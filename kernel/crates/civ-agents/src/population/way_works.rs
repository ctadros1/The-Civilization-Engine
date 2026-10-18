//! Works on ways by a household (M8b slice CQ, second part; ADR-0029 §6; the transport brief
//! §1.8; research 10-07 §1.6, §2.2): a passing place where its carts wait. At its yearly review a
//! household weighs each single-cart stretch where it holds waits (`traffic::Cause::Width`): a
//! passing place cleared to two carts' width at the stretch's middle would spare it about half of
//! them, the stretch split in two, over the years it reckons such a place lasts; making one
//! takes the hours clearing and levelling its ground takes. Where the first is more, and the
//! place is within the work's walk from home, it begins one: a way of one cell, kept by it, its
//! built width growing with each hour of its people's work until it is as wide as two carts.

use super::*;
use crate::traffic;
use civ_land::ways::{CART, Keeper, Rank, Segment, Surface, WALK, Way};

/// Purpose key for when in the year a household weighs a passing place.
const PURPOSE_WAY_REVIEW: u64 = 0x7761_7977_6f72_6b72; // "wayworkr"
/// The share of the waits at a single-cart stretch a passing place at its middle spares: the
/// stretch split in two, a cart waits for one coming the other way on half of it (a design prior).
pub const PASSING_SPARES: f64 = 0.5;

impl Population {
    /// The way household `household` is working on: one it keeps whose built width is short of
    /// its corridor.
    fn way_in_work(ctx: &Ctx, household: PermanentId) -> Option<usize> {
        ctx.land.ways.list.iter().position(|w| {
            w.segments.iter().any(|s| {
                s.keeper == Keeper::Household(household) && s.built_m + 1e-3 < s.corridor_m
            })
        })
    }

    /// The hours a year household `household` holds of waiting at the single-cart stretch whose
    /// middle is `cell`, as of `day`.
    fn width_hours_a_year(&self, ctx: &Ctx, household: PermanentId, cell: u32, day: i64) -> f64 {
        let half = ctx.params.places.use_half_life_days;
        self.traffic
            .losses_held(household, day, half)
            .filter(|l| l.cell == cell && l.cause == traffic::Cause::Width)
            .map(|l| crate::uses::per_year(f64::from(l.hours), half))
            .sum()
    }

    /// Households whose day of the year it is weigh a passing place where their carts wait.
    pub(super) fn way_work_reviews(&mut self, ctx: &mut Ctx, day: i64) {
        if !ctx
            .catalog
            .activities
            .iter()
            .any(|a| a.behavior == Behavior::Way)
        {
            return;
        }
        let due: Vec<PermanentId> = self
            .traffic
            .losses
            .keys()
            .copied()
            .filter(|h| {
                Rng64::from_key(&[ctx.seed, PURPOSE_WAY_REVIEW, h.get()]).next_u64()
                    % DAYS_PER_YEAR as u64
                    == day.rem_euclid(DAYS_PER_YEAR) as u64
            })
            .collect();
        for h in due {
            self.review_way_work(ctx, h, day);
        }
    }

    /// Household `household` weighs a passing place now, whatever its day of the year: for tests.
    #[doc(hidden)]
    pub fn review_way_work_for_tests(&mut self, ctx: &mut Ctx, household: PermanentId) {
        let day = ctx.now.day_index();
        self.review_way_work(ctx, household, day);
    }

    /// Household `household` weighs a passing place at each single-cart stretch where it holds
    /// waits, and begins the one whose saving most exceeds its work, if any does: one at a time,
    /// and none where a way already runs.
    fn review_way_work(&mut self, ctx: &mut Ctx, household: PermanentId, day: i64) {
        let Some(x) = self.household(household) else {
            return;
        };
        if x.members.is_empty() || Self::way_in_work(ctx, household).is_some() {
            return;
        }
        let Some(act) = ctx
            .catalog
            .activities
            .iter()
            .find(|a| a.behavior == Behavior::Way)
        else {
            return;
        };
        let works = ctx.land_params.way_works;
        let cell_m = f64::from(ctx.map.cell_size_m);
        let tread = civ_land::carting::TRAIL_TREAD_M as f32;
        let widen = f64::from((works.passing_width_m - tread).max(0.0));
        let labour_h = widen * cell_m * works.clear_h_m2;
        let pace = ctx.params.nav.tobler_ms(0.0).max(1e-6);
        let home = x.home;
        let half = ctx.params.places.use_half_life_days;
        let mut best: Option<(u32, f64)> = None;
        for l in self.traffic.losses_held(household, day, half) {
            if l.cause != traffic::Cause::Width || ctx.land.ways.at(l.cell).next().is_some() {
                continue;
            }
            let at = cell_centre(ctx.map, l.cell as usize);
            let walk_min = f64::from((at.0 - home.0).hypot(at.1 - home.1)) / pace / 60.0;
            if walk_min > f64::from(act.max_walk_minutes) {
                continue;
            }
            let saved_h = crate::uses::per_year(f64::from(l.hours), half)
                * PASSING_SPARES
                * works.passing_life_years;
            let gain = saved_h - labour_h;
            if gain > 0.0 && best.is_none_or(|(_, g)| gain > g) {
                best = Some((l.cell, gain));
            }
        }
        let Some((cell, _)) = best else {
            return;
        };
        let id = ctx.ids.allocate();
        ctx.land.ways.add(Way {
            id,
            begun: day,
            segments: vec![Segment {
                cells: vec![cell],
                corridor_m: works.passing_width_m.max(tread),
                built_m: tread,
                usable_m: tread,
                surface: Surface::Cleared,
                keeper: Keeper::Household(household),
                kept: day,
            }],
            modes: WALK | CART,
            trips: 0.0,
            trips_day: day,
            rank: Rank::Lane,
        });
    }

    /// The passing place household `x` is making, if it is light: where its work is done, the
    /// walk there from home, the hours of a capable adult's work it still takes, and the hours of
    /// waiting each hour of that work spares the household over the place's life.
    pub(super) fn way_facts(
        &self,
        ctx: &Ctx,
        x: &Household,
        dark: bool,
    ) -> Option<crate::decide::BoatFacts> {
        if dark {
            return None;
        }
        let w = &ctx.land.ways.list[Self::way_in_work(ctx, x.id)?];
        let s = w.segments.first()?;
        let cell = *s.cells.first()?;
        let works = ctx.land_params.way_works;
        let metres = f64::from(ctx.map.cell_size_m) * s.cells.len() as f64;
        let left_h = f64::from(s.corridor_m - s.built_m).max(0.0) * metres * works.clear_h_m2;
        let labour_h = f64::from(s.corridor_m - civ_land::carting::TRAIL_TREAD_M as f32).max(0.1)
            * metres
            * works.clear_h_m2;
        let at = cell_centre(ctx.map, cell as usize);
        let pace = ctx.params.nav.tobler_ms(0.0).max(1e-6);
        let saved_h = self.width_hours_a_year(ctx, x.id, cell, ctx.now.day_index())
            * PASSING_SPARES
            * works.passing_life_years;
        Some(crate::decide::BoatFacts {
            boat: w.id,
            at,
            walk_min: f64::from((at.0 - x.home.0).hypot(at.1 - x.home.1)) / pace / 60.0,
            left_h,
            saves_per_hour: saved_h / labour_h,
            full: false,
        })
    }

    /// `hours` of a capable adult's work by someone of `household` on the way `way` it is making:
    /// each clears and levels its share of a square metre a `clear_h_m2`, widening the stretch
    /// along its length, until it is as wide as its corridor. It is kept that day.
    pub(super) fn way_work(ctx: &mut Ctx, household: PermanentId, way: PermanentId, hours: f64) {
        let works = ctx.land_params.way_works;
        let cell_m = f64::from(ctx.map.cell_size_m);
        let day = ctx.now.day_index();
        let Some(w) = ctx.land.ways.list.iter_mut().find(|w| w.id == way) else {
            return;
        };
        let mut changed = false;
        for s in &mut w.segments {
            if s.keeper != Keeper::Household(household) || s.built_m + 1e-3 >= s.corridor_m {
                continue;
            }
            let area = cell_m * s.cells.len() as f64;
            let wider = (hours / works.clear_h_m2 / area.max(1e-6)) as f32;
            s.built_m = (s.built_m + wider).min(s.corridor_m);
            s.usable_m = s.built_m;
            s.kept = day;
            changed = true;
            break;
        }
        if changed {
            ctx.land.ways.changed();
        }
    }
}
