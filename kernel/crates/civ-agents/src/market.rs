//! A settlement's market (slice I, ADR-0006 §4): what its households offer and on what terms,
//! what was sold and what it was paid in, which good its payments settle in (its money, if it
//! has one), and what buyers wanted and found no offer for. Tallies fade with a half-life, so a
//! market remembers recent weeks (research 08-04 §1.2: demand smoothed over a memory time in
//! days, measured apart from sales).

use std::collections::VecDeque;

use civ_core::PermanentId;

use crate::ledger::Trade;

/// What a household offers, and on what terms (ADR-0006 §4): one row per good it sells and good
/// it accepts in payment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Offer {
    /// The good offered, by index in the catalog's goods.
    pub good: u16,
    /// What it accepts in payment.
    pub payment: u16,
    /// Units of the payment asked for a unit of the good.
    pub price: f32,
    /// Units of the good offered.
    pub units: f32,
    /// What a unit is worth to the seller behind these terms, hours of its own work: its cost
    /// and margin, moved by what sold.
    pub ask_h: f32,
}

/// A settlement's market.
#[derive(Clone, Debug, PartialEq)]
pub struct Market {
    /// The settlement.
    pub settlement: PermanentId,
    /// The day the tallies were last brought up to date.
    pub day: i64,
    /// Per good: what the payments settled in it were worth to those paid, hours.
    pub paid_h: Vec<f64>,
    /// Per good: units sold.
    pub sold: Vec<f64>,
    /// Per good: units buyers wanted and found no offer for (recorded demand, ADR-0006 §4).
    pub unmet: Vec<f64>,
    /// Per good: what those buyers would have given for them, hours of their own work.
    pub unmet_h: Vec<f64>,
    /// Trades remembered, fading like the tallies.
    pub trades: f64,
    /// Per good: the last trade's terms, `(payment good, paid per unit)`.
    pub last: Vec<Option<(u16, f32)>>,
    /// The latest trades, oldest first.
    pub recent: VecDeque<Trade>,
}

impl Market {
    /// An empty market for `settlement`, on `day`.
    pub fn new(settlement: PermanentId, goods: usize, day: i64) -> Market {
        Market {
            settlement,
            day,
            paid_h: vec![0.0; goods],
            sold: vec![0.0; goods],
            unmet: vec![0.0; goods],
            unmet_h: vec![0.0; goods],
            trades: 0.0,
            last: vec![None; goods],
            recent: VecDeque::new(),
        }
    }

    fn fit(&mut self, goods: usize) {
        if self.paid_h.len() < goods {
            self.paid_h.resize(goods, 0.0);
            self.sold.resize(goods, 0.0);
            self.unmet.resize(goods, 0.0);
            self.unmet_h.resize(goods, 0.0);
            self.last.resize(goods, None);
        }
    }

    /// Brings the tallies to `day`: what is remembered halves every `memory_days`.
    pub fn age_to(&mut self, day: i64, memory_days: f64) {
        if day <= self.day {
            return;
        }
        let keep = 0.5f64.powf((day - self.day) as f64 / memory_days.max(1e-6));
        for v in [
            &mut self.paid_h,
            &mut self.sold,
            &mut self.unmet,
            &mut self.unmet_h,
        ] {
            for x in v.iter_mut() {
                *x *= keep;
            }
        }
        self.trades *= keep;
        self.day = day;
    }

    /// Records a trade: the units sold, the payment and what it was worth to the seller, hours,
    /// the terms paid, and the demand it met. The list of the latest keeps `keep` trades.
    pub fn record_trade(&mut self, trade: Trade, worth_h: f64, keep: usize) {
        let (g, p) = (usize::from(trade.good), usize::from(trade.payment));
        self.fit(g.max(p) + 1);
        let units = f64::from(trade.units).max(0.0);
        self.sold[g] += units;
        self.paid_h[p] += worth_h.max(0.0);
        self.trades += 1.0;
        if units > 0.0 {
            self.last[g] = Some((trade.payment, trade.paid / trade.units));
        }
        // What was bought no longer waits for a seller.
        if self.unmet[g] > 0.0 {
            let left = (self.unmet[g] - units).max(0.0);
            self.unmet_h[g] *= left / self.unmet[g];
            self.unmet[g] = left;
        }
        self.recent.push_back(trade);
        while self.recent.len() > keep {
            self.recent.pop_front();
        }
    }

    /// Records `units` of `good` a buyer wanted and found no offer for, which it would have
    /// given `worth_h` hours of its own work for.
    pub fn record_unmet(&mut self, good: usize, units: f64, worth_h: f64) {
        if !(units > 0.0 && worth_h.is_finite()) {
            return;
        }
        self.fit(good + 1);
        self.unmet[good] += units;
        self.unmet_h[good] += worth_h.max(0.0);
    }

    /// What buyers who found no offer would have given for a unit of `good`, hours.
    pub fn unmet_worth(&self, good: usize) -> Option<f64> {
        let units = self.unmet.get(good).copied().unwrap_or(0.0);
        (units > 1e-6).then(|| self.unmet_h[good] / units)
    }

    /// Each good's share of the worth of the payments remembered (acceptance, ADR-0006 §4).
    pub fn acceptance(&self) -> Vec<f64> {
        let total: f64 = self.paid_h.iter().sum();
        self.paid_h
            .iter()
            .map(|&h| if total > 0.0 { h / total } else { 0.0 })
            .collect()
    }

    /// The settlement's money: the good that settles the largest share of its payments, once
    /// that share is at least `share` over at least `min_trades` trades remembered. Nothing in
    /// the engine names a money good (ADR-0006 §4); a settlement may never have one.
    pub fn money(&self, share: f64, min_trades: f64) -> Option<usize> {
        if self.trades < min_trades {
            return None;
        }
        let shares = self.acceptance();
        let (good, best) = shares
            .iter()
            .copied()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(&b.1).then(b.0.cmp(&a.0)))?;
        (best >= share).then_some(good)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::Channel;
    use civ_core::SimTime;

    fn id(n: u64) -> PermanentId {
        PermanentId::from_raw(n).expect("non-zero")
    }

    fn trade(good: u16, units: f32, payment: u16, paid: f32) -> Trade {
        Trade {
            at: SimTime::ZERO,
            seller: id(1),
            buyer: id(2),
            good,
            units,
            payment,
            paid,
            channel: Channel::Barter,
        }
    }

    #[test]
    fn a_good_becomes_money_only_once_it_settles_most_payments() {
        let mut m = Market::new(id(9), 3, 0);
        // Sickles (2) paid in grain (0), once in stone (1).
        m.record_trade(trade(2, 1.0, 0, 2.0), 3.0, 4);
        assert_eq!(m.money(0.5, 3.0), None, "one trade is no convention");
        m.record_trade(trade(2, 1.0, 0, 2.0), 3.0, 4);
        m.record_trade(trade(2, 1.0, 1, 5.0), 3.0, 4);
        assert_eq!(m.money(0.5, 3.0), Some(0));
        assert_eq!(m.money(0.7, 3.0), None, "two thirds is not enough here");
        assert_eq!(m.last[2], Some((1, 5.0)));
        // Payments in stone take over.
        for _ in 0..5 {
            m.record_trade(trade(2, 1.0, 1, 5.0), 3.0, 4);
        }
        assert_eq!(m.money(0.5, 3.0), Some(1));
        assert_eq!(m.recent.len(), 4, "the latest trades only");
    }

    #[test]
    fn tallies_fade_and_a_sale_answers_the_demand_it_met() {
        let mut m = Market::new(id(9), 3, 0);
        m.record_unmet(2, 3.0, 30.0);
        assert_eq!(m.unmet_worth(2), Some(10.0));
        m.age_to(30, 30.0);
        assert!((m.unmet[2] - 1.5).abs() < 1e-9, "half after a half-life");
        assert_eq!(m.unmet_worth(2), Some(10.0), "what a unit is worth stays");
        m.record_trade(trade(2, 1.0, 0, 2.0), 3.0, 4);
        assert!((m.unmet[2] - 0.5).abs() < 1e-9);
        m.record_trade(trade(2, 1.0, 0, 2.0), 3.0, 4);
        assert_eq!(m.unmet[2], 0.0);
        assert_eq!(m.unmet_worth(2), None);
    }
}
