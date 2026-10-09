//! `civ-host twin`: the M5b demo's harness (ADR-0019 §8). A save is lived twice from the same
//! moment, a day at a time at Max: once as it is, and once with purchases between settlements
//! stopped. Shared weather and seasons move two settlements' prices together whether or not
//! anyone trades (research 08-12 §4), so trade is judged by the difference between the two lives.
//! Each year's end reports, for both, the purchases and trips between settlements, what was
//! carried, and each pair's mean gap of asks; the end judges each pair's convergence as the
//! dashboard's row does ([`crate::trade`]), the twin's beside it.
//!
//! The long run's checks fail it (every settlement's accounts balance, the books, residence
//! histories: [`LongRun`]); the convergence row is reported, and red in it fails the run, as on the
//! dashboard. Nothing here steers either life: the switch only leaves the trip to buy elsewhere
//! out of every choice.

use std::collections::BTreeMap;
use std::io::Write;
use std::path::PathBuf;

use civ_content::ContentRegistry;
use civ_core::PermanentId;
use civ_core::time::MINUTES_PER_YEAR;
use civ_sim::{SPEED_MAX, Sim, persist};

use crate::economy::Grade;
use crate::smoke::LongRun;
use crate::trade::{self, PairSeen};

/// Years to live from the save, by default.
pub const YEARS: u32 = 2;

/// What to run.
#[derive(Clone, Debug)]
pub struct TwinOptions {
    /// The save both lives start from.
    pub save: PathBuf,
    /// Years to live.
    pub years: u32,
}

/// What came of it.
#[derive(Debug, Default)]
pub struct TwinOutcome {
    /// The checks that failed, in either life.
    pub failures: Vec<String>,
    /// Each pair as judged at the end: with trade, and as the twin.
    pub trade: Vec<PairSeen>,
    pub twin: Vec<PairSeen>,
}

/// One year of one life.
#[derive(Clone, Debug, Default)]
struct YearSeen {
    /// Purchases between settlements, trips to buy (those that bought nothing too) and the hours
    /// walked on them.
    purchases: u32,
    trips: u32,
    walk_h: f64,
    /// Units carried home from other settlements' markets, by good.
    carried: BTreeMap<u16, f64>,
    /// Per pair, the mean over the year's months of the mean over goods offered in both of
    /// 100 × |ln(ask ÷ ask)|.
    gaps: BTreeMap<(PermanentId, PermanentId), f64>,
}

/// One life lived.
struct Life {
    sim: Sim,
    years: Vec<YearSeen>,
    failures: Vec<String>,
}

/// Lives both and writes the report to `out`.
pub fn run(
    content: &ContentRegistry,
    options: &TwinOptions,
    out: &mut dyn Write,
) -> anyhow::Result<TwinOutcome> {
    let load = || {
        persist::load(&options.save, content)
            .map_err(|e| anyhow::anyhow!("{}: {e}", options.save.display()))
    };
    let (trade_sim, mut twin_sim) = (load()?, load()?);
    twin_sim.stop_trade_between(true);
    writeln!(
        out,
        "{} at {}: {} people in {} settlements, lived {} years with trade between them and as \
         the twin without",
        options.save.display(),
        trade_sim.now(),
        trade_sim.people().living(),
        trade_sim.land().settlements.len(),
        options.years,
    )?;
    let years = options.years;
    let (trade_life, twin_life) = std::thread::scope(|scope| {
        let a = scope.spawn(move || live(trade_sim, years));
        let b = scope.spawn(move || live(twin_sim, years));
        (a.join(), b.join())
    });
    let (Ok(trade_life), Ok(twin_life)) = (trade_life, twin_life) else {
        anyhow::bail!("a life panicked");
    };
    let sim = &trade_life.sim;
    let name = |s: PermanentId| {
        sim.land()
            .settlements
            .iter()
            .find(|x| x.id == s)
            .map_or_else(|| format!("settlement {s}"), |x| x.name.clone())
    };
    let good = |g: u16| {
        sim.rules()
            .catalog
            .goods
            .get(usize::from(g))
            .map_or_else(|| format!("good {g}"), |d| d.name.to_lowercase())
    };
    let carried = |c: &BTreeMap<u16, f64>| {
        let list: Vec<String> = c
            .iter()
            .map(|(&g, &u)| format!("{u:.1} {}", good(g)))
            .collect();
        if list.is_empty() {
            "nothing".to_owned()
        } else {
            list.join(", ")
        }
    };
    let gaps = |y: &YearSeen| {
        let list: Vec<String> = y
            .gaps
            .iter()
            .map(|(&(a, b), g)| format!("{}–{} {g:.0}", name(a), name(b)))
            .collect();
        list.join(", ")
    };
    for (i, (t, w)) in trade_life.years.iter().zip(&twin_life.years).enumerate() {
        writeln!(out, "year {}", i + 1)?;
        writeln!(
            out,
            "  with trade: {} purchases between settlements, {} trips ({:.0} hours walked), \
             carried {}; mean gaps {}",
            t.purchases,
            t.trips,
            t.walk_h,
            carried(&t.carried),
            gaps(t)
        )?;
        writeln!(
            out,
            "  the twin:   {} purchases, {} trips; mean gaps {}",
            w.purchases,
            w.trips,
            gaps(w)
        )?;
    }
    let (with, without) = (trade_life.sim.people(), twin_life.sim.people());
    let mut outcome = TwinOutcome {
        trade: trade::pairs(&with.convergence, &with.contacts),
        twin: trade::pairs(&without.convergence, &without.contacts),
        ..TwinOutcome::default()
    };
    writeln!(
        out,
        "price convergence (gaps and bands in points, 100 × log):"
    )?;
    for p in &outcome.trade {
        let (grade, why) = p.grade();
        writeln!(
            out,
            "  {}–{}: {} ({why}); {} trades, {} trips, {:.0} hours walked",
            name(p.a),
            name(p.b),
            grade.word(),
            p.trades,
            p.trips,
            p.walk_h
        )?;
        let twin = outcome.twin.iter().find(|q| (q.a, q.b) == (p.a, p.b));
        for g in &p.goods {
            let twin_gap = twin
                .and_then(|q| q.goods.iter().find(|h| h.good == g.good))
                .map_or_else(|| "-".to_owned(), |h| format!("{:.0}", h.gap));
            writeln!(
                out,
                "    {}: gap {:.0} (twin {twin_gap}), band {:.0}, before trade {}, {:.1} carried, \
                 {} months{}",
                good(g.good),
                g.gap,
                g.band,
                g.before
                    .map_or_else(|| "-".to_owned(), |b| format!("{b:.0}")),
                g.carried,
                g.months,
                if g.wrong_way > 0 {
                    format!("; {} years the wrong way", g.wrong_way)
                } else {
                    String::new()
                }
            )?;
        }
    }
    let row = trade::grade(&outcome.trade);
    writeln!(out, "the row with trade: {}", row.word())?;
    outcome.failures.extend(trade_life.failures);
    outcome
        .failures
        .extend(twin_life.failures.into_iter().map(|f| format!("twin: {f}")));
    if row == Grade::Red {
        outcome
            .failures
            .push("price convergence: a good flowed the wrong way".to_owned());
    }
    for f in &outcome.failures {
        writeln!(out, "FAILED: {f}")?;
    }
    if outcome.failures.is_empty() {
        writeln!(out, "every check passed in both lives")?;
    }
    Ok(outcome)
}

/// Lives `sim` `years` years a day at a time at Max, checking each year's end.
fn live(mut sim: Sim, years: u32) -> Life {
    let (mut seen, mut failures) = (Vec::new(), Vec::new());
    if let Err(e) = sim
        .set_speed(SPEED_MAX)
        .and_then(|()| sim.advance_to_midnight().map(|_| ()))
    {
        failures.push(format!("the clock stopped: {e}"));
        return Life {
            sim,
            years: seen,
            failures,
        };
    }
    let mut run = LongRun::begin(&sim);
    let start = sim.now().minutes();
    for y in 1..=years {
        let from_month = civ_agents::market::month_of(sim.now());
        let bought_before = bought(&sim);
        if let Err(e) = run.live_to(&mut sim, start + i64::from(y) * MINUTES_PER_YEAR) {
            failures.push(format!("the clock stopped in year {y}: {e}"));
            break;
        }
        failures.extend(run.year_end(&sim, y));
        failures.extend(sim.people().residence_problems());
        let to_month = civ_agents::market::month_of(sim.now());
        seen.push(year_seen(
            &sim,
            from_month,
            to_month,
            bought(&sim) - bought_before,
        ));
    }
    Life {
        sim,
        years: seen,
        failures,
    }
}

/// Purchases between settlements so far.
fn bought(sim: &Sim) -> u32 {
    sim.people().contacts.years.values().map(|c| c.bought).sum()
}

/// What the record says of months `from` to `to` (not including `to`).
fn year_seen(sim: &Sim, from: u32, to: u32, purchases: u32) -> YearSeen {
    let conv = &sim.people().convergence;
    let mut seen = YearSeen {
        purchases,
        ..YearSeen::default()
    };
    for (&(month, _, _), c) in &conv.carried {
        if month < from || month >= to {
            continue;
        }
        seen.trips += c.trips;
        seen.walk_h += f64::from(c.walk_h);
        for &(g, u) in &c.goods {
            *seen.carried.entry(g).or_default() += f64::from(u);
        }
    }
    let mut months: BTreeMap<(PermanentId, PermanentId), (f64, u32)> = BTreeMap::new();
    for (&(month, a, b), list) in &conv.gaps {
        if month < from || month >= to || list.is_empty() {
            continue;
        }
        let mean = list
            .iter()
            .map(|g| 100.0 * (f64::from(g.ask_h[0]) / f64::from(g.ask_h[1])).ln().abs())
            .sum::<f64>()
            / list.len() as f64;
        let m = months.entry((a, b)).or_default();
        m.0 += mean;
        m.1 += 1;
    }
    seen.gaps = months
        .into_iter()
        .map(|(k, (sum, n))| (k, sum / f64::from(n.max(1))))
        .collect();
    seen
}
