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
//! dashboard; so is the neighbours row for each life (M5b slice AR). Nothing here steers either life: the switch only leaves the trip to buy elsewhere
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
    /// Each settlement's roofs (M5b slice AR; the M5 diffusion brief §3.4).
    roofs: Vec<Roofs>,
}

/// A settlement's roofs at a year's end: its founding way's pitch, and the pitch of the year's
/// new buildings, with how many of them follow, through their chain of followed buildings, a
/// building of each other settlement.
#[derive(Clone, Debug)]
struct Roofs {
    settlement: PermanentId,
    /// Founding way's pitch, degrees.
    way: Option<f64>,
    /// The year's new buildings: how many, and their mean pitch, degrees.
    new: usize,
    pitch: f64,
    /// Of them, by the settlement their chain first crosses into.
    after: BTreeMap<PermanentId, u32>,
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
    // Each settlement as the lives begin: its people and its founding way, and how far apart
    // their hearths stand (the M5 diffusion brief §3.4: the demo picks its world by this draw).
    for s in &trade_sim.land().settlements {
        let way = trade_sim.people().founding_ways.get(&s.id).map_or_else(
            || "-".to_owned(),
            |w| format!("{:.1}°", w.pitch_centideg / 100.0),
        );
        writeln!(
            out,
            "  {}: {} people, founded to build roofs at {way}",
            s.name,
            trade_sim.people().residents(s.id)
        )?;
    }
    let places = &trade_sim.land().settlements;
    for (i, x) in places.iter().enumerate() {
        for y in places.iter().skip(i + 1) {
            let d = (x.hearth_m.0 - y.hearth_m.0).hypot(x.hearth_m.1 - y.hearth_m.1) / 1000.0;
            writeln!(out, "  {}–{}: hearths {d:.1} km apart", x.name, y.name)?;
        }
    }
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
        for (life, y) in [("with trade", t), ("the twin", w)] {
            let list: Vec<String> = y
                .roofs
                .iter()
                .filter(|r| r.new > 0)
                .map(|r| {
                    let after: Vec<String> = r
                        .after
                        .iter()
                        .map(|(&s, n)| format!("{n} after {}", name(s)))
                        .collect();
                    format!(
                        "{} {} new at {:.1}°{}",
                        name(r.settlement),
                        r.new,
                        r.pitch,
                        if after.is_empty() {
                            String::new()
                        } else {
                            format!(" ({})", after.join(", "))
                        }
                    )
                })
                .collect();
            if !list.is_empty() {
                writeln!(out, "  roofs {life}: {}", list.join("; "))?;
            }
        }
    }
    // Whether, and when, one settlement took another's way of roofing (the M5 diffusion brief
    // §3.4), in each life.
    for (life, l) in [("with trade", &trade_life), ("the twin", &twin_life)] {
        let contacts = &l.sim.people().contacts;
        let met = |a: PermanentId, b: PermanentId| {
            contacts
                .years
                .keys()
                .any(|&(_, f, t)| (f, t) == (a, b) || (f, t) == (b, a))
        };
        let ids: Vec<PermanentId> = l.sim.land().settlements.iter().map(|s| s.id).collect();
        let mut said = false;
        for &a in &ids {
            for &b in ids.iter().filter(|&&b| b != a) {
                if let Some(y) = l
                    .years
                    .iter()
                    .position(|y| took_way(&y.roofs, a, b, met(a, b)))
                {
                    writeln!(
                        out,
                        "roofs {life}: {} took {}'s way in year {}",
                        name(a),
                        name(b),
                        y + 1
                    )?;
                    said = true;
                }
            }
        }
        if !said {
            writeln!(
                out,
                "roofs {life}: no settlement took another's way (not reached)"
            )?;
        }
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
    // What crossed between the settlements each way, against the contact that could carry it
    // (M5b slice AR): red in either life is a leak, and fails the run.
    for (life, sim) in [
        ("with trade", &trade_life.sim),
        ("the twin", &twin_life.sim),
    ] {
        for p in crate::crossings::pairs(sim) {
            let (g, why) = p.grade();
            let c = |w: &crate::crossings::Way| {
                format!(
                    "{} contacts ({} through), {} buildings, {} admiring, {} brought, {} seen",
                    w.contact,
                    w.through,
                    w.crossed.buildings,
                    w.crossed.admired,
                    w.crossed.brought,
                    w.crossed.seen
                )
            };
            writeln!(
                out,
                "neighbours {life}, {}–{}: {} ({why}); to {}: {}; to {}: {}",
                name(p.a),
                name(p.b),
                g.word(),
                name(p.b),
                c(&p.to_b),
                name(p.a),
                c(&p.to_a)
            )?;
            if g == Grade::Red {
                outcome.failures.push(format!(
                    "neighbours {life}: {}–{} crossed without contact",
                    name(p.a),
                    name(p.b)
                ));
            }
        }
    }
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
    seen.roofs = roofs(sim);
    seen
}

/// Every settlement's roofs now ([`Roofs`]).
fn roofs(sim: &Sim) -> Vec<Roofs> {
    let pop = sim.people();
    let land = sim.land();
    let catalog = &sim.rules().catalog;
    let since = sim.now().minutes() - MINUTES_PER_YEAR;
    land.settlements
        .iter()
        .map(|s| {
            let clocks = pop.style_clocks(catalog, land, sim.now(), s.id);
            let mut after: BTreeMap<PermanentId, u32> = BTreeMap::new();
            for b in &land.buildings {
                if !(b.finished() && b.state == civ_land::BuildingState::Standing)
                    || b.stage_since.minutes() < since
                    || pop.building_settlement(land, b) != Some(s.id)
                {
                    continue;
                }
                let there = pop.crossing_of(land, b).and_then(|first| {
                    land.buildings
                        .iter()
                        .find(|x| x.id == first)
                        .and_then(|x| pop.building_settlement(land, x))
                });
                if let Some(there) = there {
                    *after.entry(there).or_default() += 1;
                }
            }
            Roofs {
                settlement: s.id,
                way: clocks.way.map(|w| f64::from(w.pitch_centideg) / 100.0),
                new: clocks.new.n,
                pitch: clocks.new.mean[0] / 100.0,
                after,
            }
        })
        .collect()
}

/// Whether `a` has taken `b`'s way of roofing in a year (the M5 diffusion brief §3.4): the year's
/// new buildings of `a` are pitched nearer `b`'s founding way than `a`'s own, at least one of them
/// follows a building of `b`, and there was contact between them on record.
fn took_way(roofs: &[Roofs], a: PermanentId, b: PermanentId, contact: bool) -> bool {
    let (Some(ra), Some(rb)) = (
        roofs.iter().find(|r| r.settlement == a),
        roofs.iter().find(|r| r.settlement == b),
    ) else {
        return false;
    };
    let (Some(own), Some(theirs)) = (ra.way, rb.way) else {
        return false;
    };
    contact
        && ra.new > 0
        && (ra.pitch - theirs).abs() < (ra.pitch - own).abs()
        && ra.after.get(&b).is_some_and(|&n| n > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(n: u64) -> PermanentId {
        PermanentId::from_raw(n).expect("an id")
    }

    fn roofs(s: u64, way: f64, new: usize, pitch: f64, after: &[(u64, u32)]) -> Roofs {
        Roofs {
            settlement: id(s),
            way: Some(way),
            new,
            pitch,
            after: after.iter().map(|&(x, n)| (id(x), n)).collect(),
        }
    }

    #[test]
    fn a_way_is_taken_only_nearer_the_other_s_founding_way_after_one_of_its_buildings_with_contact()
    {
        // The second settlement, founded at 45°, builds at 47.5° after a building of the first,
        // founded at 48°: nearer the first's way, with a followed link that crosses.
        let year = [
            roofs(1, 48.0, 2, 48.0, &[]),
            roofs(2, 45.0, 3, 47.5, &[(1, 1)]),
        ];
        assert!(took_way(&year, id(2), id(1), true));
        // Not without the contact on record, nor the other way about.
        assert!(!took_way(&year, id(2), id(1), false));
        assert!(!took_way(&year, id(1), id(2), true));
        // Nor nearer its own way, nor with no building following one of the first.
        let own = [
            roofs(1, 48.0, 2, 48.0, &[]),
            roofs(2, 45.0, 3, 46.0, &[(1, 1)]),
        ];
        assert!(!took_way(&own, id(2), id(1), true));
        let unlinked = [roofs(1, 48.0, 2, 48.0, &[]), roofs(2, 45.0, 3, 47.5, &[])];
        assert!(!took_way(&unlinked, id(2), id(1), true));
    }
}
