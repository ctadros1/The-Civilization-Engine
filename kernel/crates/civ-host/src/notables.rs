//! The notables' gate (ADR-0014 §4): the notables' tier, which lets only the notables and those
//! an issue reaches weigh institutional moves at the weekly review, against every adult weighing
//! them.
//!
//! Each fixture world (Gate B's two landscapes) is founded with a fixed identity, lives its first
//! month by the minute, and is saved at a midnight, its polity founded. From that save [`RUNS`]
//! runs with the tier and [`RUNS`] without it live at Max to the end of year [`LAST_YEAR`], each
//! with the scheduler's tie-break stream redrawn, and each checked at every year's end as every
//! long run is. Then each polity aggregate's mean without the tier is compared with its mean with
//! it, and the gate fails where they differ by more than the aggregate's tolerance: a share of the
//! larger of the mean with the tier and the aggregate's floor (counts this small need one).
//!
//! The tolerances were set once, from runs with the tier only (`--calibrate`), before any run
//! without it, and are recorded in PROJECT_PLAN §9. Nothing here steers a world.

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use civ_agents::polity::{Law, Outcome};
use civ_content::ContentRegistry;
use civ_core::SimTime;
use civ_schema::SAVE_EXTENSION;
use civ_sim::{NewWorld, SPEED_MAX, Sim, persist};
use commons_persist::{SaveDir, SaveKind};
use serde::Serialize;

use crate::consistency::{FIXTURES, SIZE};
use crate::smoke::LongRun;

/// The last calendar year the runs live: from the first month to its end.
pub const LAST_YEAR: i64 = 3;
/// Runs with the tier and without it from each fixture.
pub const RUNS: usize = 5;
/// Runs with the tier from each fixture when calibrating.
pub const CALIBRATION_RUNS: usize = 8;
/// Standard errors of the difference of two means of [`RUNS`] runs each, at the spread with the
/// tier, that a tolerance allows when calibrated.
pub const CALIBRATION_SIGMAS: f64 = 3.0;

/// One polity aggregate compared with the tier and without it.
#[derive(Clone, Copy, Debug)]
pub struct Aggregate {
    /// Short name.
    pub name: &'static str,
    /// What it measures.
    pub what: &'static str,
    /// The least scale a difference is a share of: a count this small needs one.
    pub floor: f64,
    /// Largest difference of the means allowed, as a share of the larger of the mean with the
    /// tier and the floor (§9).
    pub tolerance: f64,
}

/// The aggregates ADR-0014 §4 names, in the order a run reports them. The tolerances were set from
/// the calibration of 2026-10-08 (PROJECT_PLAN §9), at three standard errors of a difference of
/// two means, rounded up to the next five percent.
pub const AGGREGATES: [Aggregate; 4] = [
    Aggregate {
        name: "proposals",
        what: "laws proposed",
        floor: 1.0,
        tolerance: 0.55,
    },
    Aggregate {
        name: "enacted",
        what: "laws a gathering passed",
        floor: 1.0,
        tolerance: 0.40,
    },
    Aggregate {
        name: "offices",
        what: "offices filled: laws passed that name a holder",
        floor: 1.0,
        tolerance: 0.75,
    },
    Aggregate {
        name: "moved",
        what: "the share of the food reaped that the polity levied or gave in relief",
        floor: 0.01,
        tolerance: 0.25,
    },
];

/// What the gate does.
#[derive(Clone, Copy, Debug, Default)]
pub struct NotablesOptions {
    /// Only runs with the tier, [`CALIBRATION_RUNS`] from each fixture, to set the tolerances.
    pub calibrate: bool,
}

/// One run: a fixture lived with the tier or without it.
#[derive(Clone, Debug, Serialize)]
pub struct Run {
    /// The fixture, an index into [`FIXTURES`].
    pub fixture: usize,
    /// Whether the notables' tier was on.
    pub tier: bool,
    /// The tie-break stream.
    pub stream: u64,
    /// Seconds it took.
    pub seconds: f64,
    /// Its aggregates, in [`AGGREGATES`] order.
    pub values: Vec<f64>,
    /// The checks it failed.
    pub failures: Vec<String>,
    /// Each office's holders, for reading a difference (not graded): "day 412: Ada named by
    /// themselves; lapsed: left the valley".
    pub offices: Vec<String>,
}

/// One aggregate of one fixture, with the tier and without it.
#[derive(Clone, Debug, Serialize)]
pub struct Comparison {
    /// The fixture.
    pub fixture: usize,
    /// The aggregate's name.
    pub aggregate: &'static str,
    /// Mean and standard deviation with the tier.
    pub tier: (f64, f64),
    /// Mean and standard deviation without it (none when calibrating).
    pub every_adult: Option<(f64, f64)>,
    /// The difference of the means, as a share of the scale.
    pub difference: Option<f64>,
    /// The tolerance it is held to.
    pub tolerance: f64,
    /// The tolerance the spread with the tier would set ([`CALIBRATION_SIGMAS`] standard errors of
    /// a difference of two means of [`RUNS`] runs, as a share of the scale).
    pub calibrated: f64,
    /// Whether it is within its tolerance (always, when calibrating).
    pub pass: bool,
}

/// The gate's report.
#[derive(Clone, Debug, Serialize)]
pub struct Report {
    /// Whether this was a calibration.
    pub calibration: bool,
    /// The people of each fixture when saved.
    pub fixture_people: Vec<usize>,
    /// Every run.
    pub runs: Vec<Run>,
    /// Every comparison.
    pub comparisons: Vec<Comparison>,
    /// Seconds in all.
    pub seconds: f64,
}

impl Report {
    /// Whether every run passed its checks and every aggregate is within its tolerance.
    pub fn passed(&self) -> bool {
        self.runs.iter().all(|r| r.failures.is_empty()) && self.comparisons.iter().all(|c| c.pass)
    }
}

type Made = Result<(PathBuf, usize), String>;

/// Makes the fixture of `FIXTURES[index]` in `dir`: founded with a fixed identity, its first month
/// lived by the minute to a midnight, and saved. Returns the save and its people.
fn fixture(content: &ContentRegistry, dir: &Path, index: usize) -> Made {
    let (preset, seed) = FIXTURES[index];
    let mut identity = [0u8; 16];
    identity[..8].copy_from_slice(&0x6761_7465_5f6e_6f74u64.to_le_bytes());
    identity[8..].copy_from_slice(&seed.to_le_bytes());
    let mut sim = Sim::create_for_tests(
        &NewWorld {
            name: format!("Notables {}", index + 1),
            seed,
            preset_id: preset.to_owned(),
            size_cells: SIZE,
            band_size: 0,
            neighbours: Vec::new(),
            neighbours_known: false,
            regime_id: String::new(),
        },
        content,
        identity,
    )
    .map_err(|e| e.to_string())?;
    if let Some(problem) = sim.founding_problem() {
        return Err(format!(
            "{preset} {seed}: the band could not settle: {problem}"
        ));
    }
    sim.advance_minutes(30 * 1440).map_err(|e| e.to_string())?;
    sim.advance_to_midnight().map_err(|e| e.to_string())?;
    let saves = SaveDir::create(dir, SAVE_EXTENSION).map_err(|e| e.to_string())?;
    let saved =
        persist::save(&mut sim, &saves, SaveKind::Manual, "notables").map_err(|e| e.to_string())?;
    Ok((saved.path, sim.people().living()))
}

/// Lives `save` at Max to the end of [`LAST_YEAR`], with the notables' tier on or off, with
/// tie-break stream `stream`.
fn run(content: &ContentRegistry, save: &Path, fixture: usize, tier: bool, stream: u64) -> Run {
    let started = Instant::now();
    let mut out = Run {
        fixture,
        tier,
        stream,
        seconds: 0.0,
        values: Vec::new(),
        failures: Vec::new(),
        offices: Vec::new(),
    };
    let mut sim = match persist::load(save, content) {
        Ok(sim) => sim,
        Err(e) => {
            out.failures.push(format!("the fixture did not load: {e}"));
            return out;
        }
    };
    sim.redraw_tiebreak_for_tests(stream);
    sim.set_notable_tier(tier);
    if let Err(e) = sim.set_speed(SPEED_MAX) {
        out.failures.push(e.to_string());
        return out;
    }
    let mut long = LongRun::begin(&sim);
    let first = sim.now().date().year;
    for year in first..=LAST_YEAR {
        let end = SimTime::from_date(year + 1, 1, 1, 0, 0).map_or(0, |t| t.minutes());
        if let Err(e) = long.live_to(&mut sim, end) {
            out.failures.push(format!("the clock stopped: {e}"));
            return out;
        }
        out.failures
            .extend(long.year_end(&sim, u32::try_from(year).unwrap_or(1)));
    }
    out.values = aggregates(&sim);
    out.offices = holders(&sim);
    out.seconds = started.elapsed().as_secs_f64();
    out
}

/// Each office's holders in the order named, and how each held it.
fn holders(sim: &Sim) -> Vec<String> {
    let pop = sim.people();
    pop.polities
        .iter()
        .flat_map(|p| &p.laws)
        .filter(|l| l.outcome == Some(Outcome::Passed))
        .filter_map(|l| {
            let h = l.holder?;
            let by = if l.sponsor == h {
                "themselves".to_owned()
            } else {
                pop.name_of(l.sponsor)
            };
            let how = match pop.records.get(&h).map(|r| (r.died, r.left)) {
                Some((Some(_), _)) => "died",
                Some((_, Some(_))) => "left the valley",
                _ => "holds it",
            };
            let day = l.decided.map_or(0, SimTime::day_index);
            Some(format!(
                "day {day}: {} named by {by}; {:?}, {how}",
                pop.name_of(h),
                l.status
            ))
        })
        .collect()
}

/// A world's polity aggregates at the end of its run, in [`AGGREGATES`] order.
fn aggregates(sim: &Sim) -> Vec<f64> {
    let laws: Vec<&Law> = sim.people().polities.iter().flat_map(|p| &p.laws).collect();
    let passed: Vec<&&Law> = laws
        .iter()
        .filter(|l| l.outcome == Some(Outcome::Passed))
        .collect();
    let moved_kg: f64 = laws
        .iter()
        .map(|l| l.compliance.levied_kg + l.compliance.relief_kg)
        .sum();
    let reaped_kg: f64 = sim
        .land()
        .fields
        .iter()
        .map(|f| {
            f.soil
                .record
                .iter()
                .map(|r| f64::from(r.kg_per_ha) * f.area_ha())
                .sum::<f64>()
        })
        .sum();
    vec![
        laws.len() as f64,
        passed.len() as f64,
        passed.iter().filter(|l| l.holder.is_some()).count() as f64,
        if reaped_kg > 0.0 {
            (moved_kg / reaped_kg).max(0.0)
        } else {
            0.0
        },
    ]
}

/// Mean and standard deviation (with Bessel's correction).
fn spread(xs: &[f64]) -> (f64, f64) {
    let n = xs.len() as f64;
    if xs.is_empty() {
        return (0.0, 0.0);
    }
    let mean = xs.iter().sum::<f64>() / n;
    if xs.len() < 2 {
        return (mean, 0.0);
    }
    let var = xs.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0);
    (mean, var.sqrt())
}

/// The comparison of one aggregate's runs with the tier (`tier`) and without it (`every_adult`,
/// none when calibrating).
fn compare(
    fixture: usize,
    agg: &Aggregate,
    tier: &[f64],
    every_adult: Option<&[f64]>,
) -> Comparison {
    let with = spread(tier);
    let scale = with.0.abs().max(agg.floor);
    let calibrated = CALIBRATION_SIGMAS * with.1 * (2.0 / RUNS as f64).sqrt() / scale;
    let without = every_adult.map(spread);
    let difference = without.map(|w| (w.0 - with.0) / scale);
    Comparison {
        fixture,
        aggregate: agg.name,
        tier: with,
        every_adult: without,
        difference,
        tolerance: agg.tolerance,
        calibrated,
        pass: difference.is_none_or(|d| d.abs() <= agg.tolerance),
    }
}

/// Runs the gate (or its calibration) on `threads` threads, printing each run as it ends.
pub fn run_gate(
    content: &ContentRegistry,
    options: NotablesOptions,
    dir: &Path,
    threads: usize,
) -> Result<Report, String> {
    let started = Instant::now();
    let made: Vec<Mutex<Option<Made>>> = FIXTURES.iter().map(|_| Mutex::new(None)).collect();
    std::thread::scope(|s| {
        for (i, slot) in made.iter().enumerate() {
            let sub = dir.join(format!("fixture-{}", i + 1));
            s.spawn(move || {
                let r = fixture(content, &sub, i);
                if let Ok(mut slot) = slot.lock() {
                    *slot = Some(r);
                }
            });
        }
    });
    let mut fixtures = Vec::new();
    for slot in made {
        let made = slot
            .into_inner()
            .map_err(|_| "a fixture thread failed".to_owned())?
            .ok_or("a fixture was not made")??;
        fixtures.push(made);
    }
    for (i, (_, people)) in fixtures.iter().enumerate() {
        let (preset, seed) = FIXTURES[i];
        println!(
            "fixture {}: {preset} seed {seed}, {people} people after their first month",
            i + 1
        );
    }
    let runs_each = if options.calibrate {
        CALIBRATION_RUNS
    } else {
        RUNS
    };
    let mut jobs = Vec::new();
    for f in 0..fixtures.len() {
        for k in 0..runs_each {
            jobs.push((f, true, k as u64 + 1));
            if !options.calibrate {
                jobs.push((f, false, k as u64 + 1));
            }
        }
    }
    let next = AtomicUsize::new(0);
    let done: Mutex<Vec<Run>> = Mutex::new(Vec::new());
    std::thread::scope(|s| {
        for _ in 0..threads.max(1) {
            s.spawn(|| {
                loop {
                    let j = next.fetch_add(1, Ordering::Relaxed);
                    let Some(&(f, tier, stream)) = jobs.get(j) else {
                        return;
                    };
                    let r = run(content, &fixtures[f].0, f, tier, stream);
                    println!(
                        "  fixture {} {} run {}: {:.0} s{}",
                        f + 1,
                        if tier { "with the tier" } else { "every adult" },
                        stream,
                        r.seconds,
                        if r.failures.is_empty() {
                            String::new()
                        } else {
                            format!(", FAILED: {}", r.failures.join("; "))
                        }
                    );
                    if let Ok(mut d) = done.lock() {
                        d.push(r);
                    }
                }
            });
        }
    });
    let mut runs = done
        .into_inner()
        .map_err(|_| "a run thread failed".to_owned())?;
    runs.sort_by_key(|r| (r.fixture, !r.tier, r.stream));
    let mut comparisons = Vec::new();
    for f in 0..fixtures.len() {
        for (a, agg) in AGGREGATES.iter().enumerate() {
            let values = |tier: bool| -> Vec<f64> {
                runs.iter()
                    .filter(|r| r.fixture == f && r.tier == tier)
                    .filter_map(|r| r.values.get(a).copied())
                    .collect()
            };
            let without = (!options.calibrate).then(|| values(false));
            comparisons.push(compare(f, agg, &values(true), without.as_deref()));
        }
    }
    Ok(Report {
        calibration: options.calibrate,
        fixture_people: fixtures.iter().map(|f| f.1).collect(),
        runs,
        comparisons,
        seconds: started.elapsed().as_secs_f64(),
    })
}

/// The report in words.
pub fn print(report: &Report) {
    let failed = report
        .runs
        .iter()
        .filter(|r| !r.failures.is_empty())
        .count();
    println!();
    println!(
        "{} runs in {:.1} minutes; {} failed their checks",
        report.runs.len(),
        report.seconds / 60.0,
        failed
    );
    for (f, (preset, seed)) in FIXTURES
        .iter()
        .enumerate()
        .take(report.fixture_people.len())
    {
        println!();
        println!("fixture {}: {preset} seed {seed}", f + 1);
        println!(
            "  {:<10} {:>18} {:>18} {:>10} {:>10} {:>10}",
            "aggregate", "with the tier", "every adult", "difference", "tolerance", "calibrated"
        );
        for c in report.comparisons.iter().filter(|c| c.fixture == f) {
            let without = c
                .every_adult
                .map_or("-".to_owned(), |(m, s)| format!("{m:.3} ± {s:.3}"));
            let diff = c
                .difference
                .map_or("-".to_owned(), |d| format!("{:+.1} %", 100.0 * d));
            println!(
                "  {:<10} {:>18} {:>18} {:>10} {:>9.1}% {:>9.1}%{}",
                c.aggregate,
                format!("{:.3} ± {:.3}", c.tier.0, c.tier.1),
                without,
                diff,
                100.0 * c.tolerance,
                100.0 * c.calibrated,
                if c.pass { "" } else { "  OUT OF TOLERANCE" }
            );
        }
    }
    println!();
    if report.calibration {
        println!("calibration only: no runs without the tier, nothing graded");
    } else if report.passed() {
        println!("The notables' gate passed");
    } else {
        println!("The notables' gate FAILED");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_difference_is_a_share_of_the_larger_of_the_mean_and_its_floor() {
        let offices = &AGGREGATES[2];
        // Means of 0.2 and 0.6 offices: the floor of one office sets the scale.
        let c = compare(
            0,
            offices,
            &[0.0, 0.0, 1.0, 0.0, 0.0],
            Some(&[1.0, 0.0, 1.0, 1.0, 0.0]),
        );
        assert!((c.difference.expect("graded") - 0.4).abs() < 1e-12);
        assert!(c.pass, "0.4 is within {}", offices.tolerance);
        // Calibrating grades nothing.
        let c = compare(0, offices, &[1.0, 2.0, 3.0], None);
        assert!(c.pass && c.difference.is_none() && c.calibrated > 0.0);
        // Proposals of 4 and 8: the mean with the tier again, beyond a tolerance of a half.
        let tight = Aggregate {
            tolerance: 0.5,
            ..AGGREGATES[0]
        };
        let c = compare(0, &tight, &[4.0; 5], Some(&[8.0; 5]));
        assert!(!c.pass);
    }
}
