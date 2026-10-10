//! The places people work, and who else they see working them (M5c slice AT, ADR-0020 §5): a
//! patch block a gathering trip works, or a deposit dug. Each day's work is logged as it is done
//! and folded in at the day's end: a household counts a day at a place however many of its people
//! went, the food they got there, and, for each other settlement whose people worked the same
//! place that day, a day it saw outsiders there. What a household holds fades by the content's
//! half-life, so what it saw long ago weighs little; nobody holds what another household saw.

use std::collections::BTreeMap;

use civ_core::PermanentId;

/// A place people gather from or dig at.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Place {
    /// The block a gathering trip works, by its centre patch (as `KnownPatch::patch`).
    Patch(u32),
    /// A deposit, by its permanent id.
    Deposit(PermanentId),
}

impl Place {
    /// Its kind and number in saves: 0 a patch, 1 a deposit.
    pub fn code(self) -> (u8, u64) {
        match self {
            Place::Patch(p) => (0, u64::from(p)),
            Place::Deposit(d) => (1, d.get()),
        }
    }

    /// The place saved as `kind` and `id`.
    pub fn from_code(kind: u8, id: u64) -> Option<Place> {
        match kind {
            0 => u32::try_from(id).ok().map(Place::Patch),
            1 => PermanentId::from_raw(id).map(Place::Deposit),
            _ => None,
        }
    }
}

/// Someone of `household`, living in `settlement`, worked `place` on `day` and got `kcal` of food
/// there (none at a deposit); `person` is who, when known (a save of schema 62 did not say).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Worked {
    pub place: Place,
    pub day: i64,
    pub household: PermanentId,
    pub settlement: PermanentId,
    pub kcal: f32,
    pub person: Option<PermanentId>,
}

/// People of more than one settlement worked `place` on `day` (M5c slice AT, step two): what
/// they saw of each other there. `by` is in household order.
#[derive(Clone, Debug, PartialEq)]
pub struct Meeting {
    pub place: Place,
    pub day: i64,
    pub by: Vec<Worked>,
}

/// Outsiders a household saw at a place: people of `settlement` worked it on `days` of the days
/// its own people did (fading), the last on `last`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Outsiders {
    pub settlement: PermanentId,
    pub days: f32,
    pub last: i64,
}

/// What a household holds of a place its people work.
#[derive(Clone, Debug, PartialEq)]
pub struct PlaceUse {
    pub place: Place,
    /// Days its people worked there, fading.
    pub days: f32,
    /// Food they got there, kcal, fading.
    pub kcal: f32,
    /// Those of other settlements seen there, in settlement order.
    pub outsiders: Vec<Outsiders>,
    /// The day the fading was reckoned to.
    pub day: i64,
}

impl PlaceUse {
    fn fade_to(&mut self, day: i64, half_life_days: f64) {
        let f = fading(day - self.day, half_life_days);
        self.days *= f;
        self.kcal *= f;
        for o in &mut self.outsiders {
            o.days *= f;
        }
        self.day = self.day.max(day);
    }
}

/// What is left after `days` of a half-life of `half_life_days`.
fn fading(days: i64, half_life_days: f64) -> f32 {
    if days <= 0 || half_life_days <= 0.0 {
        return 1.0;
    }
    0.5f64.powf(days as f64 / half_life_days) as f32
}

/// What a fading sum with half-life `half_life_days` comes to over a year, as a steady rate: a
/// steady `r` a day sums to about `r · half / ln 2`, so a year's is the sum times `365 ln 2 / half`.
pub fn per_year(sum: f64, half_life_days: f64) -> f64 {
    if half_life_days <= 0.0 {
        return sum;
    }
    sum * 365.0 * std::f64::consts::LN_2 / half_life_days
}

/// Days worked below which a place a household no longer sees outsiders at is let go.
const LET_GO_DAYS: f32 = 0.05;

/// Every household's places, and today's work not yet folded in.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Uses {
    /// By household, each in place order.
    pub households: BTreeMap<PermanentId, Vec<PlaceUse>>,
    /// Work done since the last fold, in the order done.
    pub today: Vec<Worked>,
}

impl Uses {
    /// Someone worked a place.
    pub fn worked(&mut self, w: Worked) {
        self.today.push(w);
    }

    /// Folds in the work logged since the last fold (ADR-0020 §5): per day and place, each
    /// household that worked it counts the day once and the food its people got, and a day of
    /// outsiders for each other settlement whose people worked it that day. Returns where people
    /// of more than one settlement met, in day and place order.
    pub fn fold(&mut self, half_life_days: f64) -> Vec<Meeting> {
        let mut meetings = Vec::new();
        if self.today.is_empty() {
            return meetings;
        }
        let mut today = std::mem::take(&mut self.today);
        today.sort_by(|a, b| {
            (a.day, a.place, a.household)
                .cmp(&(b.day, b.place, b.household))
                .then(a.settlement.cmp(&b.settlement))
        });
        let mut i = 0;
        while i < today.len() {
            let (day, place) = (today[i].day, today[i].place);
            let end = today[i..]
                .iter()
                .position(|w| (w.day, w.place) != (day, place))
                .map_or(today.len(), |n| i + n);
            let group = &today[i..end];
            let mut settlements: Vec<PermanentId> = group.iter().map(|w| w.settlement).collect();
            settlements.sort_unstable();
            settlements.dedup();
            if settlements.len() > 1 {
                meetings.push(Meeting {
                    place,
                    day,
                    by: group.to_vec(),
                });
            }
            let mut j = 0;
            while j < group.len() {
                let (household, settlement) = (group[j].household, group[j].settlement);
                let mut k = j;
                let mut kcal = 0.0f32;
                while k < group.len() && group[k].household == household {
                    kcal += group[k].kcal;
                    k += 1;
                }
                let list = self.households.entry(household).or_default();
                let at = match list.binary_search_by(|u| u.place.cmp(&place)) {
                    Ok(at) => at,
                    Err(at) => {
                        list.insert(
                            at,
                            PlaceUse {
                                place,
                                days: 0.0,
                                kcal: 0.0,
                                outsiders: Vec::new(),
                                day,
                            },
                        );
                        at
                    }
                };
                let u = &mut list[at];
                u.fade_to(day, half_life_days);
                u.days += 1.0;
                u.kcal += kcal;
                for &s in settlements.iter().filter(|&&s| s != settlement) {
                    match u.outsiders.binary_search_by(|o| o.settlement.cmp(&s)) {
                        Ok(o) => {
                            u.outsiders[o].days += 1.0;
                            u.outsiders[o].last = u.outsiders[o].last.max(day);
                        }
                        Err(o) => u.outsiders.insert(
                            o,
                            Outsiders {
                                settlement: s,
                                days: 1.0,
                                last: day,
                            },
                        ),
                    }
                }
                j = k;
            }
            i = end;
        }
        meetings
    }

    /// The places household `household` saw people of another settlement working on day `since`
    /// or later, each with those settlements.
    pub fn outsiders_seen(
        &self,
        household: PermanentId,
        since: i64,
    ) -> impl Iterator<Item = (&PlaceUse, PermanentId)> + '_ {
        self.households
            .get(&household)
            .into_iter()
            .flatten()
            .flat_map(move |u| {
                u.outsiders
                    .iter()
                    .filter(move |o| o.last >= since)
                    .map(move |o| (u, o.settlement))
            })
    }

    /// What household `household` holds of `place`, faded to `day`.
    pub fn held(
        &self,
        household: PermanentId,
        place: Place,
        day: i64,
        half_life_days: f64,
    ) -> Option<PlaceUse> {
        let list = self.households.get(&household)?;
        let at = list.binary_search_by(|u| u.place.cmp(&place)).ok()?;
        let mut u = list[at].clone();
        u.fade_to(day, half_life_days);
        Some(u)
    }

    /// Lets go of households no more (`here` says who is), and of places faded to almost
    /// nothing where no outsiders were seen in the past `memory_days`, reckoned on `day`.
    pub fn prune(
        &mut self,
        day: i64,
        half_life_days: f64,
        memory_days: i64,
        here: impl Fn(PermanentId) -> bool,
    ) {
        self.households.retain(|&h, _| here(h));
        for list in self.households.values_mut() {
            for u in list.iter_mut() {
                u.fade_to(day, half_life_days);
                u.outsiders.retain(|o| day - o.last <= memory_days);
            }
            list.retain(|u| u.days >= LET_GO_DAYS || !u.outsiders.is_empty());
        }
        self.households.retain(|_, list| !list.is_empty());
    }

    /// What is wrong with the record, if anything: places out of order or repeated, settlements
    /// out of order or repeated, or counts that are not finite or below zero.
    pub fn problems(&self) -> Vec<String> {
        let mut out = Vec::new();
        for (h, list) in &self.households {
            if list.windows(2).any(|w| w[0].place >= w[1].place) {
                out.push(format!(
                    "household {h}'s places are out of order or repeated"
                ));
            }
            for u in list {
                let bad = |x: f32| !x.is_finite() || x < 0.0;
                if bad(u.days) || bad(u.kcal) || u.outsiders.iter().any(|o| bad(o.days)) {
                    out.push(format!(
                        "household {h} holds a count of {:?} below zero",
                        u.place
                    ));
                }
                if u.outsiders
                    .windows(2)
                    .any(|w| w[0].settlement >= w[1].settlement)
                {
                    out.push(format!(
                        "household {h}'s outsiders at {:?} are out of order or repeated",
                        u.place
                    ));
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(n: u64) -> PermanentId {
        PermanentId::from_raw(n).expect("non-zero")
    }

    fn work(place: Place, day: i64, household: u64, settlement: u64, kcal: f32) -> Worked {
        Worked {
            place,
            day,
            household: id(household),
            settlement: id(settlement),
            kcal,
            person: Some(id(household * 100)),
        }
    }

    #[test]
    fn a_day_at_a_place_counts_once_and_outsiders_that_day_are_seen() {
        let mut u = Uses::default();
        let fish = Place::Patch(7);
        // Two of household 1 (settlement 10) fish patch 7 on day 5; one of household 2
        // (settlement 20) fishes it too; household 1 digs deposit 99 alone.
        u.worked(work(fish, 5, 1, 10, 300.0));
        u.worked(work(fish, 5, 1, 10, 200.0));
        u.worked(work(fish, 5, 2, 20, 100.0));
        u.worked(work(Place::Deposit(id(99)), 5, 1, 10, 0.0));
        let met = u.fold(180.0);
        assert!(u.today.is_empty());
        // The two settlements met at the fishing place, and nowhere else.
        assert_eq!(met.len(), 1);
        assert_eq!((met[0].place, met[0].day, met[0].by.len()), (fish, 5, 3));
        let one = u.held(id(1), fish, 5, 180.0).expect("held");
        assert_eq!((one.days, one.kcal), (1.0, 500.0));
        assert_eq!(
            one.outsiders,
            vec![Outsiders {
                settlement: id(20),
                days: 1.0,
                last: 5
            }]
        );
        let two = u.held(id(2), fish, 5, 180.0).expect("held");
        assert_eq!(two.outsiders[0].settlement, id(10));
        let dug = u
            .held(id(1), Place::Deposit(id(99)), 5, 180.0)
            .expect("held");
        assert!(dug.outsiders.is_empty());
        let seen: Vec<_> = u
            .outsiders_seen(id(1), 0)
            .map(|(p, s)| (p.place, s))
            .collect();
        assert_eq!(seen, vec![(fish, id(20))]);
        assert!(u.outsiders_seen(id(1), 6).next().is_none());
        assert!(u.problems().is_empty());
    }

    #[test]
    fn what_is_held_fades_and_is_let_go() {
        let mut u = Uses::default();
        u.worked(work(Place::Patch(3), 0, 1, 10, 100.0));
        u.fold(100.0);
        // A half-life later, half is left.
        let h = u.held(id(1), Place::Patch(3), 100, 100.0).expect("held");
        assert!((h.days - 0.5).abs() < 1e-6 && (h.kcal - 50.0).abs() < 1e-3);
        // Long after, it is let go; so is a household no more.
        u.prune(2_000, 100.0, 365, |_| true);
        assert!(u.households.is_empty());
        u.worked(work(Place::Patch(3), 2_000, 1, 10, 100.0));
        u.fold(100.0);
        u.prune(2_000, 100.0, 365, |h| h != id(1));
        assert!(u.households.is_empty());
    }

    #[test]
    fn a_place_where_outsiders_were_seen_lately_is_kept_however_faded() {
        let mut u = Uses::default();
        u.worked(work(Place::Patch(4), 0, 1, 10, 10.0));
        u.worked(work(Place::Patch(4), 0, 2, 20, 10.0));
        u.fold(10.0);
        u.prune(300, 10.0, 365, |_| true);
        assert_eq!(u.outsiders_seen(id(1), 0).count(), 1);
        u.prune(400, 10.0, 365, |_| true);
        assert!(u.households.is_empty());
    }

    #[test]
    fn places_round_trip_their_codes() {
        for p in [
            Place::Patch(0),
            Place::Patch(u32::MAX),
            Place::Deposit(id(5)),
        ] {
            let (k, n) = p.code();
            assert_eq!(Place::from_code(k, n), Some(p));
        }
        assert_eq!(Place::from_code(2, 1), None);
        assert_eq!(Place::from_code(1, 0), None);
    }
}
