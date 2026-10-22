//! Engines at pits (M10a slice DS, third part; ADR-0033 §7; the M10 power brief §1.4): an engine
//! set at a pit burns fuel to pump the water that comes into it, so its diggers need not bail.
//! Each day an engine whose owner holds the engine whole and fuel for it lifts out what stands in
//! its pit, at most its rated power for the hours it is kept going, burning the owner's fuels by
//! their heat at the engine's efficiency (research 07-06 §2.3: 3.6 / (MJ η) kg a kWh), and the
//! engine wears by the hours it ran. Who sets an engine, and why, is the next part.

use super::*;
use crate::params::PowerSource;
use civ_land::power::{Engine, fuel_kg, lift_kwh};

/// The share of an engine's work its pump turns into lifting water (research 07-06 §2.2's 50 %
/// for lifting by a wheel or a pump; a tuning value).
const PUMP_EFFICIENCY: f64 = 0.5;

/// The hours a day an engine is kept going: fed, stoked and watched in daylight (a design prior;
/// its tending is not yet counted in anyone's hours).
const ENGINE_HOURS_A_DAY: f64 = 12.0;

impl Population {
    /// Every engine pumps its pit for the day (M10a slice DS, third part).
    pub(super) fn engines_day(&mut self, ctx: &mut Ctx) {
        for i in 0..ctx.land.engines.len() {
            let e = ctx.land.engines[i];
            self.pump(ctx, &e);
        }
    }

    /// Every engine pumps its pit now, whatever the hour: for tests.
    #[doc(hidden)]
    pub fn engines_day_for_tests(&mut self, ctx: &mut Ctx) {
        self.engines_day(ctx);
    }

    /// Sets an engine of good `good` that household `owner` holds at pit `pit`, for tests: the
    /// engine's record, with the good's rated power and efficiency.
    #[doc(hidden)]
    pub fn set_engine_for_tests(
        &mut self,
        ctx: &mut Ctx,
        owner: PermanentId,
        pit: PermanentId,
        good: usize,
    ) -> Option<PermanentId> {
        let power = ctx.catalog.goods.get(good)?.power?;
        if power.source != PowerSource::Fuel {
            return None;
        }
        let id = ctx.ids.allocate();
        ctx.land.engines.push(Engine {
            id,
            pit,
            owner,
            rated_kw: power.rated_kw,
            efficiency: power.efficiency,
        });
        Some(id)
    }

    /// Engine `e` lifts out what stands in its pit today, as far as its hours and its owner's
    /// fuel allow.
    fn pump(&mut self, ctx: &mut Ctx, e: &Engine) {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let Some(state) = ctx.land.pit_water_now(
            ctx.map,
            ctx.land_params,
            params.digging.influence_m,
            e.pit,
            now,
        ) else {
            return;
        };
        if state.water_m3 <= 0.0 {
            return;
        }
        let Some(&hd) = self.hh_index.get(&e.owner) else {
            return;
        };
        let Some(o) = self.households.get_mut(hd) else {
            return;
        };
        let members = o.members.len().max(1);
        o.settle_stores(now, goods, &|d| fuel_per_day(params, members, d));
        o.stores.resize(goods.len(), 0.0);
        // The engine itself, whole, among the owner's things.
        let Some(engine) = (0..goods.len()).find(|&g| {
            goods[g]
                .power
                .is_some_and(|p| p.source == PowerSource::Fuel)
                && o.stores[g] >= decide::MIN_TOOL
        }) else {
            return;
        };
        // The work it would take to lift it all out, and as much as its hours give.
        let lift = state.lift_m();
        let need_kwh = lift_kwh(state.water_m3, lift, PUMP_EFFICIENCY);
        let mut kwh = need_kwh.min(e.rated_kw * ENGINE_HOURS_A_DAY);
        // Its fuel: the owner's fuels, in the catalogue's order, by their heat.
        let before = o.stores.clone();
        let mut done = 0.0;
        for (g, d) in goods.iter().enumerate() {
            if kwh <= 1e-12 {
                break;
            }
            let Some(f) = d.fuel.filter(|_| d.purpose == GoodUse::Fuel) else {
                continue;
            };
            let per_kwh = fuel_kg(1.0, f.mj_per_kg, e.efficiency);
            if per_kwh <= 0.0 {
                continue;
            }
            let burn = (kwh * per_kwh).min(o.stores[g].max(0.0));
            o.stores[g] -= burn;
            done += burn / per_kwh;
            kwh -= burn / per_kwh;
        }
        if done <= 0.0 {
            return;
        }
        let hours = done / e.rated_kw.max(1e-9);
        make::wear(&mut o.stores, goods, &[engine], hours);
        o.flows
            .add_changes(&before, &o.stores, Flow::Made, Flow::Used);
        let left = state.water_m3 * (1.0 - (done / need_kwh.max(1e-12)).min(1.0));
        ctx.land.set_pit_water(e.pit, left, now);
    }
}
