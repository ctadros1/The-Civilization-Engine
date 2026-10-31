//! Deposits found (ADR-0010 §1; M3b slice Q): a settlement comes to know a body that shows at
//! the surface when one of its members walks near it. A buried body waits for an earthwork to cut
//! into it. Nothing here decides where anyone goes; finding is what walking brings.

use super::*;

/// How far from a body's edge a walker sees it showing, metres: an exposure in a bank or a spread
/// on the surface, seen from the way (a tuning value; the research gives no figure, and looking
/// reveals indications, not a body's outline, 03-05 §1.3).
pub const FIND_M: f64 = 50.0;

/// Slack on the quick test that a body lies beyond a walk's reach, metres: far more than any
/// rounding at a map's scale.
const BOX_MARGIN_M: f64 = 1e-3;

/// A settlement's knowledge of a deposit: who found it, and when.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DepositKnown {
    /// The settlement that knows it.
    pub settlement: PermanentId,
    /// The deposit.
    pub deposit: PermanentId,
    /// Who found it.
    pub finder: PermanentId,
    /// When.
    pub at: SimTime,
}

/// The distance from `p` to the segment from `a` to `b`, metres.
fn to_segment(p: (f64, f64), a: (f64, f64), b: (f64, f64)) -> f64 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let len2 = dx * dx + dy * dy;
    let t = if len2 > 0.0 {
        (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / len2).clamp(0.0, 1.0)
    } else {
        0.0
    };
    (p.0 - (a.0 + t * dx)).hypot(p.1 - (a.1 + t * dy))
}

/// The side of a block of ground looked over, metres: twice how far a walker sees a body showing,
/// so a walk that crosses a block has about seen it (ADR-0043 §2; a design prior).
pub const LOOKED_BLOCK_M: f64 = 2.0 * FIND_M;

/// Cells on the side of a block of ground looked over on `map`.
pub fn looked_block_cells(map: &WorldMap) -> u32 {
    ((LOOKED_BLOCK_M / f64::from(map.cell_size_m).max(1e-6)).round() as u32).max(1)
}

/// The block of ground looked over that holds `cell` of `map`: blocks of
/// [`looked_block_cells`] a side, row by row from the map's corner.
pub fn looked_block(map: &WorldMap, cell: usize) -> u32 {
    let b = looked_block_cells(map) as usize;
    let width = map.width as usize;
    let cols = width.div_ceil(b);
    let (x, y) = (cell % width, cell / width);
    ((y / b) * cols + x / b) as u32
}

/// The ground each settlement's prospectors have walked over (ADR-0043 §2), by blocks of
/// [`looked_block`], with the day each was first looked over. A household of no settlement keeps
/// its own, under its own id. Nothing fades: a body showing at the surface does not appear later.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GroundLooked {
    by: BTreeMap<PermanentId, BTreeMap<u32, i32>>,
}

impl GroundLooked {
    /// `key`'s prospectors looked over `block` on `day`; the first day is kept.
    pub fn mark(&mut self, key: PermanentId, block: u32, day: i32) {
        self.by.entry(key).or_default().entry(block).or_insert(day);
    }

    /// Whether `key`'s prospectors have looked over `block`.
    pub fn looked(&self, key: PermanentId, block: u32) -> bool {
        self.by.get(&key).is_some_and(|b| b.contains_key(&block))
    }

    /// How many blocks `key`'s prospectors have looked over.
    pub fn blocks(&self, key: PermanentId) -> usize {
        self.by.get(&key).map_or(0, BTreeMap::len)
    }

    /// Every block looked over, `(key, block, day)`, by key and block.
    pub fn entries(&self) -> impl Iterator<Item = (PermanentId, u32, i32)> + '_ {
        self.by
            .iter()
            .flat_map(|(&k, b)| b.iter().map(move |(&block, &day)| (k, block, day)))
    }
}

/// Places drawn to look at when choosing where to prospect (M11d): enough to find ground of the
/// right kind within reach where there is any, few enough to draw on every decision.
const PROSPECT_DRAWS: usize = 32;

/// Its key in the world's draws.
const PURPOSE_PROSPECT: u64 = 0x7072_6f73_7065_6374; // "prospect"

/// A walk to look for a body (M11d): out to `at`, across to `across`, and home; `walk_min` is the
/// walk out and `loop_min` the whole of it; `fresh` the share of the reachable ground of the right
/// kind not yet looked over (ADR-0043 §3).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Prospect {
    pub at: (f32, f32),
    pub across: (f32, f32),
    pub walk_min: f64,
    pub loop_min: f64,
    pub fresh: f64,
}

impl Population {
    /// Where someone of household `household` would look on day `day` for a body of good `good`
    /// (M11d): two places drawn afresh each day on ground of the slope the land profile's rule for
    /// it lays bodies on, within `max_walk_min` of its home by its walking field `reach`, and not
    /// yet looked over by its settlement's prospectors (ADR-0043 §3), the first and the one
    /// farthest from it, the walks, and the share of such ground not yet looked over. None where
    /// the profile lays no such body, no such ground is in reach, or all of it has been looked
    /// over. Looking finds what walking brings ([`Self::look_for_deposits`]).
    pub(super) fn prospect_points(
        &self,
        ctx: &Ctx,
        household: PermanentId,
        reach: &TravelField,
        good: usize,
        max_walk_min: f64,
        day: i64,
    ) -> Option<Prospect> {
        let hh = self.household(household)?;
        let (home, key) = (hh.home, hh.settlement.unwrap_or(household));
        let rule = ctx.land_params.deposits.iter().find(|r| r.good == good)?;
        let map = ctx.map;
        let kind = |cell: usize| {
            let slope = f64::from(civ_world::terrain::slope_at(map, cell));
            map.water[cell] == WATER_LAND
                && slope >= rule.slope.0
                && slope <= rule.slope.1
                && reach
                    .seconds_to(cell)
                    .is_some_and(|s| f64::from(s) / 60.0 <= max_walk_min)
        };
        let mut rng = Rng64::from_key(&[ctx.seed, PURPOSE_PROSPECT, household.get(), day as u64]);
        // As far as a walk of that long goes at an easy pace off the trails.
        let radius =
            max_walk_min * 60.0 * ctx.params.nav.tobler_ms(0.0) * ctx.params.nav.offtrail_factor;
        let (w, h) = (map.width as f64, map.height as f64);
        let cell_m = f64::from(map.cell_size_m);
        let mut found: Vec<((f32, f32), f64)> = Vec::new();
        for _ in 0..PROSPECT_DRAWS {
            let angle = rng.next_f64() * std::f64::consts::TAU;
            let r = radius * rng.next_f64().sqrt();
            let (x, y) = (
                f64::from(home.0) + r * angle.cos(),
                f64::from(home.1) + r * angle.sin(),
            );
            if !(x >= 0.0 && y >= 0.0 && x < w * cell_m && y < h * cell_m) {
                continue;
            }
            let at = (x as f32, y as f32);
            let cell = cell_of(map, at);
            if map.water[cell] != WATER_LAND {
                continue;
            }
            let slope = f64::from(civ_world::terrain::slope_at(map, cell));
            if slope < rule.slope.0 || slope > rule.slope.1 {
                continue;
            }
            let Some(secs) = reach.seconds_to(cell) else {
                continue;
            };
            let walk = f64::from(secs) / 60.0;
            // Ground they have looked over already is not looked at again (ADR-0043 §3).
            if walk <= max_walk_min && !self.ground_looked.looked(key, looked_block(map, cell)) {
                found.push((at, walk));
            }
        }
        // Anywhere of the kind within reach not yet looked over, a day at a time: the search
        // covers the ground over the days, a near place worth more than a far. From there across
        // to the place of the day's draws farthest from it, and home: a loop over the ground
        // rather than out and back along one line.
        let &(at, walk_min) = found.first()?;
        // The share of the ground of the kind within reach not yet looked over, by blocks, each
        // judged by its middle cell; none of the kind measured, all of it is fresh.
        let fresh = if self.ground_looked.blocks(key) == 0 {
            1.0
        } else {
            let b = looked_block_cells(map);
            let side = f64::from(b) * cell_m;
            let (cols, rows) = (map.width.div_ceil(b), map.height.div_ceil(b));
            let span = |c: f64, n: u32| {
                let lo = ((c - radius) / side).floor().max(0.0) as u32;
                let hi = (((c + radius) / side).floor().max(0.0) as u32).min(n.saturating_sub(1));
                lo..=hi
            };
            let (mut all, mut looked) = (0u32, 0u32);
            for by in span(f64::from(home.1), rows) {
                for bx in span(f64::from(home.0), cols) {
                    let x = (bx * b + b / 2).min(map.width - 1);
                    let y = (by * b + b / 2).min(map.height - 1);
                    let cell = (y * map.width + x) as usize;
                    if kind(cell) {
                        all += 1;
                        if self.ground_looked.looked(key, by * cols + bx) {
                            looked += 1;
                        }
                    }
                }
            }
            if all == 0 {
                1.0
            } else {
                f64::from(all - looked) / f64::from(all)
            }
        };
        if fresh <= 0.0 {
            return None;
        }
        let span = |p: (f32, f32)| f64::from((p.0 - at.0).hypot(p.1 - at.1));
        let (across, back_min) = found
            .iter()
            .copied()
            .max_by(|a, b| span(a.0).total_cmp(&span(b.0)))
            .unwrap_or((at, walk_min));
        let pace = (ctx.params.nav.tobler_ms(0.0) * ctx.params.nav.offtrail_factor).max(0.1);
        let loop_min = walk_min + span(across) / pace / 60.0 + back_min;
        Some(Prospect {
            at,
            across,
            walk_min,
            loop_min,
            fresh,
        })
    }

    /// Person `who` of household `household` walked the land cells `cells` looking for a body
    /// (ADR-0043 §2): its settlement's prospectors have looked over the blocks they cross.
    pub(super) fn look_over(
        &mut self,
        map: &WorldMap,
        household: PermanentId,
        cells: &[u32],
        day: i64,
    ) {
        let Some(hh) = self.household(household) else {
            return;
        };
        let key = hh.settlement.unwrap_or(household);
        let day = i32::try_from(day).unwrap_or(i32::MAX);
        for &c in cells {
            self.ground_looked
                .mark(key, looked_block(map, c as usize), day);
        }
    }

    /// Where someone of household `household` would look on day `day` for a body of good `good`
    /// within `max_walk_min` of home ([`Self::prospect_points`]), by its settlement's walking field
    /// once one is drawn: for tests.
    #[doc(hidden)]
    pub fn prospect_points_for_tests(
        &self,
        ctx: &Ctx,
        household: PermanentId,
        good: usize,
        max_walk_min: f64,
        day: i64,
    ) -> Option<Prospect> {
        let hh = self.household(household)?;
        let key = hh.settlement.unwrap_or(household);
        let reach = &self.homes.get(&key)?.reach;
        self.prospect_points(ctx, household, reach, good, max_walk_min, day)
    }

    /// Whether settlement `settlement` knows deposit `deposit`.
    pub fn knows_deposit(&self, settlement: PermanentId, deposit: PermanentId) -> bool {
        self.deposits_known
            .iter()
            .any(|k| k.settlement == settlement && k.deposit == deposit)
    }

    /// Person `who` of household `household` walked the route `points` (metres): their
    /// settlement finds every body showing within [`FIND_M`] of the way that it did not know, and
    /// the chronicle says so. A household of no settlement keeps no such record.
    pub(super) fn look_for_deposits(
        &mut self,
        ctx: &mut Ctx,
        who: PermanentId,
        household: PermanentId,
        points: &[(f32, f32)],
    ) {
        let Some(settlement) = self.household(household).and_then(|h| h.settlement) else {
            return;
        };
        let route: Vec<(f64, f64)> = points
            .iter()
            .map(|&(x, y)| (f64::from(x), f64::from(y)))
            .collect();
        // A body farther than its reach from the box round a stretch of the way, along either
        // axis, is out of reach of every point of it: the box is only a quick way to say no (the
        // margin covers rounding in `to_segment`).
        let span = |a: &[(f64, f64)]| {
            a.iter().fold(
                (
                    f64::INFINITY,
                    f64::INFINITY,
                    f64::NEG_INFINITY,
                    f64::NEG_INFINITY,
                ),
                |(x0, y0, x1, y1), &(x, y)| (x0.min(x), y0.min(y), x1.max(x), y1.max(y)),
            )
        };
        let beyond = |at: (f64, f64), reach: f64, (x0, y0, x1, y1): (f64, f64, f64, f64)| {
            let far = reach + BOX_MARGIN_M;
            at.0 < x0 - far || at.0 > x1 + far || at.1 < y0 - far || at.1 > y1 + far
        };
        let whole = span(&route);
        let near = |at: (f64, f64), reach: f64| {
            if beyond(at, reach, whole) {
                return false;
            }
            match route.as_slice() {
                [] => false,
                [only] => (at.0 - only.0).hypot(at.1 - only.1) <= reach,
                r => r
                    .windows(2)
                    .any(|w| !beyond(at, reach, span(w)) && to_segment(at, w[0], w[1]) <= reach),
            }
        };
        let found: Vec<(PermanentId, u16, (f64, f64))> = ctx
            .land
            .deposits
            .iter()
            .filter(|d| d.body.exposed && d.left_kg() > 0.0)
            .filter_map(|d| {
                let at = (d.body.at_cm.0 as f64 / 100.0, d.body.at_cm.1 as f64 / 100.0);
                let reach = f64::from(d.body.radius_cm) / 100.0 + FIND_M;
                near(at, reach).then_some((d.id, d.body.good, at))
            })
            .filter(|&(id, _, _)| !self.knows_deposit(settlement, id))
            .collect();
        for (deposit, good, at) in found {
            self.deposits_known.push(DepositKnown {
                settlement,
                deposit,
                finder: who,
                at: ctx.now,
            });
            let what = ctx
                .catalog
                .goods
                .get(usize::from(good))
                .map_or_else(|| "a deposit".to_owned(), |g| g.name.to_lowercase());
            self.chronicle_push(
                ctx.now,
                ChronicleKind::DepositFound,
                vec![who],
                Some(settlement),
                Some((at.0 as f32, at.1 as f32)),
                0.0,
                format!("{what} showing at the surface"),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{GroundLooked, to_segment};
    use civ_core::PermanentId;

    #[test]
    fn ground_looked_over_keeps_the_first_day_for_each_settlement() {
        let (s, t) = (
            PermanentId::from_raw(3).expect("an id"),
            PermanentId::from_raw(9).expect("an id"),
        );
        let mut g = GroundLooked::default();
        g.mark(s, 40, 10);
        g.mark(s, 40, 12);
        g.mark(s, 7, 11);
        g.mark(t, 40, 5);
        assert!(g.looked(s, 40) && g.looked(t, 40) && !g.looked(t, 7));
        assert_eq!((g.blocks(s), g.blocks(t)), (2, 1));
        let all: Vec<_> = g.entries().collect();
        assert_eq!(all, vec![(s, 7, 11), (s, 40, 10), (t, 40, 5)]);
    }

    #[test]
    fn the_distance_to_a_segment_is_to_its_nearest_point() {
        assert!((to_segment((5.0, 3.0), (0.0, 0.0), (10.0, 0.0)) - 3.0).abs() < 1e-12);
        assert!((to_segment((-4.0, 3.0), (0.0, 0.0), (10.0, 0.0)) - 5.0).abs() < 1e-12);
        assert!((to_segment((1.0, 1.0), (2.0, 2.0), (2.0, 2.0)) - 2f64.sqrt()).abs() < 1e-12);
    }
}
