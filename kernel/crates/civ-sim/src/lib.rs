//! The kernel's composition root (plan §3): one world's authoritative state and the operations
//! the host performs on it.
//!
//! - [`Sim`] owns a world: its identity, the clock and scheduler, the permanent-id counter, the
//!   terrain and water (immutable after generation, shared by `Arc`), the land and its wild
//!   stocks, the people, and the content it runs with. One thread drives it.
//! - [`persist`] saves it as, and loads it from, `commons-persist` generations (ADR-0002).
//! - [`frames`] builds boundary payloads from it (ADR-0001).
//!
//! Advancing time delivers cadence boundaries (a day boundary grows and wastes the land's
//! stocks) and the people's scheduled events, in time order (ADR-0003).

#![forbid(unsafe_code)]

pub mod frames;
pub mod persist;

use std::fmt;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use civ_agents::params::{Catalog, PeopleParams, RegimeDef};
use civ_agents::{AgentEvent, Ctx, Founded, Population, Spawned};
use civ_content::ContentRegistry;
use civ_core::time::DEFAULT_WORLD_START;
use civ_core::{Cadence, Date, Due, IdAllocator, ScheduleError, Scheduler, SimTime};
use civ_land::{Land, LandParams};
use civ_world::nav::NavGrid;
use civ_world::{GenError, GenerateRequest, MapStats, Progress, WorldMap};

/// Simulation cell size of new worlds, metres (plan §6).
pub const CELL_SIZE_M: f64 = 8.0;
/// Map sizes offered for new worlds, in cells per side: 2, 4, 8 and 16 km at 8 m cells. The plan's
/// default world is the largest (§6); the smaller ones exist for quick iteration.
pub const MAP_SIZES: [u32; 4] = [256, 512, 1024, 2048];
/// The map size new worlds get unless another is chosen.
pub const DEFAULT_MAP_SIZE: u32 = 2048;
/// Simulated seconds per real second at 1x: one in-game day per 15 real minutes (plan §4.4).
pub const SPEED_1X: f32 = 96.0;
/// The Detailed-mode speeds as multiples of 1x (plan §4.4). Accelerated mode arrives with agents.
pub const SPEED_MULTIPLIERS: [f32; 3] = [1.0, 3.0, 10.0];
/// The fastest speed, in simulated seconds per real second.
pub const MAX_SPEED: f32 = SPEED_1X * 10.0;
/// Longest world name kept, in bytes.
pub const MAX_NAME_LEN: usize = 64;
/// Name given to a world created without one.
pub const DEFAULT_NAME: &str = "Unnamed world";

/// What a new world is made from. Everything else comes from the preset.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewWorld {
    /// Display name; normalised by [`normalize_name`].
    pub name: String,
    /// World-generation seed.
    pub seed: u64,
    /// Content id of the world-generation preset, for example `core:worldgen/river_valley`.
    pub preset_id: String,
    /// Cells per side; the map is square.
    pub size_cells: u32,
    /// People in the founding band; 0 means the people profile's default.
    pub band_size: u32,
    /// Content id of the property regime (ADR-0007); empty means the content's default.
    pub regime_id: String,
}

/// Who and what a world is. Fixed when the world is created.
#[derive(Clone, Debug, PartialEq)]
pub struct WorldMeta {
    /// Stable identity across every save of the world.
    pub world_id: [u8; 16],
    /// Display name.
    pub name: String,
    /// World-generation seed.
    pub seed: u64,
    /// Preset the terrain was generated from.
    pub preset_id: String,
    /// [`civ_world::GENERATOR_VERSION`] of the build that generated the terrain.
    pub generator_version: u32,
    /// Wall-clock creation time, Unix milliseconds.
    pub created_unix_ms: i64,
    /// Effective generation parameters, by name. Provenance only: terrain is saved, never
    /// regenerated from these.
    pub params: Vec<(String, f64)>,
    /// Content id of the property regime the world lives under (ADR-0007), fixed when it is
    /// created; empty for the rules every world lived by before regimes.
    pub regime_id: String,
}

impl WorldMeta {
    /// The world id as lowercase hex.
    pub fn world_id_hex(&self) -> String {
        commons_persist::hex(&self.world_id)
    }
}

/// One content pack a world ran with.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackStamp {
    /// Pack id.
    pub id: String,
    /// Human-facing version.
    pub version: String,
    /// BLAKE3 of the pack's files.
    pub artifact_fingerprint: [u8; 32],
}

/// The content a world's state was produced with.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContentStamp {
    /// Semantic fingerprint of the content (see `civ-content`).
    pub fingerprint: [u8; 32],
    /// The packs.
    pub packs: Vec<PackStamp>,
}

impl ContentStamp {
    /// The stamp of the loaded content.
    pub fn of(registry: &ContentRegistry) -> Self {
        ContentStamp {
            fingerprint: registry.fingerprint,
            packs: registry
                .packs
                .iter()
                .map(|p| PackStamp {
                    id: p.id.clone(),
                    version: p.version.clone(),
                    artifact_fingerprint: p.artifact_fingerprint,
                })
                .collect(),
        }
    }
}

/// Events the kernel schedules.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SimEvent {
    /// A person's activity step ends (ADR-0003).
    Agent(AgentEvent),
}

/// Scheduler phase of people's events.
pub const PHASE_AGENT: u8 = 1;

/// What a world's people and land run by. Compiled from the loaded content, never saved: a world
/// saved with other content runs by the content loaded now (the save browser says so).
#[derive(Debug)]
pub struct Rules {
    /// How people live.
    pub people: PeopleParams,
    /// Habitats and wild resources.
    pub land: LandParams,
    /// The activities people can do.
    pub catalog: Catalog,
}

impl Rules {
    /// The rules of the loaded content.
    pub fn of(content: &ContentRegistry) -> Rules {
        Rules {
            people: content.people.params.clone(),
            land: content.land.params.clone(),
            catalog: content.catalog.clone(),
        }
    }
}

/// What one advance of the clock did.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Advance {
    /// Simulated minutes that passed.
    pub minutes: i64,
    /// Day boundaries crossed.
    pub days: u32,
    /// Month boundaries crossed.
    pub months: u32,
    /// Year boundaries crossed.
    pub years: u32,
}

/// Why an operation on a world failed.
#[derive(Debug)]
pub enum SimError {
    /// The content has no preset with this id.
    UnknownPreset(String),
    /// The content has no property regime with this id.
    UnknownRegime(String),
    /// A founding band must have between the people profile's smallest and largest size.
    InvalidBandSize {
        /// What was asked for.
        size: u32,
        /// The smallest band allowed.
        min: u32,
        /// The largest band allowed.
        max: u32,
    },
    /// Speeds must be finite, positive and at most [`MAX_SPEED`].
    InvalidSpeed(f32),
    /// World generation failed or was cancelled.
    Generation(GenError),
    /// The scheduler refused to advance.
    Schedule(ScheduleError),
}

impl fmt::Display for SimError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SimError::UnknownPreset(id) => write!(f, "there is no world preset `{id}`"),
            SimError::UnknownRegime(id) => write!(f, "there is no property regime `{id}`"),
            SimError::InvalidBandSize { size, min, max } => write!(
                f,
                "a founding band of {size} is out of range ({min} to {max} people)"
            ),
            SimError::InvalidSpeed(speed) => write!(
                f,
                "speed {speed} is out of range (1 to {MAX_SPEED} simulated seconds per second)"
            ),
            SimError::Generation(e) => e.fmt(f),
            SimError::Schedule(e) => write!(f, "the scheduler stopped: {e}"),
        }
    }
}

impl std::error::Error for SimError {}

/// One world and everything the kernel knows about it.
#[derive(Debug)]
pub struct Sim {
    meta: WorldMeta,
    map: Arc<WorldMap>,
    stats: MapStats,
    scheduler: Scheduler<SimEvent>,
    ids: IdAllocator,
    paused: bool,
    speed: f32,
    /// Simulated seconds owed but not yet a whole minute.
    carry_seconds: f64,
    content: ContentStamp,
    content_changed: bool,
    /// What the people and land run by (from the loaded content).
    rules: Arc<Rules>,
    /// The world's property regime, from the loaded content by the id the world keeps.
    regime: RegimeDef,
    /// Where people can walk, and how fast. Derived from the map and the rules.
    nav: Arc<NavGrid>,
    /// Habitat patches, wild stocks and settlements.
    land: Land,
    /// People, households and their history.
    people: Population,
    /// Why the founding band could not settle, when it could not. Not saved.
    founding_problem: Option<String>,
    /// The snapshot this state was last written to or loaded from.
    last_snapshot: Option<[u8; 16]>,
    /// That snapshot's generation number (0 = never saved).
    generation: u64,
    /// The state changed since it was last saved or loaded.
    dirty: bool,
}

/// Normalises a world name: control characters become spaces, runs of whitespace collapse,
/// the result is cut to [`MAX_NAME_LEN`] bytes, and an empty name becomes [`DEFAULT_NAME`].
pub fn normalize_name(name: &str) -> String {
    let words: Vec<&str> = name
        .split(|c: char| c.is_whitespace() || c.is_control())
        .collect();
    let joined = words
        .into_iter()
        .filter(|w| !w.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    let mut out = String::with_capacity(joined.len().min(MAX_NAME_LEN));
    for c in joined.chars() {
        if out.len() + c.len_utf8() > MAX_NAME_LEN {
            break;
        }
        out.push(c);
    }
    let out = out.trim_end().to_owned();
    if out.is_empty() {
        DEFAULT_NAME.to_owned()
    } else {
        out
    }
}

impl Sim {
    /// Generates a new world. `progress` is called from this thread; raising `cancel` stops
    /// generation with [`GenError::Cancelled`].
    pub fn create(
        request: &NewWorld,
        content: &ContentRegistry,
        progress: &mut dyn FnMut(Progress),
        cancel: &AtomicBool,
    ) -> Result<Sim, SimError> {
        let preset = content
            .preset(&request.preset_id)
            .ok_or_else(|| SimError::UnknownPreset(request.preset_id.clone()))?;
        let band = &content.people.params.band;
        let band_size = match request.band_size {
            0 => band.default_size,
            size if (band.min_size..=band.max_size).contains(&size) => size,
            size => {
                return Err(SimError::InvalidBandSize {
                    size,
                    min: band.min_size,
                    max: band.max_size,
                });
            }
        };
        let generate = GenerateRequest {
            seed: request.seed,
            width_cells: request.size_cells,
            height_cells: request.size_cells,
            cell_size_m: CELL_SIZE_M,
            params: preset.params.clone(),
        };
        let map = civ_world::generate(&generate, progress, cancel).map_err(SimError::Generation)?;
        let regime = if request.regime_id.is_empty() {
            content.catalog.default_regime()
        } else {
            content.catalog.regime(&request.regime_id)
        };
        let regime_id = match regime {
            Some(r) => r.id.clone(),
            None if request.regime_id.is_empty() => String::new(),
            None => return Err(SimError::UnknownRegime(request.regime_id.clone())),
        };
        let world_id = commons_persist::random_id().unwrap_or_else(|_| {
            // No OS randomness: fall back to time and seed. Identity only needs to be unique
            // among this player's worlds.
            let t = commons_persist::now_unix_ms() as u64;
            let a = civ_core::rng::key(&[t, request.seed]);
            let b = civ_core::rng::key(&[a, t]);
            let mut id = [0u8; 16];
            id[..8].copy_from_slice(&a.to_le_bytes());
            id[8..].copy_from_slice(&b.to_le_bytes());
            id
        });
        let meta = WorldMeta {
            world_id,
            name: normalize_name(&request.name),
            seed: request.seed,
            preset_id: preset.id.clone(),
            generator_version: civ_world::GENERATOR_VERSION,
            created_unix_ms: commons_persist::now_unix_ms(),
            params: preset
                .params
                .named_values()
                .into_iter()
                .map(|(name, value)| (name.to_owned(), value))
                .collect(),
            regime_id,
        };
        let tiebreak_seed = civ_core::rng::key(&[
            request.seed,
            u64::from_le_bytes(world_id[..8].try_into().unwrap_or([0; 8])),
        ]);
        let scheduler = Scheduler::new(DEFAULT_WORLD_START, tiebreak_seed);
        let rules = Arc::new(Rules::of(content));
        progress(Progress {
            stage: "Growing wild plants",
            fraction: 1.0,
        });
        let land = Land::create(
            &map,
            &rules.land,
            request.seed,
            DEFAULT_WORLD_START.day_index(),
        );
        let mut sim = Sim::assemble(
            meta,
            map,
            scheduler,
            IdAllocator::new(),
            ContentStamp::of(content),
            rules,
            land,
            Population::new(),
        );
        progress(Progress {
            stage: "The first people arrive",
            fraction: 1.0,
        });
        if let Err(why) = sim.found_band(band_size) {
            sim.founding_problem = Some(why);
        }
        // Deposits are laid down once, after the band arrives so its founding draws are as they
        // were before deposits existed (ADR-0010 §1).
        sim.land
            .place_deposits(&sim.map, &sim.rules.land, sim.meta.seed, &mut sim.ids);
        Ok(sim)
    }

    /// Brings a founding band of `size` people to the world now. They choose where to camp.
    pub fn found_band(&mut self, size: u32) -> Result<Founded, String> {
        let now = self.now();
        let mut pending = Vec::new();
        let founded = {
            let mut ctx = Ctx {
                now,
                seed: self.meta.seed,
                map: &self.map,
                nav: &self.nav,
                land: &mut self.land,
                land_params: &self.rules.land,
                params: &self.rules.people,
                catalog: &self.rules.catalog,
                regime: &self.regime,
                ids: &mut self.ids,
                schedule: &mut pending,
            };
            civ_agents::found_band(&mut self.people, &mut ctx, size)
        };
        for (at, event) in pending {
            // Agent events are never scheduled before the next minute.
            let _ = self
                .scheduler
                .schedule(at, PHASE_AGENT, SimEvent::Agent(event));
        }
        self.dirty = true;
        founded
    }

    /// The observer sends a family to `at` (god tool): it joins the settlement there or makes
    /// camp (see [`civ_agents::spawn_family`]).
    pub fn spawn_family(&mut self, at: (f32, f32)) -> Result<Spawned, String> {
        self.spawn_families(at, 1).map(|mut all| all.remove(0))
    }

    /// The observer lays down a deposit of good `good` (a content id) at `at` (metres), a disc of
    /// `radius_m` showing at the surface or under the cover the land profile's rule for the good
    /// gives (god tool, ADR-0010 §1). Its thickness, quality and density are the middle of that
    /// rule's ranges, or a metre, 0.6 and 1,800 kg/m³ for a good the profile lays down none of.
    /// Refused off the map, on water, for an unknown good or a radius outside 1 to 200 m.
    pub fn place_deposit(
        &mut self,
        at: (f32, f32),
        good: &str,
        radius_m: f32,
        exposed: bool,
    ) -> Result<civ_core::PermanentId, String> {
        let g = self
            .rules
            .catalog
            .good_index(good)
            .ok_or_else(|| format!("there is no good `{good}`"))?;
        if !(1.0..=200.0).contains(&radius_m) {
            return Err(format!(
                "a deposit's radius must be 1 to 200 m (got {radius_m})"
            ));
        }
        let cell_m = self.map.cell_size_m;
        let (cx, cy) = (at.0 / cell_m, at.1 / cell_m);
        if !(cx >= 0.0 && cy >= 0.0 && cx < self.map.width as f32 && cy < self.map.height as f32) {
            return Err("that is off the map".to_owned());
        }
        let cell = cy as usize * self.map.width as usize + cx as usize;
        if self.map.water[cell] != civ_world::WATER_LAND {
            return Err("a deposit must lie under dry land".to_owned());
        }
        let rule = self.rules.land.deposits.iter().find(|r| r.good == g);
        let mid = |(lo, hi): (f64, f64)| (lo + hi) / 2.0;
        let thickness = rule.map_or(1.0, |r| mid(r.thickness_m)).max(0.05);
        let quality = rule.map_or(0.6, |r| mid(r.quality)).clamp(0.0, 1.0);
        let density = rule.map_or(1_800.0, |r| r.density_kg_m3).max(0.0);
        let top = if exposed {
            0.0
        } else {
            rule.map_or(1.0, |r| r.top_m.1).max(0.1)
        };
        let r = f64::from(radius_m);
        let body = civ_land::deposits::Body {
            good: g as u16,
            at_cm: (
                (f64::from(at.0) * 100.0) as i64,
                (f64::from(at.1) * 100.0) as i64,
            ),
            radius_cm: (r * 100.0).round() as i32,
            top_cm: (top * 100.0).round() as i32,
            thickness_cm: (thickness * 100.0).round() as i32,
            quality: quality as f32,
            exposed,
            initial_kg: std::f64::consts::PI * r * r * thickness * density,
        };
        let id = self.ids.allocate();
        self.land.deposits.push(civ_land::deposits::Deposit {
            id,
            body,
            taken_kg: 0.0,
        });
        let nearest = self
            .land
            .settlements
            .iter()
            .min_by(|a, b| {
                let d = |s: &civ_land::Settlement| (s.hearth_m.0 - at.0).hypot(s.hearth_m.1 - at.1);
                d(a).total_cmp(&d(b))
            })
            .map(|s| s.id);
        let what = self.rules.catalog.goods[g].name.to_lowercase();
        let words = if exposed {
            format!("{what} showing at the surface")
        } else {
            format!("{what} under the ground")
        };
        let now = self.now();
        self.people.chronicle_push(
            now,
            civ_agents::history::ChronicleKind::DepositPlaced,
            Vec::new(),
            nearest,
            Some(at),
            0.0,
            words,
        );
        self.dirty = true;
        Ok(id)
    }

    /// The observer sends `families` families to the world's first settlement, for runs of
    /// villages larger than a founding band: in groups as large as the god tool sends
    /// ([`civ_agents::MAX_SPAWN_FAMILIES`]), each group about its own point 150 m from the hearth.
    /// Returns the people who came.
    pub fn send_families_to_hearth(&mut self, families: u32) -> usize {
        let Some(hearth) = self.land.settlements.first().map(|s| s.hearth_m) else {
            return 0;
        };
        let (mut left, mut group, mut people) = (families, 0u32, 0);
        while left > 0 {
            let n = left.min(civ_agents::MAX_SPAWN_FAMILIES);
            let angle = 2.4 * f64::from(group);
            let at = (
                hearth.0 + (150.0 * angle.cos()) as f32,
                hearth.1 + (150.0 * angle.sin()) as f32,
            );
            if let Ok(sent) = self.spawn_families(at, n) {
                people += sent.iter().map(|s| s.people.len()).sum::<usize>();
            }
            left -= n;
            group += 1;
        }
        people
    }

    /// The observer sends `count` families together (at most [`civ_agents::MAX_SPAWN_FAMILIES`]):
    /// the first where it placed them and the others around it (see
    /// [`civ_agents::spawn_families`]).
    pub fn spawn_families(&mut self, at: (f32, f32), count: u32) -> Result<Vec<Spawned>, String> {
        let now = self.now();
        let mut pending = Vec::new();
        let spawned = {
            let mut ctx = Ctx {
                now,
                seed: self.meta.seed,
                map: &self.map,
                nav: &self.nav,
                land: &mut self.land,
                land_params: &self.rules.land,
                params: &self.rules.people,
                catalog: &self.rules.catalog,
                regime: &self.regime,
                ids: &mut self.ids,
                schedule: &mut pending,
            };
            civ_agents::spawn_families(&mut self.people, &mut ctx, at, count)
        };
        for (when, event) in pending {
            let _ = self
                .scheduler
                .schedule(when, PHASE_AGENT, SimEvent::Agent(event));
        }
        if spawned.is_ok() {
            self.dirty = true;
        }
        spawned
    }

    /// The observer introduces a technique (god tool, ADR-0008 §6): living person `person` comes
    /// to know technique `technique` (by content id), or with `aware_only` only hears of it. The
    /// chronicle records it.
    pub fn introduce_technique(
        &mut self,
        person: civ_core::PermanentId,
        technique: &str,
        aware_only: bool,
    ) -> Result<(), String> {
        let t = self
            .rules
            .catalog
            .technique_index(technique)
            .ok_or_else(|| format!("there is no technique `{technique}`"))?;
        let now = self.now();
        let mut pending = Vec::new();
        let done = {
            let mut ctx = Ctx {
                now,
                seed: self.meta.seed,
                map: &self.map,
                nav: &self.nav,
                land: &mut self.land,
                land_params: &self.rules.land,
                params: &self.rules.people,
                catalog: &self.rules.catalog,
                regime: &self.regime,
                ids: &mut self.ids,
                schedule: &mut pending,
            };
            self.people
                .introduce_technique(&mut ctx, person, t, aware_only)
        };
        for (when, event) in pending {
            let _ = self
                .scheduler
                .schedule(when, PHASE_AGENT, SimEvent::Agent(event));
        }
        if done.is_ok() {
            self.dirty = true;
        }
        done
    }

    /// Puts a world together from its parts, paused at 1x and never saved.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn assemble(
        mut meta: WorldMeta,
        map: WorldMap,
        mut scheduler: Scheduler<SimEvent>,
        ids: IdAllocator,
        content: ContentStamp,
        rules: Arc<Rules>,
        mut land: Land,
        mut people: Population,
    ) -> Sim {
        // Subscriptions are code, not state (civ-core scheduler docs).
        for cadence in [Cadence::Day, Cadence::Month, Cadence::Year] {
            scheduler.subscribe(cadence);
        }
        // The routing view and the trails are derived from the worn ground.
        land.wear
            .survey(scheduler.now().day_index(), &rules.land.paths);
        let stats = map.stats();
        let nav = Arc::new(NavGrid::new(&map, rules.people.nav));
        people.rebuild_indexes();
        // The world's regime by the id it keeps; a world from before regimes, or whose regime the
        // content no longer has, lives under the content's default (ADR-0007 §5).
        let regime = rules
            .catalog
            .regime(&meta.regime_id)
            .or_else(|| rules.catalog.default_regime())
            .cloned()
            .unwrap_or_else(RegimeDef::legacy);
        meta.regime_id.clone_from(&regime.id);
        Sim {
            meta,
            map: Arc::new(map),
            stats,
            scheduler,
            ids,
            paused: true,
            speed: SPEED_1X,
            carry_seconds: 0.0,
            content,
            content_changed: false,
            rules,
            regime,
            nav,
            land,
            people,
            founding_problem: None,
            last_snapshot: None,
            generation: 0,
            dirty: true,
        }
    }

    /// Identity and provenance.
    pub fn meta(&self) -> &WorldMeta {
        &self.meta
    }

    /// The property regime the world lives under (ADR-0007).
    pub fn regime(&self) -> &RegimeDef {
        &self.regime
    }

    /// Terrain and water. Immutable; clone the `Arc` to read it from another thread.
    pub fn map(&self) -> &Arc<WorldMap> {
        &self.map
    }

    /// Summary figures of the map, computed once.
    pub fn stats(&self) -> &MapStats {
        &self.stats
    }

    /// The current simulation time.
    pub fn now(&self) -> SimTime {
        self.scheduler.now()
    }

    /// The current calendar date.
    pub fn date(&self) -> Date {
        self.now().date()
    }

    /// Whether the clock is stopped.
    pub fn paused(&self) -> bool {
        self.paused
    }

    /// Simulated seconds per real second while running.
    pub fn speed(&self) -> f32 {
        self.speed
    }

    /// The permanent-id counter.
    pub fn ids(&self) -> &IdAllocator {
        &self.ids
    }

    /// The content the state is produced with.
    pub fn content(&self) -> &ContentStamp {
        &self.content
    }

    /// The world was saved with different content than it runs with now.
    pub fn content_changed(&self) -> bool {
        self.content_changed
    }

    /// What the people and land run by.
    pub fn rules(&self) -> &Arc<Rules> {
        &self.rules
    }

    /// Where people can walk.
    pub fn nav(&self) -> &Arc<NavGrid> {
        &self.nav
    }

    /// Habitat patches, wild stocks and settlements.
    pub fn land(&self) -> &Land {
        &self.land
    }

    /// People, households and their history.
    pub fn people(&self) -> &Population {
        &self.people
    }

    /// People, to set up a situation in a test. The observer never edits people: it sends
    /// commands.
    #[doc(hidden)]
    pub fn people_mut_for_tests(&mut self) -> &mut Population {
        &mut self.people
    }

    /// Land, to set up a situation in a test.
    #[doc(hidden)]
    pub fn land_mut_for_tests(&mut self) -> &mut Land {
        &mut self.land
    }

    /// A fresh permanent id, for something a test sets up by hand (a firm, say).
    #[doc(hidden)]
    pub fn allocate_id_for_tests(&mut self) -> civ_core::PermanentId {
        self.dirty = true;
        self.ids.allocate()
    }

    /// Puts up a building to `spec` for `household`, through stage `stage` (the number of stages
    /// for a finished one), on a plot of its own with its program's use, for a test that needs a
    /// building no household designs yet. Its ground is not checked; the groups its finished
    /// stages put in place are sound, as if built at middling skill; its household's goods keep
    /// under it from now. `None` if the content has no such program.
    #[doc(hidden)]
    pub fn place_building_for_tests(
        &mut self,
        household: civ_core::PermanentId,
        spec: civ_grammar::BuildingSpec,
        stage: u8,
    ) -> Option<civ_core::PermanentId> {
        let catalog = &self.rules.catalog;
        let def = catalog
            .building_index(&spec.program)
            .and_then(|i| catalog.buildings.get(i))?;
        let rect = civ_agents::build::plot_rect(&spec, def);
        let use_ = def.use_;
        let now = self.now();
        let (plot, id) = (self.ids.allocate(), self.ids.allocate());
        self.land.plots.push(civ_land::Plot {
            id: plot,
            household,
            rect,
            use_,
            since: now,
        });
        // Its plot is levelled as one begun would be; past its first stage, it is done.
        let levelling = civ_agents::build::level_plot(
            &mut self.land,
            &self.map,
            &mut self.ids,
            &self.rules.people.build.levelling,
            (plot, rect),
            household,
            now,
        );
        if levelling > 0.0 && stage > 0 {
            let land = &mut self.land;
            for work in land.earthworks.iter_mut().filter(|w| w.plot == Some(plot)) {
                civ_land::earth::advance(work, &self.map, &mut land.ground, 1.0);
            }
        }
        let mut b = civ_land::Building {
            stage,
            ..civ_land::Building::new(id, household, plot, spec, now)
        };
        if let Ok(e) = civ_grammar::expand(&b.spec, &def.rules) {
            let spread = self.rules.people.build.quality_spread;
            civ_agents::condition::reconcile(&mut b, &e, self.meta.seed, spread, now);
        }
        self.land.buildings.push(b);
        self.people.buildings_changed(
            now,
            &self.land,
            &self.rules.catalog,
            &self.rules.people,
            household,
        );
        self.dirty = true;
        Some(id)
    }

    /// Someone dies now (for a test that needs a death on a given day; see
    /// [`Population::die_for_tests`]).
    #[doc(hidden)]
    pub fn die_for_tests(&mut self, person: civ_core::PermanentId) {
        let now = self.now();
        let mut pending = Vec::new();
        {
            let mut ctx = Ctx {
                now,
                seed: self.meta.seed,
                map: &self.map,
                nav: &self.nav,
                land: &mut self.land,
                land_params: &self.rules.land,
                params: &self.rules.people,
                catalog: &self.rules.catalog,
                regime: &self.regime,
                ids: &mut self.ids,
                schedule: &mut pending,
            };
            self.people.die_for_tests(&mut ctx, person);
        }
        for (at, event) in pending {
            let _ = self
                .scheduler
                .schedule(at, PHASE_AGENT, SimEvent::Agent(event));
        }
        self.dirty = true;
    }

    /// The world's regime, to set up a situation in a test (a review sooner, say). A world's
    /// regime is otherwise fixed when it is created.
    #[doc(hidden)]
    pub fn regime_mut_for_tests(&mut self) -> &mut RegimeDef {
        &mut self.regime
    }

    /// Why the founding band of a new world could not settle, when it could not.
    pub fn founding_problem(&self) -> Option<&str> {
        self.founding_problem.as_deref()
    }

    /// Number of pending scheduled events.
    pub fn pending_events(&self) -> usize {
        self.scheduler.pending()
    }

    /// The snapshot this state was last written to or loaded from.
    pub fn last_snapshot(&self) -> Option<[u8; 16]> {
        self.last_snapshot
    }

    /// Generation number of that snapshot (0 = never saved).
    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// Whether the state changed since it was last saved or loaded.
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Stops or starts the clock.
    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
        if paused {
            self.carry_seconds = 0.0;
        }
    }

    /// Changes the speed, in simulated seconds per real second.
    pub fn set_speed(&mut self, speed: f32) -> Result<(), SimError> {
        if !(speed.is_finite() && (1.0..=MAX_SPEED).contains(&speed)) {
            return Err(SimError::InvalidSpeed(speed));
        }
        self.speed = speed;
        Ok(())
    }

    /// Advances by `real_seconds` of wall-clock time at the current speed. Does nothing while
    /// paused. Fractions of a simulated minute carry over to the next call.
    pub fn advance_real(&mut self, real_seconds: f64) -> Result<Advance, SimError> {
        if self.paused || !(real_seconds.is_finite() && real_seconds > 0.0) {
            return Ok(Advance::default());
        }
        let seconds = self.carry_seconds + real_seconds * f64::from(self.speed);
        let minutes = (seconds / 60.0).floor();
        self.carry_seconds = seconds - minutes * 60.0;
        self.advance_minutes(minutes as i64)
    }

    /// Advances by whole simulated minutes, whether or not the clock is paused.
    pub fn advance_minutes(&mut self, minutes: i64) -> Result<Advance, SimError> {
        if minutes <= 0 {
            return Ok(Advance::default());
        }
        let target = self.now().plus_minutes(minutes);
        let mut advance = Advance {
            minutes,
            ..Advance::default()
        };
        let Sim {
            meta,
            map,
            scheduler,
            ids,
            rules,
            regime,
            nav,
            land,
            people,
            ..
        } = self;
        let mut pending = Vec::new();
        scheduler
            .advance_to(target, |due, followups| match due {
                Due::Cadence { cadence, at } => match cadence {
                    Cadence::Day => {
                        advance.days += 1;
                        // The day that just ended is complete.
                        land.advance_to_day(&rules.land, meta.seed, at.day_index() - 1);
                        let mut ctx = Ctx {
                            now: at,
                            seed: meta.seed,
                            map,
                            nav,
                            land,
                            land_params: &rules.land,
                            params: &rules.people,
                            catalog: &rules.catalog,
                            regime,
                            ids,
                            schedule: &mut pending,
                        };
                        people.on_day(&mut ctx);
                        // Newborns decide what to do first.
                        for (t, e) in pending.drain(..) {
                            let _ = followups.schedule(t, PHASE_AGENT, SimEvent::Agent(e));
                        }
                    }
                    Cadence::Month => {
                        advance.months += 1;
                        // People plan their routes on the paths as they stand this month.
                        land.wear.survey(at.day_index(), &rules.land.paths);
                        people.note_trails(land, at);
                    }
                    Cadence::Year => {
                        advance.years += 1;
                        // Each settlement's wealth as the year ends (ADR-0007 §4).
                        people.record_wealth(
                            &rules.catalog,
                            &rules.people,
                            &rules.land,
                            land,
                            at,
                            at.date().year - 1,
                        );
                        // Taste moves toward the year's admired buildings (M3b slice R).
                        people.review_tastes(&rules.catalog, &rules.people, &rules.land, land, at);
                    }
                    _ => {}
                },
                Due::Event {
                    at,
                    event: SimEvent::Agent(event),
                    ..
                } => {
                    let mut ctx = Ctx {
                        now: at,
                        seed: meta.seed,
                        map,
                        nav,
                        land,
                        land_params: &rules.land,
                        params: &rules.people,
                        catalog: &rules.catalog,
                        regime,
                        ids,
                        schedule: &mut pending,
                    };
                    people.on_event(&mut ctx, event);
                    for (t, e) in pending.drain(..) {
                        // Never before the current instant: agents schedule at least a minute on.
                        let _ = followups.schedule(t, PHASE_AGENT, SimEvent::Agent(e));
                    }
                }
            })
            .map_err(SimError::Schedule)?;
        self.dirty = true;
        Ok(advance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_normalised() {
        assert_eq!(
            normalize_name("  Old\tRiver \n Valley "),
            "Old River Valley"
        );
        assert_eq!(normalize_name("\u{7}\u{0}"), DEFAULT_NAME);
        assert_eq!(normalize_name(""), DEFAULT_NAME);
        let long = "é".repeat(100);
        let cut = normalize_name(&long);
        assert!(cut.len() <= MAX_NAME_LEN);
        assert!(cut.chars().all(|c| c == 'é'));
    }
}
