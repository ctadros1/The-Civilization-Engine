//! Bottlenecks for walkers (M8b slice CN; ADR-0029 §4; the transport brief §1.6; research 12-05
//! §2.1, §4.2 and 12-06 §7). Walkers pass through one another on open ground, as they always
//! have; only at a **facility** (an open crossing's deck, an open gate) does passage take its
//! turn. A facility passes walkers through its **files**, each one walker every
//! [`headway_s`] seconds (Weidmann's peak of 1.2 people a metre a second over a file of 0.6 m,
//! 12-05 §2.1); a deck too narrow for two files is one file shared by both directions, a wider
//! one gives each direction half its files.
//!
//! A walk is planned free-flow as before. Its arrival at a facility is an event; there the walker
//! passes at the facility's first free slot in its direction at or after their arrival, and a
//! wait of a minute or more re-times the rest of the walk as a new revision of the trip (ADR-0003
//! §2). Events fall on whole minutes, so arrivals within one minute are taken in the events'
//! keyed order: a facility keeps the slots it gave in the last [`SLOT_MEMORY_S`] seconds and
//! fits each walker into the first gap at or after their own arrival second, so one taken a
//! little later in that order is not made to wait for one who came after them, unless their
//! slots would overlap. A facility's unused capacity never accumulates: no slot is earlier than
//! its walker's arrival (12-06 §7.3). The walkers ahead beyond its waiting
//! space ([`APPROACH_SPACE`]) stand short of the approach; they are counted, nobody vanishes.
//!
//! What it comes to is kept two ways: each facility's passages and waits by hour of the day,
//! fading over about [`TALLY_MEMORY_DAYS`] days (the Traffic lens), and each household's waits
//! by facility, fading as its wades do (`crate::fords`), for its later choices (ADR-0029 §6).

use std::collections::BTreeMap;

use civ_core::PermanentId;

use crate::uses::fading;

/// People a metre of passage width passes a second at most (research 12-05 §2.1).
pub const SPECIFIC_FLOW: f64 = 1.2;
/// The width one file of walkers takes, metres (a design prior).
pub const FILE_M: f64 = 0.6;
/// The opening of a gate in a palisade, metres (a design prior: a sector is drawn whole, but its
/// gate is a doorway in it, about 2-3 m).
pub const GATE_OPENING_M: f64 = 2.5;
/// Walkers that can wait at a facility's approach before the next stands short of it (a design
/// prior: about 8 m of path at a walker every 0.8 m).
pub const APPROACH_SPACE: u32 = 10;
/// Days a facility's tallies remember: each passage counts `e^(−age/30)`.
pub const TALLY_MEMORY_DAYS: f64 = 30.0;
/// The direction of a one-file facility, shared by both ways.
pub const SHARED: u8 = 2;
/// Seconds in a minute.
const MINUTE_S: f64 = 60.0;
/// Seconds of slots a facility keeps behind the latest arrival: twice the events' minute.
pub const SLOT_MEMORY_S: f64 = 2.0 * MINUTE_S;

/// A place walkers take their turn. Codes are part of saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Facility {
    /// An open crossing, by its permanent id.
    Crossing(PermanentId),
    /// An open gate: its enclosure and the sector it is.
    Gate { enclosure: PermanentId, sector: u16 },
}

impl Facility {
    /// Its code in saves: kind, id and sector.
    pub fn code(self) -> (u8, u64, u16) {
        match self {
            Facility::Crossing(id) => (0, id.get(), 0),
            Facility::Gate { enclosure, sector } => (1, enclosure.get(), sector),
        }
    }

    /// The facility with code `code`.
    pub fn from_code(kind: u8, id: u64, sector: u16) -> Option<Facility> {
        let id = PermanentId::from_raw(id)?;
        match kind {
            0 => Some(Facility::Crossing(id)),
            1 => Some(Facility::Gate {
                enclosure: id,
                sector,
            }),
            _ => None,
        }
    }
}

/// How a passage `width_m` wide is walked: whether its one file is shared by both directions, and
/// how many files each direction (or the shared one) has.
pub fn files(width_m: f64) -> (bool, u32) {
    let n = (width_m / FILE_M).floor().max(1.0) as u32;
    if n < 2 { (true, 1) } else { (false, n / 2) }
}

/// Seconds between walkers through `files` files.
pub fn headway_s(files: u32) -> f64 {
    1.0 / (SPECIFIC_FLOW * FILE_M * f64::from(files.max(1)))
}

/// One direction of a facility, and the slots it has given lately, seconds from the calendar
/// origin, in order.
#[derive(Clone, Debug, PartialEq)]
pub struct Server {
    pub facility: Facility,
    pub dir: u8,
    pub slots: Vec<f64>,
}

impl Server {
    /// The first slot at or after `at_s` at least `headway_s` from every slot given, and how many
    /// slots given lie between: the walkers ahead.
    fn fit(&self, at_s: f64, headway_s: f64) -> (f64, u32) {
        let mut t = at_s;
        let from = self.slots.partition_point(|&s| s <= at_s - headway_s);
        for &s in &self.slots[from..] {
            if s >= t + headway_s {
                break;
            }
            t = t.max(s + headway_s);
        }
        let ahead = self.slots[from..]
            .iter()
            .filter(|&&s| s >= at_s && s < t)
            .count() as u32;
        (t, ahead)
    }
}

/// What a facility gave one walker.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Passage {
    /// When they reached it and when they passed, seconds from the calendar origin.
    pub at_s: f64,
    pub pass_s: f64,
    /// Walkers ahead of them when they came.
    pub ahead: u32,
}

impl Passage {
    /// Seconds they waited.
    pub fn waited_s(&self) -> f64 {
        self.pass_s - self.at_s
    }

    /// Whether they stood short of the approach, its waiting space full.
    pub fn held_short(&self) -> bool {
        self.ahead > APPROACH_SPACE
    }
}

/// A facility's passages and waits by hour of the day, fading, as of `day`.
#[derive(Clone, Debug, PartialEq)]
pub struct Tally {
    pub facility: Facility,
    pub day: i64,
    pub passages: [f32; 24],
    pub waited_min: [f32; 24],
    /// Walkers who stood short of its approach.
    pub held_short: f32,
    /// The longest wait in its memory, minutes, and the day of it.
    pub longest_min: f32,
    pub longest_day: i64,
}

impl Tally {
    fn fade_to(&mut self, day: i64) {
        if day > self.day {
            let f = (-((day - self.day) as f64) / TALLY_MEMORY_DAYS).exp() as f32;
            for v in self.passages.iter_mut().chain(self.waited_min.iter_mut()) {
                *v *= f;
            }
            self.held_short *= f;
            self.day = day;
        }
    }

    /// Its passages remembered, by hour, on `day`.
    pub fn faded(&self, day: i64) -> Tally {
        let mut t = self.clone();
        t.fade_to(day);
        t
    }

    /// Passages and minutes waited over every hour.
    pub fn totals(&self) -> (f32, f32) {
        (self.passages.iter().sum(), self.waited_min.iter().sum())
    }
}

/// What a household holds of its waits at one facility.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WaitHeld {
    pub facility: Facility,
    /// Minutes its people waited there, and how many times, fading.
    pub minutes: f32,
    pub waits: f32,
    /// The day the fading was reckoned to.
    pub day: i64,
}

/// Every facility's turn-taking, tallies and the waits households hold.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Traffic {
    /// Directions of facilities that have passed someone lately, in facility and direction order.
    pub servers: Vec<Server>,
    /// Each facility's tallies, in facility order.
    pub tallies: Vec<Tally>,
    /// The waits each household holds, in facility order.
    pub waits: BTreeMap<PermanentId, Vec<WaitHeld>>,
    /// Seconds between walkers in place of each facility's own, for tests that need a queue. Not
    /// saved; never set in a world.
    pub test_headway_s: Option<f64>,
}

impl Traffic {
    /// A walker reaches `facility` going `dir` at `at_s` (seconds from the calendar origin), on
    /// `day` in hour `hour`, where walkers pass every `headway_s` seconds: when they pass.
    pub fn pass(
        &mut self,
        facility: Facility,
        dir: u8,
        at_s: f64,
        headway_s: f64,
        day: i64,
        hour: usize,
    ) -> Passage {
        let headway_s = self.test_headway_s.unwrap_or(headway_s).max(1e-3);
        let i = match self
            .servers
            .binary_search_by(|s| (s.facility, s.dir).cmp(&(facility, dir)))
        {
            Ok(i) => i,
            Err(i) => {
                self.servers.insert(
                    i,
                    Server {
                        facility,
                        dir,
                        slots: Vec::new(),
                    },
                );
                i
            }
        };
        let server = &mut self.servers[i];
        let keep = server.slots.partition_point(|&s| s < at_s - SLOT_MEMORY_S);
        server.slots.drain(..keep);
        let (pass_s, ahead) = server.fit(at_s, headway_s);
        let k = server.slots.partition_point(|&s| s < pass_s);
        server.slots.insert(k, pass_s);
        let passage = Passage {
            at_s,
            pass_s,
            ahead,
        };
        let t = match self.tallies.binary_search_by_key(&facility, |t| t.facility) {
            Ok(t) => t,
            Err(t) => {
                self.tallies.insert(
                    t,
                    Tally {
                        facility,
                        day,
                        passages: [0.0; 24],
                        waited_min: [0.0; 24],
                        held_short: 0.0,
                        longest_min: 0.0,
                        longest_day: day,
                    },
                );
                t
            }
        };
        let tally = &mut self.tallies[t];
        tally.fade_to(day);
        let hour = hour.min(23);
        let waited = (passage.waited_s() / MINUTE_S) as f32;
        tally.passages[hour] += 1.0;
        tally.waited_min[hour] += waited;
        if passage.held_short() {
            tally.held_short += 1.0;
        }
        if waited > tally.longest_min || (day - tally.longest_day) as f64 > TALLY_MEMORY_DAYS {
            tally.longest_min = waited;
            tally.longest_day = day;
        }
        passage
    }

    /// Someone of `household` waited `minutes` at `facility` on `day`.
    pub fn waited(
        &mut self,
        household: PermanentId,
        facility: Facility,
        minutes: f32,
        day: i64,
        half_life_days: f64,
    ) {
        let list = self.waits.entry(household).or_default();
        match list.binary_search_by_key(&facility, |w| w.facility) {
            Ok(i) => {
                let w = &mut list[i];
                if day > w.day {
                    let f = fading(day - w.day, half_life_days);
                    w.minutes *= f;
                    w.waits *= f;
                    w.day = day;
                }
                w.minutes += minutes;
                w.waits += 1.0;
            }
            Err(i) => list.insert(
                i,
                WaitHeld {
                    facility,
                    minutes,
                    waits: 1.0,
                    day,
                },
            ),
        }
    }

    /// What `household` holds of its waits, faded to `day`, in facility order.
    pub fn held(
        &self,
        household: PermanentId,
        day: i64,
        half_life_days: f64,
    ) -> impl Iterator<Item = WaitHeld> + '_ {
        self.waits
            .get(&household)
            .into_iter()
            .flatten()
            .map(move |w| {
                let f = fading(day - w.day, half_life_days);
                WaitHeld {
                    minutes: w.minutes * f,
                    waits: w.waits * f,
                    day: day.max(w.day),
                    ..*w
                }
            })
    }

    /// At `now_s` on `day`: lets go of directions whose last slot is more than
    /// [`SLOT_MEMORY_S`] behind now (no later walker could fit before them), tallies that have
    /// faded below `least` passages, and waits faded below `least` or of households `keep`
    /// refuses.
    pub fn forget(
        &mut self,
        now_s: f64,
        day: i64,
        half_life_days: f64,
        least: f32,
        keep: impl Fn(PermanentId) -> bool,
    ) {
        self.servers
            .retain(|s| s.slots.last().is_some_and(|&t| t >= now_s - SLOT_MEMORY_S));
        self.tallies
            .retain(|t| t.faded(day).totals().0 >= least || t.longest_day == day);
        self.waits.retain(|&h, list| {
            if !keep(h) {
                return false;
            }
            list.retain(|w| w.waits * fading(day - w.day, half_life_days) >= least);
            !list.is_empty()
        });
    }

    /// What is wrong with the record, for the invariant checks.
    pub fn problems(&self) -> Vec<String> {
        let mut out = Vec::new();
        if self
            .servers
            .windows(2)
            .any(|w| (w[0].facility, w[0].dir) >= (w[1].facility, w[1].dir))
        {
            out.push("traffic servers are out of order or repeated".to_owned());
        }
        if self.servers.iter().any(|s| {
            s.slots.is_empty()
                || s.slots.iter().any(|t| !t.is_finite())
                || s.slots.windows(2).any(|w| w[0] > w[1])
        }) {
            out.push("a traffic server has its slots malformed".to_owned());
        }
        if self
            .tallies
            .windows(2)
            .any(|w| w[0].facility >= w[1].facility)
        {
            out.push("traffic tallies are out of order or repeated".to_owned());
        }
        for t in &self.tallies {
            if t.passages
                .iter()
                .chain(&t.waited_min)
                .chain([&t.held_short, &t.longest_min])
                .any(|v| !(v.is_finite() && *v >= 0.0))
            {
                out.push("a traffic tally is malformed".to_owned());
            }
        }
        for list in self.waits.values() {
            if list.windows(2).any(|w| w[0].facility >= w[1].facility) {
                out.push("a household's waits are out of order or repeated".to_owned());
            }
        }
        out
    }
}

/// A facility ahead on a walk.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ahead {
    pub facility: Facility,
    /// Where they reach its approach, metres, and when, minutes after setting off.
    pub at: (f32, f32),
    pub at_min: f32,
    /// The point after which along the walk's polyline they reach it.
    pub seg: usize,
    /// The point just inside it, to tell the direction they go.
    pub inside: (f32, f32),
}

/// A walk's next facility, the way it goes through it and how often that way passes a walker.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pending {
    pub ahead: Ahead,
    pub dir: u8,
    pub headway_s: f32,
}

/// The first facility along a walk's polyline `points` (timed `minutes` after setting off) that
/// it reaches at or after `from_min`, other than `skip` until the walk has been inside that one
/// and left it: the walk
/// sampled every half cell, as its cells are (`civ_land::paths::cells_along`). `facility_at` names
/// the facility at a cell; `corner(a, b)` the cell a diagonal step went by, as the walking grid
/// takes it.
#[allow(clippy::too_many_arguments)]
pub fn ahead(
    points: &[(f32, f32)],
    minutes: &[f32],
    from_min: f32,
    mut skip: Option<Facility>,
    cell_m: f32,
    width: u32,
    height: u32,
    facility_at: impl Fn(u32) -> Option<Facility>,
    corner: impl Fn(u32, u32) -> u32,
) -> Option<Ahead> {
    let cell = |x: f32, y: f32| -> Option<u32> {
        let (cx, cy) = ((x / cell_m).floor(), (y / cell_m).floor());
        (cx >= 0.0 && cy >= 0.0 && cx < width as f32 && cy < height as f32)
            .then(|| cy as u32 * width + cx as u32)
    };
    let mut inside_skip = false;
    let mut last: Option<(u32, (f32, f32), f32)> = points
        .first()
        .and_then(|&(x, y)| cell(x, y).map(|c| (c, (x, y), 0.0)));
    for (k, w) in points.windows(2).enumerate() {
        let (a, b) = (w[0], w[1]);
        let (m0, m1) = (minutes[k], minutes[k + 1]);
        let len = (b.0 - a.0).hypot(b.1 - a.1);
        let steps = (len / (cell_m * 0.5)).ceil().max(1.0) as u32;
        for s in 1..=steps {
            let t = s as f32 / steps as f32;
            let at = (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
            let at_min = m0 + (m1 - m0) * t;
            let Some(c) = cell(at.0, at.1) else {
                continue;
            };
            let Some((l, before, before_min)) = last else {
                last = Some((c, at, at_min));
                continue;
            };
            if c == l {
                continue;
            }
            let mut cells = [None, Some(c)];
            let (lx, ly, cx, cy) = (l % width, l / width, c % width, c / width);
            if lx.abs_diff(cx) == 1 && ly.abs_diff(cy) == 1 {
                cells[0] = Some(corner(ly * width + cx, cy * width + lx));
            }
            last = Some((c, at, at_min));
            if before_min < from_min {
                continue;
            }
            for c in cells.into_iter().flatten() {
                match facility_at(c) {
                    Some(f) if Some(f) == skip => inside_skip = true,
                    Some(f) => {
                        return Some(Ahead {
                            facility: f,
                            at: before,
                            at_min: before_min,
                            seg: k,
                            inside: at,
                        });
                    }
                    None if inside_skip => {
                        skip = None;
                        inside_skip = false;
                    }
                    None => {}
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(n: u64) -> PermanentId {
        PermanentId::from_raw(n).expect("non-zero")
    }

    #[test]
    fn a_file_passes_first_come_at_its_headway_and_never_saves_up_what_went_unused() {
        let mut t = Traffic::default();
        let f = Facility::Crossing(id(5));
        let h = headway_s(1);
        assert!((h - 1.0 / 0.72).abs() < 1e-9);
        // Five walkers come at once: each passes a headway after the one before.
        let passed: Vec<Passage> = (0..5).map(|_| t.pass(f, SHARED, 600.0, h, 0, 0)).collect();
        for (k, p) in passed.iter().enumerate() {
            assert!((p.pass_s - (600.0 + k as f64 * h)).abs() < 1e-9);
            assert_eq!(p.ahead, k as u32);
        }
        // An hour of nothing: the next walker passes on arrival, and the one after it a headway
        // later; the unused hour is not a burst of slots.
        let p = t.pass(f, SHARED, 4200.0, h, 0, 1);
        assert_eq!((p.pass_s, p.ahead), (4200.0, 0));
        let q = t.pass(f, SHARED, 4200.0, h, 0, 1);
        assert!((q.waited_s() - h).abs() < 1e-9);
        let tally = &t.tallies[0];
        assert_eq!((tally.passages[0], tally.passages[1]), (5.0, 2.0));
        assert!((tally.waited_min[0] - (10.0 * h / 60.0) as f32).abs() < 1e-5);
        assert!(t.problems().is_empty());
    }

    #[test]
    fn one_file_is_shared_by_both_ways_and_two_files_give_each_way_its_own() {
        assert_eq!(files(0.4), (true, 1));
        assert_eq!(files(1.0), (true, 1));
        assert_eq!(files(1.2), (false, 1));
        assert_eq!(files(GATE_OPENING_M), (false, 2));
        let mut t = Traffic::default();
        let f = Facility::Crossing(id(5));
        let g = Facility::Gate {
            enclosure: id(6),
            sector: 3,
        };
        let h = headway_s(1);
        // On a shared file, the walker coming the other way waits behind the first.
        t.pass(f, SHARED, 0.0, h, 0, 0);
        assert!(t.pass(f, SHARED, 0.0, h, 0, 0).waited_s() > 0.0);
        // Through a gate, either way has its own files.
        t.pass(g, 0, 0.0, headway_s(2), 0, 0);
        assert_eq!(t.pass(g, 1, 0.0, headway_s(2), 0, 0).waited_s(), 0.0);
        // Taken out of order within a minute: the one who came earlier fits before the one who
        // came later, unless their slots overlap.
        let mut t = Traffic::default();
        t.pass(f, SHARED, 59.0, h, 0, 0);
        assert_eq!(t.pass(f, SHARED, 30.0, h, 0, 0).waited_s(), 0.0);
        let close = t.pass(f, SHARED, 58.5, h, 0, 0);
        assert!(close.waited_s() > 0.0 && close.ahead == 1);
        // Held short once the approach is full.
        let mut t = Traffic::default();
        let last = (0..=APPROACH_SPACE + 1)
            .map(|_| t.pass(f, SHARED, 0.0, h, 0, 0))
            .last()
            .expect("passages");
        assert!(last.held_short());
        assert_eq!(t.tallies[0].held_short, 1.0);
        for code in [f.code(), g.code()] {
            assert!(Facility::from_code(code.0, code.1, code.2).is_some());
        }
        assert_eq!(Facility::from_code(1, 6, 3), Some(g));
    }

    #[test]
    fn idle_servers_and_faded_waits_are_let_go() {
        let mut t = Traffic::default();
        let f = Facility::Crossing(id(5));
        t.pass(f, SHARED, 0.0, 10.0, 0, 0);
        t.waited(id(9), f, 4.0, 0, 30.0);
        t.waited(id(9), f, 2.0, 30, 30.0);
        let held: Vec<WaitHeld> = t.held(id(9), 30, 30.0).collect();
        assert!((held[0].minutes - 4.0).abs() < 1e-5 && (held[0].waits - 1.5).abs() < 1e-5);
        t.forget(100.0, 0, 30.0, 0.1, |_| true);
        assert_eq!(t.servers.len(), 1, "its slot still within memory");
        t.forget(200.0, 400, 30.0, 0.1, |_| true);
        assert!(t.servers.is_empty() && t.tallies.is_empty() && t.waits.is_empty());
    }

    #[test]
    fn the_first_facility_ahead_is_found_at_its_approach_with_its_time() {
        // A walk east along row 1 of a 10-wide map of 8 m cells, 10 minutes over 72 m; crossings
        // on cells 15 and 17 (row 1, columns 5 and 7).
        let points = [(4.0, 12.0), (76.0, 12.0)];
        let minutes = [0.0, 10.0];
        let at = |c: u32| match c {
            15 => Some(Facility::Crossing(id(5))),
            17 => Some(Facility::Crossing(id(6))),
            _ => None,
        };
        let first = ahead(&points, &minutes, 0.0, None, 8.0, 10, 4, at, |a, _| a)
            .expect("a facility ahead");
        assert_eq!(first.facility, Facility::Crossing(id(5)));
        assert!(first.at.0 < 40.0 && first.inside.0 >= 40.0);
        assert!((first.at_min - (first.at.0 - 4.0) / 72.0 * 10.0).abs() < 1e-4);
        let next = ahead(
            &points,
            &minutes,
            first.at_min,
            Some(first.facility),
            8.0,
            10,
            4,
            at,
            |a, _| a,
        )
        .expect("the second");
        assert_eq!(next.facility, Facility::Crossing(id(6)));
        let after = ahead(
            &points,
            &minutes,
            next.at_min,
            Some(next.facility),
            8.0,
            10,
            4,
            at,
            |a, _| a,
        );
        assert!(after.is_none());
    }
}
