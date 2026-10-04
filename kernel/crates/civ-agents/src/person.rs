//! People, households, activities and trips (ADR-0003).

use std::collections::VecDeque;

use civ_core::time::MINUTES_PER_DAY;
use civ_core::{PermanentId, SimTime};

use crate::history::Receipt;
use crate::needs::Sex;
use crate::params::{GoodDef, GoodUse};

/// Most decision receipts kept per person (ADR-0003).
pub const RECEIPT_RING: usize = 64;

/// What an activity is aimed at.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Target {
    /// Nothing in particular.
    None,
    /// The household's home.
    Home,
    /// The settlement's hearth.
    Hearth,
    /// A land patch, by index.
    Patch(u32),
    /// A terrain cell next to drinkable water.
    Water(u32),
    /// A field, by permanent id.
    Field(PermanentId),
    /// New ground for a field, not yet marked out (an option weighed, never an activity's
    /// target: a chosen new field is marked out at once).
    NewField,
    /// Another household, by permanent id (asked for food).
    Household(PermanentId),
    /// A building, by permanent id.
    Building(PermanentId),
    /// A home not yet begun (an option weighed, never an activity's target: a chosen one is
    /// marked out at once).
    NewBuilding,
    /// A firm, by permanent id: the workshop worked for, or the seller bought from (slice J).
    Firm(PermanentId),
    /// A workshop not yet founded (an option weighed, never an activity's target: a chosen one
    /// is founded at once).
    NewFirm,
}

/// One step of an activity.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Step {
    /// Walk to a point, metres.
    Walk {
        /// Destination.
        to: (f32, f32),
    },
    /// Do the activity's work where one stands.
    Work {
        /// How long.
        minutes: u32,
    },
    /// Hand what is carried to the household's store.
    Deposit,
    /// Stand and wait (after a walk could not be routed, or with nothing to do).
    Wait {
        /// How long.
        minutes: u32,
    },
}

/// What a person is doing: an authored activity broken into steps. Every scheduled event carries
/// the activity's `version`; replacing or interrupting the activity bumps it, so events of the
/// old activity do nothing (ADR-0003).
#[derive(Clone, Debug, PartialEq)]
pub struct Activity {
    /// The activity, by index in the catalog.
    pub def: u16,
    /// What it is aimed at.
    pub target: Target,
    /// Its steps.
    pub steps: Vec<Step>,
    /// The step in progress.
    pub step: u8,
    /// When the activity began.
    pub started: SimTime,
    /// When the current step began.
    pub step_started: SimTime,
    /// When the current step ends.
    pub step_ends: SimTime,
    /// Version, for stale-event detection.
    pub version: u32,
}

/// A walk: a route with timings (ADR-0003). Positions in between are interpolated.
#[derive(Clone, Debug, PartialEq)]
pub struct Trip {
    /// Trip id, unique in the world.
    pub id: u64,
    /// Revision: a replanned trip keeps its id and gets a new revision.
    pub rev: u32,
    /// When it set off.
    pub depart: SimTime,
    /// Route vertices, metres.
    pub points: Vec<(f32, f32)>,
    /// Minutes after departure at each vertex.
    pub minutes: Vec<f32>,
}

impl Trip {
    /// Minutes the walk takes.
    pub fn duration_minutes(&self) -> f32 {
        self.minutes.last().copied().unwrap_or(0.0)
    }

    /// Where the walker is `t` (fractional minutes since the calendar origin).
    pub fn position_at(&self, t: f64) -> (f32, f32) {
        let since = (t - self.depart.minutes() as f64) as f32;
        if self.points.is_empty() {
            return (0.0, 0.0);
        }
        if since <= 0.0 {
            return self.points[0];
        }
        for k in 1..self.points.len() {
            let (m0, m1) = (self.minutes[k - 1], self.minutes[k]);
            if since <= m1 {
                let f = if m1 > m0 {
                    (since - m0) / (m1 - m0)
                } else {
                    1.0
                };
                let (a, b) = (self.points[k - 1], self.points[k]);
                return (a.0 + (b.0 - a.0) * f, a.1 + (b.1 - a.1) * f);
            }
        }
        self.points[self.points.len() - 1]
    }
}

/// What a person is carrying.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Load {
    /// The good carried, by index in the catalog's goods.
    pub good: Option<u16>,
    /// How much of it, kilograms.
    pub kg: f32,
    /// Water, litres.
    pub water_l: f32,
}

/// Personality: five traits and a risk disposition, standard-normal (research 04-03 §1.2).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Traits {
    /// Openness.
    pub openness: f32,
    /// Conscientiousness.
    pub conscientiousness: f32,
    /// Extraversion.
    pub extraversion: f32,
    /// Agreeableness.
    pub agreeableness: f32,
    /// Neuroticism.
    pub neuroticism: f32,
    /// Risk taking.
    pub risk: f32,
}

/// Where a woman is in the reproductive cycle (research 05-01 §1.4: a state machine; 04-08
/// §1.5: a union does not switch fertility on by itself).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Repro {
    /// Can conceive, if of age and living with her partner.
    Open,
    /// Pregnant since `conceived` by `father`. On `due` the pregnancy ends: in a birth, or in a
    /// loss if `loss` (drawn at conception with the rest of its course).
    Pregnant {
        /// When she conceived.
        conceived: SimTime,
        /// When it ends.
        due: SimTime,
        /// The father, if known.
        father: Option<PermanentId>,
        /// It ends in a loss.
        loss: bool,
    },
    /// Cannot conceive until `until` (after a birth or a loss).
    Recovering {
        /// When she can again.
        until: SimTime,
    },
}

/// A living person.
#[derive(Clone, Debug, PartialEq)]
pub struct Person {
    /// Permanent id.
    pub id: PermanentId,
    /// Given name.
    pub given: String,
    /// Sex.
    pub sex: Sex,
    /// Birth time.
    pub born: SimTime,
    /// Mother, if known.
    pub mother: Option<PermanentId>,
    /// Father, if known.
    pub father: Option<PermanentId>,
    /// Household.
    pub household: PermanentId,
    /// Personality.
    pub traits: Traits,
    /// Where the person stands when not walking, metres.
    pub pos: (f32, f32),
    /// Energy balance against normal at `needs_at`, kcal (negative = deficit).
    pub energy_kcal: f32,
    /// When the last meal stops keeping them full.
    pub satiety_until: SimTime,
    /// Sleep pressure at `needs_at`, 0–1.
    pub sleep_pressure: f32,
    /// Relatedness at `needs_at`, 0–1.
    pub relatedness: f32,
    /// When needs were last brought up to date.
    pub needs_at: SimTime,
    /// Energy use since `needs_at`, kcal per minute.
    pub burn_kcal_min: f32,
    /// Whether asleep since `needs_at`.
    pub asleep: bool,
    /// Quality of present company since `needs_at`, 0–1.
    pub company: f32,
    /// What they are doing.
    pub act: Activity,
    /// Their walk, while walking.
    pub trip: Option<Trip>,
    /// What they carry.
    pub carrying: Load,
    /// Keyed-randomness counter (ADR-0003).
    pub draws: u64,
    /// Recent decision receipts, oldest first.
    pub receipts: VecDeque<Receipt>,
    /// Their partner, while both live.
    pub partner: Option<PermanentId>,
    /// Where a woman is in the reproductive cycle (always `Open` for a man).
    pub repro: Repro,
    /// A woman's lasting fecundability factor (research 05-01 §4.5: fertility is heterogeneous);
    /// 1 for a man.
    pub fecundity: f32,
    /// The child a mother is nursing, until she can conceive again.
    pub nursing: Option<PermanentId>,
    /// Skill levels, 0–1, by index in the catalog's skills; skills never practised are absent
    /// (ADR-0006 §2).
    pub skills: Vec<(u16, f32)>,
    /// What they know, are learning or have heard of, by technique, sorted by technique
    /// (ADR-0008 §2).
    pub knows: Vec<Know>,
}

/// How a person came to know of a technique (ADR-0008 §2). Numeric in saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum KnowSource {
    /// They brought it: a founder, or one of a family the observer sent.
    Founder,
    /// They were brought up in a household where this person knew it.
    Upbringing(PermanentId),
    /// They worked beside this person, who knew it.
    Taught(PermanentId),
    /// They found it themselves.
    Found,
    /// The observer introduced it (the god tool).
    Observer,
}

impl KnowSource {
    /// Its number in saves.
    pub fn code(self) -> u8 {
        match self {
            KnowSource::Founder => 0,
            KnowSource::Upbringing(_) => 1,
            KnowSource::Taught(_) => 2,
            KnowSource::Found => 3,
            KnowSource::Observer => 4,
        }
    }

    /// The person it came from, if any.
    pub fn person(self) -> Option<PermanentId> {
        match self {
            KnowSource::Upbringing(p) | KnowSource::Taught(p) => Some(p),
            _ => None,
        }
    }

    /// The source with number `code` and person `person` (a missing person reads as a founder's
    /// knowledge).
    pub fn from_code(code: u8, person: Option<PermanentId>) -> KnowSource {
        match (code, person) {
            (1, Some(p)) => KnowSource::Upbringing(p),
            (2, Some(p)) => KnowSource::Taught(p),
            (3, _) => KnowSource::Found,
            (4, _) => KnowSource::Observer,
            _ => KnowSource::Founder,
        }
    }
}

/// What a person knows of one technique (ADR-0008 §2): known, being learnt (hours toward its
/// `learn_h`), or only heard of (no hours).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Know {
    /// The technique, by index in the catalog's techniques.
    pub technique: u16,
    /// They know it: they can do its work and teach it.
    pub known: bool,
    /// Hours of work done beside someone who knew it, while learning it.
    pub hours: f32,
    /// When they came to know it; while not yet known, when they first heard of it.
    pub since: SimTime,
    /// How it came to them.
    pub source: KnowSource,
    /// When they last did its work (`since` if never).
    pub used: SimTime,
}

impl Person {
    /// Age in years at `t`.
    pub fn age_years(&self, t: SimTime) -> f64 {
        (t.minutes() - self.born.minutes()) as f64 / civ_core::time::MINUTES_PER_YEAR as f64
    }

    /// Where the person is at time `t` (fractional minutes since the calendar origin).
    pub fn position_at(&self, t: f64) -> (f32, f32) {
        match &self.trip {
            Some(trip) => trip.position_at(t),
            None => self.pos,
        }
    }

    /// Level of skill `skill`, 0–1 (0 if never practised).
    pub fn skill(&self, skill: usize) -> f64 {
        self.skills
            .iter()
            .find(|(s, _)| usize::from(*s) == skill)
            .map_or(0.0, |(_, l)| f64::from(*l))
    }

    /// Sets the level of skill `skill`, keeping the list sorted by skill.
    pub fn set_skill(&mut self, skill: usize, level: f64) {
        let level = level.clamp(0.0, 1.0) as f32;
        match self
            .skills
            .binary_search_by_key(&(skill as u16), |(s, _)| *s)
        {
            Ok(i) => self.skills[i].1 = level,
            Err(i) => self.skills.insert(i, (skill as u16, level)),
        }
    }

    /// What they know of technique `t`, if anything.
    pub fn know(&self, t: usize) -> Option<&Know> {
        self.knows
            .binary_search_by_key(&(t as u16), |k| k.technique)
            .ok()
            .map(|i| &self.knows[i])
    }

    /// Whether they know technique `t`: can do its work and teach it.
    pub fn knows(&self, t: usize) -> bool {
        self.know(t).is_some_and(|k| k.known)
    }

    fn know_entry(&mut self, t: usize, now: SimTime, source: KnowSource) -> &mut Know {
        let i = match self
            .knows
            .binary_search_by_key(&(t as u16), |k| k.technique)
        {
            Ok(i) => i,
            Err(i) => {
                self.knows.insert(
                    i,
                    Know {
                        technique: t as u16,
                        known: false,
                        hours: 0.0,
                        since: now,
                        source,
                        used: now,
                    },
                );
                i
            }
        };
        &mut self.knows[i]
    }

    /// They come to know technique `t` (if they did not already), from `source`. Returns whether
    /// it is new to them.
    pub fn come_to_know(&mut self, t: usize, source: KnowSource, now: SimTime) -> bool {
        let k = self.know_entry(t, now, source);
        if k.known {
            return false;
        }
        k.known = true;
        k.since = now;
        k.used = now;
        k.source = source;
        true
    }

    /// They hear of technique `t` (ADR-0008 §2's awareness), unless they already know of it.
    pub fn hear_of(&mut self, t: usize, source: KnowSource, now: SimTime) {
        self.know_entry(t, now, source);
    }

    /// `hours` of work beside `teacher` on technique `t`, which takes `learn_h` to learn. Returns
    /// whether they now know it.
    pub fn learn(
        &mut self,
        t: usize,
        hours: f64,
        teacher: PermanentId,
        learn_h: f64,
        now: SimTime,
    ) -> bool {
        let k = self.know_entry(t, now, KnowSource::Taught(teacher));
        if k.known {
            return false;
        }
        k.hours += hours.max(0.0) as f32;
        k.source = KnowSource::Taught(teacher);
        if f64::from(k.hours) + 1e-6 >= learn_h {
            k.known = true;
            k.since = now;
            k.used = now;
            return true;
        }
        false
    }

    /// They did the work of technique `t`, which they know.
    pub fn practised(&mut self, t: usize, now: SimTime) {
        if let Ok(i) = self
            .knows
            .binary_search_by_key(&(t as u16), |k| k.technique)
            && self.knows[i].known
        {
            self.knows[i].used = now;
        }
    }

    /// Keeps a receipt, dropping the oldest past [`RECEIPT_RING`].
    pub fn push_receipt(&mut self, receipt: Receipt) {
        if self.receipts.len() == RECEIPT_RING {
            self.receipts.pop_front();
        }
        self.receipts.push_back(receipt);
    }
}

/// What a household has seen of a place where a resource is gathered: the return of the hours
/// worked there, weighted by how recent they were (see [`KnownPatch::belief`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KnownPatch {
    /// The land resource, by index in the land parameters.
    pub resource: u16,
    /// The place: the centre patch of the block a trip works.
    pub patch: u32,
    /// Mean return of the remembered hours, units of the resource per person-hour.
    pub rate: f32,
    /// Hours of work behind `rate`, as of `seen_day`.
    pub hours: f32,
    /// Day of the last visit.
    pub seen_day: i64,
}

impl KnownPatch {
    /// The weight of what was seen, in hours, on `day`: evidence fades over the time the resource
    /// takes to renew, since the place will have changed by then.
    pub fn weight(&self, day: i64, renewal_days: f64) -> f64 {
        let age = (day - self.seen_day).max(0) as f64;
        f64::from(self.hours) * (-age / renewal_days.max(1.0)).exp()
    }

    /// The expected return on `day`, units per person-hour, from what was seen and what land of
    /// its kind is expected to give (`prior`). The expectation counts as one unit of the
    /// resource's worth of evidence (a gamma prior on a rate): a hunt that finds nothing says
    /// little about a place where kills are rare, while a stripped patch of plants, measured in
    /// kilograms, says a lot.
    pub fn belief(&self, prior: f64, day: i64, renewal_days: f64) -> f64 {
        let w = self.weight(day, renewal_days);
        let prior_hours = 1.0 / prior.max(1e-6);
        (1.0 + f64::from(self.rate) * w) / (prior_hours + w)
    }

    /// Adds `units` gathered in `hours` on `day` to what was seen.
    pub fn observe(&mut self, units: f64, hours: f64, day: i64, renewal_days: f64) {
        let w = self.weight(day, renewal_days);
        let total = w + hours.max(0.0);
        if total > 0.0 {
            self.rate = ((f64::from(self.rate) * w + units.max(0.0)) / total) as f32;
        }
        self.hours = total as f32;
        self.seen_day = day;
    }
}

/// What became of goods: one kind of flow in or out of a household's stores (ADR-0006 §3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Flow {
    /// Brought into the world: a founding or sent family's provisions, seed and tools.
    Brought,
    /// Gathered from the land, or harvested.
    Got,
    /// Made by a recipe.
    Made,
    /// Put into a recipe.
    Used,
    /// Eaten.
    Eaten,
    /// Spoiled in store.
    Spoiled,
    /// Burned for warmth and cooking.
    Burned,
    /// Worn away by use (tools).
    Worn,
    /// Built into a building.
    Built,
    /// Sown as seed.
    Sown,
    /// Taken out of the world by a household that left.
    Departed,
    /// Received from another household (ADR-0006 §3: a transfer, by any channel).
    Received,
    /// Given to another household.
    Given,
}

impl Flow {
    /// Every kind, in a fixed order.
    pub const ALL: [Flow; 13] = [
        Flow::Brought,
        Flow::Got,
        Flow::Made,
        Flow::Used,
        Flow::Eaten,
        Flow::Spoiled,
        Flow::Burned,
        Flow::Worn,
        Flow::Built,
        Flow::Sown,
        Flow::Departed,
        Flow::Received,
        Flow::Given,
    ];

    /// Whether goods of this kind of flow enter stores (rather than leave them).
    pub fn enters(self) -> bool {
        matches!(
            self,
            Flow::Brought | Flow::Got | Flow::Made | Flow::Received
        )
    }
}

/// Amounts of each good by kind of flow since the counters began: for the conservation check
/// and reports. Counters, not world state: never saved.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Flows {
    by: Vec<Vec<f64>>,
}

impl Flows {
    /// Counts `amount` of good `good` as `flow`.
    pub fn add(&mut self, flow: Flow, good: usize, amount: f64) {
        if amount == 0.0 || !amount.is_finite() {
            return;
        }
        if self.by.is_empty() {
            self.by = vec![Vec::new(); Flow::ALL.len()];
        }
        let row = &mut self.by[flow as usize];
        if row.len() <= good {
            row.resize(good + 1, 0.0);
        }
        row[good] += amount;
    }

    /// Counts every positive change from `before` to `after` as `up` and every negative one as
    /// `down`.
    pub fn add_changes(&mut self, before: &[f64], after: &[f64], up: Flow, down: Flow) {
        for (g, (b, a)) in before.iter().zip(after).enumerate() {
            let d = a - b;
            if d > 0.0 {
                self.add(up, g, d);
            } else if d < 0.0 {
                self.add(down, g, -d);
            }
        }
    }

    /// The amount of good `good` counted as `flow`.
    pub fn get(&self, flow: Flow, good: usize) -> f64 {
        self.by
            .get(flow as usize)
            .and_then(|row| row.get(good))
            .copied()
            .unwrap_or(0.0)
    }

    /// Adds another set of counters to these.
    pub fn absorb(&mut self, other: &Flows) {
        for flow in Flow::ALL {
            if let Some(row) = other.by.get(flow as usize) {
                for (g, v) in row.iter().enumerate() {
                    self.add(flow, g, *v);
                }
            }
        }
    }

    /// Everything counted for good `good`, in and out: the scale its balance is checked against.
    pub fn turnover(&self, good: usize) -> f64 {
        Flow::ALL.iter().map(|&f| self.get(f, good).abs()).sum()
    }

    /// The net amount of good `good` that has entered stores: what entered less what left.
    pub fn net(&self, good: usize) -> f64 {
        Flow::ALL
            .iter()
            .map(|&f| {
                let v = self.get(f, good);
                if f.enters() { v } else { -v }
            })
            .sum()
    }
}

/// People who live and eat together.
#[derive(Clone, Debug, PartialEq)]
pub struct Household {
    /// Permanent id.
    pub id: PermanentId,
    /// Members, oldest first.
    pub members: Vec<PermanentId>,
    /// Where they sleep and keep their store, metres.
    pub home: (f32, f32),
    /// Their settlement.
    pub settlement: Option<PermanentId>,
    /// Goods in store at `stores_at`, kilograms, by index in the catalog's goods.
    pub stores: Vec<f64>,
    /// When stores were last brought up to date (spoilage and firewood burned).
    pub stores_at: SimTime,
    /// Water in store at `water_at`, litres.
    pub water_l: f64,
    /// When water was last brought up to date.
    pub water_at: SimTime,
    /// Patches they have gathered in, and what they found.
    pub known: Vec<KnownPatch>,
    /// Their stores are under a roof: goods spoil at their sheltered rates. Derived from their
    /// buildings (not saved); stores are settled whenever it changes.
    pub sheltered: bool,
    /// What became of their goods since the counters began (not saved).
    pub flows: Flows,
    /// What they offer for sale, and on what terms (slice I).
    pub offers: Vec<crate::market::Offer>,
}

impl Household {
    /// Water in store at `t`, given the litres the household uses per day.
    pub fn water_at_time(&self, t: SimTime, litres_per_day: f64) -> f64 {
        let days = (t.minutes() - self.water_at.minutes()).max(0) as f64 / 1440.0;
        (self.water_l - days * litres_per_day).max(0.0)
    }

    /// Brings the water store up to `t`.
    pub fn settle_water(&mut self, t: SimTime, litres_per_day: f64) {
        self.water_l = self.water_at_time(t, litres_per_day);
        self.water_at = t;
    }

    /// The stores at `t`: every good spoils by its half-life (its sheltered one under a roof),
    /// and firewood burns at `fuel_kg_per_day(day)` kilograms on each day (the household's
    /// total). Exact for any split of time.
    pub fn stores_at_time(
        &self,
        t: SimTime,
        goods: &[GoodDef],
        fuel_kg_per_day: &dyn Fn(i64) -> f64,
    ) -> Vec<f64> {
        let (t0, t1) = (self.stores_at.minutes(), t.minutes());
        let mut out = self.stores.clone();
        if t1 <= t0 {
            return out;
        }
        let days = (t1 - t0) as f64 / MINUTES_PER_DAY as f64;
        let mut burned = burned_kg(t0, t1, fuel_kg_per_day);
        for (kg, g) in out.iter_mut().zip(goods) {
            let half_life = if self.sheltered && g.sheltered_half_life_days > 0.0 {
                g.sheltered_half_life_days
            } else {
                g.half_life_days
            };
            if half_life > 0.0 {
                *kg *= 0.5f64.powf(days / half_life);
            }
            if g.purpose == GoodUse::Fuel && burned > 0.0 {
                let take = burned.min(*kg);
                *kg -= take;
                burned -= take;
            }
        }
        out
    }

    /// Brings the stores up to `t` (see [`Household::stores_at_time`]), counting what spoiled and
    /// what burned.
    pub fn settle_stores(
        &mut self,
        t: SimTime,
        goods: &[GoodDef],
        fuel_kg_per_day: &dyn Fn(i64) -> f64,
    ) {
        if t > self.stores_at {
            let after = self.stores_at_time(t, goods, fuel_kg_per_day);
            for (g, (b, a)) in self.stores.iter().zip(&after).enumerate() {
                let lost = b - a;
                if lost > 0.0 {
                    let flow = if goods.get(g).is_some_and(|d| d.purpose == GoodUse::Fuel) {
                        Flow::Burned
                    } else {
                        Flow::Spoiled
                    };
                    self.flows.add(flow, g, lost);
                }
            }
            self.stores = after;
            self.stores_at = t;
        }
    }
}

/// Food energy ready to eat in a set of stores, kcal: all of it, and what can be eaten without a
/// fire. Food that a recipe must make ready first (grain, flour) and goods kept back (seed) are
/// not counted (see [`stock_kcal`] and [`reserve_food_kcal`]).
pub fn food_kcal(stores: &[f64], goods: &[GoodDef]) -> (f64, f64) {
    let (mut all, mut raw) = (0.0, 0.0);
    for (kg, g) in stores.iter().zip(goods) {
        if g.edible() && !g.kept_back() {
            let kcal = kg.max(0.0) * g.kcal_per_kg;
            all += kcal;
            if !g.cooked() {
                raw += kcal;
            }
        }
    }
    (all, raw)
}

/// Food energy in a set of stores, kcal, ready or not (grain and flour count): what the
/// household has to live on. Goods kept back (seed) are not counted.
pub fn stock_kcal(stores: &[f64], goods: &[GoodDef]) -> f64 {
    stores
        .iter()
        .zip(goods)
        .filter(|(_, g)| g.purpose == GoodUse::Food && !g.kept_back())
        .map(|(kg, g)| kg.max(0.0) * g.kcal_per_kg)
        .sum()
}

/// Food energy of the goods kept back (seed), kcal.
pub fn reserve_food_kcal(stores: &[f64], goods: &[GoodDef]) -> f64 {
    stores
        .iter()
        .zip(goods)
        .filter(|(_, g)| g.purpose == GoodUse::Food && g.kept_back())
        .map(|(kg, g)| kg.max(0.0) * g.kcal_per_kg)
        .sum()
}

/// Firewood in a set of stores, kilograms.
pub fn fuel_kg(stores: &[f64], goods: &[GoodDef]) -> f64 {
    stores
        .iter()
        .zip(goods)
        .filter(|(_, g)| g.purpose == GoodUse::Fuel)
        .map(|(kg, _)| kg.max(0.0))
        .sum()
}

/// Kilograms burned between minutes `t0` and `t1` at `per_day(day)` per day.
fn burned_kg(t0: i64, t1: i64, per_day: &dyn Fn(i64) -> f64) -> f64 {
    let mut total = 0.0;
    let mut t = t0;
    while t < t1 {
        let day = t.div_euclid(MINUTES_PER_DAY);
        let end = ((day + 1) * MINUTES_PER_DAY).min(t1);
        total += per_day(day) * (end - t) as f64 / MINUTES_PER_DAY as f64;
        t = end;
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::Eaten;

    #[test]
    fn trips_interpolate_between_vertices() {
        let trip = Trip {
            id: 1,
            rev: 0,
            depart: SimTime::from_minutes(100),
            points: vec![(0.0, 0.0), (10.0, 0.0), (10.0, 20.0)],
            minutes: vec![0.0, 1.0, 3.0],
        };
        assert_eq!(trip.position_at(90.0), (0.0, 0.0));
        assert_eq!(trip.position_at(100.5), (5.0, 0.0));
        assert_eq!(trip.position_at(102.0), (10.0, 10.0));
        assert_eq!(trip.position_at(500.0), (10.0, 20.0));
        assert_eq!(trip.duration_minutes(), 3.0);
    }

    #[test]
    fn water_is_used_continuously() {
        let mut h = household(Vec::new());
        assert_eq!(h.water_at_time(SimTime::from_minutes(720), 40.0), 80.0);
        h.settle_water(SimTime::from_minutes(1440 * 3), 40.0);
        assert_eq!(h.water_l, 0.0);
    }

    fn household(stores: Vec<f64>) -> Household {
        Household {
            id: PermanentId::from_raw(1).expect("non-zero"),
            members: Vec::new(),
            home: (0.0, 0.0),
            settlement: None,
            stores,
            stores_at: SimTime::ZERO,
            water_l: 100.0,
            water_at: SimTime::ZERO,
            known: Vec::new(),
            sheltered: false,
            flows: Flows::default(),
            offers: Vec::new(),
        }
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
            good("meat", GoodUse::Food, 1500.0, 3.0, Eaten::Cooked),
            good("nuts", GoodUse::Food, 5000.0, 0.0, Eaten::Raw),
            good("wood", GoodUse::Fuel, 0.0, 0.0, Eaten::Never),
        ]
    }

    #[test]
    fn a_roof_slows_the_spoiling_of_what_it_shelters() {
        let mut goods = goods();
        // Nuts that spoil with a half-life of 100 days in the open and 400 under a roof.
        goods[1].half_life_days = 100.0;
        goods[1].sheltered_half_life_days = 400.0;
        let mut h = household(vec![8.0, 8.0, 0.0]);
        let t = SimTime::from_minutes(400 * 1440);
        let open = h.stores_at_time(t, &goods, &|_| 0.0);
        h.sheltered = true;
        let roofed = h.stores_at_time(t, &goods, &|_| 0.0);
        assert!((open[1] - 0.5).abs() < 1e-9, "four half-lives: {}", open[1]);
        assert!(
            (roofed[1] - 4.0).abs() < 1e-9,
            "one half-life: {}",
            roofed[1]
        );
        assert_eq!(open[0], roofed[0], "a roof does nothing for fresh meat");
    }

    #[test]
    fn one_empty_hunt_says_little_and_one_stripped_patch_says_a_lot() {
        let seen = |units, hours| {
            let mut k = KnownPatch {
                resource: 0,
                patch: 0,
                rate: 0.0,
                hours: 0.0,
                seen_day: 100,
            };
            k.observe(units, hours, 100, 365.0);
            k
        };
        // Game: about one kill in 50 hours expected. A four-hour hunt that kills nothing lowers
        // the expectation by under a tenth.
        let hunt = seen(0.0, 4.0);
        let belief = hunt.belief(0.02, 100, 365.0);
        assert!(belief < 0.02 && belief > 0.018, "{belief}");
        // A kill in four hours makes the place look several times better.
        assert!(seen(1.0, 4.0).belief(0.02, 100, 365.0) > 0.03);
        // Plants: 0.8 kg an hour expected; four hours that bring 0.4 kg mean the place is
        // stripped.
        let plants = seen(0.4, 4.0).belief(0.8, 100, 365.0);
        assert!(plants < 0.3, "{plants}");
        // What was seen fades back to the expectation over the renewal time.
        let later = seen(0.4, 4.0).belief(0.8, 100 + 10 * 365, 365.0);
        assert!((later - 0.8).abs() < 0.01, "{later}");
        // Observations add up: two empty hunts say more than one.
        let mut twice = hunt;
        twice.observe(0.0, 4.0, 101, 365.0);
        assert!(twice.belief(0.02, 101, 365.0) < belief);
    }

    #[test]
    fn stores_spoil_by_half_life_and_burn_firewood_exactly() {
        let goods = goods();
        let per_day = |day: i64| if day < 2 { 10.0 } else { 4.0 };
        let mut h = household(vec![10.0, 2.0, 30.0]);
        let three_days = SimTime::from_minutes(3 * 1440);
        let s = h.stores_at_time(three_days, &goods, &per_day);
        assert!(
            (s[0] - 10.0 / 2.0).abs() < 1e-9,
            "meat halves in three days: {}",
            s[0]
        );
        assert_eq!(s[1], 2.0, "nuts keep");
        assert!(
            (s[2] - (30.0 - 24.0)).abs() < 1e-9,
            "10 + 10 + 4 kg burned: {}",
            s[2]
        );
        // Settling in pieces gives the same result.
        for m in [700, 1500, 2900, 3 * 1440] {
            h.settle_stores(SimTime::from_minutes(m), &goods, &per_day);
        }
        for (a, b) in h.stores.iter().zip(&s) {
            assert!((a - b).abs() < 1e-9);
        }
        // Firewood never goes below nothing.
        let later = h.stores_at_time(SimTime::from_minutes(30 * 1440), &goods, &per_day);
        assert_eq!(later[2], 0.0);
        let (all, raw) = food_kcal(&later, &goods);
        assert!(
            all > raw && raw == 10_000.0,
            "meat must be cooked; nuts need no fire"
        );
        assert_eq!(fuel_kg(&later, &goods), 0.0);
    }
}
