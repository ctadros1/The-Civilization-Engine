//! Gas lit each evening (M10b slice DY, third part; ADR-0034 §8; research 12-09 §5.2, §5.5): at
//! dusk each main's district is checked, the gas its works' holder keeps and the main's capacity
//! against the burners connected to it, lit for the evening's hours, and the gas served is drawn
//! from the works' household. A connection lit is light at its home after dark, as a lamp is.

use super::{AgentEvent, Ctx, Flow, Population, fuel_per_day};

/// The hours a connection's burners burn of an evening (research 12-09 §2.3: house lamps lit
/// some four hours a day).
pub const GAS_EVENING_H: f64 = 4.0;

impl Population {
    /// Schedules today's lighting of gas at dusk (M10b slice DY), while any main is laid.
    pub(super) fn schedule_gas(&mut self, ctx: &mut Ctx) {
        if ctx.land.gas.mains.is_empty() {
            return;
        }
        let doy = civ_land::day_of_year(ctx.now.day_index());
        let (rise, set) = crate::needs::daylight(ctx.params.latitude_deg, doy);
        let start = ctx.now.plus_minutes(-ctx.now.minute_of_day());
        ctx.schedule
            .push((start.plus_minutes(set.max(rise + 1)), AgentEvent::Gas));
    }

    /// The evening's gas (M10b slice DY): each main in turn lights its connections for
    /// [`GAS_EVENING_H`] from now, as far as the main carries their burners and its works' gas
    /// lasts, and the gas served is drawn from the works' household's stores.
    pub(super) fn gas_evening(&mut self, ctx: &mut Ctx) {
        let goods = &ctx.catalog.goods;
        // The piped burner and the gas it burns.
        let Some((gas, kg_h)) = goods.iter().find_map(|d| {
            let l = d.light.filter(|l| l.piped)?;
            Some((l.fuel?, l.kg_per_hour))
        }) else {
            return;
        };
        let (now, params) = (ctx.now, ctx.params);
        let day = now.day_index();
        let mains: Vec<_> = ctx.land.gas.mains.iter().map(|m| (m.id, m.works)).collect();
        for (main, works) in mains {
            let held = match self
                .hh_index
                .get(&works)
                .and_then(|&hd| self.households.get_mut(hd))
            {
                Some(x) => {
                    let members = x.members.len().max(1);
                    x.settle_stores(now, goods, &|d| fuel_per_day(params, members, d));
                    x.stores.resize(goods.len(), 0.0);
                    x.stores[gas]
                }
                None => 0.0,
            };
            let drawn = ctx
                .land
                .gas
                .evening(main, day, now.minutes(), GAS_EVENING_H, held, kg_h);
            if drawn > 0.0
                && let Some(x) = self
                    .hh_index
                    .get(&works)
                    .and_then(|&hd| self.households.get_mut(hd))
            {
                x.stores[gas] = (x.stores[gas] - drawn).max(0.0);
                x.flows.add(Flow::Used, gas, drawn);
            }
        }
    }

    /// The evening's gas lit now: for tests.
    #[doc(hidden)]
    pub fn gas_evening_for_tests(&mut self, ctx: &mut Ctx) {
        self.gas_evening(ctx);
    }
}
