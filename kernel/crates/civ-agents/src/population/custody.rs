//! Custody at midnight (M10d slice EG, second part; ADR-0027 §4; ADR-0035 §9; brief m10-money
//! §3): the probe of slice EG found households under their own roof never hold more than it
//! keeps, but those still waiting on one keep everything in the open. At its weekly review such a
//! household leaves what keeps better under a roof, less the days of food it keeps in hand, with
//! a neighbour of its settlement whose roofs have room for it after its own goods and what it
//! keeps for others: close kin first, who ask nothing, else the nearest, who take a share each
//! month. It does when what lying in the open would lose it in a month, at its own costs, is more
//! than the fee and the walks there and back (weighed, not walked, as bringing metal to a mint
//! is). Once it has a roof of its own it takes back as much as that roof has room for, the rest
//! still kept; when its food in hand runs below half those days, the food that brings it back to
//! them; and everything when the keeper has no longer room for it. What a household gone left is
//! the keeper's. Each is applied at midnight in household id order.

use super::*;
use crate::custody::Kept;

/// Days between a keeper's fees.
const FEE_DAYS: i64 = 30;
/// Metres a second one walks on an errand to a neighbour (a design prior, as the walk is weighed,
/// not walked).
const WALK_M_S: f64 = 1.2;

impl Population {
    /// What household `x` with `stores` has room for under its roofs beyond its own goods that
    /// keep better there and what it keeps for others, kilograms: none without a roof.
    pub(super) fn custody_room(&self, ctx: &Ctx, x: &Household, stores: &[f64]) -> f64 {
        if !x.sheltered {
            return 0.0;
        }
        let goods = &ctx.catalog.goods;
        let keeping = x.keeping.with_stores(stores, goods);
        let room = keeping.raised_kg.max(0.0) + keeping.roofed_kg.max(0.0);
        let own: f64 = goods
            .iter()
            .zip(stores)
            .filter(|(d, _)| d.sheltered_half_life_days > 0.0)
            .map(|(_, kg)| kg.max(0.0))
            .sum();
        let others: f64 = self
            .kept
            .iter()
            .filter(|k| k.keeper == x.id)
            .map(Kept::kg)
            .sum();
        if room.is_finite() {
            room - own - others
        } else {
            f64::INFINITY
        }
    }

    /// The midnight's custody (see the module documentation).
    pub(super) fn custody_day(&mut self, ctx: &mut Ctx, day: i64) {
        let (now, params) = (ctx.now, ctx.params);
        let goods = &ctx.catalog.goods;
        let mp = &params.market;
        if !self.kept.is_empty() {
            for k in &mut self.kept {
                k.settle_stores(now, goods);
            }
            // What goes back, to whom, how much of each good, and the fees, decided first.
            let mut back: Vec<(PermanentId, PermanentId, Vec<f64>)> = Vec::new();
            let mut fees: Vec<(PermanentId, PermanentId, Vec<f64>)> = Vec::new();
            for k in &self.kept {
                let depositor = self
                    .household(k.depositor)
                    .filter(|x| !x.members.is_empty());
                let keeper = self.household(k.keeper).filter(|x| !x.members.is_empty());
                match (depositor, keeper) {
                    // A household gone left what it kept with another to its keeper.
                    (None, Some(_)) => back.push((k.id, k.keeper, k.stores.clone())),
                    (None, None) => {}
                    (Some(_), None) => back.push((k.id, k.depositor, k.stores.clone())),
                    (Some(d), Some(keeper)) => {
                        let keeper_stores = stores_now(keeper, now, params, goods);
                        let crowded = self.custody_room(ctx, keeper, &keeper_stores) < -1e-6;
                        if crowded || !keeper.sheltered {
                            back.push((k.id, k.depositor, k.stores.clone()));
                            continue;
                        }
                        // Under a roof of its own, as much as its room holds; short of food, what
                        // brings its food in hand back to its days.
                        let stores = stores_now(d, now, params, goods);
                        let need = d.members.len() as f64 * params.household.daily_kcal_per_person;
                        let food = stock_kcal(&stores, goods);
                        let room_share = if d.sheltered {
                            let room = self.custody_room(ctx, d, &stores).max(0.0);
                            (room / k.kg().max(1e-9)).min(1.0)
                        } else {
                            0.0
                        };
                        let kept_food = stock_kcal(&k.stores, goods);
                        let food_share = if food < 0.5 * mp.custody_keep_days * need {
                            ((mp.custody_keep_days * need - food) / kept_food.max(1e-9)).min(1.0)
                        } else {
                            0.0
                        };
                        if room_share > 0.0 || food_share > 0.0 {
                            let take: Vec<f64> = k
                                .stores
                                .iter()
                                .zip(goods)
                                .map(|(&kg, g)| {
                                    let food = g.purpose == GoodUse::Food && g.kcal_per_kg > 0.0;
                                    let share = if food {
                                        room_share.max(food_share)
                                    } else {
                                        room_share
                                    };
                                    kg * share
                                })
                                .collect();
                            back.push((k.id, k.depositor, take));
                        } else if day - k.fee_day >= FEE_DAYS
                            && self.kin_households(d).binary_search(&keeper.id).is_err()
                        {
                            let fee: Vec<f64> =
                                k.stores.iter().map(|x| x * mp.custody_fee_share).collect();
                            fees.push((k.id, k.keeper, fee));
                        }
                    }
                }
            }
            for (id, keeper, fee) in fees {
                let legs: Vec<Leg> = fee
                    .iter()
                    .enumerate()
                    .filter(|&(_, &kg)| kg > 1e-9)
                    .map(|(good, &amount)| Leg {
                        from: id,
                        to: keeper,
                        good,
                        amount,
                    })
                    .collect();
                if !legs.is_empty() {
                    self.transfer(now, params, goods, &legs, Channel::Custody);
                }
                if let Some(k) = self.kept.iter_mut().find(|k| k.id == id) {
                    k.fee_day = day;
                }
            }
            for (id, to, take) in back {
                let legs: Vec<Leg> = take
                    .iter()
                    .enumerate()
                    .filter(|&(_, &kg)| kg > 1e-9)
                    .map(|(good, &amount)| Leg {
                        from: id,
                        to,
                        good,
                        amount,
                    })
                    .collect();
                if !legs.is_empty() {
                    self.transfer(now, params, goods, &legs, Channel::Custody);
                }
            }
            let (empty, full): (Vec<Kept>, Vec<Kept>) = std::mem::take(&mut self.kept)
                .into_iter()
                .partition(Kept::is_empty);
            for k in &empty {
                self.kept_flows.absorb(&k.flows);
            }
            self.kept = full;
        }
        // Households with no roof of their own whose day it is weigh leaving goods with another.
        let review_days = i64::from(mp.review_days.max(1));
        let leaving: Vec<PermanentId> = self
            .households
            .iter()
            .map(|(_, x)| x)
            .filter(|x| {
                x.settlement.is_some()
                    && !x.members.is_empty()
                    && !x.sheltered
                    && (x.id.get() as i64 + day).rem_euclid(review_days) == 0
                    && !self.kept.iter().any(|k| k.depositor == x.id)
            })
            .map(|x| x.id)
            .collect();
        for h in leaving {
            let Some((keeper, legs)) = self.custody_choice(ctx, h) else {
                continue;
            };
            let id = ctx.ids.allocate();
            self.kept
                .push(Kept::new(id, (h, keeper), now, ctx.catalog.goods.len()));
            let legs: Vec<Leg> = legs
                .into_iter()
                .map(|(good, amount)| Leg {
                    from: h,
                    to: id,
                    good,
                    amount,
                })
                .collect();
            if !self.transfer(now, params, &ctx.catalog.goods, &legs, Channel::Custody) {
                self.kept.pop();
            }
        }
    }

    /// The keeper household `h` would leave goods with, and what it would leave (good,
    /// kilograms), if leaving them is worth more to it than it costs (see the module
    /// documentation).
    fn custody_choice(
        &self,
        ctx: &Ctx,
        h: PermanentId,
    ) -> Option<(PermanentId, Vec<(usize, f64)>)> {
        let (now, params) = (ctx.now, ctx.params);
        let goods = &ctx.catalog.goods;
        let mp = &params.market;
        let x = self.household(h)?;
        let settlement = x.settlement?;
        let stores = stores_now(x, now, params, goods);
        let need = x.members.len() as f64 * params.household.daily_kcal_per_person;
        let food = stock_kcal(&stores, goods);
        let keep = mp.custody_keep_days * need;
        // Of its food, what is beyond the days it keeps in hand, from each in proportion.
        let food_share = if food > keep && food > 0.0 {
            (food - keep) / food
        } else {
            0.0
        };
        let leave: Vec<(usize, f64)> = goods
            .iter()
            .enumerate()
            .filter(|(_, d)| d.sheltered_half_life_days > 0.0)
            .filter_map(|(g, d)| {
                let kg = stores.get(g).copied().unwrap_or(0.0).max(0.0);
                let kg = if d.purpose == GoodUse::Food && d.kcal_per_kg > 0.0 {
                    kg * food_share
                } else {
                    kg
                };
                (kg > 1e-6).then_some((g, kg))
            })
            .collect();
        let total: f64 = leave.iter().map(|l| l.1).sum();
        if total < 1.0 {
            return None;
        }
        // The keeper: close kin with room first, else the nearest with room.
        let kin_of = self.kin_households(x);
        let (keeper, kin, metres) = self
            .households_in(settlement)
            .filter(|k| k.id != h && !k.members.is_empty() && k.sheltered)
            .filter(|k| {
                let ks = stores_now(k, now, params, goods);
                self.custody_room(ctx, k, &ks) >= total
            })
            .map(|k| {
                let d = f64::from(k.home.0 - x.home.0).hypot(f64::from(k.home.1 - x.home.1));
                (k.id, kin_of.binary_search(&k.id).is_ok(), d)
            })
            .min_by(|a, b| b.1.cmp(&a.1).then(a.2.total_cmp(&b.2)).then(a.0.cmp(&b.0)))?;
        // What lying in the open would lose it in a month, against the fee and the walks.
        let costs = self.own_costs_of(ctx, x);
        let lost = |half: f64| {
            if half > 0.0 {
                1.0 - 0.5f64.powf(FEE_DAYS as f64 / half)
            } else {
                0.0
            }
        };
        let mut gain = 0.0;
        let mut worth = 0.0;
        for &(g, kg) in &leave {
            let d = &goods[g];
            let c = costs.get(g).copied().flatten().unwrap_or(0.0);
            gain += kg * c * (lost(d.half_life_days) - lost(d.sheltered_half_life_days)).max(0.0);
            worth += kg * c;
        }
        let fee = if kin {
            0.0
        } else {
            mp.custody_fee_share * worth
        };
        // There and back to leave them, and again to take them back.
        let walk_h = 4.0 * metres / WALK_M_S / 3600.0;
        (gain > fee + walk_h).then_some((keeper, leave))
    }

    /// What household `household` has left with others, and what it keeps for others,
    /// kilograms: for tests.
    #[doc(hidden)]
    pub fn custody_for_tests(&self, household: PermanentId) -> (f64, f64) {
        let left = self
            .kept
            .iter()
            .filter(|k| k.depositor == household)
            .map(Kept::kg)
            .sum();
        let keeps = self
            .kept
            .iter()
            .filter(|k| k.keeper == household)
            .map(Kept::kg)
            .sum();
        (left, keeps)
    }

    /// The households of household `household`'s people's close kin: for tests.
    #[doc(hidden)]
    pub fn kin_households_for_tests(&self, household: PermanentId) -> Vec<PermanentId> {
        self.household(household)
            .map_or_else(Vec::new, |x| self.kin_households(x))
    }

    /// The midnight's custody now, as at midnight: for tests.
    #[doc(hidden)]
    pub fn custody_day_for_tests(&mut self, ctx: &mut Ctx) {
        let day = ctx.now.day_index();
        self.custody_day(ctx, day);
    }
}
