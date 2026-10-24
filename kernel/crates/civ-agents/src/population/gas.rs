//! Gas lit each evening (M10b slice DY, third part; ADR-0034 §8; research 12-09 §5.2, §5.5): at
//! dusk each main's district is checked, the gas its works' holder keeps and the main's capacity
//! against the burners connected to it, lit for the evening's hours, and the gas served is drawn
//! from the works' household. A connection lit is light at its home after dark, as a lamp is.

use civ_core::PermanentId;
use civ_land::gas::Main;
use civ_land::ways::Keeper;

use super::{AgentEvent, Ctx, Flow, Population, cell_of, fuel_per_day};

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

    /// Begins a main from household `works`' home to cell `to` (M10b slice DY, third part, second
    /// step): routed along ways whose stretches' keepers give leave, nobody's, the works' own or
    /// its polity's ([`civ_land::gas::plan_main`]), marked out at its length with nothing yet
    /// laid, and carrying what the land profile's bore does. Returns its id; `None` where the land
    /// profile lays no main or no route keeps to such ways.
    pub fn begin_main(&self, ctx: &mut Ctx, works: PermanentId, to: u32) -> Option<PermanentId> {
        let params = ctx.land_params.gas_main?;
        let x = self.household(works)?;
        let from = cell_of(ctx.map, x.home) as u32;
        let polity = x
            .settlement
            .and_then(|s| self.polity_of(s))
            .map(|pi| self.polities[pi].id);
        let cells =
            civ_land::gas::plan_main(&ctx.land.ways, ctx.map.width, from, to, |k| match k {
                Keeper::Nobody => true,
                Keeper::Household(h) => h == works,
                Keeper::Polity(p) => Some(p) == polity,
            })?;
        let length_m: f64 =
            civ_land::rail::runs(&cells, ctx.map.width, f64::from(ctx.map.cell_size_m))
                .iter()
                .sum();
        let id = ctx.ids.allocate();
        ctx.land.gas.mains.push(Main {
            id,
            works,
            cells,
            capacity_kg_h: params.capacity_kg_h,
            broken: false,
            length_m: length_m as f32,
            laid_m: 0.0,
        });
        Some(id)
    }

    /// `hours` of work by someone of `household` on main `main` (M10b slice DY, third part,
    /// second step): its works' own household digs, lays and fills in its pipe, a metre each
    /// `lay_h_m` hours, each taking its pipe from the household's store, no more hours than the
    /// pipe held allows. The main carries gas only once whole.
    pub(super) fn main_work(
        &mut self,
        ctx: &mut Ctx,
        household: PermanentId,
        main: PermanentId,
        hours: f64,
    ) {
        let Some(params) = ctx.land_params.gas_main else {
            return;
        };
        let Some(m) = ctx
            .land
            .gas
            .mains
            .iter_mut()
            .find(|m| m.id == main && m.works == household && !m.whole())
        else {
            return;
        };
        let Some(x) = self.household_mut(household) else {
            return;
        };
        let held = x.stores.get(params.good).copied().unwrap_or(0.0).max(0.0);
        // Metres the hours lay, no more than the pipe held makes or the main has left.
        let metres = (hours.max(0.0) / params.lay_h_m.max(1e-9))
            .min(held / params.good_kg_m.max(1e-9))
            .min(f64::from(m.length_m - m.laid_m).max(0.0));
        m.laid_m = (m.laid_m + metres as f32).min(m.length_m);
        if m.length_m - m.laid_m < 1e-3 {
            m.laid_m = m.length_m;
        }
        let pipe = (metres * params.good_kg_m).min(held);
        if pipe > 0.0 {
            x.stores[params.good] = held - pipe;
            x.flows.add(Flow::Used, params.good, pipe);
        }
    }

    /// Begins a main from household `works`' home to cell `to`: for tests.
    #[doc(hidden)]
    pub fn begin_main_for_tests(
        &self,
        ctx: &mut Ctx,
        works: PermanentId,
        to: u32,
    ) -> Option<PermanentId> {
        self.begin_main(ctx, works, to)
    }

    /// `hours` of work by someone of `household` on main `main`: for tests.
    #[doc(hidden)]
    pub fn main_work_for_tests(
        &mut self,
        ctx: &mut Ctx,
        household: PermanentId,
        main: PermanentId,
        hours: f64,
    ) {
        self.main_work(ctx, household, main, hours);
    }

    /// The evening's gas lit now: for tests.
    #[doc(hidden)]
    pub fn gas_evening_for_tests(&mut self, ctx: &mut Ctx) {
        self.gas_evening(ctx);
    }
}
