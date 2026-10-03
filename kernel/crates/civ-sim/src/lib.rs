//! The kernel's composition root (plan §3): one world's authoritative state and the operations
//! the host performs on it.
//!
//! - [`Sim`] owns a world: its identity, the clock and scheduler, the permanent-id counter, the
//!   terrain and water (immutable after generation, shared by `Arc`), and the content it runs
//!   with. One thread drives it.
//! - [`persist`] saves it as, and loads it from, `commons-persist` generations (ADR-0002).
//! - [`frames`] builds boundary payloads from it (ADR-0001).
//!
//! M0 has no agents: advancing time moves the clock and delivers cadence boundaries, nothing else.

#![forbid(unsafe_code)]

pub mod frames;
pub mod persist;

use std::fmt;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use civ_content::ContentRegistry;
use civ_core::time::DEFAULT_WORLD_START;
use civ_core::{Cadence, Date, Due, IdAllocator, ScheduleError, Scheduler, SimTime};
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

/// Events the kernel schedules. M0 has none; activities and trips arrive in M1.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SimEvent {}

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
        let generate = GenerateRequest {
            seed: request.seed,
            width_cells: request.size_cells,
            height_cells: request.size_cells,
            cell_size_m: CELL_SIZE_M,
            params: preset.params.clone(),
        };
        let map = civ_world::generate(&generate, progress, cancel).map_err(SimError::Generation)?;
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
        };
        let tiebreak_seed = civ_core::rng::key(&[
            request.seed,
            u64::from_le_bytes(world_id[..8].try_into().unwrap_or([0; 8])),
        ]);
        let scheduler = Scheduler::new(DEFAULT_WORLD_START, tiebreak_seed);
        Ok(Sim::assemble(
            meta,
            map,
            scheduler,
            IdAllocator::new(),
            ContentStamp::of(content),
        ))
    }

    /// Puts a world together from its parts, paused at 1x and never saved.
    pub(crate) fn assemble(
        meta: WorldMeta,
        map: WorldMap,
        mut scheduler: Scheduler<SimEvent>,
        ids: IdAllocator,
        content: ContentStamp,
    ) -> Sim {
        // Subscriptions are code, not state (civ-core scheduler docs).
        for cadence in [Cadence::Day, Cadence::Month, Cadence::Year] {
            scheduler.subscribe(cadence);
        }
        let stats = map.stats();
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
            last_snapshot: None,
            generation: 0,
            dirty: true,
        }
    }

    /// Identity and provenance.
    pub fn meta(&self) -> &WorldMeta {
        &self.meta
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
        self.scheduler
            .advance_to(target, |due, _| match due {
                Due::Cadence { cadence, .. } => match cadence {
                    Cadence::Day => advance.days += 1,
                    Cadence::Month => advance.months += 1,
                    Cadence::Year => advance.years += 1,
                    _ => {}
                },
                Due::Event { event, .. } => match event {},
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
