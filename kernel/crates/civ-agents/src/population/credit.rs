//! Loans in goods between households (M7d slices CA–CB, ADR-0027 §1-3): at its review a
//! household whose food will not last to its harvest asks households of its settlement with food
//! to spare, those its elder regards first (a household that owes on a loan, or stands guarantor
//! for one, lends nothing). Each lender weighs the loan by what it believes ([`crate::credit`]):
//! bare first, then against the use of a field the borrower holds and works, then with a
//! guarantor, a household whose elder regards the borrower's enough to promise. It lends the
//! lesser of a share of what it can spare and what is wanted, in the food good it holds most of,
//! due a week after the borrower's harvest begins to come in.
//!
//! At the due day the debtor answers once, as a finding is met: it means to pay, or refuses. One
//! that means to pay gives what it can spare of the good each day; short at the due day it asks
//! for time, which the creditor grants by its regard. Paid in full, the lender's elder notes that
//! its word was kept; refused, or unpaid when the days of grace run out, the creditor's household
//! notes that it was not, and the creditor takes the pledge: the guarantor pays what it can spare
//! of the good and the debtor owes it that in turn; a field pledged is worked by the creditor on
//! the regime's lease, from the day it lies between crops, until the crop the creditor keeps of it
//! has paid what is left, and then goes back.

use super::*;
use crate::credit::{Claim, Pledge, Standing, default_chance, pay_points, terms};
use crate::crime::logistic;
use crate::ties::Act;

/// Purpose tag for a debtor's answer at a loan's due day.
pub const PURPOSE_LOAN_ANSWER: u64 = 0x6c6f_616e_616e_7377; // "loanansw"

/// Years a lease on a pledged field runs: until the debt is paid, when it ends.
const PLEDGE_YEARS: i64 = 100;

impl Population {
    /// The day's credit: households whose day it is weigh borrowing, loans due are answered and
    /// paid, and pledged fields change hands.
    pub(super) fn credit_day(&mut self, ctx: &mut Ctx, day: i64) {
        self.weigh_borrowing(ctx, day);
        self.repay_loans(ctx, day);
        self.pledged_fields(ctx, day);
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

    /// The field household `hh` could pledge: one it holds and works, unlet and not pledged
    /// already, the smallest (then the lowest id), where the regime lets fields.
    fn pledgeable_field(&self, ctx: &Ctx, hh: PermanentId) -> Option<PermanentId> {
        ctx.regime.lease?;
        self.fields_of(ctx.land, hh)
            .filter(|f| {
                f.holder == Party::Household(hh)
                    && f.lease.is_none()
                    && !self.order.credit.pledged(f.id)
            })
            .min_by(|a, b| {
                a.rect
                    .area_ha()
                    .total_cmp(&b.rect.area_ha())
                    .then(a.id.cmp(&b.id))
            })
            .map(|f| f.id)
    }

    /// The household that would stand guarantor for household `hh`, whose elder is `borrower`,
    /// besides `lender`: of its settlement, owing nothing and guaranteeing nothing, holding food
    /// to spare, whose elder regards `borrower` at least the content's least; the one regarding
    /// it most. With its elder and the food it can spare.
    fn guarantor_for(
        &self,
        ctx: &Ctx,
        hh: PermanentId,
        borrower: PermanentId,
        lender: PermanentId,
        day: i64,
    ) -> Option<(PermanentId, PermanentId, f64)> {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let settlement = self.household(hh)?.settlement;
        let credit = &self.order.credit;
        self.households
            .iter()
            .map(|(_, h)| h)
            .filter(|h| {
                h.id != hh
                    && h.id != lender
                    && h.settlement == settlement
                    && !h.members.is_empty()
                    && !credit.owes(h.id)
                    && !credit.guarantees(h.id)
            })
            .filter_map(|h| {
                let spare = spare_food_kcal(h, now, params, goods);
                let elder = self.elder_of(h.id, now, params)?;
                let regard = self.ties.regard(elder, borrower, day, &params.ties);
                (spare > 1.0 && regard >= params.credit.guarantor_regard)
                    .then_some((regard, h.id, elder, spare))
            })
            .max_by(|a, b| a.0.total_cmp(&b.0).then(b.1.cmp(&a.1)))
            .map(|(_, h, elder, spare)| (h, elder, spare))
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
                // One that owes lends nothing of what it borrowed; one that guarantees keeps
                // what it promised.
                .filter(|h| !self.order.credit.owes(h.id) && !self.order.credit.guarantees(h.id))
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
            let field = self.pledgeable_field(ctx, id);
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
                // Bare, then against a field's use, then with a guarantor (ADR-0027 §3).
                let mut agreed = terms(p, 0.0, t_years, cp).map(|i| (None, i));
                if agreed.is_none()
                    && let Some(f) = field
                {
                    agreed = terms(p, cp.field_recovery, t_years, cp)
                        .map(|i| (Some(Pledge::Field(f)), i));
                }
                if agreed.is_none()
                    && let Some((g, g_elder, g_spare)) =
                        self.guarantor_for(ctx, id, borrower, lender, day)
                {
                    // Lost only if both fail: the guarantor judged as a borrower would be.
                    let g_regard = self.ties.regard(elder, g_elder, day, &params.ties);
                    let p_g = default_chance(cp.p_prior, g_regard, g_spare / owed_kcal.max(1.0));
                    agreed =
                        terms(p * p_g, 0.0, t_years, cp).map(|i| (Some(Pledge::Guarantor(g)), i));
                }
                let Some((pledge, increment)) = agreed else {
                    self.order.credit.refused += 1;
                    continue;
                };
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
                    pledge,
                    answer: None,
                    asks: 0,
                    extensions: 0,
                    taken: None,
                    by_guarantor: 0.0,
                    recourse: None,
                });
                self.note_tie(ctx, borrower, elder, Act::Lent, 1.0, 0.0);
                break;
            }
        }
    }

    /// Kilograms of good `good` household `hh` can give today, keeping `keep_days` of its food.
    fn can_give(&self, ctx: &Ctx, hh: PermanentId, good: usize) -> f64 {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let Some(x) = self.household(hh) else {
            return 0.0;
        };
        let stores = stores_now(x, now, params, goods);
        let held = stores.get(good).copied().unwrap_or(0.0).max(0.0);
        let need = x.members.len().max(1) as f64 * params.household.daily_kcal_per_person;
        let spare = (stock_kcal(&stores, goods) - params.credit.keep_days * need).max(0.0);
        let per_kg = goods[good].kcal_per_kg;
        if per_kg > 0.0 {
            held.min(spare / per_kg)
        } else {
            held
        }
    }

    /// Household `from` gives `kg` of good `good` to `to` on the repaid channel: whether it was
    /// given.
    fn pay_loan(
        &mut self,
        ctx: &Ctx,
        from: PermanentId,
        to: PermanentId,
        good: usize,
        kg: f64,
    ) -> bool {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let leg = Leg {
            from,
            to,
            good,
            amount: kg,
        };
        kg > 1e-6 && self.transfer(now, params, goods, &[leg], Channel::Repaid)
    }

    /// Loans due: each debtor answers once, pays what it can spare, asks for time; a loan refused
    /// or unpaid past its days of grace fails and its pledge is taken (see the module's words).
    fn repay_loans(&mut self, ctx: &Ctx, day: i64) {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let cp = &params.credit;
        let grace = i64::from(cp.grace_days);
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
            if self.household(c.debtor).is_none() || self.household(c.creditor).is_none() {
                self.order.credit.claims[i].standing = Standing::Lapsed;
                continue;
            }
            let elders = (
                self.elder_of(c.creditor, now, params),
                self.elder_of(c.debtor, now, params),
            );
            if c.answer.is_none() {
                let held = self.household(c.debtor).map_or(0.0, |x| {
                    stock_kcal(&stores_now(x, now, params, goods), goods)
                });
                let regard = match elders {
                    (Some(a), Some(b)) => self.ties.regard(b, a, day, &params.ties),
                    _ => 0.0,
                };
                // A debt beyond all it holds weighs as all it holds.
                let cost_share = (c.left() * goods[good].kcal_per_kg / held.max(1.0)).min(1.0);
                let points = pay_points(regard, cost_share, cp);
                let key = [ctx.seed, PURPOSE_LOAN_ANSWER, u64::from(c.id), day as u64];
                let pays = Rng64::from_key(&key).next_f64() < logistic(points);
                self.order.credit.claims[i].answer = Some((pays, points as f32));
                if !pays {
                    self.order.credit.claims[i].standing = Standing::Refused;
                    self.loan_failed(ctx, i, day);
                    continue;
                }
            }
            let pay = c.left().min(self.can_give(ctx, c.debtor, good));
            if self.pay_loan(ctx, c.debtor, c.creditor, good, pay) {
                self.order.credit.claims[i].paid += pay as f32;
            }
            let claim = &self.order.credit.claims[i];
            if claim.left() <= 1e-3 {
                self.order.credit.claims[i].standing = Standing::Repaid;
                if let (Some(a), Some(b)) = elders {
                    self.note_tie(ctx, a, b, Act::Repaid, 1.0, 0.0);
                }
                continue;
            }
            // Short: it asks for time once for each due day, granted by the creditor's regard.
            if claim.asks == claim.extensions && u32::from(claim.extensions) < cp.max_extensions {
                let regard = match elders {
                    (Some(a), Some(b)) => self.ties.regard(a, b, day, &params.ties),
                    _ => 0.0,
                };
                let claim = &mut self.order.credit.claims[i];
                claim.asks += 1;
                if regard >= cp.extend_regard {
                    claim.extensions += 1;
                    claim.due = day + i64::from(cp.extension_days);
                    continue;
                }
            }
            if day > self.order.credit.claims[i].due + grace {
                self.order.credit.claims[i].standing = Standing::Defaulted;
                self.loan_failed(ctx, i, day);
            }
        }
    }

    /// Loan `i` failed today (refused, or unpaid when grace ran out): the creditor's household
    /// notes it of the debtor's elder, and the creditor takes the pledge: the guarantor is called
    /// on now; a field is taken when it lies between crops ([`Self::pledged_fields`]).
    fn loan_failed(&mut self, ctx: &Ctx, i: usize, day: i64) {
        let (now, params) = (ctx.now, ctx.params);
        let c = self.order.credit.claims[i].clone();
        if let Some(b) = self.elder_of(c.debtor, now, params) {
            let members = self
                .household(c.creditor)
                .map(|x| x.members.clone())
                .unwrap_or_default();
            for m in members {
                self.note_tie(ctx, m, b, Act::Defaulted, 1.0, 0.0);
            }
        }
        let Some(Pledge::Guarantor(g)) = c.pledge else {
            return;
        };
        let good = usize::from(c.good);
        let pay = c.left().min(self.can_give(ctx, g, good));
        self.order.credit.claims[i].taken = Some(day);
        if !self.pay_loan(ctx, g, c.creditor, good, pay) {
            return;
        }
        let claim = &mut self.order.credit.claims[i];
        claim.paid += pay as f32;
        claim.by_guarantor += pay as f32;
        // What the guarantor paid, the debtor owes it in turn, due when the days of grace end.
        let witnesses = [
            self.elder_of(g, now, params).unwrap_or(c.witnesses[0]),
            c.witnesses[1],
        ];
        let credit = &mut self.order.credit;
        credit.next += 1;
        credit.claims.push(Claim {
            id: credit.next,
            creditor: g,
            debtor: c.debtor,
            good: c.good,
            lent: pay as f32,
            owed: pay as f32,
            paid: 0.0,
            made: day,
            due: day + i64::from(params.credit.grace_days),
            standing: Standing::Open,
            witnesses,
            pledge: None,
            answer: None,
            asks: 0,
            extensions: 0,
            taken: None,
            by_guarantor: 0.0,
            recourse: Some(c.id),
        });
        // The creditor saw the guarantor keep its word.
        if let (Some(a), Some(b)) = (
            self.elder_of(c.creditor, now, params),
            self.elder_of(g, now, params),
        ) {
            self.note_tie(ctx, a, b, Act::Repaid, 1.0, 0.0);
        }
    }

    /// Fields pledged on failed loans change hands where they lie between crops: to the creditor,
    /// on the regime's lease, while the debt is unpaid; back to the debtor once the crop has paid
    /// it, or the creditor's household is no more.
    fn pledged_fields(&mut self, ctx: &mut Ctx, day: i64) {
        let Some(rules) = ctx.regime.lease else {
            return;
        };
        let now = ctx.now;
        for i in 0..self.order.credit.claims.len() {
            let c = &self.order.credit.claims[i];
            let Some(Pledge::Field(fid)) = c.pledge else {
                continue;
            };
            if !c.standing.failed() {
                continue;
            }
            let (debtor, creditor, taken, left) = (c.debtor, c.creditor, c.taken, c.left());
            let Some(fi) = ctx.land.fields.iter().position(|f| f.id == fid) else {
                continue;
            };
            let f = &ctx.land.fields[fi];
            if !super::land::between_crops(f) || f.holder != Party::Household(debtor) {
                continue;
            }
            let creditor_here = self.household(creditor).is_some();
            match taken {
                None if left > 1e-3
                    && creditor_here
                    && f.household == debtor
                    && f.lease.is_none() =>
                {
                    let f = &mut ctx.land.fields[fi];
                    f.household = creditor;
                    f.lease = Some(civ_land::Lease {
                        since: now,
                        until: now.plus_minutes(PLEDGE_YEARS * 365 * 24 * 60),
                        holder_share: rules.holder_share as f32,
                    });
                    self.fields_moved();
                    self.order.credit.claims[i].taken = Some(day);
                }
                Some(_) if (left <= 1e-3 || !creditor_here) && f.household == creditor => {
                    let f = &mut ctx.land.fields[fi];
                    f.household = debtor;
                    f.lease = None;
                    self.fields_moved();
                }
                _ => {}
            }
        }
    }

    /// At threshing on field `field` by `household`, which keeps `kept_kg` of good `crop_good`:
    /// a creditor working a field pledged to it counts what it keeps against the debt.
    pub(super) fn pledge_crop(
        &mut self,
        ctx: &Ctx,
        field: PermanentId,
        household: PermanentId,
        crop_good: usize,
        kept_kg: f64,
    ) {
        let goods = &ctx.catalog.goods;
        let Some(i) = self.order.credit.claims.iter().position(|c| {
            c.pledge == Some(Pledge::Field(field))
                && c.creditor == household
                && c.taken.is_some()
                && c.standing.failed()
                && c.left() > 1e-3
        }) else {
            return;
        };
        let c = &mut self.order.credit.claims[i];
        let per_kg = goods[usize::from(c.good)].kcal_per_kg;
        let kg = if per_kg > 0.0 {
            kept_kg * goods[crop_good].kcal_per_kg / per_kg
        } else {
            kept_kg
        };
        c.paid += kg.min(c.left()) as f32;
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

    /// Loans due by `day` are answered and paid now, and pledged fields change hands: for
    /// tests.
    #[doc(hidden)]
    pub fn repay_loans_for_tests(&mut self, ctx: &mut Ctx, day: i64) {
        self.repay_loans(ctx, day);
        self.pledged_fields(ctx, day);
    }
}
