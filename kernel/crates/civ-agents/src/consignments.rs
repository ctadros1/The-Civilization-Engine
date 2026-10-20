//! Goods in transit (M8b slice CO; ADR-0029 §3, amending ADR-0019 §3 for trips by boat): what a
//! household bought on an errand by boat is not in its store until it is unloaded at home. On the
//! way it is a consignment, a ledger holder of its own: owned by the household throughout,
//! carried by one person in one boat from one settlement to another, spoiling in the open by its
//! goods' own clocks. If the boat cannot bring it home it stays, still owned, where it was left.

use civ_core::{PermanentId, SimTime};

use crate::params::GoodDef;
use crate::person::{Flow, Flows};

/// One load in transit.
#[derive(Clone, Debug, PartialEq)]
pub struct Consignment {
    /// Its id, as a ledger holder.
    pub id: PermanentId,
    /// The household it belongs to.
    pub owner: PermanentId,
    /// Who carries it, and in which boat.
    pub carrier: PermanentId,
    pub boat: PermanentId,
    /// The settlement it was bought in and the one it is bound for.
    pub from: PermanentId,
    pub to: PermanentId,
    /// When it was loaded.
    pub made: SimTime,
    /// What it holds now, by good (catalog index), last brought up to date at `stores_at`.
    pub stores: Vec<f64>,
    pub stores_at: SimTime,
    pub flows: Flows,
}

impl Consignment {
    /// An empty consignment of `owner`'s, carried by `carrier` in `boat` from `from` to `to`,
    /// loaded at `now`.
    pub fn new(
        id: PermanentId,
        (owner, carrier, boat): (PermanentId, PermanentId, PermanentId),
        (from, to): (PermanentId, PermanentId),
        now: SimTime,
        goods: usize,
    ) -> Consignment {
        Consignment {
            id,
            owner,
            carrier,
            boat,
            from,
            to,
            made: now,
            stores: vec![0.0; goods],
            stores_at: now,
            flows: Flows::default(),
        }
    }

    /// Brings what it holds up to `t`: it spoils in the open.
    pub fn settle_stores(&mut self, t: SimTime, goods: &[GoodDef]) {
        if t <= self.stores_at {
            return;
        }
        self.stores.resize(goods.len(), 0.0);
        let days = (t.minutes() - self.stores_at.minutes()) as f64 / 1440.0;
        for (g, (kg, d)) in self.stores.iter_mut().zip(goods).enumerate() {
            if d.half_life_days > 0.0 && *kg > 0.0 {
                let after = *kg * 0.5f64.powf(days / d.half_life_days);
                self.flows.add(Flow::Spoiled, g, *kg - after);
                *kg = after;
            }
        }
        self.stores_at = t;
    }

    /// Whether it holds nothing.
    pub fn is_empty(&self) -> bool {
        self.stores.iter().all(|&x| x <= 1e-9)
    }
}
