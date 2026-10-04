//! Markets (slice I, ADR-0006 §4): each household reviews, on its own day, what it can spare and
//! the terms it posts; a buyer looks for the offer in its settlement that costs it the fewest
//! hours of its own work, the walk included; and the trade settles through the ledger when the
//! buyer arrives.

use civ_core::PermanentId;
use civ_world::nav::TravelField;

use super::{Ctx, Population, cell_of, fuel_per_day, stores_now};
use crate::decide::{TradeOption, TradeWorth};
use crate::farm;
use crate::ledger::{Channel, Leg, Trade};
use crate::make;
use crate::market::{Market, Offer};
use crate::params::{GoodUse, MarketParams};
use crate::person::{Household, stock_kcal};
use crate::value;

/// Food a household keeps beyond what sees it to its next harvest before it offers any, a share
/// of that (a tuning value).
const FOOD_KEEP_MARGIN: f64 = 0.25;
/// Least of a tool worth offering, standard tools.
const MIN_OFFER_TOOL: f64 = 0.5;
/// Least of any other good worth offering, kilograms.
const MIN_OFFER_KG: f64 = 5.0;
/// Most goods a seller accepts in payment.
const MAX_PAYMENTS: usize = 4;
/// Least a food stays good for to be offered, days: food that keeps (grain), not food made to
/// be eaten soon (bread).
const OFFER_FOOD_HALF_LIFE_DAYS: f64 = 365.0;

/// What a household keeps of each good, and how much more of each it wants, 0–1: 1 when it is
/// short of it, less as it holds more than it keeps, 0 for what it has no use for.
#[derive(Clone, Debug, Default)]
pub(crate) struct Holding {
    pub keep: Vec<f64>,
    pub want: Vec<f64>,
}

/// A purchase as it stands: from whom, what, how much, and what it is paid with.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Deal {
    pub seller: PermanentId,
    pub at: (f32, f32),
    pub walk_min: f64,
    pub good: usize,
    pub units: f64,
    pub payment: usize,
    pub paid: f64,
    /// What the seller's terms say a unit is worth to it, hours.
    pub ask_h: f64,
    /// Hours of its own work the buyer saves, the walk included.
    pub saving_h: f64,
    pub worth: TradeWorth,
}

/// The ask after a review (research 08-04 §1.2: a cost anchor with a restrained scarcity
/// response): half the way toward the anchor, nudged up by demand nobody met and down when
/// nothing sold, never more than `max_change` either way.
fn reviewed_ask(prev: f64, anchor: f64, unmet: bool, unsold: bool, max_change: f64) -> f64 {
    let mut step = 0.5 * (anchor / prev.max(1e-9)).ln();
    if unmet {
        step += max_change / 2.0;
    }
    if unsold {
        step -= max_change / 2.0;
    }
    prev * step.clamp(-max_change, max_change).exp()
}

impl Population {
    /// The market of `settlement`, once anything has been offered or wanted there.
    pub fn market(&self, settlement: PermanentId) -> Option<&Market> {
        self.markets.iter().find(|m| m.settlement == settlement)
    }

    /// The money of `settlement`: the good most of its payments settle in, if one does (ADR-0006
    /// §4).
    pub fn money_of(
        &self,
        settlement: Option<PermanentId>,
        params: &MarketParams,
    ) -> Option<usize> {
        self.market(settlement?)?
            .money(params.money_share, params.money_min_trades)
    }

    fn market_mut(
        &mut self,
        settlement: PermanentId,
        goods: usize,
        day: i64,
        memory_days: f64,
    ) -> &mut Market {
        let i = match self.markets.iter().position(|m| m.settlement == settlement) {
            Some(i) => i,
            None => {
                self.markets.push(Market::new(settlement, goods, day));
                self.markets.len() - 1
            }
        };
        let m = &mut self.markets[i];
        m.age_to(day, memory_days);
        m
    }

    /// What a household's goods cost it, hours of its own work per unit, at its best member's
    /// skills and from what it knows of the land.
    pub(crate) fn own_costs_of(&self, ctx: &Ctx, hh: &Household) -> Vec<Option<f64>> {
        let catalog = ctx.catalog;
        let mut levels: Vec<f64> = vec![0.0; catalog.skills.len()];
        for m in &hh.members {
            if let Some(p) = self.person(*m) {
                for (k, l) in levels.iter_mut().enumerate() {
                    *l = l.max(p.skill(k));
                }
            }
        }
        let n = catalog.goods.len();
        let gathered = value::gathered_costs(&ctx.land_params.resources, &hh.known, n);
        let grown = value::grown_costs(&catalog.crops, n, ctx.params.farm.plan_yield_share);
        let mut costs = value::own_costs(catalog, &levels, &gathered, &grown);
        value::food_by_energy(&catalog.goods, &mut costs);
        costs
    }

    /// What a household keeps of each good and how much more of each it wants, from its
    /// `stores`: food to see it to its next harvest and its margin (pooled by energy), the tools
    /// its members' work needs and a spare, what the tools it lacks are made of, and its
    /// firewood.
    pub(crate) fn holding_of(&self, ctx: &Ctx, hh: &Household, stores: &[f64]) -> Holding {
        let (params, catalog) = (ctx.params, ctx.catalog);
        let goods = &catalog.goods;
        let n = goods.len();
        let held = |g: usize| stores.get(g).copied().unwrap_or(0.0).max(0.0);
        let mut keep = vec![0.0; n];
        let mut want = vec![0.0; n];
        let members = hh.members.len().max(1);
        let kcal_day = members as f64 * params.household.daily_kcal_per_person;
        let outlook = catalog.crops.get(params.farm.crop).map_or(0.0, |c| {
            farm::days_to_harvest(
                c,
                ctx.land.fields.iter().filter(|f| f.household == hh.id),
                ctx.now.day_index(),
            ) + params.household.harvest_margin_days
        });
        let food_keep = outlook * kcal_day * (1.0 + FOOD_KEEP_MARGIN);
        let food = stock_kcal(stores, goods);
        let want_food = if food < food_keep || food <= 0.0 {
            1.0
        } else {
            food_keep / food
        };
        let mut spare_kcal = (food - food_keep).max(0.0);
        for (g, d) in goods.iter().enumerate() {
            if d.purpose == GoodUse::Food && d.kcal_per_kg > 0.0 && !d.kept_back() {
                want[g] = want_food;
                // The food it can spare comes out of what keeps longest.
                let keeps =
                    d.half_life_days <= 0.0 || d.half_life_days >= OFFER_FOOD_HALF_LIFE_DAYS;
                let spare = if keeps {
                    (spare_kcal / d.kcal_per_kg).min(held(g))
                } else {
                    0.0
                };
                spare_kcal -= spare * d.kcal_per_kg;
                keep[g] = held(g) - spare;
            }
        }
        let ages: Vec<f64> = hh
            .members
            .iter()
            .filter_map(|m| self.person(*m))
            .map(|p| p.age_years(ctx.now))
            .collect();
        let wants = make::tool_wants_for(catalog, &ages, params.family.independent_age);
        for (t, &w) in wants.iter().enumerate() {
            if w <= 0.0 {
                continue;
            }
            keep[t] = w + make::SPARE_TOOL;
            if make::tool_need(w, held(t)) > 0.0
                && let Some(r) = catalog
                    .recipes
                    .iter()
                    .find(|r| r.outputs.iter().any(|&(g, _)| g == t))
            {
                for &(g, amount) in &r.inputs {
                    if let Some(k) = keep.get_mut(g) {
                        *k += amount;
                    }
                }
            }
        }
        // Until its home is finished, what it builds with stays for that.
        let housed = ctx
            .land
            .buildings
            .iter()
            .any(|b| b.household == hh.id && b.finished());
        let fuel_day = fuel_per_day(params, members, ctx.now.day_index());
        for (g, d) in goods.iter().enumerate() {
            if d.purpose == GoodUse::Fuel {
                keep[g] = params.household.fuel_target_days * fuel_day;
            }
            if d.purpose == GoodUse::Material && !housed {
                keep[g] = keep[g].max(held(g));
            }
            if d.purpose != GoodUse::Food || d.kept_back() {
                want[g] = if keep[g] <= 0.0 {
                    0.0
                } else if held(g) < keep[g] {
                    1.0
                } else {
                    keep[g] / held(g)
                };
            }
        }
        Holding { keep, want }
    }

    /// Each household whose day it is reviews what it offers and on what terms (ADR-0006 §4):
    /// it offers what it holds beyond what it keeps, at its own cost and margin moved toward
    /// what the market shows, in each good it wants or its settlement's money; and the tools it
    /// needs and finds nobody offering are recorded as demand.
    pub(super) fn review_offers(&mut self, ctx: &Ctx, day: i64) {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let mp = &params.market;
        let review_days = i64::from(mp.review_days.max(1));
        let mut ids: Vec<PermanentId> = self
            .households
            .iter()
            .map(|(_, x)| x)
            .filter(|x| {
                x.settlement.is_some()
                    && !x.members.is_empty()
                    && (x.id.get() as i64 + day).rem_euclid(review_days) == 0
            })
            .map(|x| x.id)
            .collect();
        ids.sort_unstable();
        for id in ids {
            let Some(hh) = self.household(id) else {
                continue;
            };
            let Some(settlement) = hh.settlement else {
                continue;
            };
            let stores = stores_now(hh, now, params, goods);
            let costs = self.own_costs_of(ctx, hh);
            let holding = self.holding_of(ctx, hh, &stores);
            let money = self.money_of(Some(settlement), mp);
            let held = |g: usize| stores.get(g).copied().unwrap_or(0.0).max(0.0);
            // What it can spare.
            let mut offered: Vec<(usize, f64)> = Vec::new();
            for (g, d) in goods.iter().enumerate() {
                let spare = held(g) - holding.keep.get(g).copied().unwrap_or(0.0);
                let least = if d.tool.is_some() {
                    MIN_OFFER_TOOL
                } else {
                    MIN_OFFER_KG
                };
                let offerable = match d.purpose {
                    GoodUse::Tool => !d.tool.as_ref().is_some_and(|t| t.fixed),
                    GoodUse::Food => !d.kept_back(),
                    GoodUse::Material => true,
                    GoodUse::Fuel => false,
                };
                if offerable && spare >= least {
                    offered.push((g, spare));
                }
            }
            // What it takes in payment: what it wants, or its settlement's money, each valued at
            // its own cost.
            let mut accepts: Vec<(usize, f64)> = goods
                .iter()
                .enumerate()
                .filter(|(_, d)| !d.kept_back() && !d.tool.as_ref().is_some_and(|t| t.fixed))
                .filter_map(|(p, _)| {
                    let want = if money == Some(p) {
                        1.0
                    } else {
                        holding.want.get(p).copied().unwrap_or(0.0)
                    };
                    let cost = costs.get(p).copied().flatten()?;
                    (want >= mp.accept_want && cost > 0.0).then_some((p, cost * want))
                })
                .collect();
            // What it is shortest of first; then what its neighbours already pay in, what keeps
            // longest, and what is worth most for its weight (research 08-06 §1.3: acceptance
            // follows familiarity, durability and portability).
            let want_of = |p: usize| {
                if money == Some(p) {
                    1.0
                } else {
                    holding.want.get(p).copied().unwrap_or(0.0)
                }
            };
            let accepted: Vec<f64> = self
                .market(settlement)
                .map(|m| m.acceptance())
                .unwrap_or_default();
            let keeps = |p: usize| match goods[p].half_life_days {
                h if h > 0.0 => h,
                _ => f64::INFINITY,
            };
            accepts.sort_by(|a, b| {
                want_of(b.0)
                    .total_cmp(&want_of(a.0))
                    .then(
                        accepted
                            .get(b.0)
                            .copied()
                            .unwrap_or(0.0)
                            .total_cmp(&accepted.get(a.0).copied().unwrap_or(0.0)),
                    )
                    .then(keeps(b.0).total_cmp(&keeps(a.0)))
                    .then(b.1.total_cmp(&a.1))
                    .then(a.0.cmp(&b.0))
            });
            let (unmet, sold): (Vec<f64>, Vec<f64>) = match self.market(settlement) {
                Some(m) => (m.unmet.clone(), m.sold.clone()),
                None => (Vec::new(), Vec::new()),
            };
            let old = hh.offers.clone();
            let mut rows = Vec::new();
            for &(g, units) in &offered {
                let anchor = costs
                    .get(g)
                    .copied()
                    .flatten()
                    .map(|c| c * (1.0 + mp.margin));
                let prev = old
                    .iter()
                    .find(|o| usize::from(o.good) == g)
                    .map(|o| f64::from(o.ask_h));
                let ask = match (prev, anchor) {
                    (Some(prev), Some(anchor)) => reviewed_ask(
                        prev,
                        anchor,
                        unmet.get(g).copied().unwrap_or(0.0) > 0.5,
                        sold.get(g).copied().unwrap_or(0.0) < 0.1,
                        mp.max_change,
                    ),
                    (None, Some(anchor)) => anchor,
                    (Some(prev), None) => prev,
                    (None, None) => continue,
                };
                // Terms in the goods it takes, as far as one person could carry what a unit costs
                // (research 08-06 §1.3: a payment must be portable).
                let mut posted = 0;
                for &(p, value_h) in &accepts {
                    if p == g || posted == MAX_PAYMENTS {
                        continue;
                    }
                    let price = ask / value_h;
                    let unit = if goods[g].tool.is_some() {
                        1.0
                    } else {
                        MIN_OFFER_KG
                    };
                    if goods[p].tool.is_none() && price * unit > params.household.carry_kg {
                        continue;
                    }
                    rows.push(Offer {
                        good: g as u16,
                        payment: p as u16,
                        price: price as f32,
                        units: units as f32,
                        ask_h: ask as f32,
                    });
                    posted += 1;
                }
            }
            // The tools it needs and nobody in its settlement offers: demand on record.
            let offered_here: Vec<usize> = self
                .households
                .iter()
                .map(|(_, x)| x)
                .filter(|x| x.settlement == Some(settlement) && x.id != id)
                .flat_map(|x| x.offers.iter().map(|o| usize::from(o.good)))
                .collect();
            let mut wanted: Vec<(usize, f64)> = Vec::new();
            for (t, d) in goods.iter().enumerate() {
                if d.tool.is_none() || offered_here.contains(&t) {
                    continue;
                }
                let wants =
                    (holding.keep.get(t).copied().unwrap_or(0.0) - make::SPARE_TOOL).max(0.0);
                if make::tool_need(wants, held(t)) > 0.0
                    && let Some(c) = costs.get(t).copied().flatten()
                {
                    wanted.push((t, c));
                }
            }
            if let Some(x) = self.household_mut_by_id(id) {
                x.offers = rows;
            }
            let m = self.market_mut(settlement, goods.len(), day, mp.memory_days);
            for (t, worth_h) in wanted {
                m.record_unmet(t, 1.0, worth_h);
            }
        }
    }

    /// The best purchase household `hh` can make with its `stores` as they stand: the offer in
    /// its settlement that saves it the most hours of its own work over getting the good itself,
    /// the walk there and back (`walk_min(seller)`, minutes one way) included, for a tool it
    /// needs or food it is short of. `only` limits the search to one seller (a buyer at the
    /// seller's door).
    pub(crate) fn find_deal(
        &self,
        ctx: &Ctx,
        hh: &Household,
        stores: &[f64],
        walk_min: &dyn Fn(&Household) -> Option<f64>,
        only: Option<PermanentId>,
    ) -> Option<Deal> {
        let settlement = hh.settlement?;
        let (params, goods) = (ctx.params, &ctx.catalog.goods);
        let held = |g: usize| stores.get(g).copied().unwrap_or(0.0).max(0.0);
        let mut sellers = self
            .households
            .iter()
            .map(|(_, x)| x)
            .filter(|x| {
                x.id != hh.id
                    && x.settlement == Some(settlement)
                    && !x.offers.is_empty()
                    && only.is_none_or(|s| s == x.id)
            })
            .peekable();
        sellers.peek()?;
        let costs = self.own_costs_of(ctx, hh);
        let holding = self.holding_of(ctx, hh, stores);
        // Food it is short of: what it keeps less what it has, by energy.
        let food = stock_kcal(stores, goods);
        let food_keep: f64 = goods
            .iter()
            .enumerate()
            .filter(|(_, d)| d.purpose == GoodUse::Food && !d.kept_back())
            .map(|(g, d)| holding.keep[g].max(0.0) * d.kcal_per_kg)
            .sum::<f64>()
            .max(food);
        let short_kcal = (food_keep - food).max(0.0);
        let lean = if food_keep > 0.0 {
            short_kcal / food_keep
        } else {
            0.0
        };
        let mut best: Option<Deal> = None;
        for x in sellers {
            let Some(walk) = walk_min(x) else {
                continue;
            };
            for o in &x.offers {
                let (g, p) = (usize::from(o.good), usize::from(o.payment));
                let Some(d) = goods.get(g) else {
                    continue;
                };
                // What it would buy: a tool it needs (up to one standard tool, worn or not), or
                // food it is short of (what one person carries, no more than it is short).
                let (units, worth, own_h) = if d.tool.is_some() {
                    let wanted = (holding.keep[g] - make::SPARE_TOOL).max(0.0);
                    let need = make::tool_need(wanted, held(g));
                    let units = f64::from(o.units).min(1.0);
                    if need <= 0.0 || units < MIN_OFFER_TOOL - 1e-6 {
                        continue;
                    }
                    let own = costs[g].unwrap_or(f64::INFINITY) * units;
                    (units, TradeWorth::Tool { need }, own)
                } else if d.purpose == GoodUse::Food && d.kcal_per_kg > 0.0 {
                    if short_kcal <= 0.0 {
                        continue;
                    }
                    let units = (short_kcal / d.kcal_per_kg)
                        .min(params.household.carry_kg)
                        .min(f64::from(o.units));
                    if units < 1.0 {
                        continue;
                    }
                    // Short of food before the harvest, growing more is no help now: food is
                    // worth more the shorter the household is.
                    let own = costs[g].unwrap_or(f64::INFINITY) * (1.0 + lean) * units;
                    let kcal = units * d.kcal_per_kg;
                    (units, TradeWorth::Food { kcal }, own)
                } else {
                    continue;
                };
                // Food bought is limited to a payment one person can carry back.
                let units = if goods[p].tool.is_none() && d.tool.is_none() && o.price > 0.0 {
                    units.min(params.household.carry_kg / f64::from(o.price))
                } else {
                    units
                };
                let paid = f64::from(o.price) * units;
                // It pays out of what it holds beyond what it keeps, valued at its own cost.
                let spare = held(p) - holding.keep.get(p).copied().unwrap_or(0.0);
                if !(paid > 0.0 && paid <= spare + 1e-9) {
                    continue;
                }
                let Some(pay_cost) = costs.get(p).copied().flatten() else {
                    continue;
                };
                let pay_h = paid * pay_cost * holding.want.get(p).copied().unwrap_or(0.0);
                let saving = own_h - pay_h - 2.0 * walk / 60.0;
                if saving.is_nan() || saving <= 0.0 {
                    continue;
                }
                // What it cannot get itself saves all a buyer would give: rank those by price.
                let rank = if saving.is_finite() {
                    saving
                } else {
                    1e12 - pay_h
                };
                let better = best.is_none_or(|b| {
                    let b_rank = if b.saving_h.is_finite() {
                        b.saving_h
                    } else {
                        1e12 - b.paid
                    };
                    rank > b_rank + 1e-9
                        || ((rank - b_rank).abs() <= 1e-9
                            && (x.id, g, p) < (b.seller, b.good, b.payment))
                });
                if better {
                    best = Some(Deal {
                        seller: x.id,
                        at: x.home,
                        walk_min: walk,
                        good: g,
                        units,
                        payment: p,
                        paid,
                        ask_h: f64::from(o.ask_h),
                        saving_h: saving,
                        worth,
                    });
                }
            }
        }
        best
    }

    /// The purchase a person of household `hh` could go and make, for the decision: see
    /// [`Population::find_deal`]. `reach` is the walking field from the household's home.
    pub(crate) fn best_purchase(
        &self,
        ctx: &Ctx,
        hh: &Household,
        stores: &[f64],
        reach: Option<&TravelField>,
    ) -> Option<TradeOption> {
        let reach = reach?;
        let walk = |x: &Household| {
            reach
                .seconds_to(cell_of(ctx.map, x.home))
                .map(|s| f64::from(s) / 60.0)
        };
        let d = self.find_deal(ctx, hh, stores, &walk, None)?;
        Some(TradeOption {
            seller: d.seller,
            walk_min: d.walk_min,
            at: d.at,
            good: d.good,
            worth: d.worth,
        })
    }

    /// A buyer of household `buyer` at the door of `seller`: the best deal it can make there now
    /// settles through the ledger, as barter or, paid in the settlement's money, a sale; the
    /// seller's offer shrinks by what it sold, and the market remembers the trade.
    pub(super) fn settle_trade(&mut self, ctx: &Ctx, buyer: PermanentId, seller: PermanentId) {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let Some(hh) = self.household(buyer) else {
            return;
        };
        let Some(settlement) = hh.settlement else {
            return;
        };
        let stores = stores_now(hh, now, params, goods);
        let Some(d) = self.find_deal(ctx, hh, &stores, &|_| Some(0.0), Some(seller)) else {
            return;
        };
        let legs = [
            Leg {
                from: seller,
                to: buyer,
                good: d.good,
                amount: d.units,
            },
            Leg {
                from: buyer,
                to: seller,
                good: d.payment,
                amount: d.paid,
            },
        ];
        let channel = if self.money_of(Some(settlement), &params.market) == Some(d.payment) {
            Channel::Sale
        } else {
            Channel::Barter
        };
        if !self.transfer(now, params, goods, &legs, channel) {
            return;
        }
        if let Some(x) = self.household_mut_by_id(seller) {
            for o in x
                .offers
                .iter_mut()
                .filter(|o| usize::from(o.good) == d.good)
            {
                o.units = (f64::from(o.units) - d.units).max(0.0) as f32;
            }
            x.offers.retain(|o| o.units > 1e-6);
        }
        let trade = Trade {
            at: now,
            seller,
            buyer,
            good: d.good as u16,
            units: d.units as f32,
            payment: d.payment as u16,
            paid: d.paid as f32,
            channel,
        };
        let mp = &params.market;
        let m = self.market_mut(settlement, goods.len(), now.day_index(), mp.memory_days);
        m.record_trade(trade, d.ask_h * d.units, mp.recent_trades);
    }

    /// For each tool household `hh` does not need itself: how much it is worth making to sell, 0–1
    /// (research 08-04 §1.2: produce for funded demand, not for sales already made). Others in
    /// its settlement want it and found nobody offering it, and they would give more hours of
    /// their own work for one than the household's own cost and margin; it is worth the share of
    /// what they would give that the household keeps, times how much of it is wanted (up to one
    /// tool). Nothing while the household still holds one to sell.
    pub(crate) fn for_sale(
        &self,
        ctx: &Ctx,
        hh: &Household,
        stores: &[f64],
        tool_need: &[f64],
    ) -> Vec<f64> {
        let goods = &ctx.catalog.goods;
        let mut out = vec![0.0; goods.len()];
        let Some(market) = hh.settlement.and_then(|s| self.market(s)) else {
            return out;
        };
        if !goods
            .iter()
            .enumerate()
            .any(|(t, d)| d.tool.is_some() && market.unmet_worth(t).is_some())
        {
            return out;
        }
        let costs = self.own_costs_of(ctx, hh);
        let holding = self.holding_of(ctx, hh, stores);
        let margin = ctx.params.market.margin;
        for (t, d) in goods.iter().enumerate() {
            if d.tool.is_none() || tool_need.get(t).copied().unwrap_or(0.0) > 0.0 {
                continue;
            }
            let (Some(worth), Some(cost)) = (market.unmet_worth(t), costs[t]) else {
                continue;
            };
            let spare = stores.get(t).copied().unwrap_or(0.0) - holding.keep[t];
            if spare >= 1.0 {
                continue;
            }
            let share = 1.0 - cost * (1.0 + margin) / worth.max(1e-9);
            if share > 0.0 {
                out[t] = share * market.unmet[t].min(1.0);
            }
        }
        out
    }

    fn household_mut_by_id(&mut self, id: PermanentId) -> Option<&mut Household> {
        let hd = *self.hh_index.get(&id)?;
        self.households.get_mut(hd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_ask_moves_toward_its_anchor_by_at_most_the_largest_change() {
        // Far below the anchor: up by the cap.
        let up = reviewed_ask(1.0, 2.0, false, false, 0.05);
        assert!((up - 0.05f64.exp()).abs() < 1e-12);
        // Close to it: half the way, in logs.
        let near = reviewed_ask(1.0, 1.04, false, false, 0.05);
        assert!((near - 1.04f64.sqrt()).abs() < 1e-12);
        // At the anchor, demand nobody met raises it and an unsold offer lowers it.
        assert!(reviewed_ask(1.0, 1.0, true, false, 0.05) > 1.0);
        assert!(reviewed_ask(1.0, 1.0, false, true, 0.05) < 1.0);
        assert_eq!(reviewed_ask(1.0, 1.0, true, true, 0.05), 1.0);
    }
}
