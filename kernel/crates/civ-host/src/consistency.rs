//! The statistical consistency test (ADR-0011 §5, Gate B): Accelerated mode, with the
//! approximations it declares (§4), against Detailed mode, from the same midnight.
//!
//! Each fixture world is founded with a fixed identity and lives by the minute to the first of
//! January of year [`RUN_YEAR`], where it is saved. From that save [`RUNS`] runs in each mode live
//! the year, each with the scheduler's tie-break stream redrawn (a test hook), so that they differ
//! as lives of one world do. Every run's year is checked as every long run's is, exactly: nobody
//! stuck, no problems, every good accounted for ([`LongRun::year_end`]). Then each aggregate's
//! mean over the Accelerated runs is compared with its mean over the Detailed runs, and the gate
//! fails where they differ by more than the aggregate's tolerance.
//!
//! The tolerances were set once, from Detailed calibration runs (`--calibrate`) and before
//! Accelerated mode was evaluated, and are recorded in PROJECT_PLAN §9. Widening one needs its own
//! entry there. Nothing here steers a world.

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use civ_agents::Behavior;
use civ_agents::params::GoodUse;
use civ_content::ContentRegistry;
use civ_core::SimTime;
use civ_schema::SAVE_EXTENSION;
use civ_sim::{Approximations, Mode, NewWorld, SPEED_1X, SPEED_MAX, Sim, persist};
use commons_persist::{SaveDir, SaveKind};
use serde::Serialize;

use crate::smoke::LongRun;

/// The fixture worlds, by preset and seed: two landscapes.
pub const FIXTURES: [(&str, u64); 2] = [
    ("core:worldgen/river_valley", 1),
    ("core:worldgen/ria_coast", 2),
];
/// Cells per side of each fixture (the smoke's).
pub const SIZE: u32 = 768;
/// The calendar year the runs live: each fixture is saved on its first of January.
pub const RUN_YEAR: i64 = 3;
/// Runs in each mode from each fixture.
pub const RUNS: usize = 5;
/// Detailed runs from each fixture when calibrating.
pub const CALIBRATION_RUNS: usize = 8;
/// Standard errors of the difference of two means of [`RUNS`] runs each, at the Detailed spread,
/// that a tolerance allows when calibrated.
pub const CALIBRATION_SIGMAS: f64 = 3.0;

/// One aggregate compared between the modes.
#[derive(Clone, Copy, Debug)]
pub struct Aggregate {
    /// Short name.
    pub name: &'static str,
    /// What it measures.
    pub what: &'static str,
    /// Largest difference of the means allowed, as a share of the Detailed mean (§9).
    pub tolerance: f64,
}

/// The aggregates, in the order a run reports them (ADR-0011 §5: time use, food, harvest,
/// materials, roofs, population, the Gini of goods and walks). Each tolerance was set from the
/// calibration of 2026-10-07 (PROJECT_PLAN §9), before Accelerated mode was evaluated: the larger
/// of the two fixtures' calibrated values, rounded up to a whole percent, and at least 3 % (10 %
/// for the share of households roofed, one household in about ten).
pub const AGGREGATES: [Aggregate; 10] = [
    Aggregate {
        name: "people",
        what: "people living at the year's end",
        tolerance: 0.03,
    },
    Aggregate {
        name: "work",
        what: "hours a person-day at work (all but sleep, meals, leisure and walking)",
        tolerance: 0.10,
    },
    Aggregate {
        name: "leisure",
        what: "hours a person-day resting, playing or at the hearth",
        tolerance: 0.03,
    },
    Aggregate {
        name: "sleep",
        what: "hours a person-day asleep",
        tolerance: 0.03,
    },
    Aggregate {
        name: "walking",
        what: "hours a person-day walking",
        tolerance: 0.09,
    },
    Aggregate {
        name: "food",
        what: "days of the people's need held as food at the year's end",
        tolerance: 0.04,
    },
    Aggregate {
        name: "harvest",
        what: "grain reaped in the year, kilograms a person",
        tolerance: 0.04,
    },
    Aggregate {
        name: "materials",
        what: "materials held at the year's end, kilograms a household",
        tolerance: 0.14,
    },
    Aggregate {
        name: "roofed",
        what: "share of households under a roof of their own at the year's end",
        tolerance: 0.10,
    },
    Aggregate {
        name: "gini",
        what: "Gini of goods of the largest settlement at the year's end",
        tolerance: 0.08,
    },
];

/// What to run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConsistencyOptions {
    /// Only Detailed runs, [`CALIBRATION_RUNS`] from each fixture, to set the tolerances.
    pub calibrate: bool,
    /// The approximations the Accelerated runs make: all Accelerated mode declares, for the gate
    /// itself; fewer to see what each one changes.
    pub approximations: Approximations,
}

/// One run of a year from a fixture.
#[derive(Clone, Debug, Serialize)]
pub struct Run {
    /// The fixture's index in [`FIXTURES`].
    pub fixture: usize,
    /// Whether it lived in Accelerated mode.
    pub accelerated: bool,
    /// The tie-break stream it drew.
    pub stream: u64,
    /// Seconds it took.
    pub seconds: f64,
    /// The aggregates, in [`AGGREGATES`] order.
    pub values: Vec<f64>,
    /// The checks it failed.
    pub failures: Vec<String>,
}

/// One aggregate of one fixture, compared.
#[derive(Clone, Debug, Serialize)]
pub struct Comparison {
    /// The fixture's index in [`FIXTURES`].
    pub fixture: usize,
    /// The aggregate's name.
    pub aggregate: &'static str,
    /// Mean and standard deviation over the Detailed runs.
    pub detailed: (f64, f64),
    /// Mean and standard deviation over the Accelerated runs (none when calibrating).
    pub accelerated: Option<(f64, f64)>,
    /// The difference of the means as a share of the Detailed mean.
    pub difference: Option<f64>,
    /// The tolerance it is held to.
    pub tolerance: f64,
    /// The tolerance the Detailed spread would set ([`CALIBRATION_SIGMAS`] standard errors of a
    /// difference of means of [`RUNS`] runs each), as a share of the Detailed mean.
    pub calibrated: f64,
    /// Whether it is within its tolerance (always, when calibrating).
    pub pass: bool,
}

/// The whole test.
#[derive(Clone, Debug, Serialize)]
pub struct Report {
    /// Whether this was a calibration.
    pub calibration: bool,
    /// People at each fixture's save.
    pub fixture_people: Vec<usize>,
    /// Every run.
    pub runs: Vec<Run>,
    /// Every aggregate of every fixture.
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

/// A fixture made: its save and its people, or why it could not be.
type Made = Result<(PathBuf, usize), String>;

/// Makes the fixture of `FIXTURES[index]` in `dir`: founded with a fixed identity, lived by the
/// minute to the first of January of [`RUN_YEAR`], and saved. Returns the save and its people.
fn fixture(
    content: &ContentRegistry,
    dir: &Path,
    index: usize,
) -> Result<(PathBuf, usize), String> {
    let (preset, seed) = FIXTURES[index];
    let mut identity = [0u8; 16];
    identity[..8].copy_from_slice(&0x6761_7465_625f_6230u64.to_le_bytes());
    identity[8..].copy_from_slice(&seed.to_le_bytes());
    let mut sim = Sim::create_for_tests(
        &NewWorld {
            name: format!("Gate B {}", index + 1),
            seed,
            preset_id: preset.to_owned(),
            size_cells: SIZE,
            band_size: 0,
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
    let start = SimTime::from_date(RUN_YEAR, 1, 1, 0, 0)
        .ok_or("the run year has no first of January")?
        .minutes();
    sim.advance_minutes(start - sim.now().minutes())
        .map_err(|e| e.to_string())?;
    let saves = SaveDir::create(dir, SAVE_EXTENSION).map_err(|e| e.to_string())?;
    let saved =
        persist::save(&mut sim, &saves, SaveKind::Manual, "gate b").map_err(|e| e.to_string())?;
    Ok((saved.path, sim.people().living()))
}

/// Lives a year from `save`, in Accelerated mode with `approximations` if any, else in Detailed
/// mode, with tie-break stream `stream`.
fn run(
    content: &ContentRegistry,
    save: &Path,
    fixture: usize,
    approximations: Option<Approximations>,
    stream: u64,
) -> Run {
    let accelerated = approximations.is_some();
    let started = Instant::now();
    let mut out = Run {
        fixture,
        accelerated,
        stream,
        seconds: 0.0,
        values: Vec::new(),
        failures: Vec::new(),
    };
    let mut sim = match persist::load(save, content) {
        Ok(sim) => sim,
        Err(e) => {
            out.failures.push(format!("the fixture did not load: {e}"));
            return out;
        }
    };
    sim.redraw_tiebreak_for_tests(stream);
    sim.set_approximations(approximations.unwrap_or(Approximations::NONE));
    let speed = if accelerated { SPEED_MAX } else { SPEED_1X };
    if let Err(e) = sim.set_speed(speed) {
        out.failures.push(e.to_string());
        return out;
    }
    let mode = if accelerated {
        Mode::Accelerated
    } else {
        Mode::Detailed
    };
    if sim.mode() != mode {
        out.failures
            .push(format!("the run began in {:?} mode", sim.mode()));
        return out;
    }
    let mut long = LongRun::begin(&sim);
    let end = SimTime::from_date(RUN_YEAR + 1, 1, 1, 0, 0).map_or(0, |t| t.minutes());
    if let Err(e) = long.live_to(&mut sim, end) {
        out.failures.push(format!("the clock stopped: {e}"));
        return out;
    }
    out.failures = long.year_end(&sim, u32::try_from(RUN_YEAR).unwrap_or(3));
    out.values = aggregates(&sim);
    out.seconds = started.elapsed().as_secs_f64();
    out
}

/// A world's aggregates at the end of its run's year, in [`AGGREGATES`] order.
fn aggregates(sim: &Sim) -> Vec<f64> {
    let people = sim.people();
    let rules = sim.rules();
    let goods = &rules.catalog.goods;
    let living = people.living();
    // Time use: the minutes of every step finished in the run, so that a person-day is 24 hours
    // of them.
    let t = &people.time_use;
    let minutes = |b: Behavior| {
        Behavior::ALL
            .iter()
            .position(|&x| x == b)
            .and_then(|i| t.work.get(i))
            .copied()
            .unwrap_or(0.0)
    };
    let total: f64 = t.work.iter().sum::<f64>() + t.walking + t.waiting;
    let hours = |m: f64| if total > 0.0 { 24.0 * m / total } else { 0.0 };
    let leisure = minutes(Behavior::Rest) + minutes(Behavior::Play) + minutes(Behavior::Socialize);
    let sleep = minutes(Behavior::Sleep);
    let eat = minutes(Behavior::Eat);
    let work = t.work.iter().sum::<f64>() - leisure - sleep - eat;
    // What they hold.
    let held = people.goods_held();
    let food_kcal: f64 = held
        .iter()
        .zip(goods)
        .filter(|(_, g)| g.purpose == GoodUse::Food)
        .map(|(kg, g)| kg.max(0.0) * g.kcal_per_kg)
        .sum();
    let need = living as f64 * rules.people.household.daily_kcal_per_person;
    let materials: f64 = held
        .iter()
        .zip(goods)
        .filter(|(_, g)| g.purpose == GoodUse::Material)
        .map(|(kg, _)| kg.max(0.0))
        .sum();
    let households: Vec<_> = people
        .households
        .iter()
        .filter(|(_, h)| !h.members.is_empty())
        .map(|(_, h)| h.id)
        .collect();
    let land = sim.land();
    let roofed = households
        .iter()
        .filter(|&&h| {
            land.buildings.iter().any(|b| {
                b.household == h && b.roofed() && rules.catalog.is_dwelling(&b.spec.program)
            })
        })
        .count();
    let year = i32::try_from(RUN_YEAR).unwrap_or(3);
    let reaped: f64 = land
        .fields
        .iter()
        .map(|f| {
            f.soil
                .record
                .iter()
                .filter(|r| r.year == year)
                .map(|r| f64::from(r.kg_per_ha) * f.area_ha())
                .sum::<f64>()
        })
        .sum();
    let gini = people
        .wealth_years
        .iter()
        .filter(|w| w.year == RUN_YEAR)
        .max_by_key(|w| w.spread.people)
        .map_or(0.0, |w| w.spread.gini_goods);
    let per = |x: f64, n: usize| if n > 0 { x / n as f64 } else { 0.0 };
    vec![
        living as f64,
        hours(work),
        hours(leisure),
        hours(sleep),
        hours(t.walking),
        if need > 0.0 { food_kcal / need } else { 0.0 },
        per(reaped, living),
        per(materials, households.len()),
        per(roofed as f64, households.len()),
        gini,
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

/// Runs the gate (or its calibration) on `threads` threads, printing each run as it ends.
pub fn run_gate(
    content: &ContentRegistry,
    options: ConsistencyOptions,
    dir: &Path,
    threads: usize,
) -> Result<Report, String> {
    let started = Instant::now();
    // The fixtures, made side by side.
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
            "fixture {}: {preset} seed {seed}, {people} people on 1 January of year {RUN_YEAR}",
            i + 1
        );
    }
    // The runs: each fixture's Detailed runs and (unless calibrating) its Accelerated ones.
    let runs_each = if options.calibrate {
        CALIBRATION_RUNS
    } else {
        RUNS
    };
    let mut jobs = Vec::new();
    for f in 0..fixtures.len() {
        for k in 0..runs_each {
            jobs.push((f, false, k as u64 + 1));
            if !options.calibrate {
                jobs.push((f, true, k as u64 + 1));
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
                    let Some(&(f, accelerated, stream)) = jobs.get(j) else {
                        return;
                    };
                    let approximations = accelerated.then_some(options.approximations);
                    let r = run(content, &fixtures[f].0, f, approximations, stream);
                    println!(
                        "  fixture {} {} run {}: {:.0} s{}",
                        f + 1,
                        if accelerated {
                            "Accelerated"
                        } else {
                            "Detailed"
                        },
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
    runs.sort_by_key(|r| (r.fixture, r.accelerated, r.stream));
    // The comparisons.
    let mut comparisons = Vec::new();
    for f in 0..fixtures.len() {
        for (a, agg) in AGGREGATES.iter().enumerate() {
            let values = |accelerated: bool| -> Vec<f64> {
                runs.iter()
                    .filter(|r| r.fixture == f && r.accelerated == accelerated)
                    .filter_map(|r| r.values.get(a).copied())
                    .collect()
            };
            let detailed = spread(&values(false));
            let scale = detailed.0.abs().max(1e-9);
            let calibrated = CALIBRATION_SIGMAS * detailed.1 * (2.0 / RUNS as f64).sqrt() / scale;
            let accelerated = (!options.calibrate).then(|| spread(&values(true)));
            let difference = accelerated.map(|acc| (acc.0 - detailed.0) / scale);
            comparisons.push(Comparison {
                fixture: f,
                aggregate: agg.name,
                detailed,
                accelerated,
                difference,
                tolerance: agg.tolerance,
                calibrated,
                pass: difference.is_none_or(|d| d.abs() <= agg.tolerance),
            });
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
            "aggregate", "Detailed", "Accelerated", "difference", "tolerance", "calibrated"
        );
        for c in report.comparisons.iter().filter(|c| c.fixture == f) {
            let acc = c
                .accelerated
                .map_or("-".to_owned(), |(m, s)| format!("{m:.3} ± {s:.3}"));
            let diff = c
                .difference
                .map_or("-".to_owned(), |d| format!("{:+.1} %", 100.0 * d));
            println!(
                "  {:<10} {:>18} {:>18} {:>10} {:>9.1}% {:>9.1}%{}",
                c.aggregate,
                format!("{:.3} ± {:.3}", c.detailed.0, c.detailed.1),
                acc,
                diff,
                100.0 * c.tolerance,
                100.0 * c.calibrated,
                if c.pass { "" } else { "  OUT OF TOLERANCE" }
            );
        }
    }
    println!();
    if report.calibration {
        println!("calibration only: no Accelerated runs, nothing graded");
    } else if report.passed() {
        println!("Gate B passed");
    } else {
        println!("Gate B FAILED");
    }
}
