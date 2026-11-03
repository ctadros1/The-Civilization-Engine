//! Lifts in buildings (ADR-0042 §2): a household's whole passenger lifts stand in its frame
//! buildings of version 4 and more than one storey, one each, the tallest first (the first built
//! among equals). A lift's shaft takes its floor on every storey; while its keeper has power that
//! day (its installations' or its engines' shaft, on the fuel it holds for the lift's hours, at
//! least the lift's drive, or a hookup of its on a wire that was served), a trip out and back from
//! a storey above the ground takes the lift's wait and ride where that is quicker than the stair.
//! Nothing else reads a lift.

use super::*;
use crate::params::GoodLift;

/// Hours a day a lift's drive runs, for an engine's fuel to last (a design prior).
pub const LIFT_HOURS: f64 = 4.0;

impl Population {
    /// The lift standing in building `b`, if any (ADR-0042 §2): its keeper's whole lifts, each
    /// in one of its finished frame buildings of version 4 and more than one storey, the tallest
    /// first.
    pub(super) fn lift_in(&self, land: &Land, goods: &[GoodDef], b: &Building) -> Option<GoodLift> {
        if !Self::takes_lift(b) {
            return None;
        }
        let keeper = self.household(b.household)?;
        let mut lifts: Vec<GoodLift> = Vec::new();
        for (g, d) in goods.iter().enumerate() {
            if let Some(l) = d.lift {
                let whole = keeper
                    .stores
                    .get(g)
                    .copied()
                    .unwrap_or(0.0)
                    .max(0.0)
                    .floor();
                lifts.extend(std::iter::repeat_n(l, whole as usize));
            }
        }
        if lifts.is_empty() {
            return None;
        }
        let mut tall: Vec<&Building> = land
            .buildings
            .iter()
            .filter(|x| x.household == b.household && Self::takes_lift(x))
            .collect();
        tall.sort_by_key(|x| (std::cmp::Reverse(x.spec.storeys), x.id));
        let at = tall.iter().position(|x| x.id == b.id)?;
        lifts.get(at).copied()
    }

    /// Whether building `b` may hold a lift: a finished, standing frame of version 4 and more than
    /// one storey.
    fn takes_lift(b: &Building) -> bool {
        matches!(b.spec.footprint, civ_grammar::Footprint::Rect { .. })
            && b.spec.version >= 4
            && b.spec.storeys > 1
            && b.finished()
            && b.standing()
    }

    /// Whether household `keeper` has power today for a lift driven by `drive_kw` (ADR-0042 §2;
    /// ADR-0033 §5, ADR-0036): its installations' and its engines' shaft for [`LIFT_HOURS`] on
    /// the fuel it holds, or a hookup of its on a wire that was served and is not cut off.
    pub(super) fn has_power(&self, ctx: &Ctx, keeper: PermanentId, drive_kw: f64) -> bool {
        let hooked = ctx.land.wires.hookups.iter().any(|h| {
            h.household == keeper
                && !h.cut_off
                && h.cause == civ_land::wire::Cause::Served
                && h.lit_until >= 0
        });
        if hooked {
            return true;
        }
        let engines = self.household(keeper).map_or(0.0, |x| {
            let stores = stores_now(x, ctx.now, ctx.params, &ctx.catalog.goods);
            engine_kw(&ctx.catalog.goods, &stores, LIFT_HOURS).0
        });
        power_kw_of(ctx, keeper) + engines + 1e-9 >= drive_kw
    }

    /// Seconds a trip out and back from storey `storey` of building `b` takes today (ADR-0042
    /// §1–§2): by the stair, or by its lift where it has one with power and the lift is quicker.
    pub(super) fn trip_s_in(&self, ctx: &Ctx, b: &Building, storey: usize) -> f64 {
        let stair = ctx.params.climbing.stair_trip_s(storey);
        if storey == 0 {
            return stair;
        }
        let Some(lift) = self.lift_in(ctx.land, &ctx.catalog.goods, b) else {
            return stair;
        };
        if !self.has_power(ctx, b.household, lift.drive_kw) {
            return stair;
        }
        let storey_m = f64::from(b.spec.params[civ_grammar::frame_params::EAVE_CM].max(0)) / 100.0;
        let ride = crate::climb::lift_trip_s(&lift, storey, usize::from(b.spec.storeys), storey_m);
        stair.min(ride)
    }

    /// Seconds a trip out and back from storey `storey` of building `building` takes today: for
    /// tests.
    #[doc(hidden)]
    pub fn trip_s_for_tests(&self, ctx: &Ctx, building: PermanentId, storey: usize) -> f64 {
        ctx.land
            .buildings
            .iter()
            .find(|b| b.id == building)
            .map_or(0.0, |b| self.trip_s_in(ctx, b, storey))
    }
}
