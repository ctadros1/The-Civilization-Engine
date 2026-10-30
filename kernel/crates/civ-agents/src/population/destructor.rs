//! The refuse destructor (M11f, fourth part; ADR-0038 §5; research 12-10 §1.5, §2.2, §5.5): an
//! engine whose one fuel is refuse, held beside a dynamo as a gas engine is. Refuse becomes a good
//! where a tip is: each midnight a household holding a whole destructor draws from its polity's
//! open tip, into its store as `refuse`, what its destructors would burn at their rating over an
//! evening, less what it holds; the tip's decaying kilograms and that share of each disease load
//! there move with it, recorded as carried (settled at midnight, as a round's run is). Burning
//! refuse, like any engine's fuel, leaves its `ash_share` as `ash` in the burner's store; burning
//! refuse also destroys that share of the disease it held.

use super::*;
use crate::contagion::{How, Move, Node};
use crate::params::PowerSource;

/// The content id of the good a destructor burns and a tip's decaying share is drawn as
/// (ADR-0038 §5). A catalogue without it draws nothing from tips.
pub const REFUSE: &str = "core:good/refuse";

/// The kilograms of its named fuel one whole engine of good `g` burns giving its rating over an
/// evening's hours (the hours a station is drawn on, [`super::gas::GAS_EVENING_H`]), and that
/// fuel's index: none for a good that is no engine burning a named fuel.
pub fn engine_evening_kg(goods: &[GoodDef], g: usize) -> Option<(usize, f64)> {
    let p = goods
        .get(g)?
        .power
        .filter(|p| p.source == PowerSource::Fuel)?;
    let f = p.fuel?;
    let mj = goods.get(f)?.fuel?.mj_per_kg;
    let kwh = p.rated_kw * super::gas::GAS_EVENING_H;
    Some((f, civ_land::power::fuel_kg(kwh, mj, p.efficiency)))
}

impl Population {
    /// The refuse good's index, if the catalogue has it.
    fn refuse_good(goods: &[GoodDef]) -> Option<usize> {
        goods.iter().position(|d| d.id == REFUSE)
    }

    /// The open tip refuse may be drawn from by household `household` (ADR-0038 §5): its polity's
    /// first tip whose law is in force, as a round's; the polity's and the tip's indices.
    pub fn refuse_tip_of(&self, household: PermanentId) -> Option<(usize, usize)> {
        let settlement = self.household(household)?.settlement?;
        let pi = self.polity_of(settlement)?;
        Some((pi, self.round_tip(pi)?))
    }

    /// Whether household `hh` could draw fuel good `fuel` from a tip: it is refuse and its
    /// polity's open tip holds some (ADR-0038 §5). A destructor is wanted, as a gas engine is
    /// where its gas is held, only where refuse is at hand.
    pub(super) fn refuse_at_hand(&self, ctx: &Ctx, hh: &Household, fuel: usize) -> bool {
        ctx.catalog.goods.get(fuel).is_some_and(|d| d.id == REFUSE)
            && self
                .refuse_tip_of(hh.id)
                .is_some_and(|(pi, ti)| self.polities[pi].tips[ti].kg > 0.0)
    }

    /// Each midnight, every household holding a whole engine that burns refuse draws from its
    /// polity's open tip what its engines would burn at their rating over an evening, less the
    /// refuse it holds (ADR-0038 §5): the tip's decaying kilograms move into its store as
    /// `refuse`, and that share of each disease load at the tip moves to its refuse, recorded as
    /// carried. Nothing where no tip holds any.
    pub(super) fn refuse_draws(&mut self, ctx: &Ctx) {
        let goods = &ctx.catalog.goods;
        let Some(refuse) = Self::refuse_good(goods) else {
            return;
        };
        if self
            .polities
            .iter()
            .all(|p| p.tips.iter().all(|t| t.kg <= 0.0))
        {
            return;
        }
        let engines: Vec<(usize, f64)> = (0..goods.len())
            .filter_map(|g| {
                let (f, kg) = engine_evening_kg(goods, g)?;
                (f == refuse).then_some((g, kg))
            })
            .collect();
        if engines.is_empty() {
            return;
        }
        let keepers: Vec<(PermanentId, f64)> = self
            .households
            .iter()
            .map(|(_, x)| x)
            .filter_map(|x| {
                let evening: f64 = engines
                    .iter()
                    .map(|&(g, kg)| {
                        (x.stores.get(g).copied().unwrap_or(0.0) + 1e-9)
                            .floor()
                            .max(0.0)
                            * kg
                    })
                    .sum();
                let want = evening - x.stores.get(refuse).copied().unwrap_or(0.0).max(0.0);
                (want > 0.0).then_some((x.id, want))
            })
            .collect();
        let day = ctx.now.day_index();
        for (household, want) in keepers {
            let Some((pi, ti)) = self.refuse_tip_of(household) else {
                continue;
            };
            let tip = &mut self.polities[pi].tips[ti];
            if tip.kg <= 0.0 {
                continue;
            }
            let share = (want / tip.kg).min(1.0);
            let kg = if share >= 1.0 { tip.kg } else { want };
            tip.kg = if share >= 1.0 { 0.0 } else { tip.kg - kg };
            let from = Node::Tip(tip.id);
            let Some(x) = self.household_mut(household) else {
                continue;
            };
            if x.stores.len() <= refuse {
                x.stores.resize(goods.len(), 0.0);
            }
            x.stores[refuse] += kg;
            x.flows.add(Flow::Got, refuse, kg);
            let to = Node::Refuse(household);
            let diseases: Vec<u16> = self
                .contagion
                .loads
                .keys()
                .filter(|(n, _)| *n == from)
                .map(|&(_, d)| d)
                .collect();
            for disease in diseases {
                let amount = self.contagion.take(from, disease, share);
                self.contagion.add(to, disease, amount);
                self.contagion.record(Move {
                    day,
                    disease,
                    how: How::Carried,
                    from,
                    to,
                    amount,
                });
            }
            self.changed(household);
        }
    }

    /// Household `holder` burns `kg` of fuel good `fuel` in an engine (ADR-0038 §5): no more than
    /// it holds, leaving the fuel's ash in its store as `ash`; burning refuse also destroys that
    /// share of the disease its refuse held. Returns the kilograms burned.
    pub(super) fn burn_engine_fuel(
        &mut self,
        goods: &[GoodDef],
        holder: PermanentId,
        fuel: usize,
        kg: f64,
    ) -> f64 {
        let Some(x) = self.household_mut(holder) else {
            return 0.0;
        };
        let held = x.stores.get(fuel).copied().unwrap_or(0.0).max(0.0);
        let kg = kg.min(held);
        if kg.is_nan() || kg <= 0.0 {
            return 0.0;
        }
        x.stores[fuel] = held - kg;
        x.flows.add(Flow::Used, fuel, kg);
        if let Some((a, ash)) = make::leave_ash(goods, &mut x.stores, fuel, kg) {
            x.flows.add(Flow::Made, a, ash);
        }
        if goods.get(fuel).is_some_and(|d| d.id == REFUSE) {
            let node = Node::Refuse(holder);
            let diseases: Vec<u16> = self
                .contagion
                .loads
                .keys()
                .filter(|(n, _)| *n == node)
                .map(|&(_, d)| d)
                .collect();
            for disease in diseases {
                self.contagion.take(node, disease, kg / held);
            }
        }
        kg
    }

    /// Every household holding a destructor draws refuse from its tip now: for tests.
    #[doc(hidden)]
    pub fn refuse_draws_for_tests(&mut self, ctx: &mut Ctx) {
        self.refuse_draws(ctx);
    }
}
