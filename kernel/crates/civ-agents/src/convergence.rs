//! The convergence record (M5b slice AQ, ADR-0019 §7): monthly, per pair of settlements lived in,
//! the asks and realised prices of each good offered in both, and what people of each carried
//! home from the other's market. Saved, because markets' tallies fade and offers change; the
//! dashboard's price convergence row and the demo read it.

use std::collections::BTreeMap;

use civ_core::PermanentId;

/// One good offered in both settlements of a pair at a month's end.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GoodGap {
    /// By index in the catalog's goods.
    pub good: u16,
    /// The median of its sellers' asks in each settlement (the pair's first, then second),
    /// hours of the seller's own work a unit.
    pub ask_h: [f32; 2],
    /// What a unit fetched in each that month, hours of its sellers' work, or -1 where none sold.
    pub paid_h: [f32; 2],
}

/// What people of one settlement carried home from another's market in a month.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Carried {
    /// Trips to its sellers' doors, those that bought nothing too.
    pub trips: u32,
    /// Hours walked there and back on them.
    pub walk_h: f32,
    /// Units bought, by good (in good order).
    pub goods: Vec<(u16, f32)>,
}

/// The record, by month (see [`crate::market::month_of`]).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Convergence {
    /// By `(month, a, b)`, `a` before `b`: each good offered in both at the month's end, in good
    /// order.
    pub gaps: BTreeMap<(u32, PermanentId, PermanentId), Vec<GoodGap>>,
    /// By `(month, from, to)`: what people of `from` carried home from `to`'s market.
    pub carried: BTreeMap<(u32, PermanentId, PermanentId), Carried>,
}

impl Convergence {
    /// People of `from` walked to a seller's door in `to` and home, `walk_h` hours, in `month`.
    pub fn trip(&mut self, month: u32, from: PermanentId, to: PermanentId, walk_h: f64) {
        let c = self.carried.entry((month, from, to)).or_default();
        c.trips += 1;
        c.walk_h += walk_h.max(0.0) as f32;
    }

    /// People of `from` bought `units` of good `good` in `to`'s market in `month`.
    pub fn carry(&mut self, month: u32, from: PermanentId, to: PermanentId, good: u16, units: f64) {
        let c = self.carried.entry((month, from, to)).or_default();
        match c.goods.binary_search_by_key(&good, |&(g, _)| g) {
            Ok(i) => c.goods[i].1 += units as f32,
            Err(i) => c.goods.insert(i, (good, units as f32)),
        }
    }

    /// What is wrong with the record.
    pub fn problems(&self) -> Vec<String> {
        let mut out = Vec::new();
        for (&(month, a, b), gaps) in &self.gaps {
            if a >= b {
                out.push(format!("month {month}: a pair recorded as {a} and {b}"));
            }
            if gaps.windows(2).any(|w| w[0].good >= w[1].good) {
                out.push(format!("month {month}: {a} and {b}'s goods out of order"));
            }
            if gaps
                .iter()
                .any(|g| g.ask_h.iter().any(|x| !(x.is_finite() && *x > 0.0)))
            {
                out.push(format!("month {month}: {a} and {b} with an ask of nothing"));
            }
        }
        for (&(month, from, to), c) in &self.carried {
            if from == to {
                out.push(format!("month {month}: {from} carried from itself"));
            }
            if c.goods.windows(2).any(|w| w[0].0 >= w[1].0) {
                out.push(format!(
                    "month {month}: goods carried from {to} out of order"
                ));
            }
        }
        out
    }
}

/// The median of `xs` (not empty).
pub fn median(xs: &mut [f64]) -> f64 {
    xs.sort_by(f64::total_cmp);
    let n = xs.len();
    if n % 2 == 1 {
        xs[n / 2]
    } else {
        (xs[n / 2 - 1] + xs[n / 2]) / 2.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(n: u64) -> PermanentId {
        PermanentId::from_raw(n).expect("non-zero")
    }

    #[test]
    fn trips_and_what_they_carried_add_up_by_month_and_direction() {
        let mut c = Convergence::default();
        c.trip(5, id(1), id(2), 3.5);
        c.carry(5, id(1), id(2), 8, 20.0);
        c.trip(5, id(1), id(2), 3.0);
        c.carry(5, id(1), id(2), 3, 1.0);
        c.carry(5, id(1), id(2), 8, 10.0);
        c.trip(5, id(2), id(1), 3.0);
        let there = &c.carried[&(5, id(1), id(2))];
        assert_eq!(there.trips, 2);
        assert!((there.walk_h - 6.5).abs() < 1e-6);
        assert_eq!(there.goods, vec![(3, 1.0), (8, 30.0)]);
        assert_eq!(c.carried[&(5, id(2), id(1))].goods, vec![]);
        assert!(c.problems().is_empty());
    }

    #[test]
    fn the_median_of_an_odd_and_an_even_count() {
        assert_eq!(median(&mut [3.0, 1.0, 2.0]), 2.0);
        assert_eq!(median(&mut [4.0, 1.0, 3.0, 2.0]), 2.5);
    }
}
