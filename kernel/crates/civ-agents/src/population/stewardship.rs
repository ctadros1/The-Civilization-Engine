//! The store's keeper (M7c slice BY, ADR-0026 §4; research 09-09 §1.3, §1.8, §5.4): at each of
//! the polity's reviews the keeper of a common store weighs drawing from it for their own
//! household, as a watcher weighs asking a payment (ΔU = B − M − p̂F: what it is worth to their
//! household, against their objection to taking what is not theirs and the chance they believe
//! they run of being found out), up to a share of what the store holds. What they draw moves on
//! the `Diverted` channel and is counted on the store's law. Nobody is told: what follows is the
//! store, emptier, failing those who ask of it, which they already hold against the keeper's
//! office. When the store runs low, the keeper orders the asks by their regard (relief by
//! favour), and one they regard too little is passed over and holds it against them.

use super::*;
use crate::polity::{LawStatus, PolicyKind};

/// Purpose tag for the keeper's choice whether to draw from the store (M7c slice BY).
pub const PURPOSE_DIVERT: u64 = 0x6469_7665_7274_3031; // "divert01"

impl Population {
    /// The keeper of polity `pi`'s common store weighs drawing from it for their own household
    /// (M7c slice BY): not at all above their objection's filter; else up to `divert_share` of
    /// the food it holds, and no more than brings their household to the days of food it tries
    /// to keep, worth `w_demand_loss` a day of that household's food, against `w_objection` for
    /// their objection and `w_seen` for the chance they believe they run of being seen, drawn
    /// against not drawing. Kilograms drawn.
    pub(super) fn keeper_weighs_drawing(&mut self, ctx: &Ctx, pi: usize) -> f64 {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let cp = &params.crime;
        let Some(store_li) = self.polities[pi]
            .laws
            .iter()
            .position(|l| l.status == LawStatus::InForce && l.kind == PolicyKind::CommonStore)
        else {
            return 0.0;
        };
        let Some((keeper, _)) = self.polities[pi].keeper() else {
            return 0.0;
        };
        let Some(p) = self.person(keeper) else {
            return 0.0;
        };
        let (objection, risk, home) = (f64::from(p.objection), f64::from(p.risk_seen), p.household);
        if objection >= cp.objection_filter {
            return 0.0;
        }
        let Some(x) = self.household(home) else {
            return 0.0;
        };
        let need = x.members.len().max(1) as f64 * params.household.daily_kcal_per_person;
        let held = stock_kcal(&stores_now(x, now, params, goods), goods);
        let sheltered = self.polity_sheltered(pi);
        self.polities[pi].settle_stores(now, goods, sheltered);
        let store = stock_kcal(&self.polities[pi].stores, goods);
        let want = (params.household.food_target_days * need - held).max(0.0);
        let kcal = (cp.divert_share * store).min(want);
        if kcal <= 1.0 {
            return 0.0;
        }
        let worth = cp.w_demand_loss * kcal / need;
        let draw = worth - cp.w_objection * objection - cp.w_seen * risk;
        let key = [
            ctx.seed,
            PURPOSE_DIVERT,
            keeper.get(),
            now.day_index() as u64,
        ];
        let u = Rng64::from_key(&key).next_f64();
        if crate::demography::pick_softmax(&[0.0, draw], u) != Some(1) {
            return 0.0;
        }
        let id = self.polities[pi].id;
        let legs: Vec<Leg> = super::crime::take_goods(
            &self.polities[pi].stores,
            goods,
            params.household.carry_kg,
            kcal,
        )
        .into_iter()
        .map(|(good, amount)| Leg {
            from: id,
            to: home,
            good,
            amount,
        })
        .collect();
        let kg: f64 = legs.iter().map(|l| l.amount).sum();
        if kg <= 1e-9 || !self.transfer(now, params, goods, &legs, Channel::Diverted) {
            return 0.0;
        }
        let c = &mut self.polities[pi].laws[store_li].compliance;
        c.diverted += 1;
        c.diverted_kg += kg;
        kg
    }

    /// Whether the keeper of polity `pi`'s store passes over an ask from household `to` (M7c
    /// slice BY: relief by favour): only while the store holds less than `favour_reserve_days` of
    /// its people's food, and only one whose elder the keeper regards less than `favour_floor`
    /// (themselves and their own household never). `Some(true)` passed over, `Some(false)`
    /// answered by favour, `None` when the store is not low or has no keeper.
    pub(crate) fn keeper_favours(&self, ctx: &Ctx, pi: usize, to: PermanentId) -> Option<bool> {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let cp = &params.crime;
        let (keeper, _) = self.polities[pi].keeper()?;
        let people: usize = self
            .households
            .iter()
            .filter(|(_, x)| self.polities[pi].holds(x.settlement))
            .map(|(_, x)| x.members.len())
            .sum();
        let reserve =
            cp.favour_reserve_days * people as f64 * params.household.daily_kcal_per_person;
        if stock_kcal(&self.polities[pi].stores, goods) >= reserve {
            return None;
        }
        if self.person(keeper).map(|p| p.household) == Some(to) {
            return Some(false);
        }
        let elder = self.elder_of(to, now, params)?;
        let regard = self
            .ties
            .regard(keeper, elder, now.day_index(), &params.ties);
        Some(regard < cp.favour_floor)
    }

    /// Household `to`'s grown members who know the store's law `li` hold being passed over
    /// against the keeper's office (M7c slice BY): treatment, as much as the law's relief would
    /// have brought them, days.
    pub(super) fn grieve_passed_over(&mut self, ctx: &Ctx, pi: usize, li: usize, to: PermanentId) {
        let (now, params) = (ctx.now, ctx.params);
        let store_law = self.polities[pi].laws[li].id;
        let harm = f64::from(self.polities[pi].laws[li].relief_days);
        let Some(office) = self.polities[pi]
            .laws
            .iter()
            .find(|l| {
                l.status == LawStatus::InForce
                    && l.kind == PolicyKind::KeepStore
                    && l.holder.is_some()
            })
            .map(|l| l.id)
        else {
            return;
        };
        for m in self.grown_of(to, now, params) {
            if self.polities[pi].laws[li].knows(m) {
                self.grieve(
                    ctx,
                    m,
                    crate::word::Grieved::Treatment,
                    crate::word::Blamed::Office(office),
                    store_law,
                    harm,
                    crate::word::Wrong::PassedOver,
                );
            }
        }
    }

    /// [`Population::keeper_weighs_drawing`] for polity `polity`, now: for tests.
    #[doc(hidden)]
    pub fn keeper_weighs_drawing_for_tests(&mut self, ctx: &Ctx, polity: PermanentId) -> f64 {
        match self.polities.iter().position(|p| p.id == polity) {
            Some(pi) => self.keeper_weighs_drawing(ctx, pi),
            None => 0.0,
        }
    }

    /// Household `to` asks polity `polity`'s store for relief now: for tests.
    #[doc(hidden)]
    pub fn give_relief_for_tests(&mut self, ctx: &Ctx, polity: PermanentId, to: PermanentId) {
        if let Some(pi) = self.polities.iter().position(|p| p.id == polity) {
            self.give_relief(ctx, pi, to);
        }
    }
}
