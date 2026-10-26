//! The ledger's one operation (ADR-0006 §3): goods move between households, firms and polities
//! (ADR-0013 §4) in exact quantities, by a channel, all legs or none.

use std::collections::BTreeMap;

use civ_core::{PermanentId, SimTime};

use super::{Population, fuel_per_day};
use crate::ledger::{Channel, Leg};
use crate::params::{GoodDef, PeopleParams};
use crate::person::Flow;

/// How far short of a leg a giver may be and still cover it, in the good's unit: rounding, never
/// a shortfall.
const COVER_SLACK: f64 = 1e-9;
/// And as a share of the leg: amounts posted in terms (an offer keeps its units as `f32`) are
/// good to about one part in ten million of themselves, so a seller offering its whole stock may
/// post a hair more than it holds, and must still be able to sell it. What moves is never more
/// than the giver holds.
const COVER_SHARE: f64 = 1e-6;

/// A holder of goods the ledger moves between: a household, an open firm, a polity's store or a
/// faction's (M4c slice AH), or a payment an agreement owes, holding what was set aside for it
/// (M5c slice AV).
#[derive(Clone, Copy, Debug)]
enum Holder {
    Household(civ_core::Handle<crate::person::Household>),
    Firm(usize),
    Polity(usize),
    Faction(usize),
    Due(usize),
    Consignment(usize),
    Kept(usize),
}

impl Population {
    fn holder(&self, id: PermanentId) -> Option<Holder> {
        if let Some(&hd) = self.hh_index.get(&id) {
            return Some(Holder::Household(hd));
        }
        if let Some(i) = self.firms.iter().position(|f| f.id == id && f.is_open()) {
            return Some(Holder::Firm(i));
        }
        if let Some(i) = self.polities.iter().position(|p| p.id == id) {
            return Some(Holder::Polity(i));
        }
        if let Some(i) = self.factions.list.iter().position(|f| f.id == id) {
            return Some(Holder::Faction(i));
        }
        if let Some(i) = self.agreements.dues.iter().position(|d| d.id == id) {
            return Some(Holder::Due(i));
        }
        if let Some(i) = self.consignments.iter().position(|c| c.id == id) {
            return Some(Holder::Consignment(i));
        }
        self.kept.iter().position(|k| k.id == id).map(Holder::Kept)
    }

    /// The polity whose own issues coin `holder` holds are (M7d slice CE4c): a household's or a
    /// workshop's polity where it lives, or a polity's store itself.
    pub(crate) fn home_issuer(&self, holder: PermanentId) -> Option<PermanentId> {
        if self.polities.iter().any(|p| p.id == holder) {
            return Some(holder);
        }
        let settlement = match self.household(holder) {
            Some(x) => x.settlement,
            None => self
                .firms
                .iter()
                .find(|f| f.id == holder)
                .and_then(|f| self.household(f.owner))
                .and_then(|x| x.settlement),
        }?;
        self.polity_of(settlement).map(|pi| self.polities[pi].id)
    }

    /// Whether someone of household `holder` knows technique `assay` (a coin's assay, M7d slice
    /// CE4a): one who can tell how much metal a piece holds. No household, or no assay, cannot.
    pub(crate) fn assays(&self, holder: PermanentId, assay: Option<usize>) -> bool {
        let Some(t) = assay else {
            return false;
        };
        self.household(holder).is_some_and(|x| {
            x.members
                .iter()
                .any(|&m| self.person(m).is_some_and(|p| p.knows(t)))
        })
    }

    /// Puts every purse right by what its holder holds of each coin good (M7d slice CE): coin
    /// moved by a path other than [`Population::transfer`] (a household dividing its stores, a
    /// store's levy) is found again in whoever holds it, each issue's count conserved.
    pub(super) fn reconcile_coin(&mut self, ctx: &crate::Ctx) {
        let goods = &ctx.catalog.goods;
        let coins: Vec<usize> = (0..goods.len())
            .filter(|&g| goods[g].money.as_ref().is_some_and(|m| m.coin))
            .collect();
        if coins.is_empty() || self.order.coinage.cohorts.is_empty() {
            return;
        }
        let at = |s: &Vec<f64>| {
            coins
                .iter()
                .map(|&g| s.get(g).copied().unwrap_or(0.0))
                .sum::<f64>()
        };
        let mut held: Vec<(PermanentId, f64)> = Vec::new();
        held.extend(self.households.iter().map(|(_, x)| (x.id, at(&x.stores))));
        held.extend(
            self.firms
                .iter()
                .filter(|f| f.is_open())
                .map(|f| (f.id, at(&f.stores))),
        );
        held.extend(self.polities.iter().map(|p| (p.id, at(&p.stores))));
        held.extend(self.factions.list.iter().map(|f| (f.id, at(&f.stores))));
        held.extend(self.agreements.dues.iter().map(|d| (d.id, at(&d.stores))));
        held.extend(self.consignments.iter().map(|c| (c.id, at(&c.stores))));
        held.extend(self.kept.iter().map(|k| (k.id, at(&k.stores))));
        held.retain(|&(_, n)| n > 0.0);
        held.sort_unstable_by_key(|&(id, _)| id);
        let holders: Vec<PermanentId> = held.iter().map(|&(id, _)| id).collect();
        let lookup = |id: PermanentId| {
            held.binary_search_by_key(&id, |&(i, _)| i)
                .map_or(0.0, |i| held[i].1)
        };
        self.order.coinage.reconcile(&holders, &lookup);
    }

    /// Moves `amount` of `good` from one holder to another through the ledger, as a gift: for
    /// tests.
    #[doc(hidden)]
    pub fn transfer_for_tests(
        &mut self,
        ctx: &crate::Ctx,
        from: PermanentId,
        to: PermanentId,
        good: usize,
        amount: f64,
    ) -> bool {
        let legs = [Leg {
            from,
            to,
            good,
            amount,
        }];
        self.transfer(
            ctx.now,
            ctx.params,
            &ctx.catalog.goods,
            &legs,
            Channel::Gift,
        )
    }

    /// Puts every purse right by what its holder holds (see [`Population::reconcile_coin`]): for
    /// tests.
    #[doc(hidden)]
    pub fn reconcile_coin_for_tests(&mut self, ctx: &crate::Ctx) {
        self.reconcile_coin(ctx);
    }

    /// Strikes `count` pieces of coin good `coin` for `issuer` into `holder`'s stores and purse,
    /// `mass_g` grams each at `fineness`: for tests, until a mint strikes them (M7d slice CE).
    #[doc(hidden)]
    #[allow(clippy::too_many_arguments)]
    pub fn strike_for_tests(
        &mut self,
        coin: usize,
        issuer: PermanentId,
        holder: PermanentId,
        day: i64,
        count: f64,
        mass_g: f32,
        fineness: f32,
    ) -> bool {
        self.put_struck(coin, issuer, holder, day, count, mass_g, fineness)
    }

    /// Strikes `count` pieces of coin good `coin` for `issuer` on `day` into `holder`'s stores
    /// and purse, `mass_g` grams each at `fineness` (M7d slice CE). Returns `false`, striking
    /// nothing, when `holder` holds nothing.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn put_struck(
        &mut self,
        coin: usize,
        issuer: PermanentId,
        holder: PermanentId,
        day: i64,
        count: f64,
        mass_g: f32,
        fineness: f32,
    ) -> bool {
        let Some(h) = self.holder(holder) else {
            return false;
        };
        let Some((stores, flows)) = self.holder_goods(h) else {
            return false;
        };
        if stores.len() <= coin {
            stores.resize(coin + 1, 0.0);
        }
        stores[coin] += count;
        flows.add(Flow::Made, coin, count);
        self.order
            .coinage
            .strike(issuer, day, mass_g, fineness, count, holder);
        true
    }

    /// Brings a holder's stores up to `now`: a household's spoil and burn, a firm's spoil (under
    /// its owners' roof when they have one), a polity's spoil (under no roof).
    fn settle_holder(&mut self, h: Holder, now: SimTime, params: &PeopleParams, goods: &[GoodDef]) {
        match h {
            Holder::Household(hd) => {
                if let Some(x) = self.households.get_mut(hd) {
                    let members = x.members.len().max(1);
                    x.settle_stores(now, goods, &|d| fuel_per_day(params, members, d));
                    x.stores.resize(goods.len(), 0.0);
                }
            }
            Holder::Firm(i) => {
                let sheltered = self
                    .firms
                    .get(i)
                    .and_then(|f| self.household(f.owner))
                    .is_some_and(|x| x.sheltered);
                if let Some(f) = self.firms.get_mut(i) {
                    f.settle_stores(now, goods, sheltered);
                    f.stores.resize(goods.len(), 0.0);
                }
            }
            Holder::Polity(i) => {
                let sheltered = self.polity_sheltered(i);
                if let Some(p) = self.polities.get_mut(i) {
                    p.settle_stores(now, goods, sheltered);
                    p.stores.resize(goods.len(), 0.0);
                }
            }
            Holder::Faction(i) => {
                let sheltered = self.faction_sheltered(i);
                if let Some(f) = self.factions.list.get_mut(i) {
                    f.settle_stores(now, goods, sheltered);
                    f.stores.resize(goods.len(), 0.0);
                }
            }
            Holder::Due(i) => {
                if let Some(d) = self.agreements.dues.get_mut(i) {
                    d.settle_stores(now, goods);
                    d.stores.resize(goods.len(), 0.0);
                }
            }
            Holder::Consignment(i) => {
                if let Some(c) = self.consignments.get_mut(i) {
                    c.settle_stores(now, goods);
                    c.stores.resize(goods.len(), 0.0);
                }
            }
            Holder::Kept(i) => {
                if let Some(k) = self.kept.get_mut(i) {
                    k.settle_stores(now, goods);
                    k.stores.resize(goods.len(), 0.0);
                }
            }
        }
    }

    fn holder_goods(&mut self, h: Holder) -> Option<(&mut Vec<f64>, &mut crate::person::Flows)> {
        match h {
            Holder::Household(hd) => self
                .households
                .get_mut(hd)
                .map(|x| (&mut x.stores, &mut x.flows)),
            Holder::Firm(i) => self.firms.get_mut(i).map(|f| (&mut f.stores, &mut f.flows)),
            Holder::Polity(i) => self
                .polities
                .get_mut(i)
                .map(|p| (&mut p.stores, &mut p.flows)),
            Holder::Faction(i) => self
                .factions
                .list
                .get_mut(i)
                .map(|f| (&mut f.stores, &mut f.flows)),
            Holder::Due(i) => self
                .agreements
                .dues
                .get_mut(i)
                .map(|d| (&mut d.stores, &mut d.flows)),
            Holder::Consignment(i) => self
                .consignments
                .get_mut(i)
                .map(|c| (&mut c.stores, &mut c.flows)),
            Holder::Kept(i) => self.kept.get_mut(i).map(|k| (&mut k.stores, &mut k.flows)),
        }
    }

    /// Moves goods between holders in one operation (ADR-0006 §3): each leg moves `amount` of a
    /// good from one household, open firm or polity to another. Every holder touched has its stores
    /// brought up to `now` first; if any cannot cover all it gives, or a leg is malformed, nothing
    /// moves and `false` comes back. Each leg counts as given by one holder and received by the
    /// other, so goods are conserved, and as moved by `channel`.
    pub(crate) fn transfer(
        &mut self,
        now: SimTime,
        params: &PeopleParams,
        goods: &[GoodDef],
        legs: &[Leg],
        channel: Channel,
    ) -> bool {
        let mut gives: BTreeMap<(PermanentId, usize), f64> = BTreeMap::new();
        for l in legs {
            if l.from == l.to || l.good >= goods.len() || !(l.amount.is_finite() && l.amount >= 0.0)
            {
                return false;
            }
            *gives.entry((l.from, l.good)).or_default() += l.amount;
        }
        let mut touched: Vec<PermanentId> = legs.iter().flat_map(|l| [l.from, l.to]).collect();
        touched.sort_unstable();
        touched.dedup();
        let mut holders = Vec::with_capacity(touched.len());
        for id in &touched {
            match self.holder(*id) {
                Some(h) => holders.push((*id, h)),
                None => return false,
            }
        }
        for &(_, h) in &holders {
            self.settle_holder(h, now, params, goods);
        }
        let holder_of = |id: PermanentId| holders.iter().find(|(i, _)| *i == id).map(|(_, h)| *h);
        for (&(from, good), &amount) in &gives {
            let held = holder_of(from)
                .and_then(|h| match h {
                    Holder::Household(hd) => self.households.get(hd).map(|x| &x.stores),
                    Holder::Firm(i) => self.firms.get(i).map(|f| &f.stores),
                    Holder::Polity(i) => self.polities.get(i).map(|p| &p.stores),
                    Holder::Faction(i) => self.factions.list.get(i).map(|f| &f.stores),
                    Holder::Due(i) => self.agreements.dues.get(i).map(|d| &d.stores),
                    Holder::Consignment(i) => self.consignments.get(i).map(|c| &c.stores),
                    Holder::Kept(i) => self.kept.get(i).map(|k| &k.stores),
                })
                .and_then(|s| s.get(good))
                .copied()
                .unwrap_or(0.0);
            if held + COVER_SLACK + amount * COVER_SHARE < amount {
                return false;
            }
        }
        for l in legs {
            let mut moved = 0.0;
            if let Some((stores, flows)) = holder_of(l.from).and_then(|h| self.holder_goods(h)) {
                moved = l.amount.min(stores[l.good].max(0.0));
                stores[l.good] -= moved;
                flows.add(Flow::Given, l.good, moved);
            }
            if let Some((stores, flows)) = holder_of(l.to).and_then(|h| self.holder_goods(h)) {
                stores[l.good] += moved;
                flows.add(Flow::Received, l.good, moved);
            }
            self.transfers.add(channel, l.good, moved);
            // Coin moves issue by issue, the payer giving the oldest first (M7d slice CE).
            if let Some(m) = goods[l.good].money.as_ref().filter(|m| m.coin) {
                // A payer who can assay gives the pieces holding least metal first (M7d slice
                // CE4a); others give another polity's pieces first, then count them all alike, the
                // oldest first (M7d slice CE4c).
                let home = self.home_issuer(l.from);
                let assays = self.assays(l.from, m.assay);
                let metal: Vec<(u32, f64)> = self
                    .order
                    .coinage
                    .cohorts
                    .iter()
                    .map(|c| {
                        let w = if assays {
                            c.metal_kg()
                        } else if Some(c.issuer) == home {
                            1.0
                        } else {
                            0.0
                        };
                        (c.id, w)
                    })
                    .collect();
                let worth = |issue: u32| {
                    metal
                        .binary_search_by_key(&issue, |x| x.0)
                        .map_or(0.0, |i| metal[i].1)
                };
                self.order.coinage.give(l.from, l.to, moved, &worth);
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::{Eaten, GoodUse};
    use crate::person::{Flows, Household};

    fn goods() -> Vec<GoodDef> {
        let good = |id: &str, purpose| GoodDef {
            fuel: None,
            power: None,
            light: None,
            holder: None,
            money: None,
            id: id.into(),
            name: id.into(),
            purpose,
            kcal_per_kg: if purpose == GoodUse::Food {
                3000.0
            } else {
                0.0
            },
            half_life_days: 0.0,
            eaten: Eaten::Never,
            shared: false,
            reserve_for: None,
            tool: None,
            sheltered_half_life_days: 0.0,
            timber: None,
            store: None,
            fire: None,
        };
        vec![
            good("grain", GoodUse::Food),
            good("stone", GoodUse::Material),
        ]
    }

    fn pid(id: u64) -> PermanentId {
        PermanentId::from_raw(id).expect("non-zero")
    }

    fn with(stores: &[(u64, [f64; 2])]) -> Population {
        let mut pop = Population::new();
        for &(id, s) in stores {
            pop.insert_household(Household {
                id: pid(id),
                members: Vec::new(),
                home: (0.0, 0.0),
                settlement: None,
                stores: s.to_vec(),
                stores_at: SimTime::ZERO,
                water_l: 0.0,
                water_at: SimTime::ZERO,
                water_use_l: 0.0,
                water_walk_min: 0.0,
                known: Vec::new(),
                sheltered: false,
                keeping: crate::person::Keeping::default(),
                flows: Flows::default(),
                offers: Vec::new(),
                taste: Default::default(),
                admired: None,
                midden: Default::default(),
                flood_memory: Default::default(),
                flood_losses: Vec::new(),
                ditch_worth: 0.0,
                fire_losses: Vec::new(),
                gave_way: Vec::new(),
                batches: Vec::new(),
                mill_seen: None,
            });
        }
        pop
    }

    fn held(pop: &Population, id: u64) -> Vec<f64> {
        pop.household(pid(id)).expect("household").stores.clone()
    }

    #[test]
    fn a_transfer_moves_every_leg_or_none() {
        let params = crate::found::tests::params();
        let goods = goods();
        let mut pop = with(&[(1, [10.0, 5.0]), (2, [0.0, 0.0])]);
        let (a, b) = (pid(1), pid(2));
        let leg = |from, to, good, amount| Leg {
            from,
            to,
            good,
            amount,
        };
        // A barter: grain one way, stone the other, which the second household lacks.
        let barter = [leg(a, b, 0, 4.0), leg(b, a, 1, 1.0)];
        assert!(!pop.transfer(SimTime::ZERO, &params, &goods, &barter, Channel::Barter));
        assert_eq!(held(&pop, 1), vec![10.0, 5.0], "nothing moved");
        assert_eq!(held(&pop, 2), vec![0.0, 0.0]);
        // A gift it can cover moves exactly.
        assert!(pop.transfer(
            SimTime::ZERO,
            &params,
            &goods,
            &[leg(a, b, 0, 4.0), leg(a, b, 1, 2.0)],
            Channel::Gift
        ));
        assert_eq!(held(&pop, 1), vec![6.0, 3.0]);
        assert_eq!(held(&pop, 2), vec![4.0, 2.0]);
        // Goods are conserved: given and received cancel over the households.
        let flows = pop.flows();
        assert_eq!(flows.get(Flow::Given, 0), 4.0);
        assert_eq!(flows.get(Flow::Received, 0), 4.0);
        assert_eq!(flows.net(0), 0.0);
        assert_eq!(pop.transfers.get(Channel::Gift, 1), 2.0);
        // Malformed legs are refused: to oneself, a negative amount, an unknown household or
        // good.
        for bad in [
            leg(a, a, 0, 1.0),
            leg(a, b, 0, -1.0),
            leg(a, pid(9), 0, 1.0),
            leg(a, b, 7, 1.0),
        ] {
            assert!(!pop.transfer(SimTime::ZERO, &params, &goods, &[bad], Channel::Gift));
        }
        assert_eq!(held(&pop, 1), vec![6.0, 3.0]);
    }

    #[test]
    fn a_holder_can_give_all_it_holds_as_terms_rounded_to_f32_put_it() {
        let params = crate::found::tests::params();
        let goods = goods();
        // A workshop's whole stock, offered as an `f32`, reads a hair more than it holds.
        let stock = 0.9644461699345266;
        let posted = f64::from(stock as f32);
        assert!(posted > stock);
        let mut pop = with(&[(1, [stock, 0.0]), (2, [0.0, 0.0])]);
        let leg = |amount| Leg {
            from: pid(1),
            to: pid(2),
            good: 0,
            amount,
        };
        assert!(pop.transfer(
            SimTime::ZERO,
            &params,
            &goods,
            &[leg(posted)],
            Channel::Sale
        ));
        // What moves is what it held, no more: goods are conserved.
        assert_eq!(held(&pop, 1)[0], 0.0);
        assert_eq!(held(&pop, 2)[0], stock);
        assert_eq!(pop.flows().net(0), 0.0);
        // A real shortfall is still refused.
        let mut pop = with(&[(1, [0.96, 0.0]), (2, [0.0, 0.0])]);
        assert!(!pop.transfer(SimTime::ZERO, &params, &goods, &[leg(0.97)], Channel::Sale));
        assert_eq!(held(&pop, 1)[0], 0.96);
    }
}
