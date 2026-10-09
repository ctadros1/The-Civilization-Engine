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
    /// The household lived there before it moved (M5a slice AN).
    Lived = 5,
}

impl PlaceHow {
    /// Every way, in code order.
    pub const ALL: [PlaceHow; 6] = [
        PlaceHow::Founded,
        PlaceHow::Seen,
        PlaceHow::Kin,
        PlaceHow::Visited,
        PlaceHow::Told,
        PlaceHow::Lived,
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
            PlaceHow::Lived => "lived there",
        }
    }
}

/// How households come to know other places, and what a visit to one is worth (the people
/// profile's `[places]` table; content API 52, the visit's terms 53). Design priors: the
/// research gives no distance at which a village is seen, nor how much people value a visit.
#[derive(Clone, Debug, PartialEq)]
pub struct PlacesParams {
    /// Metres from a settlement's hearth within which a walk passes in sight of its homes.
    pub sight_m: f32,
    /// The chance someone tells a companion at the hearth of a place their household knows and
    /// the companion's does not (research 09-16 §2.2: 0.05–0.25 for routine news). Someone from
    /// another settlement always says where they are from.
    pub share_told: f64,
    /// Points a visit to a settlement is worth for each parent, child, brother or sister living
    /// there.
    pub w_kin: f64,
    /// Points a visit is worth at most for those the visitor knows there: `w_ties · s / (1 + s)`
    /// for `s` their familiarity and warmth with them, summed.
    pub w_ties: f64,
    /// Points a visit is worth to an unpartnered adult who looked for a partner at home within
    /// `seek_days` and found nobody (research 04-08 §1.1: partners come from those one meets,
    /// neighbouring settlements among them).
    pub w_seek: f64,
    /// Days a search at home that found nobody stays a reason to look elsewhere.
    pub seek_days: i64,
    /// Days after a member of the household was there over which the wish to go again grows
    /// back to the whole of what those there are worth.
    pub revisit_days: f64,
}

impl PlacesParams {
    /// The core content's values (`content/core/people/early_farmers.toml`), for tests.
    pub fn core() -> Self {
        PlacesParams {
            sight_m: 500.0,
            share_told: 0.15,
            w_kin: 6.0,
            w_ties: 4.0,
            w_seek: 6.0,
            seek_days: 180,
            revisit_days: 30.0,
        }
    }

    /// What those `person` would see at a settlement are worth to them: `kin` close kin living
    /// there, `known` their familiarity and warmth with those they know there, summed, and
    /// whether they look for a partner there; `since` days after a member of their household
    /// was last there (None: never), the wish to go again having grown back that far.
    pub fn company_points(&self, kin: usize, known: f64, seeking: bool, since: Option<i64>) -> f64 {
        let known = known.max(0.0);
        let again = since.map_or(1.0, |d| {
            (d as f64 / self.revisit_days.max(1e-9)).clamp(0.0, 1.0)
        });
        again
            * (self.w_kin * kin as f64
                + self.w_ties * known / (1.0 + known)
                + if seeking { self.w_seek } else { 0.0 })
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
    /// The day a member was last there (slice AM, visits).
    pub visited: Option<i64>,
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
                        visited: None,
                    },
                );
                true
            }
        }
    }

    /// A member of `household` went to `settlement` on `day` and saw that a share `fed` of
    /// those of it they met were not going hungry (None: they met none of them).
    pub fn went(
        &mut self,
        household: PermanentId,
        settlement: PermanentId,
        day: i64,
        fed: Option<f32>,
    ) {
        if let Some(list) = self.known.get_mut(&household)
            && let Ok(i) = list.binary_search_by_key(&settlement, |k| k.settlement)
        {
            list[i].visited = Some(day);
            if let Some(fed) = fed {
                list[i].food = Some(fed.clamp(0.0, 1.0));
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
                Ok(i) => {
                    list[i].last = list[i].last.max(k.last);
                    list[i].visited = list[i].visited.max(k.visited);
                }
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

/// What passed between one settlement and another in a year (M5a slice AM): counted as it
/// happens, kept forever (a few numbers a pair a year).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Contact {
    /// Visits by people of the first to the hearth of the second.
    pub visits: u32,
    /// Minutes they spent at it.
    pub minutes: u64,
    /// Marriages that took someone of the first to live in the second (slice AM, step three).
    pub marriages: u32,
    /// People who moved from the first to live in the second with their household (slice AN).
    pub moved: u32,
    /// Purchases people of the first made at sellers' doors in the second (M5b slice AP), and
    /// their trips there that bought nothing, by why (see [`crate::reports::Missed`]).
    pub bought: u32,
    pub missed: [u32; crate::reports::Missed::COUNT],
}

/// Contacts between settlements, by year (from 0), from-settlement and to-settlement.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Contacts {
    /// By `(year, from, to)`.
    pub years: BTreeMap<(i64, PermanentId, PermanentId), Contact>,
}

impl Contacts {
    /// Someone of `from` spent `minutes` at `to`'s hearth in `year`.
    pub fn visit(&mut self, year: i64, from: PermanentId, to: PermanentId, minutes: u32) {
        let c = self.years.entry((year, from, to)).or_default();
        c.visits += 1;
        c.minutes += u64::from(minutes);
    }

    /// Someone of `from` married into `to` in `year`.
    pub fn marriage(&mut self, year: i64, from: PermanentId, to: PermanentId) {
        self.years.entry((year, from, to)).or_default().marriages += 1;
    }

    /// A household of `people` moved from `from` to `to` in `year`.
    pub fn moved(&mut self, year: i64, from: PermanentId, to: PermanentId, people: u32) {
        self.years.entry((year, from, to)).or_default().moved += people;
    }

    /// Someone of `from` bought at a seller's door in `to` in `year` (M5b slice AP).
    pub fn bought(&mut self, year: i64, from: PermanentId, to: PermanentId) {
        self.years.entry((year, from, to)).or_default().bought += 1;
    }

    /// Someone of `from` went to buy at a seller's door in `to` in `year` and bought nothing, for
    /// `why`.
    pub fn missed(
        &mut self,
        year: i64,
        from: PermanentId,
        to: PermanentId,
        why: crate::reports::Missed,
    ) {
        self.years.entry((year, from, to)).or_default().missed[why as usize] += 1;
    }

    /// The contacts of `year`, in `(from, to)` order.
    pub fn of_year(&self, year: i64) -> impl Iterator<Item = (PermanentId, PermanentId, Contact)> {
        let lo = PermanentId::from_raw(1).expect("non-zero");
        self.years
            .range((year, lo, lo)..)
            .take_while(move |((y, _, _), _)| *y == year)
            .map(|(&(_, a, b), &c)| (a, b, c))
    }
}

/// What moving to another settlement is worth to a household, against staying (the people
/// profile's `[moving]` table; content API 54; ADR-0018 §5). Each term is in points, as the
/// household's other choices are; all are design priors (research 05-06 §5.1 gives the terms and
/// no values).
#[derive(Clone, Debug, PartialEq)]
pub struct MovingParams {
    /// For each close kin of a member living there, less each living at home (outside the
    /// household).
    pub w_kin: f64,
    /// At most this for those its members know there, less the same for those they know at home
    /// (`w_ties · s / (1 + s)` of each sum).
    pub w_ties: f64,
    /// For the share of those of the place a member last saw not going hungry, less the share at
    /// home now.
    pub w_fed: f64,
    /// For the most keenly felt grievance a member holds (0–1).
    pub w_grievance: f64,
    /// Against the harvest its fields here should bring, over a year's need (0–1): what it gives
    /// up.
    pub w_stake: f64,
    /// Against the work of making a new home and breaking new ground before the first harvest.
    pub cost: f64,
    /// Reviews running a place must win before the household moves there (research 10-01 §2.3:
    /// 1–3), outside an emergency.
    pub reviews: u32,
}

impl MovingParams {
    /// The core content's values, for tests.
    pub fn core() -> Self {
        MovingParams {
            w_kin: 2.0,
            w_ties: 2.0,
            w_fed: 4.0,
            w_grievance: 3.0,
            w_stake: 3.0,
            cost: 2.0,
            reviews: 2,
        }
    }
}

/// What founding a settlement of its own is worth to a household, and what a coalition must hold
/// to go (the people profile's `[founding]` table; content API 55; M5a slice AO, research 10-01
/// §1.5, §2.3, §5.2; 05-06 §1.3). All are design priors; the terms it shares with moving are
/// [`MovingParams`]'.
#[derive(Clone, Debug, PartialEq)]
pub struct FoundingParams {
    /// Against the work of breaking every field and building where there is no hearth, store or
    /// neighbour yet, besides moving's `cost`.
    pub cost: f64,
    /// The share of the believed yield a site is forecast at: a low percentile (10-01 §2.3: the
    /// 10th–30th).
    pub yield_share: f64,
    /// The farthest walk from home a site may lie, hours (05-06 §1.3: a daughter near its parent
    /// can rely on it).
    pub walk_hours: f64,
    /// Sites weighed at a review, of those a member has walked (10-01 §2.3: 16–64 bundles).
    pub candidates: u32,
    /// Months of food beyond the first harvest a coalition must hold to go (10-01 §2.3: 1–3).
    pub buffer_months: f64,
    /// Hours a day an adult can break new ground, for whether the first crop can be sown this
    /// year.
    pub work_h_per_day: f64,
}

impl FoundingParams {
    /// The core content's values, for tests.
    pub fn core() -> Self {
        FoundingParams {
            cost: 1.0,
            yield_share: 0.7,
            walk_hours: 2.0,
            candidates: 16,
            buffer_months: 2.0,
            work_h_per_day: 6.0,
        }
    }
}

/// What became of a coalition gathered to found a settlement (M5a slice AO). Codes are part of
/// saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoalitionFate {
    /// Still gathering, or waiting until it holds enough to go.
    Gathering = 0,
    /// It went and founded its settlement.
    Founded = 1,
    /// Its organizer gave the plan up.
    Dissolved = 2,
}

impl CoalitionFate {
    /// Every fate, in code order.
    pub const ALL: [CoalitionFate; 3] = [
        CoalitionFate::Gathering,
        CoalitionFate::Founded,
        CoalitionFate::Dissolved,
    ];

    /// The fate numbered `code`.
    pub fn from_code(code: u8) -> Option<CoalitionFate> {
        CoalitionFate::ALL.get(usize::from(code)).copied()
    }
}

/// What a coalition lacked when its time came to go. Codes are part of saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lacking {
    /// Nothing: it went, or its time has not come.
    Nothing = 0,
    /// Food to the first harvest and the months beyond.
    Food = 1,
    /// Seed for the ground it needs.
    Seed = 2,
}

impl Lacking {
    /// Every case, in code order.
    pub const ALL: [Lacking; 3] = [Lacking::Nothing, Lacking::Food, Lacking::Seed];

    /// The case numbered `code`.
    pub fn from_code(code: u8) -> Option<Lacking> {
        Lacking::ALL.get(usize::from(code)).copied()
    }
}

/// Households gathered to found a settlement of their own (M5a slice AO; research 05-06 §1.3,
/// 10-01 §1.5, §5.2): the organizing household's plan, the households that would go with it at
/// its last review, and what became of it. A record is kept for ever.
#[derive(Clone, Debug, PartialEq)]
pub struct Coalition {
    /// Its number, from 1, in the order gathered.
    pub id: u32,
    /// The household that organizes it (as it was when it last reviewed), and the person it is
    /// named for: that household's eldest when it began.
    pub organizer: PermanentId,
    pub named: Option<PermanentId>,
    /// The settlement they would leave.
    pub from: PermanentId,
    /// Where they mean to found, metres.
    pub site: (f32, f32),
    /// The day it began to gather, and the reviews running its plan has won.
    pub formed: i64,
    pub reviews: u32,
    /// The households that would go at its last review, the organizer first, and their people.
    pub members: Vec<PermanentId>,
    pub people: u32,
    /// What it lacked when last its time came, if anything.
    pub lacking: Lacking,
    /// What became of it, the day it ended (founded or given up), and the settlement founded.
    pub fate: CoalitionFate,
    pub ended: Option<i64>,
    pub settlement: Option<PermanentId>,
}

/// A household's leaning toward moving: the place that last won its review, and how many
/// reviews running it has won.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Leaning {
    /// The settlement.
    pub settlement: PermanentId,
    /// Reviews running it has won.
    pub reviews: u32,
    /// The day of the last of them.
    pub day: i64,
}

/// Whether a couple from two settlements settles with or beside household `a` rather than `b`
/// (M5a slice AM; ADR-0018 §5: where to live follows land and room, never sex): the one with
/// more land worked per member, then the better housed (its best dwelling's stage, roofed ones
/// first), then by `draw` (0–1) between households alike.
pub fn settles_with_first(a: (f64, u8), b: (f64, u8), draw: f64) -> bool {
    if a.0 != b.0 {
        return a.0 > b.0;
    }
    if a.1 != b.1 {
        return a.1 > b.1;
    }
    draw < 0.5
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
    fn a_visit_is_worth_kin_and_those_known_there_and_less_soon_after_the_last() {
        let pp = PlacesParams::core();
        assert_eq!(pp.company_points(0, 0.0, false, None), 0.0);
        assert_eq!(pp.company_points(2, 0.0, false, None), 2.0 * pp.w_kin);
        // Those known there saturate: half of `w_ties` at familiarity and warmth summing to 1.
        assert_eq!(pp.company_points(0, 1.0, false, None), pp.w_ties / 2.0);
        assert!(pp.company_points(0, 100.0, false, None) < pp.w_ties);
        assert_eq!(pp.company_points(0, 0.0, true, None), pp.w_seek);
        // The wish to go again grows back over `revisit_days`.
        let half = (pp.revisit_days / 2.0) as i64;
        let full = pp.company_points(1, 0.0, false, None);
        assert_eq!(pp.company_points(1, 0.0, false, Some(0)), 0.0);
        assert!((pp.company_points(1, 0.0, false, Some(half)) - full / 2.0).abs() < 1e-9);
        assert_eq!(pp.company_points(1, 0.0, false, Some(365)), full);
    }

    #[test]
    fn a_couple_from_two_settlements_goes_where_there_is_land_then_room_then_by_lot() {
        // More land a member wins over a better home.
        assert!(settles_with_first((0.4, 1), (0.2, 15), 0.9));
        assert!(!settles_with_first((0.2, 15), (0.4, 1), 0.1));
        // Alike in land, the better housed.
        assert!(settles_with_first((0.3, 15), (0.3, 3), 0.9));
        // Alike in both, the lot.
        assert!(settles_with_first((0.3, 3), (0.3, 3), 0.2));
        assert!(!settles_with_first((0.3, 3), (0.3, 3), 0.7));
    }

    #[test]
    fn contacts_are_counted_by_year_and_direction() {
        let mut c = Contacts::default();
        c.visit(1, pid(10), pid(20), 90);
        c.visit(1, pid(10), pid(20), 60);
        c.visit(1, pid(20), pid(10), 90);
        c.visit(2, pid(10), pid(20), 90);
        c.marriage(1, pid(20), pid(10));
        let year: Vec<_> = c.of_year(1).collect();
        assert_eq!(year.len(), 2);
        assert_eq!((year[0].0, year[0].1), (pid(10), pid(20)));
        assert_eq!((year[0].2.visits, year[0].2.minutes), (2, 150));
        assert_eq!((year[1].2.visits, year[1].2.marriages), (1, 1));
        assert_eq!(c.of_year(2).count(), 1);
        assert_eq!(c.of_year(3).count(), 0);
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
