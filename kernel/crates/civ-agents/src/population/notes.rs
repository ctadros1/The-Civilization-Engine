//! Notes that pass (M10d slice EH, second part; ADR-0035 §8-9; brief m10-money §4): a piece of a
//! keeper's notes is worth to a household of the keeper's settlement, who knows its door, the good
//! it is redeemed in at its own cost of the good, times its belief that the keeper pays (the share
//! of the pieces presented there that were paid, which the settlement hears of), less its share of
//! a walk to the door to present a carry of them. It wants notes as much as it wants that good,
//! so one short of grain takes notes for grain in payment. Notes are paid, never offered as a
//! ware. At its weekly review a household holding notes presents, at their keepers' doors, the
//! pieces that bring its food back to what it keeps to its harvest when it is short, and every
//! piece of an issue its settlement heard refused within the month.
//!
//! Runs (slice EI, second part; ADR-0027 §4, ADR-0035 §10): it also presents every piece of an
//! issue when its settlement heard within the month that a debtor of the keeper's loans in them
//! defaulted, or saw more than a tenth of the pieces out presented at the keeper's door within
//! the week. Once the keeper has lent in them, what a piece is worth falls with what the
//! settlement sees behind the pieces out (the goods left with the keeper and the keeper's own).

use super::custody::{FEE_DAYS, RETAKE_SHARE, WALK_M_S};
use super::*;
use crate::notes::{Issue, SEEN_DAYS};

/// Pieces of a keeper's notes one presents at a time, over which a walk to its door is spread (a
/// design prior: about what one carries of the grain they are redeemed in).
const NOTE_CARRY: f64 = 50.0;
/// The share of a keeper's pieces out that, seen presented at its door within a week, makes those
/// who hold its notes present theirs too (a design prior: "seeing others withdraw", ADR-0027 §4).
const RUN_SHARE: f64 = 0.1;

impl Population {
    /// The share of issue `c`'s pieces presented that were paid: what a household of its keeper's
    /// settlement believes of the keeper paying (1 before any was presented).
    pub(crate) fn note_belief(c: &Issue) -> f64 {
        if c.presented > 1e-9 {
            (1.0 - c.refused / c.presented).clamp(0.0, 1.0)
        } else {
            1.0
        }
    }

    /// What a household of its keeper's settlement sees behind issue `c`'s pieces out once the
    /// keeper has lent in them, as a share of what they pay (at most 1): the goods left with the
    /// keeper backing them and the keeper's own of the good. 1 before it lent (M10d slice EI).
    pub(crate) fn note_cover_seen(&self, c: &Issue) -> f64 {
        let out = c.count * f64::from(c.kg);
        if c.lent <= 1e-9 || out <= 1e-9 {
            return 1.0;
        }
        let good = usize::from(c.good);
        let lots: f64 = self
            .kept
            .iter()
            .filter(|k| k.keeper == c.issuer && k.noted)
            .map(|k| k.stores.get(good).copied().unwrap_or(0.0).max(0.0))
            .sum();
        let own = self
            .household(c.issuer)
            .and_then(|x| x.stores.get(good).copied())
            .unwrap_or(0.0)
            .max(0.0);
        ((lots + own) / out).min(1.0)
    }

    /// Where pieces of issuer `issuer`'s notes are presented, for a household of settlement
    /// `settlement`: a keeper's home, when it lives in that settlement, or the hearth of a
    /// polity whose settlement it is (M10d slice EJ: its store's notes are redeemed at the store).
    pub(crate) fn issuer_door(
        &self,
        ctx: &Ctx,
        issuer: PermanentId,
        settlement: Option<PermanentId>,
    ) -> Option<(f32, f32)> {
        let s = settlement?;
        if let Some(x) = self.household(issuer) {
            return (!x.members.is_empty() && x.settlement == Some(s)).then_some(x.home);
        }
        self.polities
            .iter()
            .find(|p| p.id == issuer && p.settlement == s)?;
        ctx.land
            .settlements
            .iter()
            .find(|y| y.id == s)
            .map(|y| y.hearth_m)
    }

    /// What a piece of issue `c` is worth to household `hh` at its own costs `costs`, in hours,
    /// if it knows the issuer's door (a keeper living in its settlement, or its settlement's
    /// store): the good it pays, at its cost, times its belief that the issuer pays, less its
    /// share of a walk to the door and back. Its keeper values its own pieces at what they would
    /// pay.
    pub(crate) fn piece_worth(
        &self,
        ctx: &Ctx,
        hh: &Household,
        c: &Issue,
        costs: &[Option<f64>],
    ) -> Option<f64> {
        let cost = costs.get(usize::from(c.good)).copied().flatten()?;
        let kg = f64::from(c.kg);
        if c.issuer == hh.id {
            return (cost > 0.0 && !hh.members.is_empty()).then_some(kg * cost);
        }
        // Notes nobody may redeem pay nothing at the door (M10d slice EJ, third part).
        if self.store_suspends(c.issuer) {
            return None;
        }
        let door = self.issuer_door(ctx, c.issuer, hh.settlement)?;
        let metres = f64::from(door.0 - hh.home.0).hypot(f64::from(door.1 - hh.home.1));
        let walk_h = 2.0 * metres / WALK_M_S / 3600.0 / NOTE_CARRY;
        let w = kg * cost * Self::note_belief(c) * self.note_cover_seen(c) - walk_h;
        (w > 0.0).then_some(w)
    }

    /// The issues of notes household `hh` knows the doors of: those it holds and those of keepers
    /// of its settlement and of its settlement's store.
    fn known_issues<'a>(
        &'a self,
        ctx: &'a Ctx,
        hh: &'a Household,
    ) -> impl Iterator<Item = &'a Issue> + 'a {
        self.notes.issues.iter().filter(move |c| {
            c.count > 1e-9
                && (self.notes.held(hh.id, c.id) > 1e-9
                    || self.issuer_door(ctx, c.issuer, hh.settlement).is_some())
        })
    }

    /// What a piece of notes is worth to household `hh` at its own costs, in hours: of the
    /// issues it holds, the one worth least to it (as it pays with those first); holding none, of
    /// those of its settlement's keepers, the one worth least (as it cannot choose which it is
    /// paid in). None where it knows none worth anything.
    pub(crate) fn note_worth(
        &self,
        ctx: &Ctx,
        hh: &Household,
        costs: &[Option<f64>],
    ) -> Option<f64> {
        let held = self
            .notes
            .purse(hh.id)
            .is_some_and(|p| !p.counts.is_empty());
        self.known_issues(ctx, hh)
            .filter(|c| !held || self.notes.held(hh.id, c.id) > 1e-9)
            .map(|c| self.piece_worth(ctx, hh, c, costs).unwrap_or(0.0))
            .reduce(f64::min)
            .filter(|&w| w > 0.0)
    }

    /// How much household `hh` wants notes, from how much it wants the goods those it knows are
    /// redeemed in (`want`, by good): as much as the least wanted, as it cannot choose which it is
    /// paid in.
    pub(super) fn note_want(&self, ctx: &Ctx, hh: &Household, want: &[f64]) -> f64 {
        self.known_issues(ctx, hh)
            .filter(|c| c.issuer != hh.id)
            .map(|c| want.get(usize::from(c.good)).copied().unwrap_or(0.0))
            .reduce(f64::min)
            .unwrap_or(0.0)
    }

    /// The pieces household `hh` presents at its review (see the module documentation): (issue,
    /// pieces).
    fn notes_presented(&self, ctx: &Ctx, hh: &Household, day: i64) -> Vec<(u32, f64)> {
        let goods = &ctx.catalog.goods;
        let Some(purse) = self.notes.purse(hh.id) else {
            return Vec::new();
        };
        let stores = stores_now(hh, ctx.now, ctx.params, goods);
        let food = stock_kcal(&stores, goods);
        let target = self.holding_of(ctx, hh, &stores).food_keep;
        let mut short = if food < RETAKE_SHARE * target {
            target - food
        } else {
            0.0
        };
        let mut out = Vec::new();
        for &(i, n) in &purse.counts {
            let Some(c) = self.notes.issue(i) else {
                continue;
            };
            // Notes nobody redeems are not presented (M10d slice EJ).
            let at_door = self.issuer_door(ctx, c.issuer, hh.settlement).is_some();
            if !at_door || c.issuer == hh.id || c.kg <= 0.0 || self.store_suspends(c.issuer) {
                continue;
            }
            // Its settlement heard the keeper refuse lately, or a debtor of its loans in them
            // default, or saw others present a tenth of those out this week: it presents them all.
            let heard = |d: i64| d >= 0 && day - d <= FEE_DAYS;
            let seen = c.seen_day >= 0
                && day - c.seen_day < SEEN_DAYS
                && c.seen > RUN_SHARE * c.count.max(1e-9);
            if heard(c.refused_day) || heard(c.defaulted_day) || seen {
                out.push((i, n));
                continue;
            }
            let kcal = goods
                .get(usize::from(c.good))
                .map_or(0.0, |g| g.kcal_per_kg)
                * f64::from(c.kg);
            if short > 1e-9 && kcal > 0.0 {
                let pieces = (short / kcal).min(n);
                short -= pieces * kcal;
                out.push((i, pieces));
            }
        }
        out
    }

    /// The pieces households whose review it is present (see the module documentation), those
    /// with goods of their own backing notes taking them back as custody does: (issue, holder,
    /// pieces).
    pub(super) fn notes_review(&self, ctx: &Ctx, day: i64) -> Vec<(u32, PermanentId, f64)> {
        if self.notes.purses.is_empty() {
            return Vec::new();
        }
        let review_days = i64::from(ctx.params.market.review_days.max(1));
        let mut out = Vec::new();
        for p in &self.notes.purses {
            let Some(hh) = self.household(p.holder).filter(|x| !x.members.is_empty()) else {
                continue;
            };
            if (hh.id.get() as i64 + day).rem_euclid(review_days) != 0
                || self.kept.iter().any(|k| k.noted && k.depositor == hh.id)
            {
                continue;
            }
            out.extend(
                self.notes_presented(ctx, hh, day)
                    .into_iter()
                    .map(|(i, n)| (i, hh.id, n)),
            );
        }
        out
    }

    /// Kilograms of good `good` keeper household `keeper` holds beyond what it keeps: what it may
    /// pay its notes with of its own once the goods left with it behind them run out (M10d slice
    /// EI).
    pub(super) fn keeper_spare(&self, ctx: &Ctx, keeper: PermanentId, good: usize) -> f64 {
        let Some(x) = self.household(keeper).filter(|x| !x.members.is_empty()) else {
            return 0.0;
        };
        let stores = stores_now(x, ctx.now, ctx.params, &ctx.catalog.goods);
        let keep = self.holding_of(ctx, x, &stores).keep;
        (stores.get(good).copied().unwrap_or(0.0) - keep.get(good).copied().unwrap_or(0.0)).max(0.0)
    }

    /// Kilograms behind issue `c`: the goods left with its keeper backing its notes, and the
    /// keeper's own beyond what it keeps.
    pub(crate) fn note_behind(&self, ctx: &Ctx, c: &Issue) -> f64 {
        let good = usize::from(c.good);
        let lots: f64 = self
            .kept
            .iter()
            .filter(|k| k.keeper == c.issuer && k.noted)
            .map(|k| k.stores.get(good).copied().unwrap_or(0.0).max(0.0))
            .sum();
        lots + self.keeper_spare(ctx, c.issuer, good)
    }

    /// What keeper household `h` may lend in its notes (M10d slice EI; ADR-0035 §10): of its
    /// issues in food, the one it may write most of while what is behind it stays at least the
    /// content's coverage of what the pieces out pay; that issue and the kilocalories it may lend.
    pub(super) fn note_lending(&self, ctx: &Ctx, h: &Household) -> Option<(u32, f64)> {
        let goods = &ctx.catalog.goods;
        let coverage = ctx.params.market.note_coverage;
        if coverage <= 0.0 {
            return None;
        }
        self.notes
            .issues
            .iter()
            .filter(|c| c.issuer == h.id && c.count > 1e-9)
            .filter_map(|c| {
                let per_kg = goods.get(usize::from(c.good))?.kcal_per_kg;
                if per_kg <= 0.0 {
                    return None;
                }
                let kg = self.note_behind(ctx, c) / coverage - c.count * f64::from(c.kg);
                (kg > 1e-6).then_some((c.id, kg * per_kg))
            })
            .max_by(|a, b| a.1.total_cmp(&b.1).then(b.0.cmp(&a.0)))
    }

    /// Keeper household `lender` lends `kg` of the good of its issue `issue` to household
    /// `borrower` in its notes (M10d slice EI): it writes the pieces and pays them on `Lent`, a
    /// recorded act that raises what it owes. Whether it lent.
    pub(super) fn lend_notes(
        &mut self,
        ctx: &Ctx,
        lender: PermanentId,
        borrower: PermanentId,
        issue: u32,
        kg: f64,
    ) -> bool {
        let (now, params) = (ctx.now, ctx.params);
        let goods = &ctx.catalog.goods;
        let Some(note) = goods
            .iter()
            .position(|g| g.money.as_ref().is_some_and(|m| m.note))
        else {
            return false;
        };
        let Some(c) = self.notes.issue(issue) else {
            return false;
        };
        let (good, per) = (usize::from(c.good), c.kg);
        let pieces = kg / f64::from(per).max(1e-9);
        if self
            .put_written(note, lender, lender, (good, per), pieces)
            .is_none()
        {
            return false;
        }
        let leg = Leg {
            from: lender,
            to: borrower,
            good: note,
            amount: pieces,
        };
        if !self.transfer(now, params, goods, &[leg], Channel::Lent) {
            self.retire_written((now, params, goods), lender, (note, issue), pieces);
            return false;
        }
        if let Some(c) = self.notes.issues.iter_mut().find(|c| c.id == issue) {
            c.lent += pieces;
        }
        true
    }

    /// What suspending the redemption of polity `pi`'s store's notes would bring each of
    /// `households` and cost it, kcal, in household order (M10d slice EJ, third part): its share
    /// of the good the store would no longer pay out for the pieces out of it, and the good its
    /// own pieces would no longer fetch.
    pub(super) fn suspend_stakes(
        &self,
        ctx: &Ctx,
        pi: usize,
        households: &[PermanentId],
    ) -> Vec<(PermanentId, f64, f64)> {
        let goods = &ctx.catalog.goods;
        let polity = self.polities[pi].id;
        let mut out: Vec<(PermanentId, f64, f64)> =
            households.iter().map(|&h| (h, 0.0, 0.0)).collect();
        out.sort_by_key(|x| x.0);
        let n = out.len().max(1) as f64;
        for c in self.notes.issues.iter().filter(|c| c.issuer == polity) {
            let kcal = f64::from(c.kg)
                * goods
                    .get(usize::from(c.good))
                    .map_or(0.0, |g| g.kcal_per_kg);
            let out_of_store = (c.count - self.notes.held(polity, c.id)).max(0.0);
            for x in &mut out {
                x.1 += out_of_store * kcal / n;
                x.2 += self.notes.held(x.0, c.id) * kcal;
            }
        }
        out
    }

    /// What legal tender in polity `pi` would bring each of `households` as a debtor and cost it
    /// as a creditor, kcal, in household order (M10d slice EJ, third part): for each loan open
    /// between them, the good the debtor could pay in the store's notes it holds, at the rate its
    /// store takes them, times how much less the notes are worth to it than the good.
    pub(super) fn tender_stakes(
        &self,
        ctx: &Ctx,
        pi: usize,
        households: &[PermanentId],
    ) -> Vec<(PermanentId, f64, f64)> {
        let goods = &ctx.catalog.goods;
        let mut out: Vec<(PermanentId, f64, f64)> =
            households.iter().map(|&h| (h, 0.0, 0.0)).collect();
        out.sort_by_key(|x| x.0);
        let Some(m) = self.notes_dues_medium(ctx, pi) else {
            return out;
        };
        let polity = self.polities[pi].id;
        let per_kg = f64::from(m.per_kg);
        for c in self
            .order
            .credit
            .claims
            .iter()
            .filter(|c| c.standing == crate::credit::Standing::Open)
        {
            let Some(hh) = self.household(c.debtor) else {
                continue;
            };
            let good = usize::from(c.good);
            let held = self.dues_held(ctx, hh, m.good, polity);
            let kg = c.left().min(held / per_kg);
            if kg <= 1e-9 || per_kg <= 0.0 {
                continue;
            }
            let costs = self.own_costs_of(ctx, hh);
            let good_h = costs.get(good).copied().flatten().unwrap_or(0.0);
            let unit_h = self
                .pay_worth_raw(ctx, hh, m.good, costs.get(m.good).copied().flatten(), held)
                .unwrap_or(0.0);
            if good_h <= 0.0 {
                continue;
            }
            let discount = (1.0 - unit_h * per_kg / good_h).clamp(0.0, 1.0);
            let kcal = kg * goods.get(good).map_or(0.0, |g| g.kcal_per_kg) * discount;
            if let Ok(i) = out.binary_search_by_key(&c.debtor, |x| x.0) {
                out[i].1 += kcal;
            }
            if let Ok(i) = out.binary_search_by_key(&c.creditor, |x| x.0) {
                out[i].2 += kcal;
            }
        }
        out
    }

    /// Under legal tender in the creditor's polity (M10d slice EJ, third part), the debtor of
    /// loan `i` pays what it can of it in the store's own notes first, when they are worth less
    /// to it than the good, at the rate its store takes them; the creditor must take them.
    /// Returns the kilograms of the good paid so.
    pub(super) fn tender_on_loan(&mut self, ctx: &Ctx, i: usize) -> f64 {
        let c = self.order.credit.claims[i].clone();
        let Some(pi) = self
            .household(c.creditor)
            .and_then(|x| x.settlement)
            .and_then(|s| self.polity_of(s))
        else {
            return 0.0;
        };
        if !self.tender_in_force(pi) {
            return 0.0;
        }
        let Some(m) = self.notes_dues_medium(ctx, pi) else {
            return 0.0;
        };
        let polity = self.polities[pi].id;
        let Some(hh) = self.household(c.debtor) else {
            return 0.0;
        };
        let per_kg = f64::from(m.per_kg);
        let held = self.dues_held(ctx, hh, m.good, polity);
        let left = c.left();
        if held <= 1e-9 || per_kg <= 0.0 || left <= 1e-9 {
            return 0.0;
        }
        let good = usize::from(c.good);
        let costs = self.own_costs_of(ctx, hh);
        let good_h = costs.get(good).copied().flatten().unwrap_or(0.0);
        let unit_h = self
            .pay_worth_raw(ctx, hh, m.good, costs.get(m.good).copied().flatten(), held)
            .unwrap_or(0.0);
        if unit_h * per_kg >= good_h {
            return 0.0;
        }
        let units = (left * per_kg).min(held);
        let before: Vec<(u32, f64)> = self
            .notes
            .purse(c.debtor)
            .map(|p| p.counts.clone())
            .unwrap_or_default();
        let leg = Leg {
            from: c.debtor,
            to: c.creditor,
            good: m.good,
            amount: units,
        };
        if !self.transfer(
            ctx.now,
            ctx.params,
            &ctx.catalog.goods,
            &[leg],
            Channel::Repaid,
        ) {
            return 0.0;
        }
        // Only the store's own pieces are tender: any other issue's that moved go back, and as
        // many of the store's go instead.
        let mut wrong = 0.0;
        for (id, b) in before {
            let moved = b - self.notes.held(c.debtor, id);
            let own = self.notes.issue(id).is_some_and(|x| x.issuer == polity);
            if moved > 1e-9 && !own {
                self.notes.give_issue(c.creditor, c.debtor, id, moved);
                wrong += moved;
            }
        }
        let own: Vec<u32> = self
            .notes
            .issues
            .iter()
            .filter(|x| x.issuer == polity)
            .map(|x| x.id)
            .collect();
        for id in own {
            if wrong <= 1e-9 {
                break;
            }
            let n = wrong.min(self.notes.held(c.debtor, id));
            self.notes.give_issue(c.debtor, c.creditor, id, n);
            wrong -= n;
        }
        let kg = units / per_kg;
        self.order.credit.claims[i].paid += kg as f32;
        kg
    }

    /// The debtor of loan `i` pays what it would of it in notes under legal tender
    /// ([`Self::tender_on_loan`]): for tests. Returns the kilograms paid so.
    #[doc(hidden)]
    pub fn tender_on_loan_for_tests(&mut self, ctx: &Ctx, i: usize) -> f64 {
        self.tender_on_loan(ctx, i)
    }

    /// What keeper household `household` may lend in its notes ([`Self::note_lending`]): for
    /// tests.
    #[doc(hidden)]
    pub fn note_lending_for_tests(&self, ctx: &Ctx, household: PermanentId) -> Option<(u32, f64)> {
        self.note_lending(ctx, self.household(household)?)
    }

    /// Keeper `lender` lends `kg` in its notes of issue `issue` to `borrower`
    /// ([`Self::lend_notes`]): for tests.
    #[doc(hidden)]
    pub fn lend_notes_for_tests(
        &mut self,
        ctx: &Ctx,
        (lender, borrower): (PermanentId, PermanentId),
        issue: u32,
        kg: f64,
    ) -> bool {
        self.lend_notes(ctx, lender, borrower, issue, kg)
    }

    /// Household `household` presents the pieces it would at its review, and they are paid as at
    /// midnight: for tests.
    #[doc(hidden)]
    pub fn notes_review_for_tests(&mut self, ctx: &Ctx, household: PermanentId) {
        let day = ctx.now.day_index();
        let Some(hh) = self.household(household) else {
            return;
        };
        let presented: Vec<(u32, PermanentId, f64)> = self
            .notes_presented(ctx, hh, day)
            .into_iter()
            .map(|(i, n)| (i, household, n))
            .collect();
        self.redeem(ctx, day, presented);
    }
}
