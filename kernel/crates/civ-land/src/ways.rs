//! Ways (M8b slice CM; ADR-0029 §1; the transport brief §1; research 10-07 §1.6, 11-12): a line
//! someone has worked on, saved below the grid as a bank is (ADR-0022 §11). A trail never becomes
//! a way by use alone: until someone clears, lays or reserves a line it stays worn ground
//! ([`crate::paths`]), and a world in which nobody works on a way keeps its digests.
//!
//! A way has an identity that survives surveys and loads, and runs as an ordered list of
//! **segments**, split where its surface, widths or keeper change. Each segment has three widths
//! (research 11-12 §1.2: the corridor reserved, the width built and the width usable today), a
//! surface, a keeper and the day it was last kept. Its **condition** fades from that day at the
//! wear's own half-life, and before each monthly survey a way holds the wear of its cells at no
//! less than its condition ([`Ways::holds`]), so a kept way walks at full and an untended one
//! fades as a trail does.
//!
//! Its **rank** (lane, street, main way, way between settlements) is derived at each survey from
//! what it joins, its build and its use, and is shown, never read by behaviour (ADR-0029 §1).

use civ_core::PermanentId;

use crate::Settlement;
use crate::paths::PathParams;

/// The modes a way serves, as bits of [`Way::modes`].
pub const WALK: u8 = 1;
/// Handcarts (M8b slice CP).
pub const CART: u8 = 2;

/// A segment's surface. Codes are part of saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Surface {
    /// Cleared ground: brush and stones taken off, the earth as it lies.
    Cleared,
    /// Shaped earth: cambered and ditched.
    Shaped,
    /// Gravel laid on shaped earth.
    Gravel,
    /// A timber causeway over soft ground.
    Causeway,
}

impl Surface {
    /// Its code in saves.
    pub fn code(self) -> u8 {
        self as u8
    }

    /// The surface with code `code`.
    pub fn from_code(code: u8) -> Option<Surface> {
        [
            Surface::Cleared,
            Surface::Shaped,
            Surface::Gravel,
            Surface::Causeway,
        ]
        .get(usize::from(code))
        .copied()
    }

    /// Its name, for the observer.
    pub fn name(self) -> &'static str {
        match self {
            Surface::Cleared => "cleared ground",
            Surface::Shaped => "shaped earth",
            Surface::Gravel => "gravel",
            Surface::Causeway => "timber causeway",
        }
    }
}

/// Who keeps a segment: who would mend it, and whose say its use is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Keeper {
    /// Nobody keeps it.
    Nobody,
    Household(PermanentId),
    Polity(PermanentId),
}

impl Keeper {
    /// Its code in saves, and the id it names (0 for nobody).
    pub fn code(self) -> (u8, u64) {
        match self {
            Keeper::Nobody => (0, 0),
            Keeper::Household(id) => (1, id.get()),
            Keeper::Polity(id) => (2, id.get()),
        }
    }

    /// The keeper with code `code` naming `id`.
    pub fn from_code(code: u8, id: u64) -> Option<Keeper> {
        match code {
            0 => Some(Keeper::Nobody),
            1 => PermanentId::from_raw(id).map(Keeper::Household),
            2 => PermanentId::from_raw(id).map(Keeper::Polity),
            _ => None,
        }
    }
}

/// A way's place in the hierarchy, derived at each survey and never read by behaviour. Codes are
/// part of saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rank {
    Lane,
    Street,
    MainWay,
    /// A way whose ends lie by two different settlements.
    Between,
}

impl Rank {
    /// Its code in saves.
    pub fn code(self) -> u8 {
        self as u8
    }

    /// The rank with code `code`.
    pub fn from_code(code: u8) -> Option<Rank> {
        [Rank::Lane, Rank::Street, Rank::MainWay, Rank::Between]
            .get(usize::from(code))
            .copied()
    }

    /// Its name, for the observer.
    pub fn name(self) -> &'static str {
        match self {
            Rank::Lane => "lane",
            Rank::Street => "street",
            Rank::MainWay => "main way",
            Rank::Between => "way between settlements",
        }
    }
}

/// Days of use a way's trip count remembers: each trip counts `e^(−age/90)`, so the count is
/// about the trips of the last 90 days (ADR-0029 §1).
pub const USE_MEMORY_DAYS: f64 = 90.0;
/// A way's end lies by a settlement within this distance of its hearth, metres (a design prior:
/// about a village's walk from its hearth to its edge).
pub const BY_SETTLEMENT_M: f32 = 400.0;
/// Usable width from which a way is a main way, metres (a design prior from research 11-12
/// §1.2's carriageway widths: two carts pass on 4.5 m).
pub const MAIN_WAY_M: f32 = 4.5;
/// Usable width from which a way is a street, metres (a design prior: a cart and walkers pass on
/// 2.5 m).
pub const STREET_M: f32 = 2.5;
/// Trips a day over its memory from which a way is a main way (a design prior).
pub const MAIN_WAY_TRIPS: f32 = 100.0;
/// Trips a day over its memory from which a way is a street (a design prior).
pub const STREET_TRIPS: f32 = 20.0;

/// What works on ways take (M8b slice CQ; ADR-0029 §6; research 10-07 §2.2, §2.4). Authored in
/// content.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WayWorkParams {
    /// The width a passing place is cleared to, metres: two carts side by side, with room.
    pub passing_width_m: f32,
    /// Hours of a capable adult's work a square metre of ground takes to clear and level.
    pub clear_h_m2: f64,
    /// Years a household reckons a passing place lasts when weighing whether to make one.
    pub passing_life_years: f64,
}

impl Default for WayWorkParams {
    /// A 2 m passing place; 0.34 h a square metre, ILO's 3.5 m³ a worker-day in medium soil over
    /// a 0.15 m cut (research 10-07 §2.2); ten years (design priors).
    fn default() -> Self {
        Self {
            passing_width_m: 2.0,
            clear_h_m2: 0.34,
            passing_life_years: 10.0,
        }
    }
}

/// One stretch of a way with one surface, set of widths and keeper.
#[derive(Clone, Debug, PartialEq)]
pub struct Segment {
    /// The map cells it runs through, in order along the way, each once.
    pub cells: Vec<u32>,
    /// The corridor reserved for it, metres: `usable ≤ built ≤ corridor`.
    pub corridor_m: f32,
    /// The width built, metres.
    pub built_m: f32,
    /// The width usable today, metres.
    pub usable_m: f32,
    pub surface: Surface,
    pub keeper: Keeper,
    /// The day it was last laid or kept: its condition is whole then.
    pub kept: i64,
}

impl Segment {
    /// Its condition on `day`, 0–1: whole on the day it was kept, fading at the wear's half-life.
    pub fn condition(&self, day: i64, params: &PathParams) -> f32 {
        params.fade(day - self.kept) as f32
    }

    /// Its steepest grade between neighbouring cells, rise over run, from the ground's
    /// elevations (`elevation` row by row, `width` cells to a row, `cell_m` metres a cell): the
    /// grade a cart would feel until grading eases it (ADR-0029 §6).
    pub fn grade(&self, elevation: &[f32], width: u32, cell_m: f32) -> f32 {
        let w = width.max(1) as usize;
        self.cells
            .windows(2)
            .filter_map(|p| {
                let (a, b) = (p[0] as usize, p[1] as usize);
                let (za, zb) = (elevation.get(a)?, elevation.get(b)?);
                let (dx, dy) = ((a % w).abs_diff(b % w), (a / w).abs_diff(b / w));
                let run = cell_m * ((dx * dx + dy * dy) as f32).sqrt();
                (run > 0.0).then(|| (zb - za).abs() / run)
            })
            .fold(0.0, f32::max)
    }
}

/// One way.
#[derive(Clone, Debug, PartialEq)]
pub struct Way {
    pub id: PermanentId,
    /// The day the first work on it began.
    pub begun: i64,
    /// Its stretches, in order: each begins where the one before it ends.
    pub segments: Vec<Segment>,
    /// The modes it serves, as bits ([`WALK`], [`CART`]).
    pub modes: u8,
    /// Trips along it, each counted as it fades over [`USE_MEMORY_DAYS`], as of `trips_day`.
    pub trips: f32,
    pub trips_day: i64,
    /// Its rank as last surveyed.
    pub rank: Rank,
}

impl Way {
    /// Its trips remembered on `day`.
    pub fn trips_on(&self, day: i64) -> f32 {
        let age = (day - self.trips_day).max(0) as f64;
        (f64::from(self.trips) * (-age / USE_MEMORY_DAYS).exp()) as f32
    }

    /// Its trips a day, averaged over its memory, on `day`.
    pub fn trips_a_day(&self, day: i64) -> f32 {
        self.trips_on(day) / USE_MEMORY_DAYS as f32
    }

    /// The cells at either end.
    pub fn ends(&self) -> Option<(u32, u32)> {
        let first = *self.segments.first()?.cells.first()?;
        let last = *self.segments.last()?.cells.last()?;
        Some((first, last))
    }

    /// Its widest usable width.
    pub fn usable_m(&self) -> f32 {
        self.segments.iter().map(|s| s.usable_m).fold(0.0, f32::max)
    }

    /// Its rank on `day` (ADR-0029 §1): between settlements when its two ends lie by different
    /// settlements; else a main way, a street or a lane by its usable width or its use, whichever
    /// ranks it higher. `near(cell)` is the settlement a cell lies by, if any.
    pub fn rank_on(&self, day: i64, near: impl Fn(u32) -> Option<PermanentId>) -> Rank {
        if let Some((a, b)) = self.ends()
            && let (Some(sa), Some(sb)) = (near(a), near(b))
            && sa != sb
        {
            return Rank::Between;
        }
        let (width, trips) = (self.usable_m(), self.trips_a_day(day));
        if width >= MAIN_WAY_M || trips >= MAIN_WAY_TRIPS {
            Rank::MainWay
        } else if width >= STREET_M || trips >= STREET_TRIPS {
            Rank::Street
        } else {
            Rank::Lane
        }
    }
}

/// Every way in the world, in the order begun, and the revision the observer draws them at.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Ways {
    pub list: Vec<Way>,
    revision: u32,
    /// Each cell some way runs through, with the way's position in `list`, sorted: what a walk
    /// is counted against. Derived.
    cells: Vec<(u32, u32)>,
}

impl Ways {
    /// Ways `list` at revision `revision` (a loaded world's).
    pub fn with_revision(list: Vec<Way>, revision: u32) -> Ways {
        let mut ways = Ways {
            list,
            revision,
            cells: Vec::new(),
        };
        ways.index();
        ways
    }

    /// Changes whenever a way is begun or changed, and at each survey while there are any. 0
    /// while there has never been one.
    pub fn revision(&self) -> u32 {
        self.revision
    }

    /// Notes that a way was begun or changed.
    pub fn changed(&mut self) {
        self.revision = self.revision.wrapping_add(1).max(1);
        self.index();
    }

    /// Adds a way and notes the change.
    pub fn add(&mut self, way: Way) {
        self.list.push(way);
        self.changed();
    }

    fn index(&mut self) {
        self.cells.clear();
        for (k, w) in self.list.iter().enumerate() {
            for s in &w.segments {
                self.cells.extend(s.cells.iter().map(|&c| (c, k as u32)));
            }
        }
        self.cells.sort_unstable();
        self.cells.dedup();
    }

    /// The ways running through `cell`, by position.
    pub fn at(&self, cell: u32) -> impl Iterator<Item = usize> + '_ {
        let from = self.cells.partition_point(|&(c, _)| c < cell);
        self.cells[from..]
            .iter()
            .take_while(move |&&(c, _)| c == cell)
            .map(|&(_, k)| k as usize)
    }

    /// Someone went across `cells` on `day` by `modes` (bits, as [`Way::modes`]): one trip along
    /// each way they used that serves one of them, so a way wears only by the modes it serves
    /// (ADR-0034 §1). Nothing is done while there are no ways.
    pub fn walked(&mut self, cells: &[u32], day: i64, modes: u8) {
        if self.list.is_empty() {
            return;
        }
        let mut used: Vec<usize> = cells
            .iter()
            .flat_map(|&c| self.at(c))
            .filter(|&k| self.list[k].modes & modes != 0)
            .collect();
        used.sort_unstable();
        used.dedup();
        for k in used {
            let way = &mut self.list[k];
            way.trips = way.trips_on(day) + 1.0;
            way.trips_day = way.trips_day.max(day);
        }
    }

    /// What each way people walk holds its cells' wear at on `day`, ahead of the survey: its
    /// condition, cell by cell (the greatest where ways meet), in cell order.
    pub fn holds(&self, day: i64, params: &PathParams) -> Vec<(u32, f32)> {
        let mut out: Vec<(u32, f32)> = self
            .list
            .iter()
            // Only a way people walk holds walking ground (ADR-0034 §1): track for wheels alone
            // is not a footpath.
            .filter(|w| w.modes & WALK != 0)
            .flat_map(|w| {
                w.segments.iter().flat_map(move |s| {
                    let c = s.condition(day, params);
                    s.cells.iter().map(move |&cell| (cell, c))
                })
            })
            .collect();
        out.sort_unstable_by(|a, b| a.0.cmp(&b.0).then(b.1.total_cmp(&a.1)));
        out.dedup_by_key(|h| h.0);
        out
    }

    /// The month's survey on `day`: each way's rank from what it joins, its build and its use
    /// (`settlements` as the land keeps them; a cell's centre is `cell_m` metres a cell of a map
    /// `width` cells wide), and a new revision while there are any ways.
    pub fn survey(&mut self, day: i64, settlements: &[Settlement], width: u32, cell_m: f32) {
        if self.list.is_empty() {
            return;
        }
        let w = width.max(1);
        let near = |cell: u32| {
            let (x, y) = (
                ((cell % w) as f32 + 0.5) * cell_m,
                ((cell / w) as f32 + 0.5) * cell_m,
            );
            settlements
                .iter()
                .filter(|s| s.abandoned.is_none())
                .map(|s| ((s.hearth_m.0 - x).hypot(s.hearth_m.1 - y), s.id))
                .filter(|&(d, _)| d <= BY_SETTLEMENT_M)
                .min_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)))
                .map(|(_, id)| id)
        };
        for way in &mut self.list {
            way.rank = way.rank_on(day, near);
        }
        self.revision = self.revision.wrapping_add(1).max(1);
    }

    /// What is wrong with the record, for the invariant checks, on a map of `cells` cells.
    pub fn problems(&self, cells: u32) -> Vec<String> {
        let mut out = Vec::new();
        if self.list.windows(2).any(|w| w[0].id >= w[1].id) {
            out.push("ways are out of order or repeated".to_owned());
        }
        for w in &self.list {
            if w.segments.is_empty() || w.segments.iter().any(|s| s.cells.is_empty()) {
                out.push(format!("way {} has a segment with no ground", w.id));
            }
            if w.segments
                .iter()
                .flat_map(|s| &s.cells)
                .any(|&c| c >= cells)
            {
                out.push(format!("way {} runs off the map", w.id));
            }
            if w.segments.iter().any(|s| {
                !(s.usable_m.is_finite()
                    && s.usable_m > 0.0
                    && s.usable_m <= s.built_m
                    && s.built_m <= s.corridor_m
                    && s.corridor_m.is_finite())
            }) {
                out.push(format!("way {} has its widths out of order", w.id));
            }
            if !(w.trips.is_finite() && w.trips >= 0.0) {
                out.push(format!("way {} has its use malformed", w.id));
            }
            if w.modes == 0 {
                out.push(format!("way {} serves no mode", w.id));
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Founding;
    use civ_core::SimTime;

    fn id(n: u64) -> PermanentId {
        PermanentId::from_raw(n).expect("non-zero")
    }

    fn params() -> PathParams {
        PathParams {
            wear_per_walk: 0.05,
            half_life_days: 120.0,
            trail_at: 0.35,
            trail_until: 0.2,
        }
    }

    fn segment(cells: Vec<u32>, usable_m: f32, kept: i64) -> Segment {
        Segment {
            cells,
            corridor_m: 6.0,
            built_m: usable_m,
            usable_m,
            surface: Surface::Cleared,
            keeper: Keeper::Household(id(3)),
            kept,
        }
    }

    fn way(n: u64, segments: Vec<Segment>) -> Way {
        Way {
            id: id(n),
            begun: 0,
            segments,
            modes: WALK,
            trips: 0.0,
            trips_day: 0,
            rank: Rank::Lane,
        }
    }

    fn settlement(n: u64, at: (f32, f32)) -> Settlement {
        Settlement {
            id: id(n),
            name: format!("S{n}"),
            founded: SimTime::from_minutes(0),
            hearth_m: at,
            food_short: false,
            harvest_kg: 0.0,
            parent: None,
            founding: Founding::Setup,
            abandoned: None,
        }
    }

    #[test]
    fn a_way_wears_only_by_the_modes_it_serves() {
        // Track for carts alone is not walked: a walker's trip leaves it as it was, a hauler's
        // (on foot beside the cart) counts on it and on the footpath it crosses alike.
        let mut ways = Ways::default();
        ways.add(Way {
            modes: CART,
            ..way(5, vec![segment(vec![1, 2, 3], 3.0, 0)])
        });
        ways.add(way(6, vec![segment(vec![3, 4], 1.5, 0)]));
        ways.walked(&[1, 2, 3, 4], 10, WALK);
        assert_eq!((ways.list[0].trips, ways.list[1].trips), (0.0, 1.0));
        ways.walked(&[1, 2, 3, 4], 10, WALK | CART);
        assert_eq!((ways.list[0].trips, ways.list[1].trips), (1.0, 2.0));
    }

    #[test]
    fn walks_count_once_a_way_and_fade_over_the_memory() {
        let mut ways = Ways::default();
        ways.walked(&[1, 2, 3], 0, WALK);
        assert_eq!(ways.revision(), 0, "no ways, nothing counted or changed");
        ways.add(way(
            5,
            vec![segment(vec![1, 2], 1.5, 0), segment(vec![3, 4], 1.5, 0)],
        ));
        ways.add(way(6, vec![segment(vec![4, 9], 1.5, 0)]));
        assert_eq!(ways.revision(), 2);
        assert_eq!(ways.at(4).collect::<Vec<_>>(), vec![0, 1]);
        ways.walked(&[0, 1, 2, 3], 10, WALK);
        ways.walked(&[2, 3, 4, 5], 10, WALK);
        assert_eq!((ways.list[0].trips, ways.list[1].trips), (2.0, 1.0));
        let later = ways.list[0].trips_on(100);
        assert!((later - 2.0 * (-1.0f32).exp()).abs() < 1e-4, "{later}");
        assert!(ways.problems(16).is_empty());
        assert!(
            !ways.problems(4).is_empty(),
            "way 6 runs off a map of 4 cells"
        );
    }

    #[test]
    fn kept_ways_hold_their_cells_at_their_condition_the_best_where_they_meet() {
        let mut ways = Ways::default();
        ways.add(way(5, vec![segment(vec![7, 8], 1.5, 0)]));
        ways.add(way(6, vec![segment(vec![8, 2], 1.5, 120)]));
        let holds = ways.holds(120, &params());
        assert_eq!(holds.len(), 3);
        assert_eq!(holds[0], (2, 1.0));
        assert_eq!(holds[1].0, 7);
        assert!((holds[1].1 - 0.5).abs() < 1e-6, "a half-life untended");
        assert_eq!(holds[2].0, 8);
        assert!(
            (holds[2].1 - 1.0).abs() < 1e-6,
            "cell 8 takes the newer keeping"
        );
        // Held on the ground: a way's cells walk as worn trail.
        let p = params();
        let mut wear = crate::paths::Wear::new(4, 4, 8.0);
        wear.hold(&holds, 120, &p);
        wear.survey(120, &p);
        assert!((wear.factor(8) - 1.0).abs() < 1e-2 && wear.is_trail(8));
        assert!((wear.factor(7) - 0.5).abs() < 1e-2 && wear.is_trail(7));
        assert_eq!(wear.factor(3), 0.0);
        // A way for wheels alone holds no walking ground (ADR-0034 §1).
        let mut wheels = way(7, vec![segment(vec![3, 4], 1.5, 120)]);
        wheels.modes = CART;
        ways.add(wheels);
        assert_eq!(ways.holds(120, &params()), holds);
    }

    #[test]
    fn rank_follows_what_a_way_joins_then_its_build_then_its_use() {
        let mut ways = Ways::default();
        // A map 100 cells wide of 8 m cells; settlements at cells 0 and 99 (in row 0).
        let settlements = [settlement(1, (4.0, 4.0)), settlement(2, (796.0, 4.0))];
        ways.add(way(5, vec![segment((0..100).collect(), 1.5, 0)]));
        ways.add(way(6, vec![segment(vec![0, 1, 2], 1.5, 0)]));
        ways.add(way(7, vec![segment(vec![0, 1, 2], 3.0, 0)]));
        ways.add(way(8, vec![segment(vec![300, 301], 5.0, 0)]));
        ways.list[1].trips = 25.0 * 90.0;
        ways.survey(0, &settlements, 100, 8.0);
        let ranks: Vec<Rank> = ways.list.iter().map(|w| w.rank).collect();
        assert_eq!(
            ranks,
            vec![Rank::Between, Rank::Street, Rank::Street, Rank::MainWay]
        );
        ways.survey(400, &settlements, 100, 8.0);
        assert_eq!(ways.list[1].rank, Rank::Lane, "its use forgotten");
        for r in [Rank::Lane, Rank::Street, Rank::MainWay, Rank::Between] {
            assert_eq!(Rank::from_code(r.code()), Some(r));
        }
        for s in [
            Surface::Cleared,
            Surface::Shaped,
            Surface::Gravel,
            Surface::Causeway,
        ] {
            assert_eq!(Surface::from_code(s.code()), Some(s));
        }
        for k in [
            Keeper::Nobody,
            Keeper::Household(id(4)),
            Keeper::Polity(id(9)),
        ] {
            let (c, i) = k.code();
            assert_eq!(Keeper::from_code(c, i), Some(k));
        }
    }

    #[test]
    fn a_segment_feels_its_steepest_step() {
        let s = segment(vec![0, 1, 3], 1.5, 0);
        // A map 2 cells wide: 0→1 east 8 m, 1→3 south 8 m.
        let z = [10.0, 10.4, 0.0, 12.0];
        assert!((s.grade(&z, 2, 8.0) - 0.2).abs() < 1e-6);
    }
}
