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

/// Places drawn to look at when choosing where to prospect (M11d): enough to find ground of the
/// right kind within reach where there is any, few enough to draw on every decision.
const PROSPECT_DRAWS: usize = 32;

/// Its key in the world's draws.
const PURPOSE_PROSPECT: u64 = 0x7072_6f73_7065_6374; // "prospect"

/// A walk to look for a body (M11d): out to `at`, across to `across`, and home; `walk_min` is the
/// walk out and `loop_min` the whole of it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Prospect {
    pub at: (f32, f32),
    pub across: (f32, f32),
    pub walk_min: f64,
    pub loop_min: f64,
}

impl Population {
    /// Where someone of household `household` would look on day `day` for a body of good `good`
    /// (M11d): two places drawn afresh each day on ground of the slope the land profile's rule for
    /// it lays bodies on, within `max_walk_min` of its home by its walking field `reach`, the first
    /// and the one farthest from it, and the walks. None where the profile lays no such body or no
    /// such ground is in reach.
    /// Looking finds what walking brings ([`Self::look_for_deposits`]).
    pub(super) fn prospect_points(
        &self,
        ctx: &Ctx,
        household: PermanentId,
        reach: &TravelField,
        good: usize,
        max_walk_min: f64,
        day: i64,
    ) -> Option<Prospect> {
        let home = self.household(household)?.home;
        let rule = ctx.land_params.deposits.iter().find(|r| r.good == good)?;
        let map = ctx.map;
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
            if walk <= max_walk_min {
                found.push((at, walk));
            }
        }
        // Anywhere of the kind within reach, a day at a time: with no memory of where they have
        // looked, the search covers the ground over the days, a near place worth more than a far.
        // From there across to the place of the day's draws farthest from it, and home: a loop
        // over the ground rather than out and back along one line.
        let &(at, walk_min) = found.first()?;
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
        })
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
    use super::to_segment;

    #[test]
    fn the_distance_to_a_segment_is_to_its_nearest_point() {
        assert!((to_segment((5.0, 3.0), (0.0, 0.0), (10.0, 0.0)) - 3.0).abs() < 1e-12);
        assert!((to_segment((-4.0, 3.0), (0.0, 0.0), (10.0, 0.0)) - 5.0).abs() < 1e-12);
        assert!((to_segment((1.0, 1.0), (2.0, 2.0), (2.0, 2.0)) - 2f64.sqrt()).abs() < 1e-12);
    }
}
