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
            let due = day + to_harvest.ceil() as i64 + 7;
            // Under a law of lending from the store, the store first, at the law's terms (M7d
            // slice CC).
            if self.borrow_from_store(ctx, id, borrower, want, due) {
                continue;
            }
            // Those with food to spare, or a keeper who may lend in its notes more than it has
            // to spare (M10d slice EI), those its elder regards first.
            let mut lenders: Vec<(f64, PermanentId, f64, Option<u32>)> = self
                .households
                .iter()
                .map(|(_, h)| h)
                .filter(|h| h.id != id && h.settlement == settlement && !h.members.is_empty())
                // One that owes lends nothing of what it borrowed; one that guarantees keeps
                // what it promised.
                .filter(|h| !self.order.credit.owes(h.id) && !self.order.credit.guarantees(h.id))
                .filter_map(|h| {
                    let spare = spare_food_kcal(h, now, params, goods);
                    let notes = self.note_lending(ctx, h);
                    match notes {
                        Some((issue, kcal)) if kcal > spare && kcal > 1.0 => {
                            Some((h.id, kcal, Some(issue)))
                        }
                        _ => (spare > 1.0).then_some((h.id, spare, None)),
                    }
                })
                .map(|(h, spare, issue)| {
                    let regard = self
                        .elder_of(h, now, params)
                        .map_or(0.0, |e| self.ties.regard(borrower, e, day, &params.ties));
                    (regard, h, spare, issue)
                })
                .collect();
            lenders.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
            lenders.truncate(cp.max_asked as usize);
            let t_years = (due - day) as f64 / civ_core::time::DAYS_PER_YEAR as f64;
            let field = self.pledgeable_field(ctx, id);
            for (_, lender, spare, issue) in lenders {
                self.order.credit.asked += 1;
                let Some(elder) = self.elder_of(lender, now, params) else {
                    continue;
                };
                // A loan in notes is in the good they are redeemed in, as far as it may write
                // them (M10d slice EI).
                let lent_in = match issue.and_then(|i| self.notes.issue(i)) {
                    Some(c) => Some((
                        usize::from(c.good),
                        spare / goods[usize::from(c.good)].kcal_per_kg,
                    )),
                    None => self.richest_food(ctx, lender),
                };
                let Some((good, kg_held)) = lent_in else {
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
                let lent = match issue {
                    Some(i) => self.lend_notes(ctx, lender, id, i, kg),
                    None => {
                        let leg = Leg {
                            from: lender,
                            to: id,
                            good,
                            amount: kg,
                        };
                        self.transfer(now, params, goods, &[leg], Channel::Lent)
                    }
                };
                if !lent {
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

    /// Household `hh`, whose elder is `borrower`, short `want` kcal before its harvest, borrows
    /// from its polity's common store under a law of lending from the store in force there that
    /// one of its members knows (M7d slice CC, ADR-0027 §4): the lesser of what it wants and the
    /// law's share of the food the store holds, in the food good the store holds most of, owed
    /// with the law's increment, due `due`. Whether it borrowed.
    fn borrow_from_store(
        &mut self,
        ctx: &Ctx,
        hh: PermanentId,
        borrower: PermanentId,
        want: f64,
        due: i64,
    ) -> bool {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let day = now.day_index();
        let Some(x) = self.household(hh) else {
            return false;
        };
        let members = x.members.clone();
        let Some(pi) = x.settlement.and_then(|s| self.polity_of(s)) else {
            return false;
        };
        let Some(def) = self.polities[pi]
            .in_force(
                &ctx.catalog.policies,
                crate::polity::PolicyKind::LendFromStore,
            )
            .find(|l| members.iter().any(|&m| l.knows(m)))
            .and_then(|l| ctx.catalog.policies.get(usize::from(l.policy)))
            .and_then(|d| d.lending)
        else {
            return false;
        };
        let sheltered = self.polity_sheltered(pi);
        self.polities[pi].settle_stores(now, goods, sheltered);
        let store = self.polities[pi].id;
        let stores = &self.polities[pi].stores;
        let Some((good, kg_held)) = (0..goods.len())
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
        else {
            return false;
        };
        let food = stock_kcal(stores, goods);
        let per_kg = goods[good].kcal_per_kg;
        let kcal = want.min(f64::from(def.share) * food).min(kg_held * per_kg);
        let kg = kcal / per_kg;
        self.order.credit.asked += 1;
        if kg <= 1e-3 {
            return false;
        }
        let leg = Leg {
            from: store,
            to: hh,
            good,
            amount: kg,
        };
        if !self.transfer(now, params, goods, &[leg], Channel::Lent) {
            return false;
        }
        let keeper = self.polities[pi].keeper().map_or(borrower, |(k, _)| k);
        let credit = &mut self.order.credit;
        credit.next += 1;
        credit.claims.push(Claim {
            id: credit.next,
            creditor: store,
            debtor: hh,
            good: good as u16,
            lent: kg as f32,
            owed: (kg * (1.0 + f64::from(def.increment))) as f32,
            paid: 0.0,
            made: day,
            due,
            standing: Standing::Open,
            witnesses: [keeper, borrower],
            pledge: None,
            answer: None,
            asks: 0,
            extensions: 0,
            taken: None,
            by_guarantor: 0.0,
            recourse: None,
        });
        true
    }

    /// Whether `id` is a creditor still there to be paid: a household, or a polity's store.
    fn creditor_here(&self, id: PermanentId) -> bool {
        self.household(id).is_some() || self.polities.iter().any(|p| p.id == id)
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
            // Wages a workshop owes are paid, or it fails (M7d slice CB).
            if self.firm(c.debtor).is_some() {
                self.repay_wages(ctx, i);
                continue;
            }
            let good = usize::from(c.good);
            if self.household(c.debtor).is_none() || !self.creditor_here(c.creditor) {
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
            // Under legal tender it pays in the store's notes first what they are worth less to
            // it than the good (M10d slice EJ, third part).
            self.tender_on_loan(ctx, i);
            let c = self.order.credit.claims[i].clone();
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

    /// Workshop `id` could not pay household `worker` `kg` of good `good` it owed in wages
    /// (ADR-0027 §3): it owes them, added to what it owes that household already, due when the
    /// days of wage grace run out.
    pub(super) fn owe_wages(
        &mut self,
        ctx: &Ctx,
        id: PermanentId,
        worker: PermanentId,
        good: usize,
        kg: f64,
    ) {
        let (now, params) = (ctx.now, ctx.params);
        let day = now.day_index();
        if kg <= 1e-9 {
            return;
        }
        let credit = &mut self.order.credit;
        if let Some(c) = credit.claims.iter_mut().find(|c| {
            c.debtor == id
                && c.creditor == worker
                && usize::from(c.good) == good
                && c.standing == Standing::Open
        }) {
            c.lent += kg as f32;
            c.owed += kg as f32;
            return;
        }
        let owner = self.firm(id).map(|f| f.owner);
        let witnesses = [
            self.elder_of(worker, now, params),
            owner.and_then(|o| self.elder_of(o, now, params)),
        ];
        let [Some(a), Some(b)] = witnesses else {
            return;
        };
        let credit = &mut self.order.credit;
        credit.next += 1;
        credit.claims.push(Claim {
            id: credit.next,
            creditor: worker,
            debtor: id,
            good: good as u16,
            lent: kg as f32,
            owed: kg as f32,
            paid: 0.0,
            made: day,
            due: day + i64::from(params.credit.wage_days),
            standing: Standing::Open,
            witnesses: [a, b],
            pledge: None,
            answer: None,
            asks: 0,
            extensions: 0,
            taken: None,
            by_guarantor: 0.0,
            recourse: None,
        });
    }

    /// Wages workshop claim `i` owes, due: the workshop pays what it holds of the good, its owners
    /// what they can spare besides, for they answer for what it owes; unpaid then, it fails, and
    /// the worker's household notes it of the owners' elder.
    fn repay_wages(&mut self, ctx: &Ctx, i: usize) {
        let (now, params) = (ctx.now, ctx.params);
        let c = self.order.credit.claims[i].clone();
        let good = usize::from(c.good);
        if self.household(c.creditor).is_none() {
            self.order.credit.claims[i].standing = Standing::Lapsed;
            return;
        }
        let Some((owner, open)) = self.firm(c.debtor).map(|f| (f.owner, f.is_open())) else {
            return;
        };
        if open {
            self.settle_firm(ctx, c.debtor);
            let held = self
                .firm(c.debtor)
                .and_then(|f| f.stores.get(good))
                .copied()
                .unwrap_or(0.0)
                .max(0.0);
            let pay = c.left().min(held);
            if self.pay_loan(ctx, c.debtor, c.creditor, good, pay) {
                self.order.credit.claims[i].paid += pay as f32;
            }
        }
        let left = self.order.credit.claims[i].left();
        let pay = left.min(self.can_give(ctx, owner, good));
        if self.household(owner).is_some() && self.pay_loan(ctx, owner, c.creditor, good, pay) {
            self.order.credit.claims[i].paid += pay as f32;
        }
        let elders = (
            self.elder_of(c.creditor, now, params),
            self.elder_of(owner, now, params),
        );
        if self.order.credit.claims[i].left() <= 1e-3 {
            self.order.credit.claims[i].standing = Standing::Repaid;
            if let (Some(a), Some(b)) = elders {
                self.note_tie(ctx, a, b, Act::Repaid, 1.0, 0.0);
            }
            return;
        }
        self.order.credit.claims[i].standing = Standing::Defaulted;
        if let Some(b) = elders.1 {
            let members = self
                .household(c.creditor)
                .map(|x| x.members.clone())
                .unwrap_or_default();
            for m in members {
                self.note_tie(ctx, m, b, Act::Defaulted, 1.0, 0.0);
            }
        }
        if open {
            self.close_firm(ctx, c.debtor, crate::firm::Exit::Failure);
        }
    }

    /// Loans household `creditor` made that were refused or left unpaid, with something still
    /// owed, whose pledge was not taken and that nobody has brought a case for (M7d slice CB):
    /// with `notes`, only notes refused and booked as debts (M10d slice EI), else only the rest.
    pub(super) fn unheard_debts(&self, creditor: PermanentId, notes: bool) -> Vec<u32> {
        self.order
            .credit
            .claims
            .iter()
            .filter(|c| {
                c.creditor == creditor
                    && c.standing.failed()
                    && c.left() > 1e-3
                    && c.taken.is_none()
                    && self.firm(c.debtor).is_none()
                    && self.notes.is_dishonoured(c.id) == notes
            })
            .map(|c| c.id)
            .filter(|&id| !self.order.cases.iter().any(|k| k.loan == Some(id)))
            .collect()
    }

    /// What each of `households` is owed on loans refused or left unpaid and not yet heard, and
    /// what it owes on such loans, kcal (M7d slice CB), in household order: with `notes`, on
    /// notes refused and booked as debts (M10d slice EI).
    pub(super) fn debt_stakes(
        &self,
        ctx: &Ctx,
        households: &[PermanentId],
        notes: bool,
    ) -> Vec<(PermanentId, f64, f64)> {
        let goods = &ctx.catalog.goods;
        let mut out: Vec<(PermanentId, f64, f64)> =
            households.iter().map(|&h| (h, 0.0, 0.0)).collect();
        out.sort_by_key(|x| x.0);
        for h in households {
            for id in self.unheard_debts(*h, notes) {
                let Some(c) = self.order.credit.claim(id) else {
                    continue;
                };
                let kcal = c.left()
                    * goods
                        .get(usize::from(c.good))
                        .map_or(0.0, |g| g.kcal_per_kg);
                if let Ok(i) = out.binary_search_by_key(&c.creditor, |x| x.0) {
                    out[i].1 += kcal;
                }
                if let Ok(i) = out.binary_search_by_key(&c.debtor, |x| x.0) {
                    out[i].2 += kcal;
                }
            }
        }
        out
    }

    /// Whether a creditor brings a loan refused or left unpaid before polity `pi`'s gathering,
    /// at its review (M7d slice CB, ADR-0027 §3): with a law of debts heard in force, for each
    /// such loan of a household of the polity's settlements that nobody brought yet, each grown
    /// member of the creditor's household weighs U = V − C as a breach is weighed (research 09-07
    /// §4.1): V what a finding would move, the debt at the law's share, in days of their
    /// household's food at the points a day is worth in a dispute; C the cost of bringing a case
    /// and their regard for the debtor's elder. The one with the most to gain, if anyone, brings
    /// it.
    pub(super) fn weigh_debts(&mut self, ctx: &mut Ctx, pi: usize) {
        self.weigh_debts_under(ctx, pi, crate::polity::PolicyKind::DebtsHeard);
        // Notes refused, under a law of bearer heard, as loans are (M10d slice EI, third part).
        self.weigh_debts_under(ctx, pi, crate::polity::PolicyKind::BearerHeard);
    }

    /// [`Self::weigh_debts`] under a law of `kind` in force: debts heard for loans, bearer heard
    /// for notes refused and booked as debts.
    fn weigh_debts_under(&mut self, ctx: &mut Ctx, pi: usize, kind: crate::polity::PolicyKind) {
        let (now, params) = (ctx.now, ctx.params);
        let (pp, cp, day) = (&params.polity, &params.crime, now.day_index());
        let notes = kind == crate::polity::PolicyKind::BearerHeard;
        let Some((law_id, def)) = self.polities[pi]
            .laws
            .iter()
            .filter(|l| l.kind == kind && l.status == crate::polity::LawStatus::InForce)
            .find_map(|l| {
                let d = ctx.catalog.policies.get(usize::from(l.policy))?.debts?;
                Some((l.id, d))
            })
        else {
            return;
        };
        let goods = &ctx.catalog.goods;
        let mut creditors: Vec<PermanentId> = self
            .households
            .iter()
            .map(|(_, x)| x)
            .filter(|x| self.polities[pi].holds(x.settlement) && !x.members.is_empty())
            .map(|x| x.id)
            .collect();
        creditors.sort_unstable();
        for creditor in creditors {
            for id in self.unheard_debts(creditor, notes) {
                let Some(c) = self.order.credit.claim(id).cloned() else {
                    continue;
                };
                let Some(accused) = self
                    .elder_of(c.debtor, now, params)
                    .filter(|_| self.household(c.debtor).is_some())
                else {
                    continue;
                };
                let kcal = c.left()
                    * goods
                        .get(usize::from(c.good))
                        .map_or(0.0, |g| g.kcal_per_kg);
                let need = self.day_need(creditor, params);
                let v = cp.w_demand_loss * kcal * f64::from(def.share) / need;
                let members: Vec<PermanentId> = self
                    .household(creditor)
                    .map(|x| x.members.clone())
                    .unwrap_or_default();
                let mut best: Option<(f64, PermanentId)> = None;
                for m in members {
                    let grown = self
                        .person(m)
                        .is_some_and(|p| p.age_years(now) >= params.family.independent_age);
                    if !grown {
                        continue;
                    }
                    let regard = self
                        .ties
                        .regard(m, accused, day, &params.ties)
                        .clamp(0.0, 1.0);
                    let u = v - cp.report_cost - pp.w_regard * regard;
                    if u > 0.0 && best.is_none_or(|b| u > b.0) {
                        best = Some((u, m));
                    }
                }
                let Some((_, by)) = best else {
                    continue;
                };
                let settlement = self.polities[pi].settlement;
                let case = self.order.cases.last().map_or(1, |k| k.id + 1);
                self.order.cases.push(crate::crime::Case {
                    id: case,
                    incident: 0,
                    settlement,
                    law: law_id,
                    accuser: creditor,
                    by,
                    accused,
                    accused_household: c.debtor,
                    kcal: kcal as f32,
                    leads: vec![c.witnesses[0]],
                    opened: day,
                    stage: crate::crime::CaseStage::Open,
                    heard: None,
                    eligible: 0,
                    stances: Vec::new(),
                    exiled: false,
                    forum: None,
                    challenge: None,
                    tags: 0,
                    departed: None,
                    loan: Some(id),
                    building: None,
                });
                let (place, name) = ctx
                    .land
                    .settlements
                    .iter()
                    .find(|s| s.id == settlement)
                    .map_or((None, String::new()), |s| {
                        (Some(s.hearth_m), s.name.clone())
                    });
                let what = if notes {
                    "notes it refused at its door"
                } else {
                    "a loan it did not repay"
                };
                let words = format!(
                    "brought a case at {name} against {}'s household for {what}.",
                    self.name_of(accused),
                );
                self.chronicle_push(
                    now,
                    ChronicleKind::CaseBrought,
                    vec![by, accused],
                    Some(settlement),
                    place,
                    0.0,
                    words,
                );
            }
        }
    }

    /// `kcal` of food paid on debt imposed for loan `id` (M7d slice CB): paid on the loan, in its
    /// good.
    pub(super) fn paid_on_loan(&mut self, ctx: &Ctx, id: u32, kcal: f64) {
        let goods = &ctx.catalog.goods;
        if let Some(c) = self.order.credit.claims.iter_mut().find(|c| c.id == id) {
            let per_kg = goods
                .get(usize::from(c.good))
                .map_or(0.0, |g| g.kcal_per_kg);
            if per_kg > 0.0 {
                let kg = (kcal / per_kg).min(c.left());
                c.paid += kg as f32;
            }
        }
    }

    /// A finding under a law of debts heard against loan `id`'s debtor (M7d slice CB): the
    /// remedy `def` names. Time: the loan is open again, unanswered, due `extend_days` from
    /// today. Imposed: what is still owed, as a finding's obligation of the debtor's household
    /// to the creditor's, due as findings are. Words for the chronicle, after "…owes it".
    pub(super) fn debt_found(
        &mut self,
        ctx: &Ctx,
        id: u32,
        case: u32,
        def: crate::polity::DebtsDef,
    ) -> String {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let day = now.day_index();
        let Some(i) = self.order.credit.claims.iter().position(|c| c.id == id) else {
            return String::new();
        };
        let c = self.order.credit.claims[i].clone();
        match def.remedy {
            crate::polity::DebtRemedy::Time => {
                let claim = &mut self.order.credit.claims[i];
                claim.standing = Standing::Open;
                claim.answer = None;
                claim.due = day + i64::from(def.extend_days);
                format!(
                    ", and is given until {} to pay it",
                    crate::polity::day_words(SimTime::from_minutes(claim.due * 24 * 60))
                )
            }
            crate::polity::DebtRemedy::Imposed => {
                let kcal = c.left()
                    * goods
                        .get(usize::from(c.good))
                        .map_or(0.0, |g| g.kcal_per_kg);
                let ob = self.order.obligations.last().map_or(1, |o| o.id + 1);
                self.order.obligations.push(crate::crime::Obligation {
                    id: ob,
                    incident: id,
                    kind: crate::crime::Owed::Debt,
                    case: Some(case),
                    debtor: c.debtor,
                    beneficiary: c.creditor,
                    kcal: kcal as f32,
                    paid_kcal: 0.0,
                    made: day,
                    due: day + i64::from(params.crime.due_days),
                    standing: crate::crime::Standing::Open,
                    answer: None,
                });
                ", which the finding imposes".to_owned()
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
        // A keeper's debtor of a loan in its notes defaulted: its settlement hears of it (M10d
        // slice EI, second part).
        if let Some(issue) = self
            .notes
            .issue_of(c.creditor, usize::from(c.good))
            .filter(|x| x.lent > 1e-9)
            .map(|x| x.id)
            && let Some(x) = self.notes.issue_mut(issue)
        {
            x.defaulted_day = day;
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

    /// Whether a creditor of polity `pi` brings a failed loan, whatever day it is: for tests.
    #[doc(hidden)]
    pub fn weigh_debts_for_tests(&mut self, ctx: &mut Ctx, pi: usize) {
        self.weigh_debts(ctx, pi);
    }

    /// A finding against loan `id`'s debtor in case `case`, by remedy `def`: for tests.
    #[doc(hidden)]
    pub fn debt_found_for_tests(
        &mut self,
        ctx: &Ctx,
        id: u32,
        case: u32,
        def: crate::polity::DebtsDef,
    ) -> String {
        self.debt_found(ctx, id, case, def)
    }

    /// Household `owner` sets up a workshop making `good`, founded by `founder`: for tests.
    #[doc(hidden)]
    pub fn found_firm_for_tests(
        &mut self,
        ctx: &mut Ctx,
        owner: PermanentId,
        founder: PermanentId,
        good: usize,
    ) -> Option<PermanentId> {
        self.found_firm(ctx, owner, founder, good)
    }

    /// Workshop `id` owes household `worker` `kg` of good `good` in wages: for tests.
    #[doc(hidden)]
    pub fn owe_wages_for_tests(
        &mut self,
        ctx: &Ctx,
        id: PermanentId,
        worker: PermanentId,
        good: usize,
        kg: f64,
    ) {
        self.owe_wages(ctx, id, worker, good, kg);
    }

    /// Loans due by `day` are answered and paid now, and pledged fields change hands: for
    /// tests.
    #[doc(hidden)]
    pub fn repay_loans_for_tests(&mut self, ctx: &mut Ctx, day: i64) {
        self.repay_loans(ctx, day);
        self.pledged_fields(ctx, day);
    }
}
