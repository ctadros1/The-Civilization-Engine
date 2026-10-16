//! Loans in goods between households (M7d slice CA, ADR-0027 §1-2): at its review a household
//! whose food will not last to its harvest asks households of its settlement with food to spare,
//! those its elder regards first (a household that owes on a loan lends nothing); each lender weighs the loan by what it believes
//! ([`crate::credit`]), lends the lesser of a share of what it can spare and what is wanted, in
//! the food good it holds most of, and the loan is due a week after the borrower's harvest
//! begins to come in. At the due day, and each day after it until the days of grace run out, the
//! debtor pays what it holds of the good; repaid in full, the lender's elder notes that its word
//! was kept; unpaid when grace runs out, that it was not.

use super::*;
use crate::credit::{Claim, Standing, default_chance, reservation_rate};
use crate::ties::Act;

impl Population {
    /// The day's credit: households whose day it is weigh borrowing, then loans due are paid.
    pub(super) fn credit_day(&mut self, ctx: &Ctx, day: i64) {
        self.weigh_borrowing(ctx, day);
        self.repay_loans(ctx, day);
    }

    /// The food good household `hh` holds most food energy in, and the kilograms of it.
    fn richest_food(&self, ctx: &Ctx, hh: PermanentId) -> Option<(usize, f64)> {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let stores = stores_now(self.household(hh)?, now, params, goods);
        (0..goods.len())
            .filter(|&g| {
                goods[g].purpose == GoodUse::Food
                    && goods[g].kcal_per_kg > 0.0
                    && !goods[g].kept_back()
            })
            .map(|g| (g, stores.get(g).copied().unwrap_or(0.0).max(0.0)))
            .filter(|&(_, kg)| kg > 1e-6)
            .max_by(|a, b| {
                (a.1 * goods[a.0].kcal_per_kg)
                    .total_cmp(&(b.1 * goods[b.0].kcal_per_kg))
                    .then(b.0.cmp(&a.0))
            })
    }

    /// Households whose review day it is, holding no loan still open, weigh borrowing (see the
    /// module's words).
    fn weigh_borrowing(&mut self, ctx: &Ctx, day: i64) {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let cp = &params.credit;
        let Some(crop) = ctx.catalog.crops.get(params.farm.crop) else {
            return;
        };
        let review = i64::from(cp.review_days.max(1));
        let mut ids: Vec<PermanentId> = self
            .households
            .iter()
            .map(|(_, x)| x)
            .filter(|x| {
                x.settlement.is_some()
                    && !x.members.is_empty()
                    && (x.id.get() as i64 + day).rem_euclid(review) == 0
            })
            .map(|x| x.id)
            .collect();
        ids.sort_unstable();
        for id in ids {
            if self.order.credit.owes(id) {
                continue;
            }
            let Some(x) = self.household(id) else {
                continue;
            };
            let settlement = x.settlement;
            let need_day = x.members.len() as f64 * params.household.daily_kcal_per_person;
            let held = stock_kcal(&stores_now(x, now, params, goods), goods);
            // A household with no field brings in no harvest to repay from.
            let mut fields = self.fields_of(ctx.land, id).peekable();
            if fields.peek().is_none() {
                continue;
            }
            let to_harvest = crate::farm::days_to_harvest(crop, fields, day);
            let want = need_day * to_harvest - held;
            if want < cp.least_days * need_day {
                continue;
            }
            let harvest = self.outlook(ctx, id).map_or(0.0, |o| o.harvest);
            let Some(borrower) = self.elder_of(id, now, params) else {
                continue;
            };
            // Those with food to spare, those its elder regards first.
            let mut lenders: Vec<(f64, PermanentId, f64)> = self
                .households
                .iter()
                .map(|(_, h)| h)
                .filter(|h| h.id != id && h.settlement == settlement && !h.members.is_empty())
                // One that owes lends nothing of what it borrowed.
                .filter(|h| !self.order.credit.owes(h.id))
                .filter_map(|h| {
                    let spare = spare_food_kcal(h, now, params, goods);
                    (spare > 1.0).then_some((h.id, spare))
                })
                .map(|(h, spare)| {
                    let regard = self
                        .elder_of(h, now, params)
                        .map_or(0.0, |e| self.ties.regard(borrower, e, day, &params.ties));
                    (regard, h, spare)
                })
                .collect();
            lenders.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
            lenders.truncate(cp.max_asked as usize);
            let due = day + to_harvest.ceil() as i64 + 7;
            let t_years = (due - day) as f64 / civ_core::time::DAYS_PER_YEAR as f64;
            for (_, lender, spare) in lenders {
                self.order.credit.asked += 1;
                let Some(elder) = self.elder_of(lender, now, params) else {
                    continue;
                };
                let Some((good, kg_held)) = self.richest_food(ctx, lender) else {
                    continue;
                };
                let per_kg = goods[good].kcal_per_kg;
                let kcal = want.min(cp.lend_share * spare).min(kg_held * per_kg);
                let kg = kcal / per_kg;
                if kg <= 1e-3 {
                    continue;
                }
                let regard = self.ties.regard(elder, borrower, day, &params.ties);
                // What it believes, at the least increment it would ask.
                let owed_kcal = kcal * (1.0 + cp.customary_increment);
                let p = default_chance(cp.p_prior, regard, harvest / owed_kcal.max(1.0));
                let rate = reservation_rate(cp.keep_return, t_years, p, 0.0);
                let increment = (rate * t_years).max(cp.customary_increment);
                if p > cp.p_max || increment > cp.max_increment {
                    self.order.credit.refused += 1;
                    continue;
                }
                let leg = Leg {
                    from: lender,
                    to: id,
                    good,
                    amount: kg,
                };
                if !self.transfer(now, params, goods, &[leg], Channel::Lent) {
                    continue;
                }
                let credit = &mut self.order.credit;
                credit.next += 1;
                credit.claims.push(Claim {
                    id: credit.next,
                    creditor: lender,
                    debtor: id,
                    good: good as u16,
                    lent: kg as f32,
                    owed: (kg * (1.0 + increment)) as f32,
                    paid: 0.0,
                    made: day,
                    due,
                    standing: Standing::Open,
                    witnesses: [elder, borrower],
                });
                self.note_tie(ctx, borrower, elder, Act::Lent, 1.0, 0.0);
                break;
            }
        }
    }

    /// Loans due: each debtor pays what it holds of the good, and a loan unpaid past its days of
    /// grace is defaulted (see the module's words).
    fn repay_loans(&mut self, ctx: &Ctx, day: i64) {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let grace = i64::from(params.credit.grace_days);
        let due: Vec<usize> = self
            .order
            .credit
            .claims
            .iter()
            .enumerate()
            .filter(|(_, c)| c.standing == Standing::Open && c.due <= day)
            .map(|(i, _)| i)
            .collect();
        for i in due {
            let c = self.order.credit.claims[i].clone();
            let good = usize::from(c.good);
            let held = self
                .household(c.debtor)
                .map(|x| stores_now(x, now, params, goods))
                .and_then(|s| s.get(good).copied())
                .unwrap_or(0.0)
                .max(0.0);
            let pay = c.left().min(held);
            if pay > 1e-6 && self.household(c.creditor).is_some() {
                let leg = Leg {
                    from: c.debtor,
                    to: c.creditor,
                    good,
                    amount: pay,
                };
                if self.transfer(now, params, goods, &[leg], Channel::Repaid) {
                    self.order.credit.claims[i].paid += pay as f32;
                }
            }
            let left = self.order.credit.claims[i].left();
            let elders = (
                self.elder_of(c.creditor, now, params),
                self.elder_of(c.debtor, now, params),
            );
            if left <= 1e-3 {
                self.order.credit.claims[i].standing = Standing::Repaid;
                if let (Some(a), Some(b)) = elders {
                    self.note_tie(ctx, a, b, Act::Repaid, 1.0, 0.0);
                }
            } else if day > c.due + grace {
                self.order.credit.claims[i].standing = Standing::Defaulted;
                if let (Some(a), Some(b)) = elders {
                    self.note_tie(ctx, a, b, Act::Defaulted, 1.0, 0.0);
                }
            }
        }
    }

    /// Every household weighs borrowing now, whatever its review day: for tests.
    #[doc(hidden)]
    pub fn weigh_borrowing_for_tests(&mut self, ctx: &Ctx) {
        let day = ctx.now.day_index();
        let review = i64::from(ctx.params.credit.review_days.max(1));
        for k in 0..review {
            self.weigh_borrowing(ctx, day + k);
        }
    }

    /// Loans due by `day` are paid now: for tests.
    #[doc(hidden)]
    pub fn repay_loans_for_tests(&mut self, ctx: &Ctx, day: i64) {
        self.repay_loans(ctx, day);
    }
}
