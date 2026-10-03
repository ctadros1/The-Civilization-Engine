//! People, households, activities and trips (ADR-0003).

use std::collections::VecDeque;

use civ_core::{PermanentId, SimTime};

use crate::history::Receipt;
use crate::needs::Sex;

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
    /// Food, kcal.
    pub food_kcal: f32,
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

    /// Keeps a receipt, dropping the oldest past [`RECEIPT_RING`].
    pub fn push_receipt(&mut self, receipt: Receipt) {
        if self.receipts.len() == RECEIPT_RING {
            self.receipts.pop_front();
        }
        self.receipts.push_back(receipt);
    }
}

/// What a household believes about a patch it has gathered in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KnownPatch {
    /// The patch.
    pub patch: u32,
    /// Remembered gathering return, units per person-hour.
    pub rate: f32,
    /// Day of the last visit.
    pub seen_day: i64,
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
    /// Food in store, kcal.
    pub food_kcal: f64,
    /// Water in store at `water_at`, litres.
    pub water_l: f64,
    /// When water was last brought up to date.
    pub water_at: SimTime,
    /// Patches they have gathered in, and what they found.
    pub known: Vec<KnownPatch>,
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
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let mut h = Household {
            id: PermanentId::from_raw(1).expect("non-zero"),
            members: Vec::new(),
            home: (0.0, 0.0),
            settlement: None,
            food_kcal: 0.0,
            water_l: 100.0,
            water_at: SimTime::ZERO,
            known: Vec::new(),
        };
        assert_eq!(h.water_at_time(SimTime::from_minutes(720), 40.0), 80.0);
        h.settle_water(SimTime::from_minutes(1440 * 3), 40.0);
        assert_eq!(h.water_l, 0.0);
    }
}
