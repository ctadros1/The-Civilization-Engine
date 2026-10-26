//! A money's worth to each household, kept in a table at midnight (M10d slice EF, first part;
//! ADR-0035 §8, §12; brief m10-money §2, §10; research 08-06 §1.3, §2.3): what a unit of each
//! good paid as money bought it in the purchases its markets remember, and how much it paid in
//! it, faded by their memory, worked out once at midnight rather than scanned in every decision;
//! and whether others take it, from payments taken and sellers of what it wanted who did not take
//! it, fading as the markets forget. Only goods with a `[money]` table have one: coin and notes,
//! not grain or firewood (plan §9's M10 design). A unit's onward worth is what it bought, at the
//! share of payments in it others took.

use super::market::{Holding, food_short};
use super::*;
use crate::market::{MoneyBelief, Offer};

/// Tallies faded below this are forgotten.
const FORGOTTEN: f32 = 0.01;

impl Population {
    /// What household `household` believes of money `good`, if anything.
    pub(crate) fn money_belief(&self, household: PermanentId, good: usize) -> Option<&MoneyBelief> {
        let good = u16::try_from(good).ok()?;
        self.money
            .binary_search_by(|b| (b.household, b.good).cmp(&(household, good)))
            .ok()
            .map(|i| &self.money[i])
    }

    /// The table afresh at midnight: each household's purchases in a money its markets remember,
    /// valued at its own costs now, and its tallies faded to today.
    pub(super) fn money_day(&mut self, ctx: &Ctx) {
        let goods = &ctx.catalog.goods;
        if self.money.is_empty() && goods.iter().all(|g| g.money.is_none()) {
            return;
        }
        let mp = &ctx.params.market;
        let today = ctx.now.day_index();
        let memory = mp.memory_days.max(1e-6);
        let is_money = |p: u16| goods.get(usize::from(p)).is_some_and(|g| g.money.is_some());
        // Each purchase in a money, by buyer and money.
        let mut paid: BTreeMap<(PermanentId, u16), Vec<&crate::ledger::Trade>> = BTreeMap::new();
        for t in self
            .markets
            .iter()
            .flat_map(|m| m.recent.iter())
            .filter(|t| t.paid > 0.0 && is_money(t.payment))
        {
            paid.entry((t.buyer, t.payment)).or_default().push(t);
        }
        let mut fresh: Vec<MoneyBelief> = Vec::new();
        let mut costs: Option<(PermanentId, Vec<Option<f64>>)> = None;
        for ((h, p), trades) in paid {
            let Some(hh) = self.household(h) else {
                continue;
            };
            if costs.as_ref().is_none_or(|c| c.0 != h) {
                costs = Some((h, self.own_costs_of(ctx, hh)));
            }
            let Some((_, own)) = costs.as_ref() else {
                continue;
            };
            let (mut bought_h, mut units) = (0.0, 0.0);
            for t in trades {
                let Some(c) = own.get(usize::from(t.good)).copied().flatten() else {
                    continue;
                };
                let age = (today - t.at.day_index()).max(0) as f64;
                let w = 0.5f64.powf(age / memory);
                bought_h += w * c * f64::from(t.units);
                units += w * f64::from(t.paid);
            }
            fresh.push(MoneyBelief {
                household: h,
                good: p,
                bought_h: if units > 1e-9 { bought_h / units } else { 0.0 },
                paid: units,
                taken: 0.0,
                refused: 0.0,
                day: today,
            });
        }
        // The tallies carried over, faded to today; a household no longer here is forgotten.
        let old = std::mem::take(&mut self.money);
        for mut b in old {
            if self.household(b.household).is_none() {
                continue;
            }
            let fade = 0.5f64.powf((today - b.day).max(0) as f64 / memory) as f32;
            let (taken, refused) = (b.taken * fade, b.refused * fade);
            match fresh.binary_search_by(|f| (f.household, f.good).cmp(&(b.household, b.good))) {
                Ok(i) => {
                    fresh[i].taken = taken;
                    fresh[i].refused = refused;
                }
                Err(i) if taken + refused >= FORGOTTEN => {
                    b.bought_h = 0.0;
                    b.paid = 0.0;
                    b.taken = taken;
                    b.refused = refused;
                    b.day = today;
                    fresh.insert(i, b);
                }
                Err(_) => {}
            }
        }
        self.money = fresh;
    }

    /// Household `household` saw `taken` payments in money `good` taken and `refused` refused
    /// today.
    pub(super) fn note_money(
        &mut self,
        ctx: &Ctx,
        household: PermanentId,
        good: usize,
        taken: f32,
        refused: f32,
    ) {
        let Ok(g) = u16::try_from(good) else {
            return;
        };
        let today = ctx.now.day_index();
        let memory = ctx.params.market.memory_days.max(1e-6);
        match self
            .money
            .binary_search_by(|b| (b.household, b.good).cmp(&(household, g)))
        {
            Ok(i) => {
                let b = &mut self.money[i];
                let fade = 0.5f64.powf((today - b.day).max(0) as f64 / memory) as f32;
                b.taken = b.taken * fade + taken;
                b.refused = b.refused * fade + refused;
                b.day = today;
            }
            Err(i) => self.money.insert(
                i,
                MoneyBelief {
                    household,
                    good: g,
                    bought_h: 0.0,
                    paid: 0.0,
                    taken,
                    refused,
                    day: today,
                },
            ),
        }
    }

    /// What household `hh` with `stores` keeps of money `g` (M10d slice EF, second part; ADR-0035
    /// §8; research 08-06 §1.4, §2.3): its balance, `money_days` of its spending in it as its
    /// purchases show; and, where the money is about (it holds some, its market has seen payments
    /// in it, or its polity's store holds some), the dues it expects to pay in it: what a common
    /// store it knows of could levy in it, its share of the grain it holds at the template's rate,
    /// as [`Population::levy_floor`] counts it. Its want then follows, so a household that owes in
    /// a money takes it in payment and sells for it.
    pub(super) fn money_keep(&self, ctx: &Ctx, hh: &Household, g: usize, stores: &[f64]) -> f64 {
        let mp = &ctx.params.market;
        let balance = self.money_belief(hh.id, g).map_or(0.0, |b| {
            mp.money_days * b.paid * std::f64::consts::LN_2 / mp.memory_days.max(1e-6)
        });
        balance + self.dues_in(ctx, hh, g, stores)
    }

    /// The dues household `hh` with `stores` expects to pay in money `g` ([`Self::money_keep`]).
    fn dues_in(&self, ctx: &Ctx, hh: &Household, g: usize, stores: &[f64]) -> f64 {
        let Some(settlement) = hh.settlement else {
            return 0.0;
        };
        let Some(pi) = self.polity_of(settlement) else {
            return 0.0;
        };
        let polity = &self.polities[pi];
        let taken = |l: &crate::polity::Law| {
            l.status == crate::polity::LawStatus::InForce
                && l.kind == crate::polity::PolicyKind::CommonStore
                && l.levy_share > 0.0
        };
        if !polity.laws.iter().any(taken) {
            return 0.0;
        }
        let held = |s: &[f64]| s.get(g).copied().unwrap_or(0.0) > 1e-9;
        let about = held(stores)
            || held(&polity.stores)
            || self
                .market(settlement)
                .is_some_and(|m| m.paid_h.get(g).copied().unwrap_or(0.0) > 0.0);
        if !about {
            return 0.0;
        }
        let Some(grain) = ctx.catalog.crops.get(ctx.params.farm.crop).map(|c| c.good) else {
            return 0.0;
        };
        let grain_held = stores.get(grain).copied().unwrap_or(0.0).max(0.0);
        polity
            .laws
            .iter()
            .filter(|l| taken(l) && hh.members.iter().any(|&m| l.knows(m)))
            .filter_map(|l| {
                let m = ctx.catalog.policies.get(usize::from(l.policy))?.medium?;
                (m.good == g && m.per_kg > 0.0)
                    .then(|| f64::from(l.levy_share) * grain_held * f64::from(m.per_kg))
            })
            .fold(0.0, f64::max)
    }

    /// The terms household `household` would post now for `offered` (good, units), as at its
    /// review with no terms before: for tests.
    #[doc(hidden)]
    pub fn terms_for_tests(
        &self,
        ctx: &Ctx,
        household: PermanentId,
        offered: &[(usize, f64)],
    ) -> Vec<Offer> {
        let Some(hh) = self.household(household) else {
            return Vec::new();
        };
        let Some(settlement) = hh.settlement else {
            return Vec::new();
        };
        let stores = stores_now(hh, ctx.now, ctx.params, &ctx.catalog.goods);
        let own = self.own_costs_of(ctx, hh);
        let costs = self.pay_costs(ctx, hh, &stores, own);
        let holding = self.holding_of(ctx, hh, &stores);
        self.post_terms(ctx, settlement, &costs, &costs, &holding, offered, &[])
    }

    /// The monies a household with `stores` holds beyond what it keeps (`holding`), and, if any,
    /// the goods it wants at its neighbours' doors: the tools that serve for one it needs, and
    /// food when it is short of it.
    pub(super) fn money_offered(
        goods: &[GoodDef],
        stores: &[f64],
        holding: &Holding,
    ) -> (Vec<usize>, Vec<usize>) {
        let held = |g: usize| stores.get(g).copied().unwrap_or(0.0).max(0.0);
        let keep = |g: usize| holding.keep.get(g).copied().unwrap_or(0.0);
        let spare: Vec<usize> = goods
            .iter()
            .enumerate()
            .filter(|&(g, d)| d.money.is_some() && held(g) - keep(g) > 1e-9)
            .map(|(g, _)| g)
            .collect();
        let mut wanted: Vec<usize> = Vec::new();
        if spare.is_empty() {
            return (spare, wanted);
        }
        for (t, d) in goods.iter().enumerate() {
            if d.tool.as_ref().is_none_or(|x| x.serves.is_some()) {
                continue;
            }
            let wants = (keep(t) - make::SPARE_TOOL).max(0.0);
            if make::tool_need(wants, make::serving(goods, t).map(held).sum()) > 0.0 {
                wanted.extend(make::serving(goods, t));
            }
        }
        if food_short(goods, &holding.keep, stores).0 > 0.0 {
            wanted.extend(goods.iter().enumerate().filter_map(|(g, d)| {
                (d.purpose == GoodUse::Food && d.kcal_per_kg > 0.0).then_some(g)
            }));
        }
        (spare, wanted)
    }

    /// Household `household` at its review counts the sellers of its settlement taking and
    /// refusing the monies it holds beyond what it keeps, as `review_offers` does: for tests.
    #[doc(hidden)]
    pub fn money_review_for_tests(&mut self, ctx: &Ctx, household: PermanentId) {
        let Some(hh) = self.household(household) else {
            return;
        };
        let Some(settlement) = hh.settlement else {
            return;
        };
        let stores = stores_now(hh, ctx.now, ctx.params, &ctx.catalog.goods);
        let holding = self.holding_of(ctx, hh, &stores);
        let (spare, wanted) = Self::money_offered(&ctx.catalog.goods, &stores, &holding);
        self.money_refusals(ctx, household, settlement, &spare, &wanted);
    }

    /// At its review, household `hh` holding a money beyond what it keeps (`spare`, by good)
    /// counts, for each, the sellers of its settlement offering what it wants (`wanted`, by
    /// good): one whose terms take the money took it, one whose do not refused it.
    pub(super) fn money_refusals(
        &mut self,
        ctx: &Ctx,
        hh: PermanentId,
        settlement: PermanentId,
        spare: &[usize],
        wanted: &[usize],
    ) {
        if spare.is_empty() || wanted.is_empty() {
            return;
        }
        let rows: Vec<&[Offer]> = self
            .households
            .iter()
            .map(|(_, x)| x)
            .filter(|x| x.settlement == Some(settlement) && x.id != hh && !x.offers.is_empty())
            .map(|x| x.offers.as_slice())
            .chain(
                self.firms
                    .iter()
                    .filter(|f| {
                        f.is_open()
                            && f.settlement == Some(settlement)
                            && f.owner != hh
                            && !f.offers.is_empty()
                    })
                    .map(|f| f.offers.as_slice()),
            )
            .collect();
        let mut counts: Vec<(usize, f32, f32)> = Vec::new();
        for &p in spare {
            let (mut taken, mut refused) = (0.0, 0.0);
            for offers in &rows {
                let sells: Vec<&Offer> = offers
                    .iter()
                    .filter(|o| wanted.contains(&usize::from(o.good)))
                    .collect();
                if sells.is_empty() {
                    continue;
                }
                if sells.iter().any(|o| usize::from(o.payment) == p) {
                    taken += 1.0;
                } else {
                    refused += 1.0;
                }
            }
            counts.push((p, taken, refused));
        }
        for (p, taken, refused) in counts {
            if taken + refused > 0.0 {
                self.note_money(ctx, hh, p, taken, refused);
            }
        }
    }

    /// The table afresh now, as at midnight: for tests that set purchases and read worth the
    /// same day.
    #[doc(hidden)]
    pub fn money_day_for_tests(&mut self, ctx: &Ctx) {
        self.money_day(ctx);
    }

    /// What household `household` believes of money `good`: for tests.
    #[doc(hidden)]
    pub fn money_belief_for_tests(
        &self,
        household: PermanentId,
        good: usize,
    ) -> Option<MoneyBelief> {
        self.money_belief(household, good).copied()
    }
}
