//! The population: people and households, and how their activities run (ADR-0003).
//!
//! An activity is a list of steps (walk, work, deposit). Each step that takes time schedules one
//! event at its end carrying the person's permanent id and the activity version; when it fires,
//! the step's effects are applied and the next step starts, or, after the last, the person
//! decides what to do next. Needs are brought up to date only at step boundaries.

use std::collections::{BTreeMap, HashMap};

use civ_core::time::MINUTES_PER_DAY;
use civ_core::{GenTable, Handle, IdAllocator, PermanentId, Rng64, SimTime};
use civ_grammar::{BuildingSpec, Stage, StageNeeds};
use civ_land::{
    Building, CropParams, Field, FieldStage, FieldTask, Land, LandParams, Plot, PlotUse,
};
use civ_world::nav::{NavGrid, RouteResult, TravelField};
use civ_world::{WATER_LAKE, WATER_RIVER, WorldMap};

use crate::build::{self, HomeWork};
use crate::decide::{
    self, BuildOption, Facts, FieldOption, GiverOption, Limits, PatchOption, WaterOption,
};
use crate::farm::{self, FarmView, Site};
use crate::history::{ChronicleEvent, ChronicleKind, PersonRecord, Reason, Receipt};
use crate::needs;
use crate::params::{
    Behavior, Catalog, GoodDef, GoodUse, HouseholdParams, PeopleParams, interpolate,
};
use crate::person::{
    Activity, Household, KnownPatch, Load, Person, Step, Target, Trip, food_kcal, fuel_kg,
    reserve_food_kcal,
};

/// Purpose tag for decision draws (ADR-0003 keyed randomness).
pub const PURPOSE_DECIDE: u64 = 0x6465_6369_6465_3031; // "decide01"
/// Most cells one route search may expand.
const ROUTE_BUDGET: usize = 600_000;
/// Route simplification tolerance, metres (ADR-0003: about 4 m).
const TRIP_TOLERANCE_M: f32 = 4.0;
/// Days a settlement's travel-time field is reused before it is recomputed. Walking costs do not
/// change until trails wear in, so this only bounds how stale a field can get.
const FIELD_REFRESH_DAYS: i64 = 30;
/// Minutes a person waits before deciding again when a walk cannot be routed.
const WAIT_AFTER_FAILURE_MIN: u32 = 10;
/// Most routes kept in the route cache before it is emptied.
const ROUTE_CACHE_MAX: usize = 50_000;
/// Farthest a household moves its home from where it stands to find clear ground to build on,
/// metres (a tuning value).
const HOME_SHIFT_M: f64 = 30.0;

/// A route at standard walking speed: vertices (cell centres) and seconds to each.
type CachedRoute = std::sync::Arc<(Vec<(f32, f32)>, Vec<f32>)>;

/// Something a person scheduled.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AgentEvent {
    /// The current step of the person's activity ends.
    Step {
        /// Who.
        person: PermanentId,
        /// The activity version it belongs to.
        version: u32,
    },
}

/// The world a person acts in, borrowed for one event.
#[derive(Debug)]
pub struct Ctx<'a> {
    /// The current time.
    pub now: SimTime,
    /// The world seed (for keyed draws).
    pub seed: u64,
    /// Terrain and water.
    pub map: &'a WorldMap,
    /// Walkability.
    pub nav: &'a NavGrid,
    /// Land state.
    pub land: &'a mut Land,
    /// Land parameters.
    pub land_params: &'a LandParams,
    /// People parameters.
    pub params: &'a PeopleParams,
    /// Activities.
    pub catalog: &'a Catalog,
    /// Permanent ids.
    pub ids: &'a mut IdAllocator,
    /// Events to schedule.
    pub schedule: &'a mut Vec<(SimTime, AgentEvent)>,
}

impl Ctx<'_> {
    fn schedule_step(&mut self, at: SimTime, person: PermanentId, version: u32) {
        let at = at.max(self.now.plus_minutes(1));
        self.schedule
            .push((at, AgentEvent::Step { person, version }));
    }
}

/// Where gathering trips go for resources of one range and medium: aligned blocks of patches of
/// side `2·range + 1`, the area one trip works. Blocks do not overlap, so land a trip has worked
/// is never mistaken for untouched land by the next. Derived.
#[derive(Debug)]
struct Places {
    range: u32,
    in_water: bool,
    /// Per block, row by row: its centre patch, and the cell people walk to (dry walkable land
    /// nearest the centre; at the water's edge for resources that live in water), if any.
    blocks: Vec<Option<(u32, u32)>>,
}

/// Travel times from a settlement's hearth (or a lone household's home), and the water and places
/// within reach. Shared by every household of the settlement: homes stand a few metres from the
/// hearth. Derived.
#[derive(Debug)]
struct HomeField {
    cell: usize,
    day: i64,
    water: Option<(u32, f32)>,
    /// Per kind of place (the index in `Population::places`), blocks by walking time to their
    /// cell, seconds.
    places: Vec<Vec<(u32, f32)>>,
    /// Walking times to every cell in reach, for fields.
    reach: TravelField,
}

/// What a gathering activity is expected to bring from each patch on one day, at equilibrium
/// (see [`Land::typical_yields`]). Derived.
#[derive(Debug, Default)]
struct Prior {
    day: i64,
    per_patch: Vec<f32>,
}

/// Every person and household of a world.
#[derive(Debug, Default)]
pub struct Population {
    /// Living people.
    pub people: GenTable<Person>,
    /// Households.
    pub households: GenTable<Household>,
    /// Every person who ever lived here, by permanent id.
    pub records: BTreeMap<PermanentId, PersonRecord>,
    /// The chronicle, oldest first.
    pub chronicle: Vec<ChronicleEvent>,
    /// The last trip id handed out.
    pub next_trip: u64,
    index: HashMap<PermanentId, Handle<Person>>,
    hh_index: HashMap<PermanentId, Handle<Household>>,
    homes: HashMap<PermanentId, HomeField>,
    /// The kinds of place gathering trips go to.
    places: Vec<Places>,
    /// Routes by (from cell, to cell); `None` when there is none. Derived: walking costs do not
    /// change while routes are kept (trail wear will empty it when it arrives).
    routes: HashMap<(u32, u32), Option<CachedRoute>>,
    /// Per activity, what a trip is expected to bring from each patch today.
    priors: Vec<Prior>,
    /// Per household, the new ground it would mark out for a field, as found on a day.
    sites: HashMap<PermanentId, (i64, Option<Site>)>,
    /// Per household without a home under way, the hut it would build and where, as found on a
    /// day.
    home_sites: HashMap<PermanentId, (i64, Option<NewHome>)>,
    /// Per building, what each of its stages needs (its design never changes). Derived.
    stage_needs: HashMap<PermanentId, Vec<StageNeeds>>,
}

/// A hut a household would begin: its design (which says where it stands) and what each stage
/// needs.
#[derive(Clone, Debug)]
struct NewHome {
    spec: BuildingSpec,
    stages: Vec<StageNeeds>,
}

/// What a household's home needs: the building under way (or the one it would begin), the work
/// left on it, the program it follows and the day it wants its roof by.
#[derive(Clone, Debug)]
struct HomePlan {
    building: Option<PermanentId>,
    spec: BuildingSpec,
    work: HomeWork,
    def: usize,
    deadline: i64,
}

/// The terrain cell under a point.
pub fn cell_of(map: &WorldMap, (x, y): (f32, f32)) -> usize {
    let c = map.cell_size_m;
    let cx = ((x / c).floor().max(0.0) as u32).min(map.width - 1);
    let cy = ((y / c).floor().max(0.0) as u32).min(map.height - 1);
    (cy * map.width + cx) as usize
}

/// The centre of a terrain cell, metres.
pub fn cell_centre(map: &WorldMap, cell: usize) -> (f32, f32) {
    let w = map.width as usize;
    let c = map.cell_size_m;
    (((cell % w) as f32 + 0.5) * c, ((cell / w) as f32 + 0.5) * c)
}

fn draw(seed: u64, person: &mut Person) -> f64 {
    let mut rng = Rng64::from_key(&[seed, PURPOSE_DECIDE, person.id.get(), person.draws]);
    person.draws += 1;
    rng.next_f64()
}

/// The energy a person's body can draw on in a shortage, kcal: the floor of their balance.
pub fn reserve_kcal(p: &Person, now: SimTime, params: &PeopleParams) -> f64 {
    let mass = needs::mass_kg(&params.energy, p.sex, p.age_years(now));
    params.energy.reserve_kcal_per_kg * mass
}

/// Firewood a household of `members` burns on day `day`, kilograms.
pub fn fuel_per_day(params: &PeopleParams, members: usize, day: i64) -> f64 {
    let month = SimTime::from_minutes(day * MINUTES_PER_DAY).date().month;
    let m = usize::from(month.clamp(1, 12)) - 1;
    params.household.fuel_kg_per_person_day[m] * members as f64
}

/// A household's stores at `now`: spoiled by each good's half-life, less the firewood burned.
pub fn stores_now(
    h: &Household,
    now: SimTime,
    params: &PeopleParams,
    goods: &[GoodDef],
) -> Vec<f64> {
    let members = h.members.len();
    h.stores_at_time(now, goods, &|d| fuel_per_day(params, members, d))
}

/// Brings a person's needs up to `now` at the rates in force since they were last settled.
pub fn settle(p: &mut Person, now: SimTime, params: &PeopleParams) {
    let dt = (now.minutes() - p.needs_at.minutes()) as f64;
    if dt <= 0.0 {
        return;
    }
    let energy = f64::from(p.energy_kcal) - f64::from(p.burn_kcal_min) * dt;
    let floor = -reserve_kcal(p, now, params);
    p.energy_kcal = energy.clamp(floor, params.energy.max_surplus_kcal) as f32;
    p.sleep_pressure =
        needs::sleep_pressure(&params.sleep, f64::from(p.sleep_pressure), dt, p.asleep) as f32;
    p.relatedness = needs::relatedness(
        f64::from(p.relatedness),
        f64::from(p.company),
        dt,
        params.social.tau_h,
    ) as f32;
    p.needs_at = now;
}

/// Basal metabolism of a person now, kcal per day.
pub fn bmr(p: &Person, now: SimTime, params: &PeopleParams) -> f64 {
    let age = p.age_years(now);
    let mass = needs::mass_kg(&params.energy, p.sex, age);
    needs::bmr_kcal_day(&params.energy, p.sex, age, mass)
}

/// Hunger now: 0 while full, rising to 1 over the hunger ramp, plus deficit units.
pub fn hunger(p: &Person, now: SimTime, params: &PeopleParams) -> f64 {
    let after = (now.minutes() - p.satiety_until.minutes()) as f64 / 60.0;
    let ramp = (after / params.energy.hunger_ramp_hours).clamp(0.0, 1.0);
    ramp + (-energy_now(p, now, params)).max(0.0) / params.energy.deficit_unit_kcal
}

/// A person's energy balance at `now`, kcal: negative in deficit, no lower than their body's
/// reserve allows.
pub fn energy_now(p: &Person, now: SimTime, params: &PeopleParams) -> f64 {
    let elapsed = (now.minutes() - p.needs_at.minutes()).max(0) as f64;
    (f64::from(p.energy_kcal) - f64::from(p.burn_kcal_min) * elapsed)
        .max(-reserve_kcal(p, now, params))
}

/// Whether a person is hungry enough to eat food kept back (seed): they have drawn
/// `energy.eat_reserve_at_deficit` of their body's reserve (research 08-02 §2.3: households can
/// eat seed in a crisis, at the cost of the next sowing).
pub fn may_eat_reserve(p: &Person, now: SimTime, params: &PeopleParams) -> bool {
    let reserve = reserve_kcal(p, now, params);
    -energy_now(p, now, params) >= params.energy.eat_reserve_at_deficit * reserve
}

/// Food energy a household holds beyond what it keeps for itself, kcal: what it can spare a
/// household in need (ordinary food above twice the days of food it tries to keep).
pub fn spare_food_kcal(
    h: &Household,
    now: SimTime,
    params: &PeopleParams,
    goods: &[GoodDef],
) -> f64 {
    let stores = stores_now(h, now, params, goods);
    let (food, _) = food_kcal(&stores, goods);
    let need = h.members.len() as f64 * params.household.daily_kcal_per_person;
    (food - 2.0 * params.household.food_target_days * need).max(0.0)
}

impl Population {
    /// An empty population.
    pub fn new() -> Population {
        Population::default()
    }

    /// Adds a household.
    pub fn insert_household(&mut self, h: Household) -> Handle<Household> {
        let id = h.id;
        let handle = self.households.insert(h);
        self.hh_index.insert(id, handle);
        handle
    }

    /// Adds a living person (their record must be added separately).
    pub fn insert_person(&mut self, p: Person) -> Handle<Person> {
        let id = p.id;
        let handle = self.people.insert(p);
        self.index.insert(id, handle);
        handle
    }

    /// Rebuilds the id indexes and drops derived caches (after loading).
    pub fn rebuild_indexes(&mut self) {
        self.index = self.people.iter().map(|(h, p)| (p.id, h)).collect();
        self.hh_index = self.households.iter().map(|(h, x)| (x.id, h)).collect();
        self.homes.clear();
        self.places.clear();
        self.routes.clear();
        self.priors.clear();
        self.sites.clear();
        self.home_sites.clear();
        self.stage_needs.clear();
    }

    /// Brings every household's shelter up to date with the land's buildings: a household whose
    /// home has its roof on keeps its stores under it (after loading, where it is not saved).
    pub fn derive_shelter(&mut self, land: &Land) {
        for (_, h) in self.households.iter_mut() {
            h.sheltered = land
                .buildings
                .iter()
                .any(|b| b.household == h.id && b.roofed());
        }
    }

    /// A living person by permanent id.
    pub fn person(&self, id: PermanentId) -> Option<&Person> {
        self.index.get(&id).and_then(|&h| self.people.get(h))
    }

    /// A household by permanent id.
    pub fn household(&self, id: PermanentId) -> Option<&Household> {
        self.hh_index.get(&id).and_then(|&h| self.households.get(h))
    }

    /// The name of anyone who ever lived here.
    pub fn name_of(&self, id: PermanentId) -> String {
        self.records
            .get(&id)
            .map_or_else(|| format!("person {id}"), |r| r.given.clone())
    }

    /// Number of living people.
    pub fn living(&self) -> usize {
        self.people.len()
    }

    /// What is wrong with the population, if anything: references that do not resolve, steps
    /// out of range, malformed trips. A save holding any of these is refused (ADR-0002).
    /// `next_id` is the permanent-id counter; `catalog_len` the number of activities.
    pub fn problems(&self, next_id: u64, catalog_len: usize) -> Vec<String> {
        let mut out = Vec::new();
        let mut ids = std::collections::HashSet::new();
        for (_, p) in self.people.iter() {
            if !ids.insert(p.id) || p.id.get() >= next_id {
                out.push(format!("person {} has a duplicate or unallocated id", p.id));
            }
            match self.household(p.household) {
                Some(h) if h.members.contains(&p.id) => {}
                _ => out.push(format!(
                    "person {} belongs to household {}, which does not list them",
                    p.id, p.household
                )),
            }
            match self.records.get(&p.id) {
                Some(r) if r.died.is_none() => {}
                _ => out.push(format!("person {} has no living record", p.id)),
            }
            if usize::from(p.act.def) >= catalog_len.max(1) {
                out.push(format!("person {} does an unknown activity", p.id));
            }
            if usize::from(p.act.step) > p.act.steps.len() {
                out.push(format!(
                    "person {} is past the last step of their activity",
                    p.id
                ));
            }
            if let Some(t) = &p.trip
                && (t.points.len() < 2
                    || t.points.len() != t.minutes.len()
                    || t.minutes.windows(2).any(|w| w[1].is_nan() || w[1] < w[0])
                    || t.id > self.next_trip)
            {
                out.push(format!("person {} has a malformed trip", p.id));
            }
            let finite = [
                p.pos.0,
                p.pos.1,
                p.energy_kcal,
                p.sleep_pressure,
                p.relatedness,
                p.burn_kcal_min,
                p.company,
            ];
            if finite.iter().any(|v| !v.is_finite()) {
                out.push(format!("person {} has a value that is not a number", p.id));
            }
        }
        let mut hh_ids = std::collections::HashSet::new();
        for (_, h) in self.households.iter() {
            if !hh_ids.insert(h.id) || h.id.get() >= next_id {
                out.push(format!(
                    "household {} has a duplicate or unallocated id",
                    h.id
                ));
            }
            for m in &h.members {
                if self.person(*m).is_none_or(|p| p.household != h.id) {
                    out.push(format!(
                        "household {} lists {m}, who does not live there",
                        h.id
                    ));
                }
            }
            if !(h.stores.iter().all(|v| v.is_finite()) && h.water_l.is_finite()) {
                out.push(format!(
                    "household {} has a store that is not a number",
                    h.id
                ));
            }
        }
        for (id, r) in &self.records {
            if *id != r.id || id.get() >= next_id {
                out.push(format!("record {id} is filed under the wrong id"));
            }
            if r.died.is_none() && self.person(*id).is_none() {
                out.push(format!("record {id} is alive but the person is missing"));
            }
        }
        for (i, e) in self.chronicle.iter().enumerate() {
            if e.seq != i as u64 + 1 {
                out.push(format!("chronicle entry {} is out of sequence", e.seq));
                break;
            }
        }
        out
    }

    /// Adds a chronicle entry.
    #[allow(clippy::too_many_arguments)]
    pub fn chronicle_push(
        &mut self,
        at: SimTime,
        kind: ChronicleKind,
        people: Vec<PermanentId>,
        settlement: Option<PermanentId>,
        place: Option<(f32, f32)>,
        number: f64,
        name: String,
    ) {
        let seq = self.chronicle.len() as u64 + 1;
        self.chronicle.push(ChronicleEvent {
            seq,
            at,
            kind,
            people,
            settlement,
            place,
            number,
            name,
        });
    }

    /// Handles a scheduled event. Events of dead people or replaced activities do nothing.
    pub fn on_event(&mut self, ctx: &mut Ctx, event: AgentEvent) {
        match event {
            AgentEvent::Step { person, version } => {
                let Some(&h) = self.index.get(&person) else {
                    return;
                };
                if self.people.get(h).map(|p| p.act.version) != Some(version) {
                    return;
                }
                self.end_step(ctx, h);
                if let Some(p) = self.people.get_mut(h) {
                    p.act.step += 1;
                }
                self.run_steps(ctx, h, 0);
            }
        }
    }

    /// Drops a person's activity so they decide again when an event of the returned version
    /// fires (used when a loaded world's content no longer has their activity). The caller
    /// schedules that event.
    pub fn restart(&mut self, id: PermanentId) -> Option<u32> {
        let &h = self.index.get(&id)?;
        let p = self.people.get_mut(h)?;
        let now = p.needs_at;
        p.act = Activity {
            def: 0,
            target: Target::None,
            steps: Vec::new(),
            step: 0,
            started: now,
            step_started: now,
            step_ends: now,
            version: p.act.version.wrapping_add(1),
        };
        p.trip = None;
        Some(p.act.version)
    }

    /// Starts a person's life in the simulation: they decide what to do first.
    pub fn begin(&mut self, ctx: &mut Ctx, id: PermanentId) {
        if let Some(&h) = self.index.get(&id) {
            self.decide(ctx, h, 0);
        }
    }

    /// Lays out the places gathering trips go, once per world: one kind of place per range and
    /// medium among the land's resources.
    fn ensure_places(&mut self, ctx: &Ctx) {
        let mut kinds: Vec<(u32, bool)> = ctx
            .land_params
            .resources
            .iter()
            .map(|r| (r.range_patches, r.in_water))
            .collect();
        kinds.sort_unstable();
        kinds.dedup();
        if self.places.len() == kinds.len()
            && self
                .places
                .iter()
                .zip(&kinds)
                .all(|(p, k)| (p.range, p.in_water) == *k)
        {
            return;
        }
        self.places = kinds
            .into_iter()
            .map(|(range, in_water)| lay_out_places(ctx, range, in_water))
            .collect();
        self.homes.clear();
    }

    /// The kind of place resource `r` is gathered in.
    fn places_for(&self, params: &LandParams, r: usize) -> Option<usize> {
        let res = params.resources.get(r)?;
        self.places
            .iter()
            .position(|p| p.range == res.range_patches && p.in_water == res.in_water)
    }

    /// Brings the expected yields of every gathering activity up to `day`.
    fn ensure_priors(&mut self, ctx: &Ctx, day: i64) {
        let defs = &ctx.catalog.activities;
        if self.priors.len() != defs.len() {
            self.priors = defs.iter().map(|_| Prior::default()).collect();
        }
        for (a, prior) in defs.iter().zip(&mut self.priors) {
            let Some(r) = a.resource.filter(|&r| r < ctx.land_params.resources.len()) else {
                continue;
            };
            if prior.day == day && prior.per_patch.len() == ctx.land.patches.len() {
                continue;
            }
            let hours = f64::from(a.max_minutes) / 60.0;
            prior.per_patch = ctx.land.typical_yields(ctx.land_params, r, hours, day);
            prior.day = day;
        }
    }

    fn field_reach_seconds(catalog: &Catalog) -> f32 {
        let max_min = catalog
            .activities
            .iter()
            .map(|a| a.max_walk_minutes)
            .max()
            .unwrap_or(60);
        // Off-trail walking is slower than the walk the decision allows for; keep some margin.
        (f64::from(max_min) * 60.0 * 1.2) as f32
    }

    fn home_field(&mut self, ctx: &Ctx, key: PermanentId, origin: (f32, f32)) {
        let cell = cell_of(ctx.map, origin);
        let day = ctx.now.day_index();
        if let Some(f) = self.homes.get(&key)
            && f.cell == cell
            && day - f.day < FIELD_REFRESH_DAYS
        {
            return;
        }
        self.ensure_places(ctx);
        let reach = Self::field_reach_seconds(ctx.catalog);
        let field = ctx
            .nav
            .travel_field(&ctx.map.elevation, cell, reach, &|_| 0.0);
        let water = nearest_water(ctx.map, &field);
        let places = self
            .places
            .iter()
            .map(|kind| {
                let mut out: Vec<(u32, f32)> = kind
                    .blocks
                    .iter()
                    .enumerate()
                    .filter_map(|(b, place)| {
                        let (_, cell) = (*place)?;
                        field.seconds_to(cell as usize).map(|s| (b as u32, s))
                    })
                    .collect();
                out.sort_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)));
                out
            })
            .collect();
        self.homes.insert(
            key,
            HomeField {
                cell,
                day,
                water,
                places,
                reach: field,
            },
        );
    }

    /// The new ground household `hh` would mark out for a field today (found once a day).
    fn site_for(
        &mut self,
        ctx: &Ctx,
        hh: &Household,
        field_key: PermanentId,
        crop: &CropParams,
    ) -> Option<Site> {
        let day = ctx.now.day_index();
        if let Some((seen, site)) = self.sites.get(&hh.id)
            && *seen == day
        {
            return *site;
        }
        let reach = &self.homes.get(&field_key)?.reach;
        // Clear of every home of the settlement and of its hearth.
        let mut homes: Vec<(f32, f32)> = self
            .households
            .iter()
            .filter(|(_, x)| {
                x.id == hh.id || (hh.settlement.is_some() && x.settlement == hh.settlement)
            })
            .map(|(_, x)| x.home)
            .collect();
        homes.extend(
            ctx.land
                .settlements
                .iter()
                .filter(|s| Some(s.id) == hh.settlement)
                .map(|s| s.hearth_m),
        );
        let site = farm::find_site(
            ctx.map,
            ctx.nav,
            ctx.land,
            ctx.land_params,
            reach,
            &homes,
            ctx.params,
            crop,
            &[ctx.seed, farm::PURPOSE_SITE, hh.id.get(), day as u64],
            hh.id,
        );
        self.sites.insert(hh.id, (day, site));
        site
    }

    /// What each stage of `building` needs (expanded once per building).
    fn needs_of(
        &mut self,
        building: &Building,
        def: &crate::params::BuildingDef,
    ) -> Option<Vec<StageNeeds>> {
        if let Some(n) = self.stage_needs.get(&building.id) {
            return Some(n.clone());
        }
        let needs = build::stage_needs(&building.spec, def)?;
        self.stage_needs.insert(building.id, needs.clone());
        Some(needs)
    }

    /// What household `hh`'s home needs: its building under way, or else the hut it would begin
    /// and where (found once a day: its home if the ground there is clear, else the nearest clear
    /// ground within reach of the hearth). `Built` once its home is finished; `NoPlace` when
    /// there is no program to build to or no clear ground.
    fn home_plan(
        &mut self,
        ctx: &Ctx,
        hh: &Household,
        hearth: Option<(f32, f32)>,
        field_key: PermanentId,
    ) -> Result<HomePlan, Reason> {
        let catalog = ctx.catalog;
        let mut built = false;
        for b in ctx.land.buildings.iter().filter(|b| b.household == hh.id) {
            if b.finished() {
                built = true;
                continue;
            }
            let def = catalog
                .building_index(&b.spec.program)
                .ok_or(Reason::NoPlace)?;
            let needs = self
                .needs_of(b, &catalog.buildings[def])
                .ok_or(Reason::NoPlace)?;
            return Ok(HomePlan {
                building: Some(b.id),
                spec: b.spec.clone(),
                work: HomeWork::of(b, needs),
                def,
                deadline: build::roof_deadline(
                    b.started.day_index(),
                    catalog.buildings[def].roof_by_day,
                ),
            });
        }
        if built {
            return Err(Reason::Built);
        }
        let def = ctx.params.home_program;
        let program = catalog.buildings.get(def).ok_or(Reason::NoPlace)?;
        let day = ctx.now.day_index();
        let found = match self.home_sites.get(&hh.id) {
            Some((seen, site)) if *seen == day => site.clone(),
            _ => {
                let reach = self.homes.get(&field_key).map(|f| &f.reach);
                let residents = hh.members.len().max(1);
                let design_at = |at: (f32, f32)| {
                    let reachable =
                        reach.is_none_or(|r| r.seconds_to(cell_of(ctx.map, at)).is_some());
                    reachable.then(|| build::design(program, &catalog.goods, residents, at, hearth))
                };
                let site = build::home_site(
                    ctx.land,
                    ctx.map,
                    ctx.nav,
                    program,
                    &design_at,
                    hh.home,
                    hearth,
                    HOME_SHIFT_M,
                )
                .and_then(|spec| {
                    let stages = build::stage_needs(&spec, program)?;
                    Some(NewHome { spec, stages })
                });
                self.home_sites.insert(hh.id, (day, site.clone()));
                site
            }
        };
        let site = found.ok_or(Reason::NoPlace)?;
        Ok(HomePlan {
            building: None,
            spec: site.spec,
            work: HomeWork::begin(site.stages),
            def,
            deadline: build::roof_deadline(day, program.roof_by_day),
        })
    }

    /// Household `household` claims the ground for the hut it planned today and begins it: the
    /// plot is marked out, the building is begun, and the household's home moves to it. `None`
    /// if the ground is no longer clear.
    fn begin_home(&mut self, ctx: &mut Ctx, household: PermanentId) -> Option<PermanentId> {
        let (_, site) = self.home_sites.remove(&household)?;
        let site = site?;
        let def = ctx.catalog.buildings.get(ctx.params.home_program)?;
        let rect = build::plot_rect(&site.spec, def);
        if !build::plot_clear(ctx.land, ctx.map, ctx.nav, &rect) {
            return None;
        }
        let (plot, id) = (ctx.ids.allocate(), ctx.ids.allocate());
        ctx.land.plots.push(Plot {
            id: plot,
            household,
            rect,
            use_: PlotUse::Dwelling,
            since: ctx.now,
        });
        ctx.land.buildings.push(Building {
            id,
            household,
            plot,
            spec: site.spec.clone(),
            stage: 0,
            work_h: 0.0,
            started: ctx.now,
            stage_since: ctx.now,
        });
        self.stage_needs.insert(id, site.stages);
        let at = build::centre_m(&site.spec);
        if let Some(&hd) = self.hh_index.get(&household)
            && let Some(x) = self.households.get_mut(hd)
        {
            x.home = at;
        }
        Some(id)
    }

    /// The household of `hh`'s settlement that could give it the most food now, when it holds
    /// `held` kcal of a need of `kcal_day` a day and is short; the walk to its home is counted
    /// from the settlement's hearth (from `field_key`'s travel field).
    fn best_giver(
        &self,
        ctx: &Ctx,
        hh: &Household,
        field_key: PermanentId,
        kcal_day: f64,
        held: f64,
    ) -> Option<GiverOption> {
        let settlement = hh.settlement?;
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let want = (params.household.food_target_days * kcal_day - held).max(0.0);
        if want <= 0.0 {
            return None;
        }
        let reach = &self.homes.get(&field_key)?.reach;
        let mut best: Option<GiverOption> = None;
        for (_, x) in self.households.iter() {
            if x.id == hh.id || x.settlement != Some(settlement) {
                continue;
            }
            let kcal = spare_food_kcal(x, now, params, goods).min(want);
            if kcal <= 0.0 {
                continue;
            }
            let Some(secs) = reach.seconds_to(cell_of(ctx.map, x.home)) else {
                continue;
            };
            let walk_min = f64::from(secs) / 60.0;
            let better = best.is_none_or(|b| {
                kcal > b.kcal + 1e-6 || ((kcal - b.kcal).abs() <= 1e-6 && walk_min < b.walk_min)
            });
            if better {
                best = Some(GiverOption {
                    household: x.id,
                    walk_min,
                    at: x.home,
                    kcal,
                });
            }
        }
        best
    }

    fn decide(&mut self, ctx: &mut Ctx, h: Handle<Person>, depth: u32) {
        let now = ctx.now;
        let params = ctx.params;
        let Some(p) = self.people.get_mut(h) else {
            return;
        };
        settle(p, now, params);
        let (hh_id, age, pos) = (p.household, p.age_years(now), p.pos);
        let Some(hh) = self.household(hh_id).cloned() else {
            return;
        };
        let hearth = hh.settlement.and_then(|s| {
            ctx.land
                .settlements
                .iter()
                .find(|x| x.id == s)
                .map(|x| x.hearth_m)
        });
        // One travel field per settlement, from its hearth.
        let field_key = match (hh.settlement, hearth) {
            (Some(s), Some(_)) => s,
            _ => hh_id,
        };
        self.home_field(ctx, field_key, hearth.unwrap_or(hh.home));
        let goods = &ctx.catalog.goods;
        let member_count = hh.members.len().max(1);
        let members = member_count as f64;
        let kcal_day = members * params.household.daily_kcal_per_person;
        let fuel_day = |d: i64| fuel_per_day(params, member_count, d);
        let stores = hh.stores_at_time(now, goods, &fuel_day);
        let (food_all, food_raw) = food_kcal(&stores, goods);
        // Seed is food only to someone in real hunger, and then only once nothing else is left.
        let reserve_ok = self
            .people
            .get(h)
            .is_some_and(|p| may_eat_reserve(p, now, params));
        let (seed_all, seed_raw) = if reserve_ok {
            // Less the seed to sow the ground already cropped, which is never eaten.
            let mut spare = stores.clone();
            let mine = ctx.land.fields.iter().filter(|f| f.household == hh_id);
            if let Some((g, kg)) = farm::protected_seed(mine, &ctx.catalog.crops)
                && let Some(s) = spare.get_mut(g)
            {
                *s = (*s - kg).max(0.0);
            }
            reserve_food_kcal(&spare, goods)
        } else {
            (0.0, 0.0)
        };
        let fuel = fuel_kg(&stores, goods);
        let edible = if fuel > 0.0 { food_all } else { food_raw };
        let edible_seed = if fuel > 0.0 { seed_all } else { seed_raw };
        let household_fuel_day = fuel_per_day(params, member_count, now.day_index());
        let water_l = hh.water_at_time(now, members * params.household.water_l_per_person_day);
        let doy = civ_land::day_of_year(now.day_index());
        let sun = needs::daylight(params.latitude_deg, doy);
        let minute = now.minute_of_day();
        let dark = minute < sun.0 || minute >= sun.1;
        let until_sunrise = if minute < sun.0 {
            sun.0 - minute
        } else {
            MINUTES_PER_DAY - minute + needs::daylight(params.latitude_deg, doy + 1).0
        };
        let evening_end = (sun.1 + 240).min(MINUTES_PER_DAY);
        let evening = if minute >= sun.1 - 60 && minute < evening_end {
            1.0
        } else {
            0.0
        };
        let Some(p) = self.people.get(h) else {
            return;
        };
        let circ = needs::circadian(&params.sleep, minute, sun);
        let facts = Facts {
            age,
            capacity: interpolate(&params.capacity_by_age, age),
            hunger: hunger(p, now, params).min(params.decision.max_hunger_drive),
            sleep_drive: f64::from(p.sleep_pressure) * circ,
            sleep_pressure: f64::from(p.sleep_pressure),
            loneliness: 1.0 - f64::from(p.relatedness),
            dark,
            daylight_left_min: if dark { 0.0 } else { (sun.1 - minute) as f64 },
            evening,
            until_sunrise_min: until_sunrise as f64,
            food_days: food_all / kcal_day.max(1.0),
            food_target_days: params.household.food_target_days,
            water_days: water_l / (members * params.household.water_l_per_person_day).max(1e-6),
            water_target_days: params.household.water_target_days,
            household_kcal_day: kcal_day,
            has_food: edible + edible_seed > 1.0,
            food_needs_fire: food_all + seed_all > 1.0 && edible + edible_seed <= 1.0,
            fuel_days: fuel / household_fuel_day.max(1e-6),
            fuel_target_days: params.household.fuel_target_days,
            household_fuel_day,
            at_home: (pos.0 - hh.home.0).abs() < 1.0 && (pos.1 - hh.home.1).abs() < 1.0,
            home: hh.home,
            hearth,
        };
        let limits = Limits {
            sleep_needed_min: needs::minutes_to_rest(&params.sleep, f64::from(p.sleep_pressure)),
            sleep_min: params.sleep.min_hours * 60.0,
            sleep_max: params.sleep.max_hours * 60.0,
            nap: (params.sleep.nap_min_minutes, params.sleep.nap_max_minutes),
            sleep_threshold: params.sleep.wake_pressure,
            meal_min: params.energy.meal_minutes,
            carry_kg: params.household.carry_kg,
        };
        self.ensure_priors(ctx, now.day_index());
        // Farming: what the household can put into its fields, what it holds, what it needs.
        let today = now.day_index();
        let crop = ctx.catalog.crops.get(params.farm.crop);
        let grain_kcal = crop
            .and_then(|c| goods.get(c.good))
            .map_or(0.0, |g| g.kcal_per_kg);
        let held = |g: Option<usize>| {
            g.and_then(|g| stores.get(g))
                .copied()
                .unwrap_or(0.0)
                .max(0.0)
        };
        let seed_kg = held(crop.map(|c| c.seed_good));
        let grain_days = held(crop.map(|c| c.good)) * grain_kcal / kcal_day.max(1.0);
        let target_days = params.farm.grain_target_days.max(1e-6);
        let labour_per_day = self.labour_per_day(&hh.members, now, params);
        let farm_view = crop.map(|c| FarmView {
            crop: c,
            kcal_per_kg: grain_kcal,
            fields: ctx
                .land
                .fields
                .iter()
                .filter(|f| f.household == hh_id)
                .collect(),
            day: today,
            labour_per_day,
            seed_kg,
            room: target_days / (target_days + grain_days),
            need_ha: farm::need_area_ha(member_count, params, c, grain_kcal),
        });
        let field_ha = params.farm.field_m * params.farm.field_m / 10_000.0;
        let wants_land = farm_view
            .as_ref()
            .is_some_and(|v| v.wants_new_field(field_ha, 0.0));
        let site = match crop {
            Some(c) if wants_land => self.site_for(ctx, &hh, field_key, c),
            _ => None,
        };
        // Building: the home under way or the one the household would begin, what it still has
        // to bring for it, and how pressing it is.
        let plan = self.home_plan(ctx, &hh, hearth, field_key);
        let (build_need, build_urgency) = match &plan {
            Ok(p) => {
                let def = &ctx.catalog.buildings[p.def];
                let need = p.work.need_by_good(def, Stage::Roof, &stores, goods.len());
                // The work left counts the loads still to be cut and carried home.
                let loads = need.iter().sum::<f64>() / params.household.carry_kg.max(1e-6);
                let hours = p.work.hours_left(Stage::Roof) + loads * build::HAUL_H_PER_LOAD;
                (
                    need,
                    build::urgency(hours, p.deadline, labour_per_day, today),
                )
            }
            Err(_) => (Vec::new(), 0.0),
        };
        let home = self.homes.get(&field_key);
        let water = home.and_then(|f| {
            f.water.map(|(cell, s)| WaterOption {
                cell,
                walk_min: f64::from(s) / 60.0,
                at: cell_centre(ctx.map, cell as usize),
            })
        });
        let land_params = ctx.land_params;
        let map = ctx.map;
        let catalog = ctx.catalog;
        let (places, priors) = (&self.places, &self.priors);
        let place_kinds: Vec<Option<usize>> = catalog
            .activities
            .iter()
            .map(|a| a.resource.and_then(|r| self.places_for(land_params, r)))
            .collect();
        let best_patch = |def: usize| -> Option<PatchOption> {
            let a = &catalog.activities[def];
            let r = a.resource?;
            let res = land_params.resources.get(r)?;
            let good = catalog.goods.get(res.good)?;
            let home = home?;
            let hours = f64::from(a.max_minutes) / 60.0;
            let prior_yield = &priors.get(def)?.per_patch;
            // Days of the household's need it already holds of this good.
            let held = stores.get(res.good).copied().unwrap_or(0.0).max(0.0);
            let stored_days = match good.purpose {
                GoodUse::Food => held * good.kcal_per_kg / kcal_day.max(1.0),
                GoodUse::Fuel => held / household_fuel_day.max(1e-6),
                GoodUse::Material => 0.0,
            };
            // A material is worth bringing only toward what the household is building.
            let (need_kg, urgency) = match good.purpose {
                GoodUse::Material => (
                    build_need.get(res.good).copied().unwrap_or(0.0),
                    build_urgency,
                ),
                GoodUse::Food | GoodUse::Fuel => (0.0, 0.0),
            };
            let kind = place_kinds.get(def).copied().flatten()?;
            // What the settlement knows of this resource, by place.
            let memory: HashMap<u32, &KnownPatch> = hh
                .known
                .iter()
                .filter(|k| usize::from(k.resource) == r)
                .map(|k| (k.patch, k))
                .collect();
            let mut best: Option<(f64, PatchOption)> = None;
            for &(block, secs) in home.places.get(kind)? {
                let walk = f64::from(secs) / 60.0;
                if walk > f64::from(a.max_walk_minutes) {
                    break;
                }
                let Some((patch, cell)) =
                    places[kind].blocks.get(block as usize).copied().flatten()
                else {
                    continue;
                };
                let prior = prior_yield
                    .get(patch as usize)
                    .map_or(0.0, |&y| f64::from(y) / hours.max(1e-6));
                // What they have seen there, weighed against what they would expect.
                let rate = match memory.get(&patch) {
                    Some(k) => k.belief(prior, today, res.renewal_days()),
                    None => prior,
                };
                if rate <= 0.0 {
                    continue;
                }
                let kg_per_hour = rate * res.unit_kg;
                let value = kg_per_hour * hours / (hours + 2.0 * walk / 60.0);
                if best.is_none_or(|(bv, _)| value > bv) {
                    best = Some((
                        value,
                        PatchOption {
                            patch,
                            walk_min: walk,
                            kg_per_hour,
                            purpose: good.purpose,
                            kcal_per_kg: good.kcal_per_kg,
                            stored_days,
                            need_kg,
                            urgency,
                            at: cell_centre(map, cell as usize),
                        },
                    ));
                }
            }
            best.map(|(_, o)| o)
        };
        let reach = home.map(|f| &f.reach);
        let best_field = |def: usize| -> Result<FieldOption, Reason> {
            let a = &catalog.activities[def];
            let (Some(task), Some(view)) = (a.task, farm_view.as_ref()) else {
                return Err(Reason::NoPlace);
            };
            let walk = |f: &Field| {
                let cell = cell_of(map, f.rect.centre_m());
                reach?.seconds_to(cell).map(|s| f64::from(s) / 60.0)
            };
            view.best(task, f64::from(a.max_minutes) / 60.0, site.as_ref(), &walk)
        };
        let giver = self.best_giver(ctx, &hh, field_key, kcal_day, food_all);
        let build = plan.as_ref().map_err(|why| *why).and_then(|p| {
            let def = &catalog.buildings[p.def];
            let held = p.work.held_by_slot(def, &stores);
            let at = build::centre_m(&p.spec);
            let at_home = (at.0 - hh.home.0).abs() < 1.0 && (at.1 - hh.home.1).abs() < 1.0;
            let walk_min = if at_home {
                0.0
            } else {
                let secs = reach.and_then(|r| r.seconds_to(cell_of(map, at)));
                f64::from(secs.ok_or(Reason::Unreachable)?) / 60.0
            };
            Ok(BuildOption {
                building: p.building,
                at,
                walk_min,
                workable_h: p.work.workable_h(&held),
                left_h: p
                    .work
                    .needs()
                    .map_or(0.0, |s| (s.labour_h - p.work.done_h).max(0.0)),
                urgency: build_urgency,
            })
        });
        let (cands, excluded) = decide::candidates(
            &catalog.activities,
            &params.decision,
            &facts,
            &limits,
            &best_patch,
            &best_field,
            water,
            giver,
            build,
        );
        let Some(p) = self.people.get_mut(h) else {
            return;
        };
        if cands.is_empty() {
            // Nothing can be done: wait and look again.
            self.wait(ctx, h, WAIT_AFTER_FAILURE_MIN);
            return;
        }
        let totals: Vec<f32> = cands.iter().map(|c| c.scored.total).collect();
        let u = draw(ctx.seed, p);
        let (choice, probability, temperature) = decide::choose(&totals, &params.decision, u);
        let mut order: Vec<usize> = (0..cands.len()).filter(|&i| i != choice).collect();
        order.sort_by(|&a, &b| cands[b].scored.total.total_cmp(&cands[a].scored.total));
        let mut receipt = Receipt {
            at: now,
            chosen: cands[choice].scored.clone(),
            runner_up: order.first().map(|&i| cands[i].scored.clone()),
            others: order
                .iter()
                .skip(1)
                .take(2)
                .map(|&i| (cands[i].scored.def, cands[i].scored.total))
                .collect(),
            excluded: excluded.into_iter().take(4).collect(),
            probability: probability as f32,
            temperature: temperature as f32,
            needs: [
                facts.hunger as f32,
                facts.sleep_drive as f32,
                facts.loneliness as f32,
                facts.food_days as f32,
                facts.water_days as f32,
            ],
        };
        let chosen = &cands[choice];
        let mut target = chosen.scored.target;
        if target == Target::NewField {
            // New ground is marked out when someone sets off to break it.
            match site {
                Some(site) => {
                    let id = ctx.ids.allocate();
                    ctx.land.fields.push(Field {
                        id,
                        household: hh_id,
                        rect: site.rect,
                        crop: params.farm.crop as u16,
                        stage: FieldStage::Fallow,
                        stage_since: now,
                        work_h: 0.0,
                        tended_h: 0.0,
                        ground: site.ground as f32,
                        clear_h_per_ha: site.clear_h_per_ha as f32,
                        broken: false,
                        sown_day: 0,
                        sheaves_kg: 0.0,
                        harvests: 0,
                    });
                    self.sites.remove(&hh_id);
                    target = Target::Field(id);
                }
                None => target = Target::None,
            }
            receipt.chosen.target = target;
        }
        let mut steps = chosen.steps.clone();
        if target == Target::NewBuilding {
            // Ground for a home is claimed when someone sets to work on it; the household's home
            // moves there.
            target = self
                .begin_home(ctx, hh_id)
                .map_or(Target::None, Target::Building);
            receipt.chosen.target = target;
            if let Some(new_home) = self.household(hh_id).map(|x| x.home) {
                for s in &mut steps {
                    if let Step::Walk { to } = s
                        && *to == hh.home
                    {
                        *to = new_home;
                    }
                }
            }
        }
        let Some(p) = self.people.get_mut(h) else {
            return;
        };
        p.act = Activity {
            def: chosen.scored.def,
            target,
            steps,
            step: 0,
            started: now,
            step_started: now,
            step_ends: now,
            version: p.act.version.wrapping_add(1),
        };
        p.push_receipt(receipt);
        self.run_steps(ctx, h, depth + 1);
    }

    /// Replaces the activity with waiting where the person stands.
    fn wait(&mut self, ctx: &mut Ctx, h: Handle<Person>, minutes: u32) {
        let Some(p) = self.people.get_mut(h) else {
            return;
        };
        let def = p.act.def;
        p.act = Activity {
            def,
            target: Target::None,
            steps: vec![Step::Wait { minutes }],
            step: 0,
            started: ctx.now,
            step_started: ctx.now,
            step_ends: ctx.now,
            version: p.act.version.wrapping_add(1),
        };
        p.trip = None;
        self.start_work(ctx, h, minutes, None);
    }

    /// Starts the current step; runs instant steps; decides again after the last.
    fn run_steps(&mut self, ctx: &mut Ctx, h: Handle<Person>, depth: u32) {
        for _ in 0..8 {
            let Some(p) = self.people.get(h) else {
                return;
            };
            let Some(step) = p.act.steps.get(p.act.step as usize).copied() else {
                if depth > 2 {
                    // An activity of instant steps only: never loop at one instant.
                    self.wait(ctx, h, 1);
                } else {
                    self.decide(ctx, h, depth);
                }
                return;
            };
            match step {
                Step::Deposit => {
                    self.deposit(ctx, h);
                    if let Some(p) = self.people.get_mut(h) {
                        p.act.step += 1;
                    }
                }
                Step::Walk { to } => {
                    if !self.start_walk(ctx, h, to) {
                        self.wait(ctx, h, WAIT_AFTER_FAILURE_MIN);
                    }
                    return;
                }
                Step::Work { minutes } => {
                    let behavior = ctx
                        .catalog
                        .activities
                        .get(p.act.def as usize)
                        .map(|a| a.behavior);
                    self.start_work(ctx, h, minutes, behavior);
                    return;
                }
                Step::Wait { minutes } => {
                    self.start_work(ctx, h, minutes, None);
                    return;
                }
            }
        }
        self.wait(ctx, h, 1);
    }

    fn start_walk(&mut self, ctx: &mut Ctx, h: Handle<Person>, to: (f32, f32)) -> bool {
        let now = ctx.now;
        let params = ctx.params;
        let Some(p) = self.people.get_mut(h) else {
            return false;
        };
        let from_cell = cell_of(ctx.map, p.pos);
        let to_cell = cell_of(ctx.map, to);
        let speed = interpolate(&params.walk_speed_by_age, p.age_years(now)).max(0.05);
        let (points, minutes): (Vec<(f32, f32)>, Vec<f32>) = if from_cell == to_cell {
            let d = ((to.0 - p.pos.0).powi(2) + (to.1 - p.pos.1).powi(2)).sqrt();
            let v = params.nav.tobler_ms(0.0) * params.nav.offtrail_factor * speed;
            let m = (f64::from(d) / v / 60.0) as f32;
            (vec![p.pos, to], vec![0.0, m])
        } else {
            let key = (from_cell as u32, to_cell as u32);
            let cached = match self.routes.get(&key) {
                Some(known) => known.clone(),
                None => {
                    // No trails yet: walking is off-trail everywhere.
                    let routed = ctx.nav.route_bounded(
                        &ctx.map.elevation,
                        from_cell,
                        to_cell,
                        &|_| 0.0,
                        0.0,
                        ROUTE_BUDGET,
                    );
                    let found = match routed {
                        RouteResult::Found(route) => Some(std::sync::Arc::new(
                            ctx.nav.polyline(&route, TRIP_TOLERANCE_M),
                        )),
                        RouteResult::Unreachable | RouteResult::BudgetExhausted => None,
                    };
                    if self.routes.len() >= ROUTE_CACHE_MAX {
                        self.routes.clear();
                    }
                    self.routes.insert(key, found.clone());
                    found
                }
            };
            let Some(route) = cached else {
                return false;
            };
            let (pts, secs) = &*route;
            let mut pts = pts.clone();
            if let Some(first) = pts.first_mut() {
                *first = p.pos;
            }
            if let Some(last) = pts.last_mut() {
                *last = to;
            }
            let mins = secs
                .iter()
                .map(|s| (f64::from(*s) / 60.0 / speed) as f32)
                .collect();
            (pts, mins)
        };
        let duration = minutes.last().copied().unwrap_or(0.0).ceil().max(1.0);
        self.next_trip += 1;
        let trip = Trip {
            id: self.next_trip,
            rev: 0,
            depart: now,
            points,
            minutes,
        };
        settle(p, now, params);
        p.burn_kcal_min = (bmr(p, now, params) * params.energy.walk_par / 1440.0) as f32;
        p.asleep = false;
        p.company = 0.0;
        p.trip = Some(trip);
        p.act.step_started = now;
        p.act.step_ends = now.plus_minutes(duration as i64);
        let (id, version, ends) = (p.id, p.act.version, p.act.step_ends);
        ctx.schedule_step(ends, id, version);
        true
    }

    fn companions_at_hearth(&self, ctx: &Ctx, me: PermanentId, hearth: Target) -> usize {
        self.people
            .iter()
            .filter(|(_, q)| {
                q.id != me
                    && q.act.target == hearth
                    && q.trip.is_none()
                    && matches!(
                        q.act.steps.get(q.act.step as usize),
                        Some(Step::Work { .. })
                    )
                    && ctx
                        .catalog
                        .activities
                        .get(q.act.def as usize)
                        .is_some_and(|a| a.behavior == Behavior::Socialize)
            })
            .count()
    }

    fn start_work(
        &mut self,
        ctx: &mut Ctx,
        h: Handle<Person>,
        minutes: u32,
        behavior: Option<Behavior>,
    ) {
        let now = ctx.now;
        let params = ctx.params;
        let companions = match (behavior, self.people.get(h)) {
            (Some(Behavior::Socialize), Some(p)) => {
                self.companions_at_hearth(ctx, p.id, p.act.target)
            }
            _ => 0,
        };
        let Some(p) = self.people.get_mut(h) else {
            return;
        };
        settle(p, now, params);
        let def_par = ctx
            .catalog
            .activities
            .get(p.act.def as usize)
            .map_or(params.energy.idle_par, |a| a.par);
        let (par, asleep, company) = match behavior {
            Some(Behavior::Sleep) => (def_par, true, params.social.household_quality),
            Some(Behavior::Socialize) => (
                def_par,
                false,
                (params.social.quality_per_companion * companions as f64).min(1.0),
            ),
            Some(Behavior::Eat | Behavior::Rest | Behavior::Play) => {
                (def_par, false, params.social.household_quality)
            }
            Some(
                Behavior::Gather
                | Behavior::FetchWater
                | Behavior::Farm
                | Behavior::Ask
                | Behavior::Build,
            ) => (def_par, false, 0.0),
            None => (params.energy.idle_par, false, 0.0),
        };
        p.burn_kcal_min = (bmr(p, now, params) * par / 1440.0) as f32;
        p.asleep = asleep;
        p.company = company as f32;
        p.trip = None;
        p.act.step_started = now;
        p.act.step_ends = now.plus_minutes(i64::from(minutes.max(1)));
        let (id, version, ends) = (p.id, p.act.version, p.act.step_ends);
        ctx.schedule_step(ends, id, version);
    }

    fn end_step(&mut self, ctx: &mut Ctx, h: Handle<Person>) {
        let now = ctx.now;
        let params = ctx.params;
        let Some(p) = self.people.get_mut(h) else {
            return;
        };
        settle(p, now, params);
        let step = p.act.steps.get(p.act.step as usize).copied();
        let def = ctx.catalog.activities.get(p.act.def as usize).cloned();
        match step {
            Some(Step::Walk { to }) => {
                p.pos = to;
                p.trip = None;
            }
            Some(Step::Work { minutes }) => match def.as_ref().map(|d| d.behavior) {
                Some(Behavior::Sleep) => p.asleep = false,
                Some(Behavior::Eat) => {
                    let hh = p.household;
                    self.eat(ctx, h, hh);
                }
                Some(Behavior::Gather) => {
                    if let (Some(d), Target::Patch(patch)) = (def.as_ref(), p.act.target)
                        && let Some(r) = d.resource
                        && let Some(res) = ctx.land_params.resources.get(r)
                    {
                        let eff = interpolate(&params.capacity_by_age, p.age_years(now));
                        let hours = f64::from(minutes) / 60.0;
                        let u = draw(ctx.seed, p);
                        let got =
                            ctx.land
                                .harvest(ctx.land_params, r, patch as usize, hours, eff, u);
                        // Carry a load at most: plants and fish beyond it stay where they were;
                        // meat beyond it is left at the kill.
                        let can = params.household.carry_kg / res.unit_kg.max(1e-9);
                        let kept = got.min(can);
                        if got > kept && !res.discrete {
                            ctx.land.stocks[r][patch as usize] += (got - kept) as f32;
                        }
                        p.carrying.good = Some(res.good as u16);
                        p.carrying.kg = (kept * res.unit_kg) as f32;
                        let hh_id = p.household;
                        let seen = (r as u16, patch, got, hours, now.day_index());
                        self.remember_patch(hh_id, seen, res.renewal_days());
                    }
                }
                Some(Behavior::FetchWater) => {
                    p.carrying.water_l = params.household.carry_water_l as f32;
                }
                Some(Behavior::Farm) => {
                    if let (Some(d), Target::Field(field)) = (def.as_ref(), p.act.target)
                        && let Some(task) = d.task
                    {
                        let eff = interpolate(&params.capacity_by_age, p.age_years(now));
                        let hours = f64::from(minutes) / 60.0 * eff;
                        let (who, household) = (p.id, p.household);
                        self.field_work(ctx, who, household, field, task, hours);
                    }
                }
                Some(Behavior::Ask) => {
                    if let Target::Household(giver) = p.act.target {
                        let household = p.household;
                        self.give_food(ctx, giver, household);
                    }
                }
                Some(Behavior::Build) => {
                    if let Target::Building(building) = p.act.target {
                        let eff = interpolate(&params.capacity_by_age, p.age_years(now));
                        let hours = f64::from(minutes) / 60.0 * eff;
                        let (who, household) = (p.id, p.household);
                        self.build_work(ctx, who, household, building, hours);
                    }
                }
                _ => {}
            },
            Some(Step::Wait { .. } | Step::Deposit) | None => {}
        }
        if let Some(p) = self.people.get_mut(h) {
            p.burn_kcal_min = (bmr(p, now, params) * params.energy.idle_par / 1440.0) as f32;
            p.asleep = false;
        }
    }

    /// Records what a trip found at a place: `(resource, place, units, hours, day)`. Households of a
    /// settlement share what they learn (they talk at the hearth), so every one of them remembers
    /// it; what has faded to almost nothing is dropped.
    fn remember_patch(
        &mut self,
        household: PermanentId,
        (resource, patch, units, hours, day): (u16, u32, f64, f64, i64),
        renewal_days: f64,
    ) {
        let settlement = self.household(household).and_then(|h| h.settlement);
        for (_, hh) in self.households.iter_mut() {
            if hh.id != household && (settlement.is_none() || hh.settlement != settlement) {
                continue;
            }
            hh.known.retain(|k| {
                k.resource != resource || k.weight(day, renewal_days) >= 0.05 * f64::from(k.hours)
            });
            match hh
                .known
                .iter_mut()
                .find(|k| k.resource == resource && k.patch == patch)
            {
                Some(k) => k.observe(units, hours, day, renewal_days),
                None => {
                    let mut k = KnownPatch {
                        resource,
                        patch,
                        rate: 0.0,
                        hours: 0.0,
                        seen_day: day,
                    };
                    k.observe(units, hours, day, renewal_days);
                    hh.known.push(k);
                }
            }
        }
    }

    /// Household `giver` gives household `to`, which asked, food it can spare: up to what brings
    /// `to` to the days of food it tries to keep, and no more than one person carries home. The
    /// most perishable food goes first. No debt is kept (research 08-11 §5.4: need-based help).
    fn give_food(&mut self, ctx: &Ctx, giver: PermanentId, to: PermanentId) {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let (Some(&gh), Some(&th)) = (self.hh_index.get(&giver), self.hh_index.get(&to)) else {
            return;
        };
        if gh == th {
            return;
        }
        let (Some(g), Some(t)) = (self.households.get(gh), self.households.get(th)) else {
            return;
        };
        let spare = spare_food_kcal(g, now, params, goods);
        let need = t.members.len() as f64 * params.household.daily_kcal_per_person;
        let (held, _) = food_kcal(&stores_now(t, now, params, goods), goods);
        let mut want = spare.min((params.household.food_target_days * need - held).max(0.0));
        if want <= 0.0 {
            return;
        }
        let mut given = vec![0.0; goods.len()];
        if let Some(g) = self.households.get_mut(gh) {
            let members = g.members.len().max(1);
            g.settle_stores(now, goods, &|d| fuel_per_day(params, members, d));
            g.stores.resize(goods.len(), 0.0);
            let mut order: Vec<usize> = (0..goods.len())
                .filter(|&i| {
                    goods[i].purpose == GoodUse::Food
                        && goods[i].kcal_per_kg > 0.0
                        && !goods[i].reserve
                })
                .collect();
            let keeps = |i: usize| match goods[i].half_life_days {
                h if h > 0.0 => h,
                _ => f64::INFINITY,
            };
            order.sort_by(|&a, &b| keeps(a).total_cmp(&keeps(b)).then(a.cmp(&b)));
            let mut carry = params.household.carry_kg;
            for i in order {
                if want <= 0.0 || carry <= 0.0 {
                    break;
                }
                let kg = (want / goods[i].kcal_per_kg)
                    .min(g.stores[i].max(0.0))
                    .min(carry);
                g.stores[i] -= kg;
                given[i] += kg;
                carry -= kg;
                want -= kg * goods[i].kcal_per_kg;
            }
        }
        if let Some(t) = self.households.get_mut(th) {
            let members = t.members.len().max(1);
            t.settle_stores(now, goods, &|d| fuel_per_day(params, members, d));
            t.stores.resize(goods.len(), 0.0);
            for (s, kg) in t.stores.iter_mut().zip(&given) {
                *s += kg;
            }
        }
    }

    /// Hours of a capable adult's field work `members` can give a day.
    fn labour_per_day(&self, members: &[PermanentId], now: SimTime, params: &PeopleParams) -> f64 {
        members
            .iter()
            .filter_map(|m| self.person(*m))
            .map(|q| interpolate(&params.capacity_by_age, q.age_years(now)))
            .sum::<f64>()
            * params.farm.work_hours_per_day
    }

    /// Applies `hours` of a capable adult's `task` on `field` by person `who` of `household`:
    /// seed comes out of the household's store, and threshed grain goes in, next season's seed
    /// first (the area it plans to sow at the crop's seed rate, with what the seed loses in store
    /// until sowing: research 08-01 §1.5, seed is kept by planned area, never as a share of the
    /// harvest).
    fn field_work(
        &mut self,
        ctx: &mut Ctx,
        who: PermanentId,
        household: PermanentId,
        field: PermanentId,
        task: FieldTask,
        hours: f64,
    ) {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let Some(fi) = ctx.land.fields.iter().position(|f| f.id == field) else {
            return;
        };
        let Some(crop) = ctx.catalog.crops.get(usize::from(ctx.land.fields[fi].crop)) else {
            return;
        };
        let Some(&hh) = self.hh_index.get(&household) else {
            return;
        };
        let area: f64 = ctx
            .land
            .fields
            .iter()
            .filter(|f| f.household == household)
            .map(Field::area_ha)
            .sum();
        // Seed for the coming season is set aside before any grain is eaten.
        let seed_wanted = {
            let Some(x) = self.households.get(hh) else {
                return;
            };
            let kcal = goods.get(crop.good).map_or(0.0, |g| g.kcal_per_kg);
            let need = farm::need_area_ha(x.members.len(), params, crop, kcal);
            let labour = self.labour_per_day(&x.members, now, params);
            let plan = farm::plan_area_ha(need, area, labour, crop);
            let half_life = goods.get(crop.seed_good).map_or(0.0, |g| g.half_life_days);
            farm::seed_to_keep(plan, crop, half_life, now.day_index())
        };
        let Some(x) = self.households.get_mut(hh) else {
            return;
        };
        let members = x.members.len().max(1);
        x.settle_stores(now, goods, &|d| fuel_per_day(params, members, d));
        x.stores.resize(goods.len(), 0.0);
        let seed = x
            .stores
            .get(crop.seed_good)
            .copied()
            .unwrap_or(0.0)
            .max(0.0);
        let climate = ctx.land.climate.factor;
        let done = ctx.land.fields[fi].work(crop, task, hours, now.day_index(), climate, seed, now);
        if let Some(s) = x.stores.get_mut(crop.seed_good) {
            *s = (seed - done.seed_kg).max(0.0);
        }
        if done.grain_kg > 0.0 {
            let have = x.stores.get(crop.seed_good).copied().unwrap_or(0.0);
            let to_seed = (seed_wanted - have).clamp(0.0, done.grain_kg);
            if let Some(s) = x.stores.get_mut(crop.seed_good) {
                *s += to_seed;
            }
            if let Some(g) = x.stores.get_mut(crop.good) {
                *g += done.grain_kg - to_seed;
            }
            // Threshing leaves the straw at home too.
            if let Some((good, kg_per_kg)) = crop.straw
                && let Some(s) = x.stores.get_mut(good)
            {
                *s += done.grain_kg * kg_per_kg;
            }
        }
        let settlement = x.settlement;
        let Some(si) = settlement.and_then(|s| ctx.land.settlements.iter().position(|x| x.id == s))
        else {
            return;
        };
        ctx.land.settlements[si].harvest_kg += done.grain_kg;
        let first = task == FieldTask::Sow
            && done.finished
            && !self
                .chronicle
                .iter()
                .any(|e| e.kind == ChronicleKind::FirstSowing && e.settlement == settlement);
        if first {
            let s = &ctx.land.settlements[si];
            let (name, place) = (s.name.clone(), ctx.land.fields[fi].rect.centre_m());
            self.chronicle_push(
                now,
                ChronicleKind::FirstSowing,
                vec![who],
                settlement,
                Some(place),
                0.0,
                name,
            );
        }
    }

    /// Applies `hours` of a capable adult's work by person `who` of `household` to its building
    /// `building`: the stage under way advances as far as the work and the materials in the
    /// household's store allow, and the materials it uses leave the store. When the roof goes
    /// on, the household's stores are under it from then on.
    fn build_work(
        &mut self,
        ctx: &mut Ctx,
        who: PermanentId,
        household: PermanentId,
        building: PermanentId,
        hours: f64,
    ) {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let Some(bi) = ctx
            .land
            .buildings
            .iter()
            .position(|b| b.id == building && b.household == household)
        else {
            return;
        };
        let Some(def) = ctx
            .catalog
            .building_index(&ctx.land.buildings[bi].spec.program)
            .and_then(|i| ctx.catalog.buildings.get(i))
        else {
            return;
        };
        let Some(needs) = self.needs_of(&ctx.land.buildings[bi], def) else {
            return;
        };
        let Some(&hd) = self.hh_index.get(&household) else {
            return;
        };
        let Some(x) = self.households.get_mut(hd) else {
            return;
        };
        let members = x.members.len().max(1);
        x.settle_stores(now, goods, &|d| fuel_per_day(params, members, d));
        x.stores.resize(goods.len(), 0.0);
        let work = HomeWork::of(&ctx.land.buildings[bi], needs);
        let Some(stage) = work.needs() else {
            return;
        };
        let held = work.held_by_slot(def, &x.stores);
        let done =
            ctx.land.buildings[bi].work(hours, stage.labour_h, &stage.materials_kg, &held, now);
        for (slot, kg) in done.used_kg.iter().enumerate() {
            if let Some(s) = def.materials.get(slot).and_then(|&g| x.stores.get_mut(g)) {
                *s = (*s - kg).max(0.0);
            }
        }
        if done.finished != Some(Stage::Roof) {
            return;
        }
        // Stores were settled to now above, so they spoil at the sheltered rates from here on.
        x.sheltered = true;
        let settlement = x.settlement;
        let first = !self
            .chronicle
            .iter()
            .any(|e| e.kind == ChronicleKind::FirstRoof && e.settlement == settlement);
        let name = settlement
            .and_then(|s| ctx.land.settlements.iter().find(|x| x.id == s))
            .map(|s| s.name.clone());
        if let (true, Some(name)) = (first, name) {
            let place = build::centre_m(&ctx.land.buildings[bi].spec);
            self.chronicle_push(
                now,
                ChronicleKind::FirstRoof,
                vec![who],
                settlement,
                Some(place),
                0.0,
                name,
            );
        }
    }

    fn eat(&mut self, ctx: &mut Ctx, h: Handle<Person>, household: PermanentId) {
        let now = ctx.now;
        let params = ctx.params;
        let Some(p) = self.people.get(h) else {
            return;
        };
        let day_kcal = bmr(p, now, params) * 1.6;
        let want =
            (-f64::from(p.energy_kcal)).max(0.0) + day_kcal / 24.0 * params.energy.satiety_hours;
        let reserve_ok = may_eat_reserve(p, now, params);
        let protected = farm::protected_seed(
            ctx.land.fields.iter().filter(|f| f.household == household),
            &ctx.catalog.crops,
        );
        let Some(&hh) = self.hh_index.get(&household) else {
            return;
        };
        let Some(hh) = self.households.get_mut(hh) else {
            return;
        };
        let goods = &ctx.catalog.goods;
        let members = hh.members.len().max(1);
        hh.settle_stores(now, goods, &|d| fuel_per_day(params, members, d));
        hh.stores.resize(goods.len(), 0.0);
        // The seed to sow the ground already cropped is set aside before the meal.
        let set_aside =
            protected.map(|(g, kg)| (g, hh.stores.get(g).copied().unwrap_or(0.0).clamp(0.0, kg)));
        if let Some((g, kg)) = set_aside {
            hh.stores[g] -= kg;
        }
        let take = eat_from(&mut hh.stores, goods, want, reserve_ok);
        if let Some((g, kg)) = set_aside {
            hh.stores[g] += kg;
        }
        let Some(p) = self.people.get_mut(h) else {
            return;
        };
        p.energy_kcal =
            (f64::from(p.energy_kcal) + take).min(params.energy.max_surplus_kcal) as f32;
        let full = if want > 0.0 {
            (take / want).clamp(0.0, 1.0)
        } else {
            1.0
        };
        p.satiety_until = now.plus_minutes((params.energy.satiety_hours * 60.0 * full) as i64);
    }

    fn deposit(&mut self, ctx: &mut Ctx, h: Handle<Person>) {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let Some(p) = self.people.get_mut(h) else {
            return;
        };
        let load: Load = std::mem::take(&mut p.carrying);
        let hh_id = p.household;
        let Some(&own) = self.hh_index.get(&hh_id) else {
            return;
        };
        let settlement = self.households.get(own).and_then(|x| x.settlement);
        if let Some(x) = self.households.get_mut(own) {
            let members = x.members.len().max(1) as f64;
            x.settle_water(now, members * params.household.water_l_per_person_day);
            x.water_l += f64::from(load.water_l);
        }
        let Some(g) = load.good.map(usize::from) else {
            return;
        };
        let Some(good) = goods.get(g) else {
            return;
        };
        if load.kg <= 0.0 {
            return;
        }
        // A shared good goes to every household of the settlement, by members.
        let recipients: Vec<(Handle<Household>, usize)> = match (good.shared, settlement) {
            (true, Some(s)) => self
                .households
                .iter()
                .filter(|(_, x)| x.settlement == Some(s) && !x.members.is_empty())
                .map(|(hd, x)| (hd, x.members.len()))
                .collect(),
            _ => vec![(own, 1)],
        };
        let total = recipients.iter().map(|r| r.1).sum::<usize>().max(1) as f64;
        for (hd, n) in recipients {
            if let Some(x) = self.households.get_mut(hd) {
                let members = x.members.len().max(1);
                x.settle_stores(now, goods, &|d| fuel_per_day(params, members, d));
                x.stores.resize(goods.len(), 0.0);
                x.stores[g] += f64::from(load.kg) * n as f64 / total;
            }
        }
    }

    /// Days of food in the stores of a settlement's households at its people's needs, at `now`;
    /// `None` if nobody lives there.
    pub fn settlement_food_days(
        &self,
        settlement: PermanentId,
        now: SimTime,
        params: &PeopleParams,
        goods: &[GoodDef],
    ) -> Option<f64> {
        let mut households: Vec<&Household> = self
            .households
            .iter()
            .map(|(_, x)| x)
            .filter(|x| x.settlement == Some(settlement) && !x.members.is_empty())
            .collect();
        // A fixed order, so the sum is the same however the table is laid out.
        households.sort_by_key(|x| x.id);
        let (mut kcal, mut members) = (0.0, 0usize);
        for x in households {
            kcal += food_kcal(&stores_now(x, now, params, goods), goods).0;
            members += x.members.len();
        }
        (members > 0)
            .then(|| kcal / (members as f64 * params.household.daily_kcal_per_person).max(1.0))
    }

    /// The end of a day: notes in the chronicle when a settlement's food runs short, and when it
    /// has enough again (with a gap between the two, so a store hovering near the line is not
    /// noted every day).
    pub fn on_day(&mut self, ctx: &mut Ctx) {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        // The season turns for every field: a sowing window passes, a crop left standing is lost.
        let day = now.day_index();
        for f in &mut ctx.land.fields {
            if let Some(crop) = ctx.catalog.crops.get(usize::from(f.crop)) {
                f.new_day(crop, day, now);
            }
        }
        // A settlement's harvest is in once none of its fields has a crop growing or unthreshed.
        for i in 0..ctx.land.settlements.len() {
            let s = &ctx.land.settlements[i];
            if s.harvest_kg <= 0.0 {
                continue;
            }
            let id = s.id;
            let busy = ctx.land.fields.iter().any(|f| {
                matches!(f.stage, FieldStage::Sown | FieldStage::Reaped)
                    && self.household(f.household).and_then(|h| h.settlement) == Some(id)
            });
            if busy {
                continue;
            }
            let s = &mut ctx.land.settlements[i];
            let (kg, name, place) = (s.harvest_kg, s.name.clone(), s.hearth_m);
            s.harvest_kg = 0.0;
            self.chronicle_push(
                now,
                ChronicleKind::HarvestIn,
                Vec::new(),
                Some(id),
                Some(place),
                kg,
                name,
            );
        }
        for i in 0..ctx.land.settlements.len() {
            let id = ctx.land.settlements[i].id;
            let Some(days) = self.settlement_food_days(id, now, params, goods) else {
                continue;
            };
            let s = &mut ctx.land.settlements[i];
            let Some(kind) = shortage_change(s.food_short, days, &params.household) else {
                continue;
            };
            s.food_short = kind == ChronicleKind::FoodRanShort;
            let (name, place) = (s.name.clone(), s.hearth_m);
            self.chronicle_push(now, kind, Vec::new(), Some(id), Some(place), days, name);
        }
    }
}

/// Whether a settlement's food has just run short, or just recovered, given whether it was short
/// and its days of food now. The two lines are apart, so a store hovering near one of them is not
/// noted every day.
pub fn shortage_change(short: bool, days: f64, h: &HouseholdParams) -> Option<ChronicleKind> {
    if !short && days < h.short_food_days {
        Some(ChronicleKind::FoodRanShort)
    } else if short && days > h.recovered_food_days {
        Some(ChronicleKind::FoodRecovered)
    } else {
        None
    }
}

/// Takes up to `want` kcal from food stores, the most perishable first and goods kept back (seed)
/// last of all and only when `reserve` allows, and only what needs no fire when there is no
/// firewood. Returns the kcal taken.
pub fn eat_from(stores: &mut [f64], goods: &[GoodDef], want: f64, reserve: bool) -> f64 {
    let fire = fuel_kg(stores, goods) > 0.0;
    let keeps = |g: &GoodDef| {
        if g.half_life_days > 0.0 {
            g.half_life_days
        } else {
            f64::INFINITY
        }
    };
    let mut order: Vec<usize> = (0..goods.len().min(stores.len()))
        .filter(|&g| {
            let good = &goods[g];
            good.purpose == crate::params::GoodUse::Food
                && good.kcal_per_kg > 0.0
                && (fire || !good.cooked)
                && (reserve || !good.reserve)
        })
        .collect();
    order.sort_by(|&a, &b| {
        let (ga, gb) = (&goods[a], &goods[b]);
        ga.reserve
            .cmp(&gb.reserve)
            .then(keeps(ga).total_cmp(&keeps(gb)))
            .then(a.cmp(&b))
    });
    let mut taken = 0.0;
    for g in order {
        let need = want - taken;
        if need <= 0.0 {
            break;
        }
        let have = stores[g].max(0.0) * goods[g].kcal_per_kg;
        let take = have.min(need);
        stores[g] -= take / goods[g].kcal_per_kg;
        taken += take;
    }
    taken
}

/// Lays out the aligned blocks of side `2·range + 1` patches and the cell people walk to in each.
fn lay_out_places(ctx: &Ctx, range: u32, in_water: bool) -> Places {
    let map = ctx.map;
    let patches = &ctx.land.patches;
    let (pc, side) = (patches.patch_cells, 2 * range + 1);
    let (w, h) = (i64::from(map.width), i64::from(map.height));
    let by_water = |i: usize| {
        let (x, y) = ((i as i64) % w, (i as i64) / w);
        civ_world::grid::D8.iter().any(|&(dx, dy)| {
            let (nx, ny) = (x + i64::from(dx), y + i64::from(dy));
            nx >= 0
                && ny >= 0
                && nx < w
                && ny < h
                && map.water[(ny * w + nx) as usize] != civ_world::WATER_LAND
        })
    };
    let (cols, rows) = (patches.cols.div_ceil(side), patches.rows.div_ceil(side));
    let mut blocks = Vec::with_capacity((cols * rows) as usize);
    for by in 0..rows {
        for bx in 0..cols {
            let cpx = (bx * side + range).min(patches.cols - 1);
            let cpy = (by * side + range).min(patches.rows - 1);
            let centre = cpy * patches.cols + cpx;
            let (cx, cy) = (cpx * pc + pc / 2, cpy * pc + pc / 2);
            let mut best: Option<(u32, u32)> = None;
            for y in by * side * pc..((by + 1) * side * pc).min(map.height) {
                for x in bx * side * pc..((bx + 1) * side * pc).min(map.width) {
                    let i = y as usize * map.width as usize + x as usize;
                    if map.water[i] != civ_world::WATER_LAND || !ctx.nav.walkable(i) {
                        continue;
                    }
                    let d = x.abs_diff(cx).pow(2) + y.abs_diff(cy).pow(2);
                    if best.is_none_or(|(bd, _)| d < bd) && (!in_water || by_water(i)) {
                        best = Some((d, i as u32));
                    }
                }
            }
            blocks.push(best.map(|(_, cell)| (centre, cell)));
        }
    }
    Places {
        range,
        in_water,
        blocks,
    }
}

/// The reachable cell closest in walking time that is next to fresh water (river or lake, not
/// the sea), with its walking time in seconds.
fn nearest_water(map: &WorldMap, field: &TravelField) -> Option<(u32, f32)> {
    let (w, h) = (map.width as i64, map.height as i64);
    let mut best: Option<(u32, f32)> = None;
    for (i, s) in field.iter() {
        let better = best.is_none_or(|(bc, bs)| s < bs || (s == bs && (i as u32) < bc));
        if !better {
            continue;
        }
        let (x, y) = ((i as i64) % w, (i as i64) / w);
        let next_to_water = map.water[i] == WATER_RIVER
            || civ_world::grid::D8.iter().any(|&(dx, dy)| {
                let (nx, ny) = (x + i64::from(dx), y + i64::from(dy));
                nx >= 0
                    && ny >= 0
                    && nx < w
                    && ny < h
                    && matches!(map.water[(ny * w + nx) as usize], WATER_RIVER | WATER_LAKE)
            });
        if next_to_water {
            best = Some((i as u32, s));
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::GoodUse;

    fn household() -> HouseholdParams {
        HouseholdParams {
            water_l_per_person_day: 20.0,
            carry_water_l: 15.0,
            water_target_days: 1.5,
            food_target_days: 5.0,
            carry_kg: 20.0,
            fuel_kg_per_person_day: [1.5; 12],
            fuel_target_days: 3.0,
            short_food_days: 2.0,
            recovered_food_days: 5.0,
            daily_kcal_per_person: 2100.0,
        }
    }

    #[test]
    fn shortages_are_noted_once_until_food_has_clearly_recovered() {
        let h = household();
        let mut short = false;
        let mut notes = Vec::new();
        for days in [9.0, 3.0, 1.9, 1.0, 3.0, 4.9, 1.5, 5.1, 6.0, 1.9] {
            if let Some(kind) = shortage_change(short, days, &h) {
                short = kind == ChronicleKind::FoodRanShort;
                notes.push((days, kind));
            }
        }
        assert_eq!(
            notes,
            vec![
                (1.9, ChronicleKind::FoodRanShort),
                (5.1, ChronicleKind::FoodRecovered),
                (1.9, ChronicleKind::FoodRanShort),
            ]
        );
    }

    fn goods() -> Vec<GoodDef> {
        let good = |id: &str, purpose, kcal, half, cooked| GoodDef {
            id: id.into(),
            name: id.into(),
            purpose,
            kcal_per_kg: kcal,
            half_life_days: half,
            cooked,
            shared: false,
            reserve: false,
            sheltered_half_life_days: 0.0,
        };
        vec![
            good("grain", GoodUse::Food, 3000.0, 1000.0, false),
            good("meat", GoodUse::Food, 1500.0, 3.0, true),
            good("berries", GoodUse::Food, 600.0, 5.0, false),
            good("wood", GoodUse::Fuel, 0.0, 0.0, false),
        ]
    }

    #[test]
    fn meals_come_from_the_most_perishable_food_and_cooked_food_needs_firewood() {
        let goods = goods();
        // Meat spoils fastest, then berries; grain keeps.
        let mut stores = vec![10.0, 2.0, 1.0, 5.0];
        let taken = eat_from(&mut stores, &goods, 3600.0, false);
        assert_eq!(taken, 3600.0);
        assert!((stores[1] - 0.0).abs() < 1e-12, "meat first: {stores:?}");
        assert!((stores[2] - 0.0).abs() < 1e-12, "then berries: {stores:?}");
        assert!((stores[0] - (10.0 - 0.0)).abs() < 1e-12, "grain untouched");
        // Without firewood the meat cannot be eaten.
        let mut cold = vec![1.0, 2.0, 0.0, 0.0];
        let taken = eat_from(&mut cold, &goods, 100_000.0, false);
        assert_eq!(taken, 3000.0, "only the grain");
        assert_eq!(cold[1], 2.0);
    }
}
