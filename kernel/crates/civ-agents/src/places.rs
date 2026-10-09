//! The places households know (ADR-0018 §4; M5a slice AM). Another settlement is known only by
//! contact: founded alongside it, seen within sight on a walk, kin living there, visited, or told
//! of by someone at the hearth (research 13-01 §1.1: awareness comes by traders, migrants, kin and
//! travellers; 09-16 §1.2: news waits for a carrier). Knowing a place is knowing that a settlement
//! stands there; what a household believes of it comes only from what a member saw or heard. A
//! household's own settlement is not among the places it knows.

use std::collections::BTreeMap;

use civ_core::PermanentId;

/// How a household came to know a place. Codes are part of saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PlaceHow {
    /// Its founders came to the valley alongside theirs and knew where they camped.
    Founded = 0,
    /// A member walked within sight of its homes.
    Seen = 1,
    /// Kin of a member live there.
    Kin = 2,
    /// A member went there.
    Visited = 3,
    /// Someone at the hearth told a member of it.
    Told = 4,
}

impl PlaceHow {
    /// Every way, in code order.
    pub const ALL: [PlaceHow; 5] = [
        PlaceHow::Founded,
        PlaceHow::Seen,
        PlaceHow::Kin,
        PlaceHow::Visited,
        PlaceHow::Told,
    ];

    /// Its number in saves.
    pub fn code(self) -> u8 {
        self as u8
    }

    /// The way of knowing a number in a save stands for.
    pub fn from_code(code: u8) -> Option<PlaceHow> {
        Self::ALL.get(usize::from(code)).copied()
    }

    /// How it came to be known, in words.
    pub fn words(self) -> &'static str {
        match self {
            PlaceHow::Founded => "came to the valley alongside its founders",
            PlaceHow::Seen => "seen on a walk",
            PlaceHow::Kin => "kin live there",
            PlaceHow::Visited => "visited",
            PlaceHow::Told => "told of it",
        }
    }
}

/// How households come to know other places (the people profile's `[places]` table; content
/// API 52). Design priors: the research gives no distance at which a village is seen.
#[derive(Clone, Debug, PartialEq)]
pub struct PlacesParams {
    /// Metres from a settlement's hearth within which a walk passes in sight of its homes.
    pub sight_m: f32,
    /// The chance someone tells a companion at the hearth of a place their household knows and
    /// the companion's does not (research 09-16 §2.2: 0.05–0.25 for routine news). Someone from
    /// another settlement always says where they are from.
    pub share_told: f64,
}

impl PlacesParams {
    /// The core content's values (`content/core/people/early_farmers.toml`), for tests.
    pub fn core() -> Self {
        PlacesParams {
            sight_m: 500.0,
            share_told: 0.15,
        }
    }
}

/// A place a household knows.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KnownPlace {
    /// The settlement.
    pub settlement: PermanentId,
    /// How it first came to be known.
    pub how: PlaceHow,
    /// Who told of it, or the kin who live there, if anyone.
    pub from: Option<PermanentId>,
    /// The day it was first known.
    pub first: i64,
    /// The day a member last saw it, went there or heard of it.
    pub last: i64,
    /// What a member last saw of its food there: the share of those they met who were not
    /// going hungry. None until a member has been there (slice AM, visits).
    pub food: Option<f32>,
}

/// Every household's known places, each household's in settlement order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Places {
    /// By household.
    pub known: BTreeMap<PermanentId, Vec<KnownPlace>>,
}

impl Places {
    /// The places `household` knows, in settlement order.
    pub fn of(&self, household: PermanentId) -> &[KnownPlace] {
        self.known.get(&household).map_or(&[], Vec::as_slice)
    }

    /// Whether `household` knows `settlement`.
    pub fn knows(&self, household: PermanentId, settlement: PermanentId) -> bool {
        self.of(household)
            .binary_search_by_key(&settlement, |k| k.settlement)
            .is_ok()
    }

    /// `household` learns of `settlement` on `day`, `how`, from `from`: a place known already is
    /// only known as of `day` too, keeping how it first came to be known. Whether it is new to
    /// them.
    pub fn learn(
        &mut self,
        household: PermanentId,
        settlement: PermanentId,
        day: i64,
        how: PlaceHow,
        from: Option<PermanentId>,
    ) -> bool {
        let list = self.known.entry(household).or_default();
        match list.binary_search_by_key(&settlement, |k| k.settlement) {
            Ok(i) => {
                list[i].last = list[i].last.max(day);
                false
            }
            Err(i) => {
                list.insert(
                    i,
                    KnownPlace {
                        settlement,
                        how,
                        from,
                        first: day,
                        last: day,
                        food: None,
                    },
                );
                true
            }
        }
    }

    /// People of `from` came to live in `to`: `to` knows what they knew, but its own settlement
    /// `home`. A place both knew keeps `to`'s record, known as of the later day.
    pub fn bring(&mut self, from: PermanentId, to: PermanentId, home: Option<PermanentId>) {
        let brought: Vec<KnownPlace> = self
            .of(from)
            .iter()
            .filter(|k| Some(k.settlement) != home)
            .copied()
            .collect();
        if brought.is_empty() {
            return;
        }
        let list = self.known.entry(to).or_default();
        for k in brought {
            match list.binary_search_by_key(&k.settlement, |x| x.settlement) {
                Ok(i) => list[i].last = list[i].last.max(k.last),
                Err(i) => list.insert(i, k),
            }
        }
    }

    /// `household` is no more.
    pub fn forget(&mut self, household: PermanentId) {
        self.known.remove(&household);
    }

    /// What is wrong with these records, given the settlement a household lives in (none for a
    /// missing household) and whether a settlement exists.
    pub fn problems(
        &self,
        household: impl Fn(PermanentId) -> Option<Option<PermanentId>>,
        settlement_exists: impl Fn(PermanentId) -> bool,
    ) -> Vec<String> {
        let mut out = Vec::new();
        for (&h, list) in &self.known {
            let Some(home) = household(h) else {
                out.push(format!("places known by missing household {h}"));
                continue;
            };
            if !list.windows(2).all(|w| w[0].settlement < w[1].settlement) {
                out.push(format!(
                    "household {h}'s places are not in settlement order"
                ));
            }
            for k in list {
                if Some(k.settlement) == home {
                    out.push(format!("household {h} knows its own settlement as a place"));
                }
                if !settlement_exists(k.settlement) {
                    out.push(format!(
                        "household {h} knows a missing settlement {}",
                        k.settlement
                    ));
                }
                if k.last < k.first {
                    out.push(format!(
                        "household {h} last knew {} before it first knew it",
                        k.settlement
                    ));
                }
            }
        }
        out
    }
}

/// The least distance from `p` to the walk through `points`, metres.
pub fn distance_to_walk(points: &[(f32, f32)], p: (f32, f32)) -> f32 {
    let along = |a: (f32, f32), b: (f32, f32)| {
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let len2 = dx * dx + dy * dy;
        let t = if len2 > 0.0 {
            (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / len2).clamp(0.0, 1.0)
        } else {
            0.0
        };
        (a.0 + t * dx - p.0).hypot(a.1 + t * dy - p.1)
    };
    match points {
        [] => f32::INFINITY,
        [a] => (a.0 - p.0).hypot(a.1 - p.1),
        _ => points
            .windows(2)
            .map(|w| along(w[0], w[1]))
            .fold(f32::INFINITY, f32::min),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pid(n: u64) -> PermanentId {
        PermanentId::from_raw(n).expect("non-zero")
    }

    #[test]
    fn a_place_is_learnt_once_and_known_as_of_the_last_time() {
        let mut p = Places::default();
        assert!(p.learn(pid(1), pid(20), 5, PlaceHow::Seen, None));
        assert!(!p.learn(pid(1), pid(20), 9, PlaceHow::Told, Some(pid(3))));
        assert!(p.learn(pid(1), pid(10), 7, PlaceHow::Told, Some(pid(3))));
        let k = p.of(pid(1));
        assert_eq!(k.len(), 2);
        assert_eq!(k[0].settlement, pid(10), "kept in settlement order");
        assert_eq!((k[1].how, k[1].first, k[1].last), (PlaceHow::Seen, 5, 9));
        assert!(p.knows(pid(1), pid(20)) && !p.knows(pid(2), pid(20)));
        assert!(p.problems(|_| Some(None), |_| true).is_empty());
    }

    #[test]
    fn people_who_move_bring_what_they_knew_but_where_they_now_live() {
        let mut p = Places::default();
        p.learn(pid(1), pid(20), 5, PlaceHow::Seen, None);
        p.learn(pid(1), pid(30), 6, PlaceHow::Told, Some(pid(4)));
        p.learn(pid(2), pid(20), 8, PlaceHow::Founded, None);
        p.bring(pid(1), pid(2), Some(pid(30)));
        let k = p.of(pid(2));
        assert_eq!(k.len(), 1, "their new home is no place they know");
        assert_eq!((k[0].how, k[0].first, k[0].last), (PlaceHow::Founded, 8, 8));
        p.forget(pid(1));
        assert!(p.of(pid(1)).is_empty());
        let home = |h| (h == pid(2)).then_some(Some(pid(20)));
        assert!(!p.problems(home, |_| true).is_empty());
    }

    #[test]
    fn the_distance_to_a_walk_is_to_its_nearest_stretch() {
        let walk = [(0.0, 0.0), (100.0, 0.0), (100.0, 100.0)];
        assert_eq!(distance_to_walk(&walk, (50.0, 30.0)), 30.0);
        assert_eq!(distance_to_walk(&walk, (130.0, 50.0)), 30.0);
        assert_eq!(distance_to_walk(&walk, (-40.0, 30.0)), 50.0);
        assert_eq!(distance_to_walk(&[(1.0, 1.0)], (4.0, 5.0)), 5.0);
        assert!(distance_to_walk(&[], (0.0, 0.0)).is_infinite());
        for code in 0..PlaceHow::ALL.len() as u8 {
            assert_eq!(PlaceHow::from_code(code).map(PlaceHow::code), Some(code));
        }
    }
}
