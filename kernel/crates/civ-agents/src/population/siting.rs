//! Where a household builds a home, as a choice (M8c slice CS, second part; ADR-0030 §1). A
//! household about to build a home seeks clear ground about a few points: where it would have
//! built before (its home), the point whose walks to the places its members go from home sum
//! least, and halfway from there to each of the places they go most. It weighs each plot it finds
//! in hours: its members' walks a year from there to where they go, there and back, at the pace
//! people walk off trails, over the content's years, and the levelling the plot needs once. It
//! takes the cheapest, and the plot keeps the receipt. A household whose people have gone nowhere
//! from home yet seeks only about its home, as before.

use super::*;

/// The most places gone to that a home's plot is also sought toward.
const SITE_TOWARD: usize = 6;
/// Steps toward the point whose walks sum least (Weiszfeld's).
const MEDIAN_STEPS: usize = 24;
/// Two points to seek a plot about closer than this, metres, are one.
const ANCHOR_APART_M: f32 = 8.0;

impl Population {
    /// Where household `hh`'s members go from home, a year: each place's centre, metres, and the
    /// trips there, in place order.
    pub(super) fn goings_of(&self, ctx: &Ctx, hh: &Household) -> Vec<((f32, f32), f64)> {
        let (day, half) = (ctx.now.day_index(), ctx.params.places.use_half_life_days);
        let mut by: BTreeMap<u32, f64> = BTreeMap::new();
        for m in &hh.members {
            if let Some(g) = self.travel.goings.get(m) {
                for (place, n) in g.per_year(day, half) {
                    *by.entry(place).or_default() += n;
                }
            }
        }
        let (w, cs) = (ctx.map.width, ctx.map.cell_size_m);
        by.into_iter()
            .map(|(p, n)| (crate::travel::going_centre(p, w, cs), n))
            .collect()
    }

    /// Where household `household` would begin a first home today, and why: the plot's centre
    /// and its receipt; for tests.
    #[doc(hidden)]
    pub fn home_site_for_tests(
        &self,
        ctx: &Ctx,
        household: PermanentId,
    ) -> Option<((f32, f32), Option<civ_land::SiteReceipt>)> {
        let hh = self.household(household)?;
        let settlement = ctx
            .land
            .settlements
            .iter()
            .find(|s| Some(s.id) == hh.settlement)?;
        let site = self.new_building(
            ctx,
            hh,
            Some(settlement.hearth_m),
            settlement.id,
            24.0,
            None,
        )?;
        Some((crate::build::centre_m(&site.spec), site.receipt))
    }
}

/// The points a home's plot is sought about, `home` first: then the point whose walks to
/// `places` (centre, trips a year) sum least, and halfway from there to each of the
/// [`SITE_TOWARD`] places gone to most (the first among equals); none within [`ANCHOR_APART_M`]
/// of one before it. Only `home` when nobody went anywhere.
pub(super) fn site_anchors(home: (f32, f32), places: &[((f32, f32), f64)]) -> Vec<(f32, f32)> {
    let mut out = vec![home];
    let total: f64 = places.iter().map(|p| p.1).sum();
    if total <= 0.0 || !total.is_finite() {
        return out;
    }
    let median = least_walk_point(places);
    let mut push = |p: (f32, f32)| {
        if out
            .iter()
            .all(|q| (q.0 - p.0).hypot(q.1 - p.1) >= ANCHOR_APART_M)
        {
            out.push(p);
        }
    };
    push(median);
    let mut most: Vec<&((f32, f32), f64)> = places.iter().collect();
    most.sort_by(|a, b| b.1.total_cmp(&a.1));
    for &&(c, _) in most.iter().take(SITE_TOWARD) {
        push(((median.0 + c.0) / 2.0, (median.1 + c.1) / 2.0));
    }
    out
}

/// The point whose walks to `places` (centre, trips) sum least, as Weiszfeld's steps find it from
/// their weighted mean.
fn least_walk_point(places: &[((f32, f32), f64)]) -> (f32, f32) {
    let total: f64 = places.iter().map(|p| p.1).sum();
    let mut at = places.iter().fold((0.0, 0.0), |a, &(c, n)| {
        (
            a.0 + f64::from(c.0) * n / total,
            a.1 + f64::from(c.1) * n / total,
        )
    });
    for _ in 0..MEDIAN_STEPS {
        let (mut sx, mut sy, mut sw) = (0.0, 0.0, 0.0);
        for &((x, y), n) in places {
            let d = (f64::from(x) - at.0).hypot(f64::from(y) - at.1).max(0.5);
            sx += n * f64::from(x) / d;
            sy += n * f64::from(y) / d;
            sw += n / d;
        }
        if sw > 0.0 {
            at = (sx / sw, sy / sw);
        }
    }
    (at.0 as f32, at.1 as f32)
}

/// Hours a year of walking from `at` to `places` (centre, trips a year) and back, at `speed_ms`.
pub(super) fn walk_hours_year(at: (f32, f32), places: &[((f32, f32), f64)], speed_ms: f64) -> f64 {
    places
        .iter()
        .map(|&((x, y), n)| n * 2.0 * f64::from((x - at.0).hypot(y - at.1)))
        .sum::<f64>()
        / speed_ms.max(0.1)
        / 3600.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_home_is_sought_about_its_old_point_the_least_walk_and_toward_where_people_go() {
        // Nobody went anywhere: only the old point.
        assert_eq!(site_anchors((0.0, 0.0), &[]), vec![(0.0, 0.0)]);
        // Fields 400 m east gone to twice as often as the hearth at the old point: the least walk
        // lies at the fields (the weighted median of two points is the heavier one).
        let places = [((0.0, 0.0), 100.0), ((400.0, 0.0), 200.0)];
        let anchors = site_anchors((0.0, 0.0), &places);
        assert_eq!(anchors[0], (0.0, 0.0));
        assert!((anchors[1].0 - 400.0).abs() < 2.0, "{anchors:?}");
        // Halfway from there to the hearth.
        assert!(
            anchors.iter().any(|a| (a.0 - 200.0).abs() < 2.0),
            "{anchors:?}"
        );
        // The walks: 100 there-and-backs of 400 m and none to the fields, at 1 m/s.
        let h = walk_hours_year((400.0, 0.0), &places, 1.0);
        assert!((h - 100.0 * 800.0 / 3600.0).abs() < 1e-6);
        assert!(walk_hours_year((0.0, 0.0), &places, 1.0) > h);
    }
}
