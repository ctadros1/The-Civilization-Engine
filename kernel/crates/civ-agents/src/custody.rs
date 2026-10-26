//! Goods kept for another (M10d slice EG, second part; ADR-0027 §4; ADR-0035 §9; brief m10-money
//! §3): what a household with no roof of its own leaves with a neighbour who has room is kept for
//! it, a ledger holder of its own, as a load in transit is: the depositor's throughout, never
//! counted as the keeper's, under the keeper's roof and spoiling at its goods' sheltered clocks.
//! Goods enter and leave it on the `Custody` channel, and the keeper's fee is taken from it the
//! same way.

use civ_core::{PermanentId, SimTime};

use crate::params::GoodDef;
use crate::person::{Flow, Flows};

/// Goods one household keeps for another.
#[derive(Clone, Debug, PartialEq)]
pub struct Kept {
    /// Its id, as a ledger holder.
    pub id: PermanentId,
    /// The household whose goods they are, and the one keeping them.
    pub depositor: PermanentId,
    pub keeper: PermanentId,
    /// The day they were left, and the day the keeper's fee was last taken.
    pub since: i64,
    pub fee_day: i64,
    /// What it holds now, by good (catalog index), last brought up to date at `stores_at`.
    pub stores: Vec<f64>,
    pub stores_at: SimTime,
    pub flows: Flows,
    /// The receipt someone of the keeper's household who can write wrote when they were left:
    /// who wrote it and the day (M10d slice EG, third part). Only a record: it never pays.
    pub receipt: Option<(PermanentId, i64)>,
}

impl Kept {
    /// Nothing yet of `depositor`'s kept by `keeper`, from `now`.
    pub fn new(
        id: PermanentId,
        (depositor, keeper): (PermanentId, PermanentId),
        now: SimTime,
        goods: usize,
    ) -> Kept {
        Kept {
            id,
            depositor,
            keeper,
            since: now.day_index(),
            fee_day: now.day_index(),
            stores: vec![0.0; goods],
            stores_at: now,
            flows: Flows::default(),
            receipt: None,
        }
    }

    /// Brings what it holds up to `t`: it keeps as under a roof.
    pub fn settle_stores(&mut self, t: SimTime, goods: &[GoodDef]) {
        if t <= self.stores_at {
            return;
        }
        self.stores.resize(goods.len(), 0.0);
        let days = (t.minutes() - self.stores_at.minutes()) as f64 / 1440.0;
        for (g, (kg, d)) in self.stores.iter_mut().zip(goods).enumerate() {
            let half = if d.sheltered_half_life_days > 0.0 {
                d.sheltered_half_life_days
            } else {
                d.half_life_days
            };
            if half > 0.0 && *kg > 0.0 {
                let after = *kg * 0.5f64.powf(days / half);
                self.flows.add(Flow::Spoiled, g, *kg - after);
                *kg = after;
            }
        }
        self.stores_at = t;
    }

    /// Kilograms it holds.
    pub fn kg(&self) -> f64 {
        self.stores.iter().map(|x| x.max(0.0)).sum()
    }

    /// Whether it holds nothing.
    pub fn is_empty(&self) -> bool {
        self.stores.iter().all(|&x| x <= 1e-9)
    }
}
