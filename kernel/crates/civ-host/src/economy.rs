//! The village economy in the long smoke run (M3a slice L; plan §4.7): what the run samples at the
//! start of each month and checks at each year's end, and how it grades what it saw.
//!
//! - **Red** is a broken mechanism and fails the world: land held or worked against its regime's
//!   rules (ADR-0007), a field lost or moved, wealth measures that are not numbers or contradict
//!   the regime, or food stocks that never rise after a harvest.
//! - **Amber** is outside what the research leads one to expect: worth a look, not a failure.
//! - **Gray** is too little to judge.
//! - **Green** is as expected.
//!
//! Prices are judged over several harvests and from stocks, not as a seasonal wave (research
//! 16-01: seasonal prices come from harvests, stocks and trade, and short samples exaggerate
//! them). Nothing here steers a world.

use std::collections::BTreeMap;

use civ_agents::ChronicleKind;
use civ_agents::params::{LandHolder, Succession};
use civ_agents::person::stock_kcal;
use civ_agents::population::stores_now;
use civ_core::PermanentId;
use civ_core::time::SimTime;
use civ_land::RectCm;
use civ_sim::Sim;

/// How a check came out.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Grade {
    /// As expected.
    Green,
    /// Too little to judge.
    Gray,
    /// Outside what the research leads one to expect.
    Amber,
    /// A broken mechanism: the world fails.
    Red,
}

impl Grade {
    /// The grade in the report.
    pub fn word(self) -> &'static str {
        match self {
            Grade::Green => "green",
            Grade::Gray => "gray",
            Grade::Amber => "amber",
            Grade::Red => "RED",
        }
    }
}

/// One check of a world's economy at the end of its run.
#[derive(Clone, Debug)]
pub struct Check {
    /// What was checked: `stocks`, `asks`, `inequality` or `workshops`.
    pub name: &'static str,
    /// How it came out.
    pub grade: Grade,
    /// What was seen, in words.
    pub text: String,
}

/// Goods Gini of a settlement below which, from its fifth year, it is flagged: research 08-14
/// §3.1 found agricultural populations' composite wealth Ginis averaging 0.48 ± 0.04 (a broader
/// measure than goods alone, so the flag is set well below it).
pub const LOW_GINI: f64 = 0.3;
/// Years a settlement lives before its inequality is judged.
pub const GINI_FROM_YEAR: i64 = 5;
/// Workshops a world needs before their sizes are judged.
pub const MIN_WORKSHOPS: usize = 10;
/// Gap between grain asks before and after the harvest, log points, above which asks are seen to
/// follow the harvest (research 16-01 §2.2: mean seasonal gaps of 17 to 61 log points across foods
/// in 193 markets of seven African countries).
pub const ASK_GAP_LOG_POINTS: f64 = 2.0;

/// One settlement at the start of a month.
#[derive(Clone, Debug)]
struct Sample {
    settlement: PermanentId,
    at: SimTime,
    /// Food energy its households hold, kcal.
    food_kcal: f64,
    /// What each seller of the crop's grain asks for it, hours of its own work a unit.
    asks: Vec<f64>,
}

/// What a world's run has seen of its economy.
#[derive(Clone, Debug, Default)]
pub struct Economy {
    samples: Vec<Sample>,
    /// The fields at the last year's end.
    fields: BTreeMap<PermanentId, RectCm>,
}

impl Economy {
    /// Samples each lived-in settlement as a month starts: the food its households hold, and
    /// what its households and workshops ask for the crop's grain, one ask a seller.
    pub fn sample(&mut self, sim: &Sim) {
        let rules = sim.rules();
        let (now, goods) = (sim.now(), &rules.catalog.goods);
        let grain = rules
            .catalog
            .crops
            .get(rules.people.farm.crop)
            .map(|c| c.good);
        let ask = |offers: &[civ_agents::market::Offer]| {
            offers
                .iter()
                .find(|o| Some(usize::from(o.good)) == grain)
                .map(|o| f64::from(o.ask_h))
        };
        for s in &sim.land().settlements {
            let mut lived = false;
            let mut food_kcal = 0.0;
            let mut asks = Vec::new();
            for (_, h) in sim.people().households.iter() {
                if h.settlement != Some(s.id) || h.members.is_empty() {
                    continue;
                }
                lived = true;
                food_kcal += stock_kcal(&stores_now(h, now, &rules.people, goods), goods);
                asks.extend(ask(&h.offers));
            }
            for f in &sim.people().firms {
                if f.is_open() && f.settlement == Some(s.id) {
                    asks.extend(ask(&f.offers));
                }
            }
            if lived {
                self.samples.push(Sample {
                    settlement: s.id,
                    at: now,
                    food_kcal,
                    asks,
                });
            }
        }
    }

    /// Checks the world at the end of year `y` of its run: its land claims fit its regime, no field
    /// was lost or moved since the last year's end, and the latest year's wealth measures are
    /// numbers in their ranges that fit the regime. Every problem fails the world.
    pub fn year_end(&mut self, sim: &Sim, y: u32) -> Vec<String> {
        let mut out = Vec::new();
        let regime = sim.regime();
        let problems = sim
            .people()
            .claims_problems(sim.land(), regime, sim.ids().peek_next());
        if let Some(first) = problems.first() {
            out.push(format!(
                "{} land claim problems in year {y}, first: {first}",
                problems.len()
            ));
        }
        let fields: BTreeMap<PermanentId, RectCm> =
            sim.land().fields.iter().map(|f| (f.id, f.rect)).collect();
        let lost = self
            .fields
            .iter()
            .filter(|(id, rect)| fields.get(id) != Some(rect))
            .count();
        if lost > 0 {
            out.push(format!("{lost} fields lost or moved in year {y}"));
        }
        self.fields = fields;
        let years = &sim.people().wealth_years;
        let latest = years.iter().map(|w| w.year).max();
        for w in years.iter().filter(|w| Some(w.year) == latest) {
            let s = &w.spread;
            let shares = [
                s.gini_goods,
                s.gini_held,
                s.gini_worked,
                s.gini_floor,
                s.top_tenth_goods,
                s.holding_none,
                s.working_none,
            ];
            let amounts = [
                s.goods_h_per_head,
                s.worked_ha_per_head,
                s.floor_m2_per_house,
                s.common_ha,
                s.roofed_m2_per_house,
                s.storage_kg_per_house,
            ];
            if shares.iter().any(|v| !(0.0..=1.0).contains(v))
                || amounts.iter().any(|v| !(v.is_finite() && *v >= 0.0))
            {
                out.push(format!(
                    "the wealth measures of year {} are not numbers in their ranges",
                    w.year
                ));
            }
            if regime.holder == LandHolder::Settlement && s.households > 0 && s.holding_none < 1.0 {
                out.push(format!(
                    "households hold land under {} in year {}",
                    regime.name, w.year
                ));
            }
            if regime.holder == LandHolder::Breaker
                && regime.succession != Succession::Settlement
                && s.common_ha > 0.0
            {
                out.push(format!(
                    "a settlement holds land under {} in year {}",
                    regime.name, w.year
                ));
            }
        }
        out
    }

    /// Grades what the run saw, at its end.
    pub fn grade(&self, sim: &Sim) -> Vec<Check> {
        let (stocks, asks) = self.stocks_and_asks(sim);
        vec![stocks, asks, inequality(sim), workshops(sim)]
    }

    /// For each settlement and year after its first: the samples just before its crop ripens and
    /// just after its harvest was in.
    fn harvests(&self, sim: &Sim) -> Vec<(&Sample, &Sample)> {
        let rules = sim.rules();
        let Some(crop) = rules.catalog.crops.get(rules.people.farm.crop) else {
            return Vec::new();
        };
        let ripe_from = i64::from(crop.sow_from_day) + i64::from(crop.grow_days);
        let mut out = Vec::new();
        for e in &sim.people().chronicle {
            let Some(settlement) = e.settlement else {
                continue;
            };
            if e.kind != ChronicleKind::HarvestIn {
                continue;
            }
            let year = e.at.date().year;
            let founded = sim
                .land()
                .settlements
                .iter()
                .find(|s| s.id == settlement)
                .map(|s| s.founded.date().year);
            if founded.is_none_or(|f| f >= year) {
                continue;
            }
            let before = self
                .samples
                .iter()
                .filter(|s| {
                    let d = s.at.date();
                    s.settlement == settlement
                        && d.year == year
                        && i64::from(d.day_of_year) <= ripe_from
                })
                .max_by_key(|s| s.at);
            let after = self
                .samples
                .iter()
                .filter(|s| s.settlement == settlement && s.at > e.at)
                .min_by_key(|s| s.at);
            if let (Some(b), Some(a)) = (before, after) {
                out.push((b, a));
            }
        }
        out
    }

    /// Whether food stocks rise after the harvest, and whether grain is asked more for before it
    /// than after.
    fn stocks_and_asks(&self, sim: &Sim) -> (Check, Check) {
        let pairs = self.harvests(sim);
        let years = pairs.len();
        let rose = pairs
            .iter()
            .filter(|(b, a)| a.food_kcal > b.food_kcal)
            .count();
        let stocks = match (years, rose) {
            (0 | 1, _) => Check {
                name: "stocks",
                grade: Grade::Gray,
                text: format!("{years} harvests after the first to compare"),
            },
            (n, 0) => Check {
                name: "stocks",
                grade: Grade::Red,
                text: format!("food stocks rose after none of {n} harvests"),
            },
            (n, r) => Check {
                name: "stocks",
                grade: if r == n { Grade::Green } else { Grade::Amber },
                text: format!("food stocks rose after {r} of {n} harvests"),
            },
        };
        let gaps: Vec<f64> = pairs
            .iter()
            .filter_map(|(b, a)| {
                let (before, after) = (median(&b.asks)?, median(&a.asks)?);
                (before > 0.0 && after > 0.0).then(|| (before / after).ln() * 100.0)
            })
            .collect();
        let asks = if gaps.len() < 2 {
            Check {
                name: "asks",
                grade: Grade::Gray,
                text: format!(
                    "grain asked for both before and after {} harvests",
                    gaps.len()
                ),
            }
        } else {
            let gap = gaps.iter().sum::<f64>() / gaps.len() as f64;
            Check {
                name: "asks",
                grade: if gap > ASK_GAP_LOG_POINTS {
                    Grade::Green
                } else {
                    Grade::Amber
                },
                text: format!(
                    "grain asked {gap:+.0} log points more before the harvest than after, over {} \
                     harvests",
                    gaps.len()
                ),
            }
        };
        (stocks, asks)
    }
}

/// The middle of `values`; `None` when there are none.
fn median(values: &[f64]) -> Option<f64> {
    let mut v: Vec<f64> = values.iter().copied().filter(|x| x.is_finite()).collect();
    if v.is_empty() {
        return None;
    }
    v.sort_by(f64::total_cmp);
    let n = v.len();
    Some(if n % 2 == 1 {
        v[n / 2]
    } else {
        (v[n / 2 - 1] + v[n / 2]) / 2.0
    })
}

/// The Gini of goods of the world's largest settlement at its latest year's end, from its fifth.
fn inequality(sim: &Sim) -> Check {
    let latest = sim
        .people()
        .wealth_years
        .iter()
        .filter(|w| {
            let founded = sim
                .land()
                .settlements
                .iter()
                .find(|s| s.id == w.spread.settlement)
                .map_or(i64::MAX, |s| s.founded.date().year);
            w.year - founded + 1 >= GINI_FROM_YEAR
        })
        .max_by(|a, b| {
            a.year
                .cmp(&b.year)
                .then(a.spread.people.cmp(&b.spread.people))
        });
    match latest {
        None => Check {
            name: "inequality",
            grade: Grade::Gray,
            text: format!("no settlement has lived {GINI_FROM_YEAR} years"),
        },
        Some(w) => Check {
            name: "inequality",
            grade: if w.spread.gini_goods < LOW_GINI {
                Grade::Amber
            } else {
                Grade::Green
            },
            text: format!(
                "Gini of goods {:.2} at the end of year {}",
                w.spread.gini_goods, w.year
            ),
        },
    }
}

/// How the hours worked for each workshop over its life spread: firm sizes are skewed, a few
/// large among many small (research 16-01 treats this as a diagnostic that needs enough of them).
fn workshops(sim: &Sim) -> Check {
    let mut hours: Vec<f64> = sim
        .people()
        .firms
        .iter()
        .map(|f| {
            f.books
                .months
                .iter()
                .map(|m| f64::from(m.owner_h) + f64::from(m.hired_h))
                .sum::<f64>()
        })
        .filter(|h| *h > 0.0)
        .collect();
    let n = hours.len();
    if n < MIN_WORKSHOPS {
        return Check {
            name: "workshops",
            grade: Grade::Gray,
            text: format!("{n} workshops worked in, too few to judge their sizes"),
        };
    }
    hours.sort_by(f64::total_cmp);
    let total: f64 = hours.iter().sum();
    let mean = total / n as f64;
    let middle = median(&hours).unwrap_or(mean);
    let ratio = mean / middle.max(1e-9);
    Check {
        name: "workshops",
        grade: if ratio > 1.0 {
            Grade::Green
        } else {
            Grade::Amber
        },
        text: format!(
            "{n} workshops: the mean {ratio:.1} times the median hours worked, the largest {:.0}% \
             of them all",
            hours[n - 1] / total * 100.0
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_middle_of_a_list() {
        assert_eq!(median(&[]), None);
        assert_eq!(median(&[3.0]), Some(3.0));
        assert_eq!(median(&[4.0, 1.0, 3.0]), Some(3.0));
        assert_eq!(median(&[4.0, 1.0, 3.0, 2.0]), Some(2.5));
        assert_eq!(median(&[f64::NAN, 2.0]), Some(2.0));
        assert!(Grade::Red > Grade::Amber && Grade::Amber > Grade::Gray);
    }
}
