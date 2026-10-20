//! Shaking (M8d slice DA, ADR-0031 §4): the observer's tool puts a lateral share of weight over
//! the map, and every standing building is checked once in its lateral modes. Each event keeps
//! what it was and what each building did; a mass wall that cracked keeps no cohesion until it
//! is mended. No earthquake is natural in v1, and people cannot tell one sent from one that
//! was not.

use crate::floods::Origin;
use civ_core::{PermanentId, SimTime};

/// The least and the most lateral share of weight the observer may send (ADR-0031 §4).
pub const STRENGTH_RANGE: (f64, f64) = (0.02, 0.6);

/// The spread of one building's response about the event's strength, in log units: 11-06
/// §2.2's dispersion of about 0.3, a starting reference (ADR-0031 §4).
pub const RESPONSE_SPREAD: f64 = 0.30;

/// What one building did when the ground shook. Codes are part of saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Nothing gave way or cracked.
    Stood = 0,
    /// A mass wall cracked, and nothing fell.
    Cracked = 1,
    /// Something gave way.
    Fell = 2,
}

impl Outcome {
    /// Every outcome, in code order.
    pub const ALL: [Outcome; 3] = [Outcome::Stood, Outcome::Cracked, Outcome::Fell];

    /// Its number in saves.
    pub fn code(self) -> u8 {
        self as u8
    }

    /// The outcome numbered `code`.
    pub fn from_code(code: u8) -> Option<Outcome> {
        Outcome::ALL.get(usize::from(code)).copied()
    }
}

/// One building's response to one event.
#[derive(Clone, Debug, PartialEq)]
pub struct Shaken {
    pub building: PermanentId,
    pub household: PermanentId,
    /// Its owners' settlement then.
    pub settlement: Option<PermanentId>,
    /// Its response draw, a standard normal keyed by world, event and building.
    pub z: f32,
    /// The share of its weight it met sideways: the event's strength times `exp(0.30·z)`.
    pub cs: f32,
    pub outcome: Outcome,
    /// The groups that cracked, in its condition's order.
    pub cracked: Vec<u32>,
    /// Those inside who died when it gave way.
    pub deaths: u16,
}

/// One event: when, how strong, sent by whom, and what every building it reached did.
#[derive(Clone, Debug, PartialEq)]
pub struct Quake {
    /// Its id, from 1.
    pub id: u32,
    /// The intervention that sent it.
    pub influence: u32,
    pub at: SimTime,
    /// The lateral share of weight it put over the map.
    pub strength: f32,
    /// The settlement it was sent to, or the whole world.
    pub settlement: Option<PermanentId>,
    pub origin: Origin,
    /// Every building it reached, in the order they were checked.
    pub shaken: Vec<Shaken>,
}

/// Every event there has been. Saved.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Quakes {
    pub list: Vec<Quake>,
}

impl Quakes {
    /// Whether group `group` of building `building`, last mended at `repaired`, is cracked: some
    /// event since cracked it.
    pub fn cracked(&self, building: PermanentId, group: u32, repaired: SimTime) -> bool {
        self.list
            .iter()
            .rev()
            .take_while(|q| q.at > repaired)
            .any(|q| {
                q.shaken
                    .iter()
                    .any(|s| s.building == building && s.cracked.contains(&group))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(n: u64) -> PermanentId {
        PermanentId::from_raw(n).expect("id")
    }

    #[test]
    fn a_crack_holds_until_it_is_mended() {
        let shaken = Shaken {
            building: id(7),
            household: id(3),
            settlement: None,
            z: 0.0,
            cs: 0.2,
            outcome: Outcome::Cracked,
            cracked: vec![42],
            deaths: 0,
        };
        let quakes = Quakes {
            list: vec![Quake {
                id: 1,
                influence: 1,
                at: SimTime::from_minutes(1000),
                strength: 0.2,
                settlement: None,
                origin: Origin::recorded(vec![1]),
                shaken: vec![shaken],
            }],
        };
        assert!(quakes.cracked(id(7), 42, SimTime::from_minutes(10)));
        assert!(!quakes.cracked(id(7), 43, SimTime::from_minutes(10)));
        assert!(!quakes.cracked(id(8), 42, SimTime::from_minutes(10)));
        // Mended after it shook, it is whole again.
        assert!(!quakes.cracked(id(7), 42, SimTime::from_minutes(2000)));
        for o in Outcome::ALL {
            assert_eq!(Outcome::from_code(o.code()), Some(o));
        }
    }
}
