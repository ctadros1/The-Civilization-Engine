//! The population: people and households, and how their activities run (ADR-0003).
//!
//! An activity is a list of steps (walk, work, deposit). Each step that takes time schedules one
//! event at its end carrying the person's permanent id and the activity version; when it fires,
//! the step's effects are applied and the next step starts, or, after the last, the person
//! decides what to do next. Needs are brought up to date only at step boundaries.

use std::collections::{BTreeMap, HashMap};

use civ_core::time::MINUTES_PER_DAY;
use civ_core::{GenTable, Handle, IdAllocator, PermanentId, Rng64, SimTime};
use civ_land::{Land, LandParams};
use civ_world::nav::{NavGrid, RouteResult, TravelField};
use civ_world::{WATER_LAKE, WATER_RIVER, WorldMap};

use crate::decide::{self, Facts, Limits, PatchOption, WaterOption};
use crate::history::{ChronicleEvent, ChronicleKind, PersonRecord, Receipt};
use crate::needs;
use crate::params::{Behavior, Catalog, PeopleParams, interpolate};
use crate::person::{Activity, Household, KnownPatch, Person, Step, Target, Trip};

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

/// Travel times from a settlement's hearth (or a lone household's home), and the water and patches
/// within reach. Shared by every household of the settlement: homes stand a few metres from the
/// hearth. Derived.
#[derive(Debug)]
struct HomeField {
    cell: usize,
    day: i64,
    water: Option<(u32, f32)>,
    patches: Vec<(u32, f32)>,
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
    access: Vec<Option<u32>>,
    /// Routes by (from cell, to cell); `None` when there is none. Derived: walking costs do not
    /// change while routes are kept (trail wear will empty it when it arrives).
    routes: HashMap<(u32, u32), Option<CachedRoute>>,
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

/// Brings a person's needs up to `now` at the rates in force since they were last settled.
pub fn settle(p: &mut Person, now: SimTime, params: &PeopleParams) {
    let dt = (now.minutes() - p.needs_at.minutes()) as f64;
    if dt <= 0.0 {
        return;
    }
    let energy = f64::from(p.energy_kcal) - f64::from(p.burn_kcal_min) * dt;
    p.energy_kcal = energy.min(params.energy.max_surplus_kcal) as f32;
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
    let elapsed = (now.minutes() - p.needs_at.minutes()).max(0) as f64;
    let energy = f64::from(p.energy_kcal) - f64::from(p.burn_kcal_min) * elapsed;
    ramp + (-energy).max(0.0) / params.energy.deficit_unit_kcal
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
        self.access.clear();
        self.routes.clear();
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
            if !(h.food_kcal.is_finite() && h.water_l.is_finite()) {
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

    fn ensure_patch_access(&mut self, ctx: &Ctx) {
        if self.access.len() == ctx.land.patches.len() {
            return;
        }
        let patches = &ctx.land.patches;
        let w = ctx.map.width as usize;
        let pc = patches.patch_cells;
        let mut access = Vec::with_capacity(patches.len());
        for p in 0..patches.len() {
            let (px, py) = (p as u32 % patches.cols, p as u32 / patches.cols);
            let (cx, cy) = (px * pc + pc / 2, py * pc + pc / 2);
            let mut best: Option<(u32, u32)> = None;
            for y in py * pc..((py + 1) * pc).min(ctx.map.height) {
                for x in px * pc..((px + 1) * pc).min(ctx.map.width) {
                    let i = y as usize * w + x as usize;
                    if ctx.map.water[i] != civ_world::WATER_LAND || !ctx.nav.walkable(i) {
                        continue;
                    }
                    let d = x.abs_diff(cx).pow(2) + y.abs_diff(cy).pow(2);
                    if best.is_none_or(|(bd, _)| d < bd) {
                        best = Some((d, i as u32));
                    }
                }
            }
            access.push(best.map(|(_, i)| i));
        }
        self.access = access;
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
        self.ensure_patch_access(ctx);
        let reach = Self::field_reach_seconds(ctx.catalog);
        let field = ctx
            .nav
            .travel_field(&ctx.map.elevation, cell, reach, &|_| 0.0);
        let water = nearest_water(ctx.map, &field);
        let mut patches = Vec::new();
        for (p, a) in self.access.iter().enumerate() {
            if let Some(cell) = a
                && let Some(s) = field.seconds_to(*cell as usize)
            {
                patches.push((p as u32, s));
            }
        }
        patches.sort_by(|a, b| a.1.total_cmp(&b.1));
        self.homes.insert(
            key,
            HomeField {
                cell,
                day,
                water,
                patches,
            },
        );
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
        let members = hh.members.len().max(1) as f64;
        let kcal_day = members * params.household.daily_kcal_per_person;
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
            hunger: hunger(p, now, params),
            sleep_drive: f64::from(p.sleep_pressure) * circ,
            sleep_pressure: f64::from(p.sleep_pressure),
            loneliness: 1.0 - f64::from(p.relatedness),
            dark,
            daylight_left_min: if dark { 0.0 } else { (sun.1 - minute) as f64 },
            evening,
            until_sunrise_min: until_sunrise as f64,
            food_days: hh.food_kcal / kcal_day.max(1.0),
            food_target_days: params.household.food_target_days,
            water_days: water_l / (members * params.household.water_l_per_person_day).max(1e-6),
            water_target_days: params.household.water_target_days,
            household_kcal_day: kcal_day,
            has_food: hh.food_kcal > 1.0,
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
            carry_kcal: params.household.carry_food_kcal,
        };
        let home = self.homes.get(&field_key);
        let water = home.and_then(|f| {
            f.water.map(|(cell, s)| WaterOption {
                cell,
                walk_min: f64::from(s) / 60.0,
                at: cell_centre(ctx.map, cell as usize),
            })
        });
        let land = &*ctx.land;
        let land_params = ctx.land_params;
        let map = ctx.map;
        let catalog = ctx.catalog;
        let access = &self.access;
        let best_patch = |def: usize| -> Option<PatchOption> {
            let a = &catalog.activities[def];
            let r = a.resource?;
            let res = land_params.resources.get(r)?;
            let home = home?;
            let hours = f64::from(a.max_minutes) / 60.0;
            let mut best: Option<(f64, PatchOption)> = None;
            for &(patch, secs) in &home.patches {
                let walk = f64::from(secs) / 60.0;
                if walk > f64::from(a.max_walk_minutes) {
                    break;
                }
                let prior = prior_rate(land_params, land, r, patch as usize);
                let rate = match hh.known.iter().find(|k| k.patch == patch) {
                    Some(k) => {
                        let days = (now.day_index() - k.seen_day).max(0) as f64;
                        let forget = 1.0 - (-days / 30.0).exp();
                        f64::from(k.rate) + (prior - f64::from(k.rate)) * forget
                    }
                    None => prior,
                };
                if rate <= 0.0 {
                    continue;
                }
                let gain = (rate * hours).min(res.max_rate_per_hour * hours);
                let value = gain / (hours + 2.0 * walk / 60.0);
                if best.is_none_or(|(bv, _)| value > bv) {
                    let Some(cell) = access.get(patch as usize).copied().flatten() else {
                        continue;
                    };
                    best = Some((
                        value,
                        PatchOption {
                            patch,
                            walk_min: walk,
                            rate,
                            at: cell_centre(map, cell as usize),
                        },
                    ));
                }
            }
            best.map(|(_, o)| o)
        };
        let (cands, excluded) = decide::candidates(
            &catalog.activities,
            &params.decision,
            &facts,
            &limits,
            &best_patch,
            water,
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
        let receipt = Receipt {
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
        p.act = Activity {
            def: chosen.scored.def,
            target: chosen.scored.target,
            steps: chosen.steps.clone(),
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
            Some(Behavior::Gather | Behavior::FetchWater) => (def_par, false, 0.0),
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
                    {
                        let eff = interpolate(&params.capacity_by_age, p.age_years(now));
                        let hours = f64::from(minutes) / 60.0;
                        // Gather no more than can be carried: stop when the load is full.
                        let carry = params.household.carry_food_kcal;
                        let got =
                            ctx.land
                                .gather_around(ctx.land_params, r, patch as usize, hours, eff);
                        let kept = got.min(carry);
                        if got > kept {
                            ctx.land.stocks[r][patch as usize] += (got - kept) as f32;
                        }
                        p.carrying.food_kcal += kept as f32;
                        let rate = (got / hours.max(1e-6)) as f32;
                        let hh_id = p.household;
                        self.remember_patch(hh_id, patch, rate, now.day_index());
                    }
                }
                Some(Behavior::FetchWater) => {
                    p.carrying.water_l = params.household.carry_water_l as f32;
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

    fn remember_patch(&mut self, household: PermanentId, patch: u32, rate: f32, day: i64) {
        let Some(&hh) = self.hh_index.get(&household) else {
            return;
        };
        let Some(hh) = self.households.get_mut(hh) else {
            return;
        };
        match hh.known.iter_mut().find(|k| k.patch == patch) {
            Some(k) => {
                k.rate = rate;
                k.seen_day = day;
            }
            None => hh.known.push(KnownPatch {
                patch,
                rate,
                seen_day: day,
            }),
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
        let Some(&hh) = self.hh_index.get(&household) else {
            return;
        };
        let Some(hh) = self.households.get_mut(hh) else {
            return;
        };
        let take = want.min(hh.food_kcal).max(0.0);
        hh.food_kcal -= take;
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
        let params = ctx.params;
        let Some(p) = self.people.get_mut(h) else {
            return;
        };
        let load = std::mem::take(&mut p.carrying);
        let hh_id = p.household;
        let Some(&hh) = self.hh_index.get(&hh_id) else {
            return;
        };
        let Some(hh) = self.households.get_mut(hh) else {
            return;
        };
        let members = hh.members.len().max(1) as f64;
        hh.settle_water(ctx.now, members * params.household.water_l_per_person_day);
        hh.food_kcal += f64::from(load.food_kcal);
        hh.water_l += f64::from(load.water_l);
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

/// What people expect from a patch they have never gathered in: its habitat's typical return.
pub fn prior_rate(params: &LandParams, land: &Land, r: usize, patch: usize) -> f64 {
    let res = &params.resources[r];
    let class = land.patches.class[patch] as usize;
    let per_ha = res.production_per_ha_yr.get(class).copied().unwrap_or(0.0);
    if per_ha <= 0.0 || res.loss_per_day <= 0.0 {
        return 0.0;
    }
    let daily = per_ha * land.patches.area_ha() / 365.0;
    let stock = daily / res.loss_per_day;
    let half = res.half_rate_stock_per_ha * land.patches.area_ha();
    res.max_rate_per_hour * stock / (stock + half)
}
