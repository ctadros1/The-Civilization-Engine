//! Price convergence between settlements (M5b slice AQ, step three; ADR-0019 §7): what the
//! dashboard's row and the twin's report read from the convergence record. Per pair of
//! settlements, each good offered in both is judged by how far apart its median asks stand month
//! by month, against the band what carrying it costs would leave (research 08-05 §1.7: a gap
//! smaller than the cost of carrying a good persists without trade). The M5 trade brief's
//! proposal, every threshold a tuning value: the row applies to a pair with at least
//! [`MIN_TRADES`] purchases between them; it is green when each good traded between them stands
//! within its band over the last ten years, or narrower than in the year before their first
//! trade; amber when not, or when no traded good has [`MIN_MONTHS`] months of asks on both sides;
//! and red only for an inconsistency: a good flowing on net from the dearer settlement to the
//! cheaper over a year in which the gap exceeded the band. Monthly gaps overlap and repeat, so
//! they are summarised, never treated as independent observations (16-01 §4.1, §4.5), and a
//! correlation of the two settlements' prices is never graded (08-12 §4).

use std::collections::BTreeMap;

use civ_agents::convergence::Convergence;
use civ_agents::places::Contacts;
use civ_core::PermanentId;
use serde::Serialize;

use crate::economy::Grade;

/// Purchases between a pair before the row judges it (the brief's 30).
pub const MIN_TRADES: u32 = 30;
/// Points added to every band for what the record cannot see (the brief's 5).
pub const BAND_MARGIN: f64 = 5.0;
/// Months judged, back from the last on record (ten years).
pub const WINDOW_MONTHS: u32 = 120;
/// Months of asks on both sides a traded good needs to be judged (two years).
pub const MIN_MONTHS: usize = 24;

/// One good offered in both settlements of a pair, as judged.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct GoodSeen {
    /// By index in the catalog's goods.
    pub good: u16,
    /// Months of the window in which both offered it.
    pub months: usize,
    /// The median over those months of 100 × |ln(ask in the first ÷ ask in the second)|.
    pub gap: f64,
    /// The same over the twelve months before the pair's first trade in it, if both offered it
    /// then.
    pub before: Option<f64>,
    /// The median of 100 × |ln(what a unit fetched in the first ÷ in the second)| over the
    /// window's months in which it sold in both, if any: realised prices beside the asks.
    pub paid_gap: Option<f64>,
    /// Months of the window in which it sold in both.
    pub paid_months: usize,
    /// 100 × ln(1 + hours walked per unit carried between the pair ÷ its median ask), plus
    /// [`BAND_MARGIN`].
    pub band: f64,
    /// Units carried between the pair, both ways, all months on record.
    pub carried: f64,
    /// Years in which it flowed on net from the dearer settlement to the cheaper while their
    /// mean gap exceeded the band.
    pub wrong_way: u32,
}

/// One pair of settlements, `a` before `b`, as judged.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PairSeen {
    #[serde(serialize_with = "raw_id")]
    pub a: PermanentId,
    #[serde(serialize_with = "raw_id")]
    pub b: PermanentId,
    /// Purchases by people of either at the other's sellers' doors, all years.
    pub trades: u32,
    /// Trips between them, those that bought nothing too, and the hours walked on them.
    pub trips: u32,
    pub walk_h: f64,
    /// Each good offered in both in the window, in good order.
    pub goods: Vec<GoodSeen>,
}

impl PairSeen {
    /// The goods traded between the pair that the row judges.
    pub fn judged(&self) -> impl Iterator<Item = &GoodSeen> {
        self.goods
            .iter()
            .filter(|g| g.carried > 0.0 && g.months >= MIN_MONTHS)
    }

    /// How the pair comes out, and why.
    pub fn grade(&self) -> (Grade, String) {
        if self.trades < MIN_TRADES {
            return (
                Grade::Gray,
                format!("{} trades, fewer than {MIN_TRADES}", self.trades),
            );
        }
        let judged: Vec<&GoodSeen> = self.judged().collect();
        if judged.is_empty() {
            return (
                Grade::Amber,
                format!("no traded good with {MIN_MONTHS} months of asks on both sides"),
            );
        }
        if judged.iter().any(|g| g.wrong_way > 0) {
            return (
                Grade::Red,
                "a good flowed from the dearer settlement to the cheaper while the gap exceeded \
                 the band"
                    .to_owned(),
            );
        }
        let closed = |g: &&GoodSeen| g.gap <= g.band || g.before.is_some_and(|b| g.gap < b);
        if judged.iter().all(closed) {
            (
                Grade::Green,
                "every traded good within its band or closer than before".to_owned(),
            )
        } else {
            (
                Grade::Amber,
                "a traded good wider than its band and no closer than before".to_owned(),
            )
        }
    }
}

fn raw_id<S: serde::Serializer>(id: &PermanentId, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_u64(id.get())
}

fn median(xs: &mut [f64]) -> Option<f64> {
    if xs.is_empty() {
        return None;
    }
    Some(civ_agents::convergence::median(xs))
}

/// Each pair of settlements with any month on record, judged from the convergence record and the
/// purchases between them.
pub fn pairs(conv: &Convergence, contacts: &Contacts) -> Vec<PairSeen> {
    let Some(last) = conv
        .gaps
        .keys()
        .map(|k| k.0)
        .chain(conv.carried.keys().map(|k| k.0))
        .max()
    else {
        return Vec::new();
    };
    let first = last.saturating_sub(WINDOW_MONTHS - 1);
    let mut keys: Vec<(PermanentId, PermanentId)> = conv
        .gaps
        .keys()
        .map(|&(_, a, b)| (a, b))
        .chain(conv.carried.keys().map(|&(_, f, t)| (f.min(t), f.max(t))))
        .collect();
    keys.sort_unstable();
    keys.dedup();
    let mut out = Vec::new();
    for (a, b) in keys {
        let trades = contacts
            .years
            .iter()
            .filter(|((_, f, t), _)| (*f, *t) == (a, b) || (*f, *t) == (b, a))
            .map(|(_, c)| c.bought)
            .sum();
        // What was carried: by month and good, the flow from `a`'s market to `b` (people of `b`
        // buying there) less the flow the other way.
        let (mut trips, mut walk_h, mut units) = (0u32, 0.0f64, 0.0f64);
        let mut flow: BTreeMap<(u32, u16), f64> = BTreeMap::new();
        let mut carried: BTreeMap<u16, f64> = BTreeMap::new();
        for (&(month, from, to), c) in &conv.carried {
            let sign = if (from, to) == (b, a) {
                1.0
            } else if (from, to) == (a, b) {
                -1.0
            } else {
                continue;
            };
            trips += c.trips;
            walk_h += f64::from(c.walk_h);
            for &(g, u) in &c.goods {
                let u = f64::from(u);
                units += u;
                *flow.entry((month, g)).or_default() += sign * u;
                *carried.entry(g).or_default() += u;
            }
        }
        let carry_h = if units > 0.0 { walk_h / units } else { 0.0 };
        // Each good's asks, by month: (month, ask in `a`, ask in `b`), and what it fetched in each
        // in the months it sold in both.
        let mut asks: BTreeMap<u16, Vec<(u32, f64, f64)>> = BTreeMap::new();
        let mut paid: BTreeMap<u16, Vec<(u32, f64, f64)>> = BTreeMap::new();
        for (&(month, x, y), list) in &conv.gaps {
            if (x, y) != (a, b) {
                continue;
            }
            for g in list {
                asks.entry(g.good).or_default().push((
                    month,
                    f64::from(g.ask_h[0]),
                    f64::from(g.ask_h[1]),
                ));
                if g.paid_h.iter().all(|&p| p > 0.0) {
                    paid.entry(g.good).or_default().push((
                        month,
                        f64::from(g.paid_h[0]),
                        f64::from(g.paid_h[1]),
                    ));
                }
            }
        }
        let gap = |x: f64, y: f64| 100.0 * (x / y).ln().abs();
        let mut goods = Vec::new();
        for (good, months) in asks {
            let window: Vec<&(u32, f64, f64)> = months
                .iter()
                .filter(|m| m.0 >= first && m.0 <= last)
                .collect();
            if window.is_empty() {
                continue;
            }
            let mut gaps: Vec<f64> = window.iter().map(|m| gap(m.1, m.2)).collect();
            let mut prices: Vec<f64> = window.iter().map(|m| (m.1 + m.2) / 2.0).collect();
            let price = median(&mut prices).unwrap_or(0.0);
            let band = if price > 0.0 {
                100.0 * (1.0 + carry_h / price).ln() + BAND_MARGIN
            } else {
                BAND_MARGIN
            };
            let first_trade = flow
                .iter()
                .filter(|((_, g), u)| *g == good && u.abs() > 0.0)
                .map(|((m, _), _)| *m)
                .min();
            let before = first_trade.and_then(|t| {
                let mut xs: Vec<f64> = months
                    .iter()
                    .filter(|m| m.0 < t && m.0 + 12 >= t)
                    .map(|m| gap(m.1, m.2))
                    .collect();
                median(&mut xs)
            });
            // Years in which it flowed on net from the dearer to the cheaper, the gap wider than
            // the band: by year, (sum of asks in `a`, in `b`, net flow from `a` to `b`, months).
            let mut years: BTreeMap<u32, (f64, f64, f64, u32)> = BTreeMap::new();
            for m in &months {
                let y = years.entry(m.0 / 12).or_default();
                y.0 += m.1;
                y.1 += m.2;
                y.3 += 1;
            }
            for ((month, g), u) in &flow {
                if *g == good
                    && let Some(y) = years.get_mut(&(month / 12))
                {
                    y.2 += u;
                }
            }
            let wrong_way = years
                .values()
                .filter(|&&(sum_a, sum_b, net, n)| {
                    let (ask_a, ask_b) = (sum_a / f64::from(n), sum_b / f64::from(n));
                    let dearer_to_cheaper =
                        (ask_a > ask_b && net > 0.0) || (ask_b > ask_a && net < 0.0);
                    dearer_to_cheaper && gap(ask_a, ask_b) > band
                })
                .count() as u32;
            let mut paid_gaps: Vec<f64> = paid
                .get(&good)
                .into_iter()
                .flatten()
                .filter(|m| m.0 >= first && m.0 <= last)
                .map(|m| gap(m.1, m.2))
                .collect();
            goods.push(GoodSeen {
                good,
                months: window.len(),
                gap: median(&mut gaps).unwrap_or(0.0),
                before,
                paid_months: paid_gaps.len(),
                paid_gap: median(&mut paid_gaps),
                band,
                carried: carried.get(&good).copied().unwrap_or(0.0),
                wrong_way,
            });
        }
        out.push(PairSeen {
            a,
            b,
            trades,
            trips,
            walk_h,
            goods,
        });
    }
    out
}

/// The row over every pair: the worst grade of the pairs it applies to, or grey if it applies to
/// none.
pub fn grade(pairs: &[PairSeen]) -> Grade {
    pairs
        .iter()
        .map(|p| p.grade().0)
        .filter(|g| *g != Grade::Gray)
        .max()
        .unwrap_or(Grade::Gray)
}

#[cfg(test)]
mod tests {
    use super::*;
    use civ_agents::convergence::{Carried, GoodGap};

    fn id(n: u64) -> PermanentId {
        PermanentId::from_raw(n).expect("non-zero")
    }

    /// A pair whose good 3 is asked `ask_a(month)` hours in the first and 1 hour in the second
    /// over `months` months, people of the second buying `units` of it in the first's market
    /// each month from `from` on, walking `walk_h` hours a month for it; `trades` purchases.
    fn record(
        months: u32,
        ask_a: impl Fn(u32) -> f32,
        from: u32,
        units: f32,
        walk_h: f32,
        trades: u32,
    ) -> (Convergence, Contacts) {
        let mut conv = Convergence::default();
        for m in 0..months {
            conv.gaps.insert(
                (m, id(1), id(2)),
                vec![GoodGap {
                    good: 3,
                    ask_h: [ask_a(m), 1.0],
                    paid_h: [-1.0, -1.0],
                }],
            );
            if m >= from && units > 0.0 {
                conv.carried.insert(
                    (m, id(2), id(1)),
                    Carried {
                        trips: 1,
                        walk_h,
                        goods: vec![(3, units)],
                    },
                );
            }
        }
        let mut contacts = Contacts::default();
        contacts.years.entry((0, id(2), id(1))).or_default().bought = trades;
        (conv, contacts)
    }

    #[test]
    fn a_pair_too_few_trades_apart_is_not_judged() {
        let (conv, contacts) = record(36, |_| 2.0, 0, 1.0, 1.0, 5);
        let pairs = pairs(&conv, &contacts);
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].grade().0, Grade::Gray);
        assert_eq!(grade(&pairs), Grade::Gray);
    }

    #[test]
    fn a_gap_within_what_carrying_costs_is_green() {
        // Asked 1.1 hours against 1: a gap of about 9.5 points. Carrying a unit costs half an
        // hour's walking: a band of 100 ln(1 + 0.5 ÷ 1.05) + 5, about 44 points. People of the
        // second buy in the first, the dearer, but within the band: no inconsistency.
        let (conv, contacts) = record(36, |_| 1.1, 0, 2.0, 1.0, 40);
        let p = &pairs(&conv, &contacts)[0];
        let g = &p.goods[0];
        assert!((g.gap - 100.0 * 1.1f64.ln()).abs() < 1e-3, "{g:?}");
        assert!(
            (g.band - (100.0 * (1.0 + 0.5 / 1.05f64).ln() + 5.0)).abs() < 1e-3,
            "{g:?}"
        );
        assert_eq!((g.months, g.carried, g.wrong_way), (36, 72.0, 0));
        assert_eq!((p.trades, p.trips), (40, 36));
        assert_eq!(p.grade().0, Grade::Green);
        // Nothing sold in either: no realised gap. Sold in both in ten months, fetching 1.2
        // hours in the first and 1 in the second, and in only the first in another.
        assert_eq!((g.paid_gap, g.paid_months), (None, 0));
        let (mut conv, contacts) = (conv, contacts);
        for (&(m, _, _), list) in conv.gaps.iter_mut() {
            if m < 10 {
                list[0].paid_h = [1.2, 1.0];
            } else if m == 10 {
                list[0].paid_h = [1.2, -1.0];
            }
        }
        let g = pairs(&conv, &contacts).remove(0).goods.remove(0);
        assert_eq!(g.paid_months, 10);
        assert!((g.paid_gap.expect("sold in both") - 100.0 * 1.2f64.ln()).abs() < 1e-3);
        assert!(
            (g.gap - 100.0 * 1.1f64.ln()).abs() < 1e-3,
            "the asks' gap is the asks'"
        );
    }

    #[test]
    fn a_gap_wider_than_its_band_and_no_closer_than_before_is_amber_and_narrowing_is_green() {
        // People of the first buy in the second, the cheaper: the right way. Carrying is nearly
        // free, a band of about 5 points, and the gap stays at 69 points.
        let right_way = |ask_a: &dyn Fn(u32) -> f32| {
            let (mut conv, contacts) = record(48, ask_a, 0, 0.0, 0.0, 40);
            for m in 12..48 {
                conv.carried.insert(
                    (m, id(1), id(2)),
                    Carried {
                        trips: 1,
                        walk_h: 0.1,
                        goods: vec![(3, 10.0)],
                    },
                );
            }
            pairs(&conv, &contacts).remove(0)
        };
        let p = right_way(&|_| 2.0);
        assert_eq!(p.goods[0].before.map(f64::round), Some(69.0));
        assert_eq!(p.goods[0].wrong_way, 0);
        assert_eq!(p.grade().0, Grade::Amber, "{p:?}");
        // The same gap falling once trade begins, from 2 hours to 1.5: closer than before.
        let p = right_way(&|m| if m < 12 { 2.0 } else { 1.5 });
        assert_eq!(p.grade().0, Grade::Green, "{p:?}");
    }

    #[test]
    fn a_good_flowing_from_the_dearer_to_the_cheaper_beyond_the_band_is_red() {
        // The first asks 2 hours, the second 1, and people of the second buy in the first:
        // from the dearer to the cheaper, with the gap far beyond the band.
        let (conv, contacts) = record(36, |_| 2.0, 0, 10.0, 0.1, 40);
        let p = &pairs(&conv, &contacts)[0];
        assert_eq!(p.goods[0].wrong_way, 3);
        assert_eq!(p.grade().0, Grade::Red);
        assert_eq!(grade(std::slice::from_ref(p)), Grade::Red);
    }
}
