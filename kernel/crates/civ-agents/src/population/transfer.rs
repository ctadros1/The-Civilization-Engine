//! The ledger's one operation (ADR-0006 §3): goods move between households in exact quantities,
//! by a channel, all legs or none.

use std::collections::BTreeMap;

use civ_core::{PermanentId, SimTime};

use super::{Population, fuel_per_day};
use crate::ledger::{Channel, Leg};
use crate::params::{GoodDef, PeopleParams};
use crate::person::Flow;

/// How far short of a leg a giver may be and still cover it, in the good's unit: rounding, never
/// a shortfall.
const COVER_SLACK: f64 = 1e-9;

impl Population {
    /// Moves goods between households in one operation (ADR-0006 §3): each leg moves `amount`
    /// of a good from one household to another. Every household touched has its stores brought
    /// up to `now` first; if any cannot cover all it gives, or a leg is malformed, nothing moves
    /// and `false` comes back. Each leg counts as given by one household and received by the
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
        if touched.iter().any(|id| !self.hh_index.contains_key(id)) {
            return false;
        }
        for id in &touched {
            if let Some(x) = self
                .hh_index
                .get(id)
                .and_then(|&hd| self.households.get_mut(hd))
            {
                let members = x.members.len().max(1);
                x.settle_stores(now, goods, &|d| fuel_per_day(params, members, d));
                x.stores.resize(goods.len(), 0.0);
            }
        }
        for (&(from, good), &amount) in &gives {
            let held = self
                .household(from)
                .and_then(|x| x.stores.get(good))
                .copied()
                .unwrap_or(0.0);
            if held + COVER_SLACK < amount {
                return false;
            }
        }
        for l in legs {
            let mut moved = 0.0;
            if let Some(x) = self
                .hh_index
                .get(&l.from)
                .and_then(|&hd| self.households.get_mut(hd))
            {
                moved = l.amount.min(x.stores[l.good].max(0.0));
                x.stores[l.good] -= moved;
                x.flows.add(Flow::Given, l.good, moved);
            }
            if let Some(x) = self
                .hh_index
                .get(&l.to)
                .and_then(|&hd| self.households.get_mut(hd))
            {
                x.stores[l.good] += moved;
                x.flows.add(Flow::Received, l.good, moved);
            }
            self.transfers.add(channel, l.good, moved);
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
                known: Vec::new(),
                sheltered: false,
                flows: Flows::default(),
                offers: Vec::new(),
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
}
