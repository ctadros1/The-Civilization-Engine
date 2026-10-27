//! Notes that pass (M10d slice EH, second part; ADR-0035 §8-9; brief m10-money §4): a piece of a
//! keeper's notes is worth to a household of the keeper's settlement, who knows its door, the good
//! it is redeemed in at its own cost of the good, times its belief that the keeper pays (the share
//! of the pieces presented there that were paid, which the settlement hears of), less its share of
//! a walk to the door to present a carry of them. It wants notes as much as it wants that good,
//! so one short of grain takes notes for grain in payment. Notes are paid, never offered as a
//! ware. At its weekly review a household holding notes presents, at their keepers' doors, the
//! pieces that bring its food back to what it keeps to its harvest when it is short, and every
//! piece of an issue its settlement heard refused within the month.

use super::custody::{FEE_DAYS, RETAKE_SHARE, WALK_M_S};
use super::*;
use crate::notes::Issue;

/// Pieces of a keeper's notes one presents at a time, over which a walk to its door is spread (a
/// design prior: about what one carries of the grain they are redeemed in).
const NOTE_CARRY: f64 = 50.0;

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

    /// What a piece of issue `c` is worth to household `hh` at its own costs `costs`, in hours,
    /// if it knows the keeper's door (the keeper lives, in its settlement): the good it pays, at
    /// its cost, times its belief that the keeper pays, less its share of a walk to the door and
    /// back. Its keeper values its own pieces at what they would pay.
    pub(crate) fn piece_worth(
        &self,
        hh: &Household,
        c: &Issue,
        costs: &[Option<f64>],
    ) -> Option<f64> {
        let issuer = self.household(c.issuer).filter(|x| !x.members.is_empty())?;
        let cost = costs.get(usize::from(c.good)).copied().flatten()?;
        let kg = f64::from(c.kg);
        if issuer.id == hh.id {
            return (cost > 0.0).then_some(kg * cost);
        }
        if issuer.settlement.is_none() || issuer.settlement != hh.settlement {
            return None;
        }
        let metres =
            f64::from(issuer.home.0 - hh.home.0).hypot(f64::from(issuer.home.1 - hh.home.1));
        let walk_h = 2.0 * metres / WALK_M_S / 3600.0 / NOTE_CARRY;
        let w = kg * cost * Self::note_belief(c) - walk_h;
        (w > 0.0).then_some(w)
    }

    /// The issues of notes household `hh` knows the doors of: those it holds and those of keepers
    /// of its settlement.
    fn known_issues<'a>(&'a self, hh: &'a Household) -> impl Iterator<Item = &'a Issue> + 'a {
        self.notes.issues.iter().filter(move |c| {
            c.count > 1e-9
                && (self.notes.held(hh.id, c.id) > 1e-9
                    || self
                        .household(c.issuer)
                        .is_some_and(|x| x.settlement.is_some() && x.settlement == hh.settlement))
        })
    }

    /// What a piece of notes is worth to household `hh` at its own costs, in hours: of the
    /// issues it holds, the one worth least to it (as it pays with those first); holding none, of
    /// those of its settlement's keepers, the one worth least (as it cannot choose which it is
    /// paid in). None where it knows none worth anything.
    pub(crate) fn note_worth(&self, hh: &Household, costs: &[Option<f64>]) -> Option<f64> {
        let held = self
            .notes
            .purse(hh.id)
            .is_some_and(|p| !p.counts.is_empty());
        self.known_issues(hh)
            .filter(|c| !held || self.notes.held(hh.id, c.id) > 1e-9)
            .map(|c| self.piece_worth(hh, c, costs).unwrap_or(0.0))
            .reduce(f64::min)
            .filter(|&w| w > 0.0)
    }

    /// How much household `hh` wants notes, from how much it wants the goods those it knows are
    /// redeemed in (`want`, by good): as much as the least wanted, as it cannot choose which it is
    /// paid in.
    pub(super) fn note_want(&self, hh: &Household, want: &[f64]) -> f64 {
        self.known_issues(hh)
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
            let at_door = self
                .household(c.issuer)
                .is_some_and(|x| !x.members.is_empty() && x.settlement == hh.settlement);
            if !at_door || c.issuer == hh.id {
                continue;
            }
            // Its settlement heard the keeper refuse lately: it presents them all.
            if c.refused_day >= 0 && day - c.refused_day <= FEE_DAYS {
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
