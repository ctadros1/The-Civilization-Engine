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
use civ_land::paths::cells_along;
use civ_land::{
    Building, CropParams, Field, FieldStage, FieldTask, Land, LandParams, Party, Plot, PlotUse,
};
use civ_world::nav::{NavGrid, RouteResult, TravelField};
use civ_world::{WATER_LAKE, WATER_LAND, WATER_RIVER, WorldMap};

use crate::build::{self, HomeWork};
use crate::decide::{
    self, BuildOption, Facts, FieldOption, GiverOption, Limits, MakeOption, MakeWorth, PatchOption,
    WaterOption,
};
use crate::farm::{self, FarmView, Site};
use crate::history::{ChronicleEvent, ChronicleKind, PersonRecord, Reason, Receipt, Union};
use crate::ledger::{Channel, Leg};
use crate::make;
use crate::needs;
use crate::params::{
    ActivityDef, Behavior, Catalog, GoodDef, GoodUse, HouseholdParams, LandHolder, PeopleParams,
    RegimeDef, interpolate,
};
use crate::person::{
    Activity, Flow, Flows, Household, KnownPatch, Load, Person, Step, Target, Trip, food_kcal,
    fuel_kg, reserve_food_kcal, stock_kcal,
};

mod firm;
mod knowledge;
mod land;
mod life;
mod market;
mod transfer;

use knowledge::Gate;
pub use knowledge::{LOST_AWARE, LOST_MADE_REMAIN};

pub use life::{depleted, extra_kcal_day};

/// Purpose tag for decision draws (ADR-0003 keyed randomness).
pub const PURPOSE_DECIDE: u64 = 0x6465_6369_6465_3031; // "decide01"
/// Most cells one route search may expand.
const ROUTE_BUDGET: usize = 600_000;
/// Days a settlement's travel-time field is reused before it is recomputed. It is walked out again
/// sooner when the paths are surveyed, since walking costs change then.
const FIELD_REFRESH_DAYS: i64 = 30;
/// Minutes a person waits before deciding again when a walk cannot be routed.
const WAIT_AFTER_FAILURE_MIN: u32 = 10;
/// Most routes kept in the route cache before it is emptied.
const ROUTE_CACHE_MAX: usize = 50_000;
/// Farthest a household moves its home from where it stands to find clear ground to build on,
/// metres (a tuning value).
const HOME_SHIFT_M: f64 = 30.0;
/// Least food a household makes ready in one go, in days of its needs: below this it waits until
/// it needs more (a tuning value).
const MIN_BATCH_DAYS: f64 = 0.25;
/// A settlement's first trail, for the chronicle: at least this long, metres...
pub const FIRST_TRAIL_M: f32 = 150.0;
/// ...and passing within this of the settlement's hearth, metres.
pub const FIRST_TRAIL_NEAR_M: f32 = 80.0;

/// A route at standard walking speed, pulled straight across open ground: vertices (cell centres)
/// and seconds to each.
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
    /// The world's property regime (ADR-0007).
    pub regime: &'a RegimeDef,
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
    /// The paths survey it was walked out on ([`civ_land::Wear::rev`]).
    rev: u32,
    water: Option<(u32, f32)>,
    /// Per kind of place (the index in `Population::places`), blocks by walking time to their
    /// cell, seconds.
    places: Vec<Vec<(u32, f32)>>,
    /// Walking times to every cell in reach, for fields.
    reach: TravelField,
    /// The cells in reach where new ground could be broken, in cell order ([`farm::find_site`]).
    breakable: Vec<(u32, f32)>,
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
    /// Every couple there has been, oldest first (research 04-08 §5.1).
    pub unions: Vec<Union>,
    /// The last trip id handed out.
    pub next_trip: u64,
    index: HashMap<PermanentId, Handle<Person>>,
    hh_index: HashMap<PermanentId, Handle<Household>>,
    homes: HashMap<PermanentId, HomeField>,
    /// The kinds of place gathering trips go to.
    places: Vec<Places>,
    /// Routes by (from cell, to cell); `None` when there is none. Derived: kept until the paths
    /// are next surveyed, when walking costs change.
    routes: HashMap<(u32, u32), Option<CachedRoute>>,
    /// The paths survey the routes were planned on ([`civ_land::Wear::rev`]).
    routes_rev: u32,
    /// Per activity, what a trip is expected to bring from each patch today.
    priors: Vec<Prior>,
    /// Per household, the new ground it would mark out for a field, as found on a day.
    sites: HashMap<PermanentId, (i64, Option<Site>)>,
    /// Per household without a home under way, the hut it would build and where, as found on a
    /// day.
    home_sites: HashMap<PermanentId, (i64, Option<NewHome>)>,
    /// Per building, what each of its stages needs (its design never changes). Derived.
    stage_needs: HashMap<PermanentId, Vec<StageNeeds>>,
    /// Households whose last member died today, with that member (within a day's step only).
    emptied: Vec<(PermanentId, PermanentId)>,
    /// What became of the goods of households that are no more (counters, not saved).
    pub flows_gone: Flows,
    /// What moved between households, by channel (counters, not saved).
    pub transfers: crate::ledger::Transfers,
    /// Each settlement's market (slice I).
    pub markets: Vec<crate::market::Market>,
    /// Every firm there has been, open and closed, in the order they were founded (slice J).
    pub firms: Vec<crate::firm::Firm>,
    /// Each settlement's wealth measures at the end of each year, oldest first (ADR-0007 §4).
    pub wealth_years: Vec<crate::wealth::WealthYear>,
    /// Each settlement's record of the techniques it came to know and lost, oldest first
    /// (ADR-0008 §2).
    pub knowledge: Vec<crate::knowledge::KnowledgeEvent>,
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

/// Distance from `p` to the segment `a`–`b`, metres.
fn segment_distance(p: (f32, f32), a: (f32, f32), b: (f32, f32)) -> f32 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let len2 = dx * dx + dy * dy;
    let t = if len2 > 0.0 {
        (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / len2).clamp(0.0, 1.0)
    } else {
        0.0
    };
    ((p.0 - a.0 - t * dx).powi(2) + (p.1 - a.1 - t * dy).powi(2)).sqrt()
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

/// Least food, in days of one person's need, that must be about to spoil for a household to keep
/// it by a recipe that preserves (a tuning value).
pub const MIN_PRESERVE_DAYS: f64 = 0.5;

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

/// Goods not accounted for between two readings of the books, each `(held, flows)`: `(good,
/// amount)` wherever what households hold changed by other than what entered less what left
/// (ADR-0006 §3: goods are conserved; moving between households is neither). Empty when the
/// books balance, to rounding. `goods` is the number of goods.
pub fn unaccounted(
    goods: usize,
    before: (&[f64], &Flows),
    after: (&[f64], &Flows),
) -> Vec<(usize, f64)> {
    let held = |v: &[f64], g: usize| v.get(g).copied().unwrap_or(0.0);
    (0..goods)
        .filter_map(|g| {
            let change = held(after.0, g) - held(before.0, g);
            let gap = change - (after.1.net(g) - before.1.net(g));
            let scale = after.1.turnover(g)
                + before.1.turnover(g)
                + held(after.0, g).abs()
                + held(before.0, g).abs();
            (gap.abs() > 1e-9 * scale + 1e-6).then_some((g, gap))
        })
        .collect()
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

/// Energy a person spends a minute at physical activity level `par`, kcal: their basal metabolism
/// times the level, and what a pregnancy costs (research 05-02 §2.4).
pub fn burn_rate(p: &Person, now: SimTime, params: &PeopleParams, par: f64) -> f32 {
    ((bmr(p, now, params) * par + extra_kcal_day(p, now, params)) / 1440.0) as f32
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
/// household in need (food, ready or not, above twice the days of food it tries to keep).
pub fn spare_food_kcal(
    h: &Household,
    now: SimTime,
    params: &PeopleParams,
    goods: &[GoodDef],
) -> f64 {
    let stores = stores_now(h, now, params, goods);
    let food = stock_kcal(&stores, goods);
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

    /// Gives people and households of a world saved before tools and skills (save schema 8 and
    /// earlier, ADR-0006 §6) what a founder brings: every household the tools its adults want,
    /// and every person skills drawn as a founder's are, keyed by the world seed and their id.
    pub fn give_founders_kit(
        &mut self,
        catalog: &Catalog,
        params: &PeopleParams,
        seed: u64,
        now: SimTime,
    ) {
        const PURPOSE_KIT: u64 = 0x6b69_7430_3030_3031; // "kit00001"
        let ages: HashMap<PermanentId, f64> = self
            .people
            .iter()
            .map(|(_, p)| (p.id, p.age_years(now)))
            .collect();
        for (_, h) in self.households.iter_mut() {
            let members: Vec<f64> = h
                .members
                .iter()
                .filter_map(|m| ages.get(m).copied())
                .collect();
            h.stores.resize(catalog.goods.len(), 0.0);
            // The tools for the work founders know (ADR-0008 §1).
            let founders_know = |t: usize| {
                params
                    .knowledge
                    .founders
                    .iter()
                    .any(|&(x, s)| x == t && s > 0.0)
            };
            let wants = make::tool_wants_for(
                catalog,
                &members,
                params.family.independent_age,
                &founders_know,
            );
            for (g, want) in wants.into_iter().enumerate() {
                h.stores[g] += want;
                h.flows.add(Flow::Brought, g, want);
            }
        }
        for (_, p) in self.people.iter_mut() {
            let mut rng = Rng64::from_key(&[seed, PURPOSE_KIT, p.id.get()]);
            p.skills = crate::found::founder_skills(
                &catalog.skills,
                p.age_years(now),
                params.family.independent_age,
                &mut rng,
            );
        }
    }

    /// What became of every household's and firm's goods since the counters began, those that
    /// are no more included.
    pub fn flows(&self) -> Flows {
        let mut all = self.flows_gone.clone();
        for (_, h) in self.households.iter() {
            all.absorb(&h.flows);
        }
        for f in &self.firms {
            all.absorb(&f.flows);
        }
        all
    }

    /// What all households and firms hold, by good, as each one's stores were last brought up to
    /// date: the balance that [`Population::flows`] accounts for (ADR-0006 §3).
    pub fn goods_held(&self) -> Vec<f64> {
        let mut out: Vec<f64> = Vec::new();
        let stores = self
            .households
            .iter()
            .map(|(_, h)| &h.stores)
            .chain(self.firms.iter().map(|f| &f.stores));
        for s in stores {
            if out.len() < s.len() {
                out.resize(s.len(), 0.0);
            }
            for (o, kg) in out.iter_mut().zip(s) {
                *o += kg;
            }
        }
        out
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
                Some(r) if r.died.is_none() && r.left.is_none() => {}
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
        let mut firm_ids = std::collections::HashSet::new();
        for f in &self.firms {
            if !firm_ids.insert(f.id) || hh_ids.contains(&f.id) || f.id.get() >= next_id {
                out.push(format!("firm {} has a duplicate or unallocated id", f.id));
            }
            // A firm closed when its household was no more may name it; an open one may not.
            if f.is_open() && self.household(f.owner).is_none() {
                out.push(format!(
                    "firm {} is open but its owner {} is not a household",
                    f.id, f.owner
                ));
            }
            if !f.stores.iter().all(|v| v.is_finite())
                || (!f.is_open() && f.stores.iter().any(|&v| v != 0.0))
            {
                out.push(format!(
                    "firm {} has a store that is not a number or outlived it",
                    f.id
                ));
            }
        }
        for (id, r) in &self.records {
            if *id != r.id || id.get() >= next_id {
                out.push(format!("record {id} is filed under the wrong id"));
            }
            if r.died.is_none() && r.left.is_none() && self.person(*id).is_none() {
                out.push(format!("record {id} is alive but the person is missing"));
            }
        }
        for (i, e) in self.chronicle.iter().enumerate() {
            if e.seq != i as u64 + 1 {
                out.push(format!("chronicle entry {} is out of sequence", e.seq));
                break;
            }
        }
        for (_, p) in self.people.iter() {
            if let Some(q) = p.partner
                && self
                    .person(q)
                    .is_none_or(|x| x.partner != Some(p.id) || x.sex == p.sex)
            {
                out.push(format!(
                    "person {}'s partner {q} is not partnered with them",
                    p.id
                ));
            }
            if p.sex == crate::needs::Sex::Male && p.repro != crate::person::Repro::Open {
                out.push(format!("person {} is a man with a pregnancy", p.id));
            }
            if let crate::person::Repro::Pregnant { conceived, due, .. } = p.repro
                && due <= conceived
            {
                out.push(format!("person {}'s pregnancy ends before it began", p.id));
            }
            if !(p.fecundity.is_finite() && p.fecundity >= 0.0) {
                out.push(format!(
                    "person {} has a fecundity that is not a number",
                    p.id
                ));
            }
            if p.nursing.is_some_and(|c| !self.records.contains_key(&c)) {
                out.push(format!("person {} nurses a child nobody knows", p.id));
            }
        }
        for u in &self.unions {
            if !(self.records.contains_key(&u.woman) && self.records.contains_key(&u.man)) {
                out.push(format!(
                    "a union of {} and {} names nobody known",
                    u.woman, u.man
                ));
            } else if u.ended.is_none()
                && (self.person(u.woman).and_then(|p| p.partner) != Some(u.man)
                    || self.person(u.man).and_then(|p| p.partner) != Some(u.woman))
            {
                out.push(format!(
                    "the union of {} and {} has not ended but they are not partners",
                    u.woman, u.man
                ));
            }
        }
        out
    }

    /// Adds a chronicle entry about firm `firm`.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn chronicle_push_firm(
        &mut self,
        at: SimTime,
        kind: ChronicleKind,
        people: Vec<PermanentId>,
        settlement: Option<PermanentId>,
        place: Option<(f32, f32)>,
        number: f64,
        name: String,
        firm: PermanentId,
    ) {
        self.chronicle_push(at, kind, people, settlement, place, number, name);
        if let Some(e) = self.chronicle.last_mut() {
            e.firm = Some(firm);
        }
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
            firm: None,
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
        let rev = ctx.land.wear.rev();
        if let Some(f) = self.homes.get(&key)
            && f.cell == cell
            && f.rev == rev
            && day - f.day < FIELD_REFRESH_DAYS
        {
            return;
        }
        self.ensure_places(ctx);
        let reach = Self::field_reach_seconds(ctx.catalog);
        let wear = &ctx.land.wear;
        let field = ctx
            .nav
            .travel_field(&ctx.map.elevation, cell, reach, &|c| wear.factor(c));
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
        let breakable = farm::breakable_in_reach(
            ctx.map,
            ctx.nav,
            ctx.land,
            ctx.land_params,
            &field,
            ctx.params,
        );
        self.homes.insert(
            key,
            HomeField {
                cell,
                day,
                rev,
                water,
                places,
                reach: field,
                breakable,
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
        let breakable = &self.homes.get(&field_key)?.breakable;
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
            breakable,
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
    /// ground within reach of the hearth). A first home is as large as its members need and
    /// what it puts in of its means makes it ([`build::radius_within`]); a household that has a
    /// home builds a new one only when its means pay for all of one markedly larger
    /// ([`build::rebuild_radius`]), beside the old. Its means are [`Population::home_means`], for
    /// a household giving `labour_per_day`. `Built` while it has a finished home and builds no
    /// other; `NoPlace` when there is no program to build to or no clear ground.
    fn home_plan(
        &mut self,
        ctx: &Ctx,
        hh: &Household,
        hearth: Option<(f32, f32)>,
        field_key: PermanentId,
        labour_per_day: f64,
    ) -> Result<HomePlan, Reason> {
        let catalog = ctx.catalog;
        // The radius of the finished home it lives in, if it has one.
        let mut home: Option<i32> = None;
        for b in ctx.land.buildings.iter().filter(|b| b.household == hh.id) {
            if b.finished() {
                let civ_grammar::Footprint::Round { radius, .. } = b.spec.footprint;
                home = Some(home.map_or(radius, |r| r.max(radius)));
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
        let def = ctx.params.home_program;
        let Some(program) = catalog.buildings.get(def) else {
            return Err(if home.is_some() {
                Reason::Built
            } else {
                Reason::NoPlace
            });
        };
        let day = ctx.now.day_index();
        let found = match self.home_sites.get(&hh.id) {
            Some((seen, site)) if *seen == day => site.clone(),
            _ => {
                let (budget_h, time_h) = self.home_means(ctx, hh, program, labour_per_day, day);
                let (residents, carry) = (hh.members.len().max(1), ctx.params.household.carry_kg);
                let goods = &catalog.goods;
                let radius = match home {
                    None => Some(build::radius_within(
                        program, goods, residents, carry, budget_h, time_h,
                    )),
                    Some(current) => build::rebuild_radius(
                        program, goods, residents, current, carry, budget_h, time_h,
                    ),
                };
                let reach = self.homes.get(&field_key).map(|f| &f.reach);
                let site = radius.and_then(|radius| {
                    let design_at = |at: (f32, f32)| {
                        let reachable =
                            reach.is_none_or(|r| r.seconds_to(cell_of(ctx.map, at)).is_some());
                        reachable.then(|| build::design(program, goods, radius, at, hearth))
                    };
                    build::home_site(
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
                    })
                });
                self.home_sites.insert(hh.id, (day, site.clone()));
                site
            }
        };
        let Some(site) = found else {
            return Err(if home.is_some() {
                Reason::Built
            } else {
                Reason::NoPlace
            });
        };
        Ok(HomePlan {
            building: None,
            spec: site.spec,
            work: HomeWork::begin(site.stages),
            def,
            deadline: build::roof_deadline(day, program.roof_by_day),
        })
    }

    /// What household `hh` can put into a home on day `day`, hours of work: what it puts in
    /// ([`build::HOUSE_INVESTMENT_SHARE`]) of the goods it could spare, as it would offer them
    /// for sale, beyond what it keeps (food to see it to its next harvest, its tools, and until
    /// its home is finished what it builds with; never its seed or firewood), valued in hours of
    /// its own work; and the time a home may take in all, [`build::HOUSE_TIME_SHARE`] of what a
    /// household giving `labour_per_day` can spare for building before its roof deadline. It
    /// reckons without founders' provisions, what carries a band to dependable harvests (research
    /// 05-06 §5.4) and stands for the herds real colonists drove in: they are not to spend, and
    /// what it keeps must come from the rest.
    fn home_means(
        &self,
        ctx: &Ctx,
        hh: &Household,
        program: &crate::params::BuildingDef,
        labour_per_day: f64,
        day: i64,
    ) -> (f64, f64) {
        let goods = &ctx.catalog.goods;
        let mut stores = stores_now(hh, ctx.now, ctx.params, goods);
        // Reckoned without its provisions, which neither count nor cover what it keeps.
        if let Some(p) = stores.get_mut(ctx.params.band.provisions_good) {
            *p = 0.0;
        }
        let costs = self.own_costs_of(ctx, hh);
        let keep = self.holding_of(ctx, hh, &stores).keep;
        let spare_h: f64 = stores
            .iter()
            .enumerate()
            .filter(|&(g, _)| goods.get(g).is_some_and(market::can_offer))
            .map(|(g, &held)| {
                let spare = held.max(0.0) - keep.get(g).copied().unwrap_or(0.0);
                let cost = costs.get(g).copied().flatten().unwrap_or(0.0);
                if spare > 0.0 && cost > 0.0 {
                    spare * cost
                } else {
                    0.0
                }
            })
            .sum();
        let days = (build::roof_deadline(day, program.roof_by_day) - day).max(1) as f64;
        let time_h = labour_per_day * build::BUILD_LABOUR_SHARE * days * build::HOUSE_TIME_SHARE;
        (build::HOUSE_INVESTMENT_SHARE * spare_h, time_h)
    }

    /// Household `household` claims the ground for the hut it planned today and begins it: the
    /// plot is marked out, the building is begun, and the household's home moves to it (a
    /// household that has a home moves when the new one's roof is on). `None` if the ground is no
    /// longer clear.
    fn begin_home(&mut self, ctx: &mut Ctx, household: PermanentId) -> Option<PermanentId> {
        let (_, site) = self.home_sites.remove(&household)?;
        let site = site?;
        let def = ctx.catalog.buildings.get(ctx.params.home_program)?;
        let rect = build::plot_rect(&site.spec, def);
        if !build::plot_clear(ctx.land, ctx.map, ctx.nav, &rect) {
            return None;
        }
        // A household building a new home lives in the old until the new one's roof is on.
        let housed = ctx
            .land
            .buildings
            .iter()
            .any(|b| b.household == household && b.finished());
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
        if !housed
            && let Some(&hd) = self.hh_index.get(&household)
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
        // Food ready to eat (bread, meat, berries), and all the household has to live on (grain
        // and flour count, made ready by a recipe).
        let (food_all, food_raw) = food_kcal(&stores, goods);
        // Seed is food only to someone in real hunger, once ground, and never the seed to sow
        // the ground already cropped.
        let reserve_ok = self
            .people
            .get(h)
            .is_some_and(|p| may_eat_reserve(p, now, params));
        let protected = farm::protected_seed(
            ctx.land.fields.iter().filter(|f| f.household == hh_id),
            &ctx.catalog.crops,
        );
        let seed_kcal = if reserve_ok {
            let mut spare = stores.clone();
            if let Some((g, kg)) = protected
                && let Some(s) = spare.get_mut(g)
            {
                *s = (*s - kg).max(0.0);
            }
            reserve_food_kcal(&spare, goods)
        } else {
            0.0
        };
        let stock = stock_kcal(&stores, goods) + seed_kcal;
        let fuel = fuel_kg(&stores, goods);
        let edible = if fuel > 0.0 { food_all } else { food_raw };
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
        // The food that would see the household through to its next harvest, with a margin
        // (research 08-01 §2.3: a seasonal reserve until the next reliable food, plus 0-90 days).
        let food_outlook_days = ctx.catalog.crops.get(params.farm.crop).map_or(0.0, |c| {
            farm::days_to_harvest(
                c,
                ctx.land.fields.iter().filter(|f| f.household == hh_id),
                now.day_index(),
            ) + params.household.harvest_margin_days
        });
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
            food_days: stock / kcal_day.max(1.0),
            food_target_days: params.household.food_target_days,
            food_outlook_days,
            water_days: water_l / (members * params.household.water_l_per_person_day).max(1e-6),
            water_target_days: params.household.water_target_days,
            household_kcal_day: kcal_day,
            has_food: edible > 1.0,
            food_needs_fire: food_all > 1.0 && edible <= 1.0,
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
        let plan = self.home_plan(ctx, &hh, hearth, field_key, labour_per_day);
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
        // Tools: what the household wants for its working members, what of it is free (not in
        // the hands of members at work), and what the tools it lacks are made of.
        let catalog = ctx.catalog;
        let ages: Vec<f64> = hh
            .members
            .iter()
            .filter_map(|m| self.person(*m))
            .map(|q| q.age_years(now))
            .collect();
        let knows = |t: usize| self.household_knows(&hh.members, t);
        let wants = make::tool_wants_for(catalog, &ages, params.family.independent_age, &knows);
        let me = self.people.get(h).map(|p| p.id);
        let mut free_tools: Vec<f64> = stores
            .iter()
            .zip(goods)
            .map(|(s, g)| if g.tool.is_some() { s.max(0.0) } else { 0.0 })
            .collect();
        for m in &hh.members {
            let Some(q) = self.person(*m).filter(|q| Some(q.id) != me) else {
                continue;
            };
            let at_work = matches!(
                q.act.steps.get(q.act.step as usize),
                Some(Step::Walk { .. } | Step::Work { .. })
            );
            if let (true, Some(def)) = (at_work, catalog.activities.get(usize::from(q.act.def))) {
                for t in catalog.tools_of(def) {
                    if let Some(f) = free_tools.get_mut(t) {
                        *f -= 1.0;
                    }
                }
            }
        }
        let held_of = |g: usize| stores.get(g).copied().unwrap_or(0.0).max(0.0);
        let tool_need: Vec<f64> = (0..goods.len())
            .map(|g| make::tool_need(wants[g], held_of(g)))
            .collect();
        // Tools others want and nobody offers, which the household makes for fewer hours than
        // they would give: worth making to sell (slice I).
        let for_sale = self.for_sale(ctx, &hh, &stores, &tool_need);
        let mut tool_material = vec![0.0; goods.len()];
        let mut tool_material_blocked = vec![false; goods.len()];
        for (t, need) in tool_need.iter().enumerate() {
            if *need <= 0.0 && for_sale.get(t).copied().unwrap_or(0.0) <= 0.0 {
                continue;
            }
            let Some(r) = catalog
                .recipes
                .iter()
                .find(|r| r.outputs.iter().any(|(g, _)| *g == t))
            else {
                continue;
            };
            for &(g, amount) in &r.inputs {
                if let Some(m) = tool_material.get_mut(g) {
                    *m += (amount - held_of(g)).max(0.0);
                }
                if held_of(t) < decide::MIN_TOOL
                    && let Some(b) = tool_material_blocked.get_mut(g)
                {
                    *b = true;
                }
            }
        }
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
        let (places, priors) = (&self.places, &self.priors);
        let place_kinds: Vec<Option<usize>> = catalog
            .activities
            .iter()
            .map(|a| a.resource.and_then(|r| self.places_for(land_params, r)))
            .collect();
        // What the settlement knows of each resource, by place (the last of any repeats wins).
        let memory: civ_core::FastMap<(u16, u32), &KnownPatch> = hh
            .known
            .iter()
            .map(|k| ((k.resource, k.patch), k))
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
                GoodUse::Material | GoodUse::Tool => 0.0,
            };
            // A material is worth bringing only toward what the household is building, or the
            // tools it lacks.
            let (need_kg, urgency) = match good.purpose {
                GoodUse::Material | GoodUse::Tool => (
                    build_need.get(res.good).copied().unwrap_or(0.0),
                    build_urgency,
                ),
                GoodUse::Food | GoodUse::Fuel => (0.0, 0.0),
            };
            let tool_need_kg = tool_material.get(res.good).copied().unwrap_or(0.0);
            let tool_blocked = tool_material_blocked
                .get(res.good)
                .copied()
                .unwrap_or(false);
            let kind = place_kinds.get(def).copied().flatten()?;
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
                let rate = match memory.get(&(r as u16, patch)) {
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
                            tool_need_kg,
                            tool_blocked,
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
        // Whom to ask, what to buy and where to work are each worked out only if an option
        // needs them (they cost a search of the settlement), and then once.
        let giver_once = std::cell::OnceCell::new();
        let giver =
            || *giver_once.get_or_init(|| self.best_giver(ctx, &hh, field_key, kcal_day, stock));
        // A purchase that costs the household fewer hours than getting the good itself.
        let trade_once = std::cell::OnceCell::new();
        let trade = || *trade_once.get_or_init(|| self.best_purchase(ctx, &hh, &stores, reach));
        // Paid work at a workshop of another household (slice J).
        let session_min = catalog
            .activities
            .iter()
            .filter(|a| a.behavior == Behavior::Hire)
            .map(|a| f64::from(a.max_minutes))
            .fold(0.0, f64::max);
        let job_once = std::cell::OnceCell::new();
        let job = || {
            *job_once.get_or_init(|| {
                if session_min > 0.0 {
                    self.best_job(ctx, &hh, &stores, reach, session_min)
                } else {
                    None
                }
            })
        };
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
        // Recipes: how much the person could make at home this session, and what it is worth.
        let person = self.people.get(h);
        let capacity = facts.capacity;
        let ready_target = params.household.ready_food_days.max(1e-6);
        let processed_target = params.household.processed_food_days.max(1e-6);
        let best_make = |def: usize,
                         _blocked: &[usize]|
         -> Result<MakeOption, (Reason, Option<usize>)> {
            let a = &catalog.activities[def];
            let r = a
                .recipe
                .and_then(|r| catalog.recipes.get(r))
                .ok_or((Reason::NoPlace, None))?;
            let &(out, _) = r.outputs.first().ok_or((Reason::NotNeeded, None))?;
            let out_good = goods.get(out).ok_or((Reason::NotNeeded, None))?;
            if trade().is_some_and(|t| t.good == out) {
                return Err((Reason::Cheaper, None));
            }
            let level = match (r.skill, person) {
                (Some(k), Some(p)) => p.skill(k),
                _ => 0.0,
            };
            let speed = r
                .skill
                .and_then(|k| catalog.skills.get(k))
                .map_or(1.0, |k| interpolate(&k.speed, level))
                * capacity
                * a.rate;
            if speed <= 0.0 {
                return Err((Reason::TooYoung, None));
            }
            let have = |g: usize| make::available(&stores, goods, g, reserve_ok, protected);
            let from_inputs = make::units_from_inputs(r, &have);
            // What making it is worth, and how much of it is wanted.
            let (worth, wanted) = if let Some(_tool) = &out_good.tool {
                let need = tool_need.get(out).copied().unwrap_or(0.0);
                let sale = for_sale.get(out).copied().unwrap_or(0.0);
                if need > 0.0 {
                    (MakeWorth::Tool { tool: out, need }, 1.0)
                } else if sale > 0.0 {
                    (MakeWorth::Sale { share: sale }, 1.0)
                } else {
                    return Err((Reason::NotNeeded, None));
                }
            } else if let Some(input) = make::preserves(r, goods) {
                // Keeping food that would spoil before it is eaten (drying, smoking): as much of
                // it as would spoil, worth the food kept.
                let spoil = crate::knowledge::spoiling_kcal(
                    goods,
                    &stores,
                    hh.sheltered,
                    kcal_day,
                    &[input],
                );
                let per_unit = r
                    .inputs
                    .iter()
                    .find(|&&(g, _)| g == input)
                    .map_or(0.0, |&(_, amount)| amount * goods[input].kcal_per_kg);
                if per_unit <= 0.0
                    || spoil < MIN_PRESERVE_DAYS * params.household.daily_kcal_per_person
                {
                    return Err((Reason::NotNeeded, None));
                }
                (MakeWorth::Preserve { kcal: per_unit }, spoil / per_unit)
            } else if out_good.purpose == GoodUse::Food {
                let per_unit: f64 = r
                    .outputs
                    .iter()
                    .filter_map(|&(g, amount)| goods.get(g).map(|d| amount * d.kcal_per_kg))
                    .sum();
                if per_unit <= 0.0 {
                    return Err((Reason::NotNeeded, None));
                }
                // Ready food answers for itself; food a step from ready (flour) is wanted only
                // as far as ready food and what is already a step from it fall short.
                let ready_days = food_all / kcal_day.max(1.0);
                let (pool_days, target) = if out_good.edible() {
                    (ready_days, ready_target)
                } else {
                    (
                        ready_days + held_of(out) * out_good.kcal_per_kg / kcal_day.max(1.0),
                        processed_target.max(ready_target),
                    )
                };
                let short = (1.0 - pool_days / target).clamp(0.0, 1.0);
                let room = target / (target + pool_days.max(0.0));
                // Enough to bring what it makes up to its days and no more: what is made ahead
                // of need spoils (bread in days).
                let want_days = target - pool_days;
                if want_days < MIN_BATCH_DAYS {
                    return Err((Reason::NotNeeded, None));
                }
                let want_kcal = want_days * kcal_day.max(1.0);
                let toward_meal = out_good.edible() || held_of(out) * out_good.kcal_per_kg < 1.0;
                (
                    MakeWorth::Food {
                        kcal: 0.0,
                        short,
                        room,
                        toward_meal,
                    },
                    want_kcal / per_unit,
                )
            } else {
                return Err((Reason::NotNeeded, None));
            };
            if from_inputs <= 1e-6 {
                return Err((Reason::NoInputs, None));
            }
            if let Some(&t) = r
                .tools
                .iter()
                .find(|&&t| free_tools.get(t).copied().unwrap_or(0.0) < decide::MIN_TOOL)
            {
                return Err((Reason::NoTool, Some(t)));
            }
            let by_time = make::units_in(r, f64::from(a.max_minutes), speed);
            let units = from_inputs.min(wanted).min(by_time);
            if units.is_nan() || units <= 1e-6 {
                return Err((Reason::NoInputs, None));
            }
            let worth = match worth {
                MakeWorth::Food {
                    short,
                    room,
                    toward_meal,
                    ..
                } => MakeWorth::Food {
                    kcal: units
                        * r.outputs
                            .iter()
                            .filter_map(|&(g, amount)| goods.get(g).map(|d| amount * d.kcal_per_kg))
                            .sum::<f64>(),
                    short,
                    room,
                    toward_meal,
                },
                // The food kept: the units made of what would have spoiled.
                MakeWorth::Preserve { kcal: per_unit } => MakeWorth::Preserve {
                    kcal: units * per_unit,
                },
                tool => tool,
            };
            let firm = match worth {
                MakeWorth::Sale { .. } => self.workshop_of(hh_id, out),
                _ => None,
            };
            Ok(MakeOption {
                units,
                minutes: make::minutes_for(r, units, speed),
                worth,
                firm,
            })
        };
        let makes_tool = |def: usize| {
            catalog.activities[def]
                .recipe
                .and_then(|r| catalog.recipes.get(r))
                .and_then(|r| r.outputs.first())
                .and_then(|&(g, _)| goods.get(g))
                .is_some_and(|g| g.tool.is_some())
        };
        let shop = decide::Workshop {
            free_tools: &free_tools,
            best_make: &best_make,
            makes_tool: &makes_tool,
            knows: &|t| self.people.get(h).is_some_and(|p| p.knows(t)),
        };
        // Trying toward a technique that would answer a problem at home (ADR-0008 §3): the one
        // whose problem would cost the household the most food, among those the person could
        // find.
        let try_gap_min = (params.knowledge.try_gap_days * MINUTES_PER_DAY as f64) as i64;
        let trying = || -> Result<decide::TryOption, Reason> {
            let p = self.people.get(h).ok_or(Reason::NoProblem)?;
            if p.tried
                .is_some_and(|t| now.minutes() - t.minutes() < try_gap_min)
            {
                return Err(Reason::TriedLately);
            }
            let half = (params.decision.trip_half_worth_days * kcal_day).max(1.0);
            let knows = |t: usize| p.knows(t);
            let mut best: Option<(f64, usize)> = None;
            for (t, def) in catalog.techniques.iter().enumerate() {
                if def.answers_spoilage.is_empty()
                    || p.knows(t)
                    || !crate::knowledge::could_find(catalog, t, &knows, &stores)
                {
                    continue;
                }
                let kcal = crate::knowledge::spoiling_kcal(
                    goods,
                    &stores,
                    hh.sheltered,
                    kcal_day,
                    &def.answers_spoilage,
                );
                let share = kcal / (kcal + half);
                if best.is_none_or(|(b, _)| share > b) {
                    best = Some((share, t));
                }
            }
            let (share, t) = best
                .filter(|&(share, _)| share >= crate::knowledge::MIN_PROBLEM_SHARE)
                .ok_or(Reason::NoProblem)?;
            Ok(decide::TryOption {
                technique: t as u16,
                points: params.knowledge.w_try * share,
            })
        };
        let (cands, excluded) = decide::candidates(
            &catalog.activities,
            &params.decision,
            &facts,
            &limits,
            &best_patch,
            &best_field,
            water,
            &giver,
            &trade,
            &job,
            build,
            &shop,
            &trying,
        );
        // Work that needs a technique the person does not know is left out, unless a member of
        // their household who knows it is at that work there now: then they may work beside them
        // and learn it (ADR-0008 §4).
        let (mut cands, mut excluded) = (cands, excluded);
        if let Some(me) = self.people.get(h) {
            let mut i = 0;
            while i < cands.len() {
                let (def, target) = (cands[i].scored.def, cands[i].scored.target);
                let gate = catalog
                    .activities
                    .get(usize::from(def))
                    .and_then(|a| self.technique_for(ctx, a, target))
                    .map_or(Gate::Open, |t| self.gate(ctx, me, t, def, target));
                match gate {
                    Gate::Open => i += 1,
                    Gate::Learner(_) => {
                        decide::add_term(&mut cands[i], Reason::Learning, params.knowledge.w_learn);
                        i += 1;
                    }
                    Gate::Closed => {
                        cands.remove(i);
                        excluded.push((def, Reason::DoesNotKnow));
                    }
                }
            }
        }
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
                    // Who holds new ground is the regime's rule (ADR-0007 §2).
                    let holder = match (ctx.regime.holder, hh.settlement) {
                        (LandHolder::Settlement, Some(s)) => Party::Settlement(s),
                        _ => Party::Household(hh_id),
                    };
                    ctx.land.fields.push(Field {
                        id,
                        household: hh_id,
                        holder,
                        lease: None,
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
        if target == Target::NewFirm {
            // A workshop is set up when someone sets to work making what it sells.
            let out = catalog
                .activities
                .get(usize::from(chosen.scored.def))
                .and_then(|a| a.recipe)
                .and_then(|r| catalog.recipes.get(r))
                .and_then(|r| r.outputs.first())
                .map(|&(g, _)| g);
            let founder = self.people.get(h).map(|p| p.id);
            target = match (out, founder) {
                (Some(g), Some(founder)) => self
                    .found_firm(ctx, hh_id, founder, g)
                    .map_or(Target::Home, Target::Firm),
                _ => Target::Home,
            };
            receipt.chosen.target = target;
        }
        // Paid work taken is held for the person (slice J).
        if let (Target::Firm(firm), Some(Behavior::Hire)) = (
            target,
            catalog
                .activities
                .get(usize::from(chosen.scored.def))
                .map(|a| a.behavior),
        ) {
            let minutes: u32 = chosen
                .steps
                .iter()
                .map(|s| match s {
                    Step::Work { minutes } => *minutes,
                    _ => 0,
                })
                .sum();
            self.take_work(firm, f64::from(minutes));
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
        let wear = &ctx.land.wear;
        if wear.rev() != self.routes_rev {
            self.routes.clear();
            self.routes_rev = wear.rev();
        }
        let (points, minutes): (Vec<(f32, f32)>, Vec<f32>) = if from_cell == to_cell {
            let d = ((to.0 - p.pos.0).powi(2) + (to.1 - p.pos.1).powi(2)).sqrt();
            let surface = params.nav.offtrail_factor
                + (1.0 - params.nav.offtrail_factor) * f64::from(wear.factor(from_cell));
            let v = params.nav.tobler_ms(0.0) * surface * speed;
            let m = (f64::from(d) / v / 60.0) as f32;
            (vec![p.pos, to], vec![0.0, m])
        } else {
            let key = (from_cell as u32, to_cell as u32);
            let cached = match self.routes.get(&key) {
                Some(known) => known.clone(),
                None => {
                    // Planned on the paths as last surveyed; with no worn ground the search can
                    // assume off-trail walking everywhere and stays tight.
                    let routed = ctx.nav.route_bounded(
                        &ctx.map.elevation,
                        from_cell,
                        to_cell,
                        &|c| wear.factor(c),
                        wear.max_factor(),
                        ROUTE_BUDGET,
                    );
                    let found = match routed {
                        RouteResult::Found(route) => Some(std::sync::Arc::new(ctx.nav.straighten(
                            &ctx.map.elevation,
                            &route,
                            &|c| wear.factor(c),
                        ))),
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
        p.burn_kcal_min = burn_rate(p, now, params, params.energy.walk_par);
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
            // Work at home is done among the household.
            Some(
                Behavior::Eat | Behavior::Rest | Behavior::Play | Behavior::Make | Behavior::Try,
            ) => (def_par, false, params.social.household_quality),
            Some(
                Behavior::Gather
                | Behavior::FetchWater
                | Behavior::Farm
                | Behavior::Ask
                | Behavior::Trade
                | Behavior::Hire
                | Behavior::Build,
            ) => (def_par, false, 0.0),
            None => (params.energy.idle_par, false, 0.0),
        };
        p.burn_kcal_min = burn_rate(p, now, params, par);
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
        let (who, act_def, act_target, started) =
            (p.id, p.act.def, p.act.target, p.act.step_started);
        // The technique the work needs, found before the work changes what it is aimed at.
        let technique = match (step, def.as_ref()) {
            (Some(Step::Work { .. }), Some(d)) => self.technique_for(ctx, d, act_target),
            _ => None,
        };
        let Some(p) = self.people.get_mut(h) else {
            return;
        };
        match step {
            Some(Step::Walk { to }) => {
                p.pos = to;
                if let Some(trip) = p.trip.take() {
                    let map = ctx.map;
                    let mut cells =
                        cells_along(&trip.points, map.cell_size_m, map.width, map.height);
                    cells.retain(|&c| map.water[c as usize] == WATER_LAND);
                    ctx.land
                        .wear
                        .walk(&cells, now.day_index(), &ctx.land_params.paths);
                }
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
                        let eff = interpolate(&params.capacity_by_age, p.age_years(now)) * d.rate;
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
                        let eff = interpolate(&params.capacity_by_age, p.age_years(now)) * d.rate;
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
                Some(Behavior::Trade) => {
                    if let Target::Household(seller) | Target::Firm(seller) = p.act.target {
                        let household = p.household;
                        self.settle_trade(ctx, household, seller);
                    }
                }
                Some(Behavior::Build) => {
                    if let Target::Building(building) = p.act.target {
                        let rate = def.as_ref().map_or(1.0, |d| d.rate);
                        let eff = interpolate(&params.capacity_by_age, p.age_years(now)) * rate;
                        let hours = f64::from(minutes) / 60.0 * eff;
                        let (who, household) = (p.id, p.household);
                        self.build_work(ctx, who, household, building, hours);
                    }
                }
                Some(Behavior::Hire) => {
                    if let Target::Firm(firm) = p.act.target {
                        self.hired_work(ctx, h, firm, minutes);
                    }
                }
                Some(Behavior::Make) => {
                    if let Some(d) = def.as_ref() {
                        match p.act.target {
                            Target::Firm(firm) => self.firm_make(ctx, h, d, minutes, firm),
                            _ => self.make_work(ctx, h, d, minutes),
                        }
                    }
                }
                _ => {}
            },
            Some(Step::Wait { .. } | Step::Deposit) | None => {}
        }
        // A knower has practised the work's technique; a learner learns by it (ADR-0008 §4).
        if let (Some(Step::Work { minutes }), Some(t)) = (step, technique) {
            self.practise(ctx, who, t, act_def, act_target, f64::from(minutes) / 60.0);
        }
        // The work may have found something new (ADR-0008 §3).
        if let Some(Step::Work { minutes }) = step {
            if def.as_ref().is_some_and(|d| d.behavior == Behavior::Try)
                && let Some(p) = self.people.get_mut(h)
            {
                p.tried = Some(now);
            }
            let hours = f64::from(minutes) / 60.0;
            self.discover(ctx, who, (act_def, act_target), hours, started);
        }
        // Work wears the tools it needs (a recipe's are worn where it is worked).
        if let (Some(Step::Work { minutes }), Some(d)) = (step, def.as_ref())
            && d.behavior != Behavior::Make
            && !d.tools.is_empty()
            && let Some(household) = self.people.get(h).map(|p| p.household)
        {
            self.wear_tools(ctx, household, &d.tools, f64::from(minutes) / 60.0);
        }
        if let Some(p) = self.people.get_mut(h) {
            p.burn_kcal_min = burn_rate(p, now, params, params.energy.idle_par);
            p.asleep = false;
        }
    }

    /// Wears `tools` of `household` by `hours` of use.
    fn wear_tools(&mut self, ctx: &Ctx, household: PermanentId, tools: &[usize], hours: f64) {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let Some(x) = self
            .hh_index
            .get(&household)
            .and_then(|&hd| self.households.get_mut(hd))
        else {
            return;
        };
        let members = x.members.len().max(1);
        x.settle_stores(now, goods, &|d| fuel_per_day(params, members, d));
        x.stores.resize(goods.len(), 0.0);
        let before = x.stores.clone();
        make::wear(&mut x.stores, goods, tools, hours);
        x.flows
            .add_changes(&before, &x.stores, Flow::Made, Flow::Worn);
    }

    /// Applies `minutes` of person `h` working the recipe of make activity `def` at home: what
    /// their skill and age let them make in that time, from what the household holds (seed only
    /// in hunger, and never the seed for the ground already cropped), the recipe's tools worn by
    /// the time worked, and their skill practised.
    fn make_work(&mut self, ctx: &mut Ctx, h: Handle<Person>, def: &ActivityDef, minutes: u32) {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let Some(recipe) = def.recipe.and_then(|r| ctx.catalog.recipes.get(r)) else {
            return;
        };
        let Some(p) = self.people.get(h) else {
            return;
        };
        let household = p.household;
        let skill = recipe
            .skill
            .and_then(|k| ctx.catalog.skills.get(k).map(|s| (k, s)));
        let level = skill.map_or(0.0, |(k, _)| p.skill(k));
        let speed = skill.map_or(1.0, |(_, s)| interpolate(&s.speed, level))
            * interpolate(&params.capacity_by_age, p.age_years(now))
            * def.rate;
        let quality = skill.map_or(1.0, |(_, s)| interpolate(&s.quality, level));
        let reserve_ok = may_eat_reserve(p, now, params);
        let protected = farm::protected_seed(
            ctx.land.fields.iter().filter(|f| f.household == household),
            &ctx.catalog.crops,
        );
        let hours = f64::from(minutes) / 60.0;
        let units = make::units_in(recipe, f64::from(minutes), speed);
        if let Some(x) = self
            .hh_index
            .get(&household)
            .and_then(|&hd| self.households.get_mut(hd))
        {
            let members = x.members.len().max(1);
            x.settle_stores(now, goods, &|d| fuel_per_day(params, members, d));
            x.stores.resize(goods.len(), 0.0);
            let before = x.stores.clone();
            make::apply(
                recipe,
                &mut x.stores,
                goods,
                units,
                quality,
                reserve_ok,
                protected,
            );
            x.flows
                .add_changes(&before, &x.stores, Flow::Made, Flow::Used);
            let before = x.stores.clone();
            make::wear(&mut x.stores, goods, &recipe.tools, hours);
            x.flows
                .add_changes(&before, &x.stores, Flow::Made, Flow::Worn);
        }
        if let (Some((k, s)), Some(p)) = (skill, self.people.get_mut(h)) {
            let next = s.practised(p.skill(k), hours);
            p.set_skill(k, next);
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
        let held = stock_kcal(&stores_now(t, now, params, goods), goods);
        let mut want = spare.min((params.household.food_target_days * need - held).max(0.0));
        if want <= 0.0 {
            return;
        }
        let stores = stores_now(g, now, params, goods);
        let mut order: Vec<usize> = (0..goods.len())
            .filter(|&i| {
                goods[i].purpose == GoodUse::Food
                    && goods[i].kcal_per_kg > 0.0
                    && !goods[i].kept_back()
            })
            .collect();
        let keeps = |i: usize| match goods[i].half_life_days {
            h if h > 0.0 => h,
            _ => f64::INFINITY,
        };
        order.sort_by(|&a, &b| keeps(a).total_cmp(&keeps(b)).then(a.cmp(&b)));
        let mut carry = params.household.carry_kg;
        let mut legs = Vec::new();
        for i in order {
            if want <= 0.0 || carry <= 0.0 {
                break;
            }
            let kg = (want / goods[i].kcal_per_kg)
                .min(stores.get(i).copied().unwrap_or(0.0).max(0.0))
                .min(carry);
            if kg <= 0.0 {
                continue;
            }
            legs.push(Leg {
                from: giver,
                to,
                good: i,
                amount: kg,
            });
            carry -= kg;
            want -= kg * goods[i].kcal_per_kg;
        }
        self.transfer(now, params, goods, &legs, Channel::Gift);
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
            x.flows
                .add(Flow::Sown, crop.seed_good, seed.min(done.seed_kg));
        }
        if done.grain_kg > 0.0 {
            x.flows.add(Flow::Got, crop.good, done.grain_kg);
            let have = x.stores.get(crop.seed_good).copied().unwrap_or(0.0);
            let to_seed = (seed_wanted - have).clamp(0.0, done.grain_kg);
            if let Some(s) = x.stores.get_mut(crop.seed_good) {
                *s += to_seed;
            }
            if let Some(g) = x.stores.get_mut(crop.good) {
                *g += done.grain_kg - to_seed;
            }
            // Grain set aside as seed is counted as got as seed, not grain.
            x.flows.add(Flow::Got, crop.good, -to_seed);
            x.flows.add(Flow::Got, crop.seed_good, to_seed);
            // Threshing leaves the straw at home too.
            if let Some((good, kg_per_kg)) = crop.straw
                && let Some(s) = x.stores.get_mut(good)
            {
                *s += done.grain_kg * kg_per_kg;
                x.flows.add(Flow::Got, good, done.grain_kg * kg_per_kg);
            }
        }
        let settlement = x.settlement;
        // A let field's holder takes its share of the grain threshed from it, through the ledger
        // (ADR-0007 §3): in grain, and in seed grain for what the grain does not cover.
        if done.grain_kg > 0.0
            && let (Some(lease), Party::Household(holder)) =
                (ctx.land.fields[fi].lease, ctx.land.fields[fi].holder)
            && holder != household
            && self.household(holder).is_some()
        {
            let rent = done.grain_kg * f64::from(lease.holder_share);
            let held = |pop: &Population, g: usize| {
                pop.household(household)
                    .and_then(|x| x.stores.get(g))
                    .copied()
                    .unwrap_or(0.0)
                    .max(0.0)
            };
            let in_grain = rent.min(held(self, crop.good));
            let in_seed = (rent - in_grain).min(held(self, crop.seed_good));
            let legs: Vec<Leg> = [(crop.good, in_grain), (crop.seed_good, in_seed)]
                .into_iter()
                .filter(|&(_, kg)| kg > 1e-9)
                .map(|(good, amount)| Leg {
                    from: household,
                    to: holder,
                    good,
                    amount,
                })
                .collect();
            if !legs.is_empty() {
                self.transfer(now, params, goods, &legs, Channel::Rent);
            }
        }
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
            if let Some(&g) = def.materials.get(slot)
                && let Some(s) = x.stores.get_mut(g)
            {
                let used = s.max(0.0).min(*kg);
                *s = (*s - kg).max(0.0);
                x.flows.add(Flow::Built, g, used);
            }
        }
        if done.finished != Some(Stage::Roof) {
            return;
        }
        // Stores were settled to now above, so they spoil at the sheltered rates from here on.
        x.sheltered = true;
        let settlement = x.settlement;
        // A household that built itself a new home moves in once its roof is on, and the home it
        // leaves is taken down and its ground given up.
        let old: Vec<(PermanentId, PermanentId)> = ctx
            .land
            .buildings
            .iter()
            .filter(|b| b.household == household && b.id != building && b.finished())
            .map(|b| (b.id, b.plot))
            .collect();
        if !old.is_empty() {
            x.home = build::centre_m(&ctx.land.buildings[bi].spec);
            ctx.land
                .buildings
                .retain(|b| !old.iter().any(|&(id, _)| id == b.id));
            ctx.land
                .plots
                .retain(|p| !old.iter().any(|&(_, plot)| plot == p.id));
            for (id, _) in &old {
                self.stage_needs.remove(id);
            }
        }
        let Some(bi) = ctx.land.buildings.iter().position(|b| b.id == building) else {
            return;
        };
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

    /// After the paths were surveyed: notes in the chronicle each settlement whose first trail out
    /// has been worn in, a trail at least [`FIRST_TRAIL_M`] long that passes within
    /// [`FIRST_TRAIL_NEAR_M`] of its hearth.
    pub fn note_trails(&mut self, land: &Land, now: SimTime) {
        for s in &land.settlements {
            let noted = self
                .chronicle
                .iter()
                .any(|e| e.kind == ChronicleKind::FirstTrail && e.settlement == Some(s.id));
            if noted {
                continue;
            }
            let best = land
                .wear
                .trails()
                .iter()
                .filter(|t| {
                    t.length_m() >= FIRST_TRAIL_M
                        && t.points
                            .windows(2)
                            .any(|w| segment_distance(s.hearth_m, w[0], w[1]) <= FIRST_TRAIL_NEAR_M)
                })
                .max_by(|a, b| a.length_m().total_cmp(&b.length_m()));
            if let Some(t) = best {
                let middle = t.points[t.points.len() / 2];
                self.chronicle_push(
                    now,
                    ChronicleKind::FirstTrail,
                    Vec::new(),
                    Some(s.id),
                    Some(middle),
                    f64::from(t.length_m()),
                    s.name.clone(),
                );
            }
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
        let before = hh.stores.clone();
        let take = eat_from(&mut hh.stores, goods, want, reserve_ok);
        hh.flows
            .add_changes(&before, &hh.stores, Flow::Eaten, Flow::Eaten);
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
        let kg = f64::from(load.kg);
        // The load comes into the household's store.
        let Some(x) = self.households.get_mut(own) else {
            return;
        };
        let members = x.members.len().max(1);
        x.settle_stores(now, goods, &|d| fuel_per_day(params, members, d));
        x.stores.resize(goods.len(), 0.0);
        let target = params.household.food_target_days
            * members as f64
            * params.household.daily_kcal_per_person;
        let short = (target - stock_kcal(&x.stores, goods)).max(0.0);
        x.stores[g] += kg;
        x.flows.add(Flow::Got, g, kg);
        // A shared good is shared out of what the household can spare: it first keeps what
        // brings its food to the days it tries to keep, and the rest goes to every household of
        // the settlement, by members (research 06-01 §2.3 and §3.2: help comes from disposable
        // surplus, after a household's own subsistence). In plenty, all of a kill is shared; in
        // hunger, a family keeps what it catches.
        let Some(s) = settlement.filter(|_| good.shared && good.kcal_per_kg > 0.0) else {
            return;
        };
        let spare = kg - kg.min(short / good.kcal_per_kg);
        if spare <= 0.0 {
            return;
        }
        let mut recipients: Vec<(PermanentId, usize)> = self
            .households
            .iter()
            .filter(|(_, x)| x.settlement == Some(s) && !x.members.is_empty())
            .map(|(_, x)| (x.id, x.members.len()))
            .collect();
        recipients.sort_unstable();
        let total = recipients.iter().map(|r| r.1).sum::<usize>().max(1) as f64;
        let legs: Vec<Leg> = recipients
            .into_iter()
            .filter(|&(id, _)| id != hh_id)
            .map(|(id, n)| Leg {
                from: hh_id,
                to: id,
                good: g,
                amount: spare * n as f64 / total,
            })
            .collect();
        self.transfer(now, params, goods, &legs, Channel::Share);
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
            kcal += stock_kcal(&stores_now(x, now, params, goods), goods);
            members += x.members.len();
        }
        (members > 0)
            .then(|| kcal / (members as f64 * params.household.daily_kcal_per_person).max(1.0))
    }

    /// The end of a day: the season turns for every field, harvests and food shortages are noted
    /// in the chronicle (with a gap between running short and recovering, so a store hovering
    /// near the line is not noted every day), and then a day of life: births, deaths, couples and
    /// the households they make (see `life`). Newborns' first decisions go to `ctx.schedule`.
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
        // The yearly land review (ADR-0007 §2): fields given out by need, or ground nobody holds
        // any more taken up.
        if day.rem_euclid(365) == i64::from(ctx.regime.review_day) {
            let ids: Vec<PermanentId> = ctx.land.settlements.iter().map(|s| s.id).collect();
            for s in ids {
                self.review_land(ctx, s);
            }
        }
        // Households whose day it is review what they offer and on what terms.
        self.review_offers(ctx, day);
        // Births, deaths, couples and the households they make.
        self.live_day(ctx);
        // Children who have reached the age of their household's work learn it (ADR-0008 §4).
        self.bring_up(ctx);
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

/// Takes up to `want` kcal from the food in store that can be eaten as it is, the most perishable
/// first and goods kept back last of all and only when `reserve` allows, and only what needs no
/// fire when there is no firewood. Food a recipe must make ready first (grain, flour) is never
/// eaten. Returns the kcal taken.
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
            good.edible() && (fire || !good.cooked()) && (reserve || !good.kept_back())
        })
        .collect();
    order.sort_by(|&a, &b| {
        let (ga, gb) = (&goods[a], &goods[b]);
        ga.kept_back()
            .cmp(&gb.kept_back())
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
    use crate::params::{Eaten, GoodUse};

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
            leave_at_depletion: 0.3,
            leave_per_day: 0.1,
            leave_unless_ripe_within_days: 30.0,
            ready_food_days: 2.0,
            harvest_margin_days: 30.0,
            processed_food_days: 5.0,
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
        let good = |id: &str, purpose, kcal, half, eaten| GoodDef {
            id: id.into(),
            name: id.into(),
            purpose,
            kcal_per_kg: kcal,
            half_life_days: half,
            eaten,
            shared: false,
            reserve_for: None,
            tool: None,
            sheltered_half_life_days: 0.0,
        };
        vec![
            good("bread", GoodUse::Food, 2500.0, 1000.0, Eaten::Raw),
            good("meat", GoodUse::Food, 1500.0, 3.0, Eaten::Cooked),
            good("berries", GoodUse::Food, 600.0, 5.0, Eaten::Raw),
            good("wood", GoodUse::Fuel, 0.0, 0.0, Eaten::Never),
            good("grain", GoodUse::Food, 3300.0, 1500.0, Eaten::Never),
        ]
    }

    #[test]
    fn meals_come_from_the_most_perishable_food_and_cooked_food_needs_firewood() {
        let goods = goods();
        // Meat spoils fastest, then berries; the bread keeps longest of what is ready, and the
        // grain must be ground or pounded first.
        let mut stores = vec![10.0, 2.0, 1.0, 5.0, 100.0];
        let taken = eat_from(&mut stores, &goods, 3600.0, false);
        assert_eq!(taken, 3600.0);
        assert!((stores[1] - 0.0).abs() < 1e-12, "meat first: {stores:?}");
        assert!((stores[2] - 0.0).abs() < 1e-12, "then berries: {stores:?}");
        assert!((stores[0] - (10.0 - 0.0)).abs() < 1e-12, "bread untouched");
        // Without firewood the meat cannot be eaten, nor ever the grain.
        let mut cold = vec![1.0, 2.0, 0.0, 0.0, 100.0];
        let taken = eat_from(&mut cold, &goods, 100_000.0, false);
        assert_eq!(taken, 2500.0, "only the bread");
        assert_eq!(cold[1], 2.0);
        assert_eq!(cold[4], 100.0);
    }
}
