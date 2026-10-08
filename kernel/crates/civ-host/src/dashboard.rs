//! The sanity dashboard (plan §4.7, M3c slice T): five river-valley worlds lived fifty years and
//! graded against fixed thresholds, with no statistical inference.
//!
//! Each world lives its first month by the minute, as the smoke's does (its checks need it), then
//! a day at a time at Max (ADR-0011). Every year's end makes the checks every long run makes
//! ([`smoke::LongRun`]); any that fails fails the dashboard. At the end the rows of §4.7 that apply
//! at M3c are graded:
//!
//! - **Population:** at least [`KEEPING_WORLDS`] of the five keep [`KEEP`] people at the end, and
//!   no band ever grows past `max(3, 1.05^y)` times its founders by the end of year `y`.
//! - **Food prices:** in every world with harvests enough to judge, food stocks rose after a
//!   harvest at least once and grain was asked more for before the harvest than after
//!   ([`crate::economy`]).
//! - **Wealth Gini:** the median Gini of goods over years 26–50 of each settlement still lived in
//!   by [`KEEP`]: green within 0.3–0.75; amber from 0.1, as hoe farmers with land to spare may be
//!   (§9, 2026-10-05); red beyond.
//! - **Firm sizes:** in every world with workshops enough to judge, the mean of their lifetime
//!   hours above the median.
//! - **Structural failures:** failures of buildings lived in, per 1,000 years they stood lived
//!   in, at most [`MAX_FAILURES_PER_1000`]. Those of buildings nobody lives in are counted apart.
//!
//! The rest of §4.7 is grey, each with its reason, and never counts as a pass. Five worlds and fifty
//! years are fixed, so the dashboard cannot grow into a campaign, and a failing run is not rerun
//! until it passes (research 16-03 §4.7). Nothing here steers a world.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Instant;

use civ_content::ContentRegistry;
use civ_core::time::MINUTES_PER_YEAR;
use civ_schema::SAVE_EXTENSION;
use civ_sim::{NewWorld, SPEED_MAX, Sim, persist};
use commons_persist::{SaveDir, SaveKind};
use serde::Serialize;

use crate::economy::Grade;
use crate::smoke::{self, LongRun};

/// The landscape the worlds are made from.
pub const PRESET: &str = "core:worldgen/river_valley";
/// Worlds: seeds 1 to 5.
pub const WORLDS: u64 = 5;
/// Years each world lives after its first month.
pub const YEARS: u32 = 50;
/// People a world must keep at the end to count as keeping its band.
pub const KEEP: usize = smoke::MIN_ALIVE;
/// Worlds of the five that must keep their bands.
pub const KEEPING_WORLDS: usize = 3;
/// Growth allowed by the end of year `y`, as a multiple of the founders: the larger of this and
/// [`GROWTH_RATE`] to the power `y` (tuning; research 05-06 §5.2, and 05-01's Hutterites, whose
/// 4.15 % a year was exceptional).
pub const GROWTH_FLOOR: f64 = 3.0;
/// See [`GROWTH_FLOOR`].
pub const GROWTH_RATE: f64 = 1.05;
/// The calendar years whose Gini of goods is judged, by their median (the world's first is 1).
pub const GINI_YEARS: (i64, i64) = (26, 50);
/// Least years of that window a settlement must be lived in by [`KEEP`] for its Gini to count
/// (tuning).
pub const GINI_MIN_YEARS: usize = 5;
/// The Gini of goods expected of farming communities (plan §4.7).
pub const GINI_BAND: (f64, f64) = (0.3, 0.75);
/// Below this the Gini is no longer what hoe farmers with land to spare may show, but more likely
/// a fault in sharing (§9, 2026-10-05; tuning: the research gives no floor).
pub const GINI_FLOOR: f64 = 0.1;
/// Most failures of buildings lived in, per 1,000 years they stood lived in (research 11-06
/// §2.4's top scenario, one, doubled to allow for chance: tuning).
pub const MAX_FAILURES_PER_1000: f64 = 2.0;

/// What to run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DashboardOptions {
    /// Cells per side of each world.
    pub size: u32,
    /// Years each world lives after its first month: [`YEARS`] for the dashboard itself, fewer
    /// only for a quick look, which grades the rows over what it saw.
    pub years: u32,
}

/// A graded check and what was seen, in words.
pub type Seen = (Grade, String);

/// One world's run.
#[derive(Clone, Debug, Default, Serialize)]
pub struct WorldRun {
    /// Seed.
    pub seed: u64,
    /// The property regime it lived under.
    pub regime: String,
    /// Seconds to make and live it.
    pub seconds: f64,
    /// The founding band.
    pub founders: usize,
    /// People at the end of each year, from the first.
    pub people: Vec<usize>,
    /// Each calendar year's Gini of goods of its largest settlement, with that settlement's
    /// people (the year the world began is 1).
    pub gini: Vec<(i64, f64, u32)>,
    /// Months buildings lived in stood.
    pub lived_building_months: u64,
    /// Failures of buildings lived in.
    pub lived_failures: u32,
    /// What gave way in buildings lived in, in the chronicle's words, by calendar year.
    pub lived_failed: Vec<(i64, String)>,
    /// Failures of buildings nobody lived in.
    pub empty_failures: u32,
    /// Food stocks after the harvest, as graded.
    pub stocks: Option<Seen>,
    /// Grain asked before the harvest against after, as graded.
    pub asks: Option<Seen>,
    /// How the workshops' sizes spread, as graded.
    pub workshops: Option<Seen>,
    /// The checks it failed.
    pub failures: Vec<String>,
    /// How its band ended, if it fell below [`KEEP`].
    pub ended: Option<String>,
    /// The saves kept at each tenth year's end.
    pub saves: Vec<PathBuf>,
    /// What each settlement's polity would be called at the end, with what qualifies it
    /// (ADR-0013 §6).
    pub labels: Vec<String>,
}

impl WorldRun {
    /// People at the end of the run.
    pub fn living(&self) -> usize {
        self.people.last().copied().unwrap_or(0)
    }
}

/// One row of §4.7.
#[derive(Clone, Debug, Serialize)]
pub struct Row {
    /// What is checked.
    pub name: &'static str,
    /// How it is measured.
    pub measure: &'static str,
    /// What passes.
    pub threshold: &'static str,
    /// Where the threshold comes from.
    pub source: &'static str,
    /// How it came out.
    pub grade: Grade,
    /// What was seen, or why the row does not apply yet.
    pub text: String,
}

impl Row {
    fn new(name: &'static str, measure: &'static str, threshold: &'static str) -> Row {
        Row {
            name,
            measure,
            threshold,
            source: "",
            grade: Grade::Gray,
            text: String::new(),
        }
    }

    fn source(mut self, source: &'static str) -> Row {
        self.source = source;
        self
    }

    fn graded(mut self, grade: Grade, text: String) -> Row {
        self.grade = grade;
        self.text = text;
        self
    }
}

/// The whole dashboard.
#[derive(Clone, Debug, Serialize)]
pub struct Dashboard {
    /// The engine's version.
    pub version: &'static str,
    /// The content's fingerprint.
    pub content: String,
    /// How the worlds advanced.
    pub mode: &'static str,
    /// The landscape.
    pub preset: &'static str,
    /// Cells per side.
    pub size: u32,
    /// Years each world lived.
    pub years: u32,
    /// Seconds the whole took.
    pub seconds: f64,
    /// The worlds, by seed.
    pub worlds: Vec<WorldRun>,
    /// The rows of §4.7.
    pub rows: Vec<Row>,
}

impl Dashboard {
    /// Whether it passed: no red row, and no world failed a check.
    pub fn passed(&self) -> bool {
        !self.rows.iter().any(|r| r.grade == Grade::Red)
            && self.worlds.iter().all(|w| w.failures.is_empty())
    }

    /// The rows counted by grade, a grey row never as a pass: "3 passed, 1 amber, 4 not yet
    /// applicable, 1 failed".
    pub fn summary(&self) -> String {
        let count = |g: Grade| self.rows.iter().filter(|r| r.grade == g).count();
        let mut parts = vec![format!("{} passed", count(Grade::Green))];
        if count(Grade::Amber) > 0 {
            parts.push(format!("{} amber", count(Grade::Amber)));
        }
        parts.push(format!("{} not yet applicable", count(Grade::Gray)));
        if count(Grade::Red) > 0 {
            parts.push(format!("{} failed", count(Grade::Red)));
        }
        let failed = self
            .worlds
            .iter()
            .filter(|w| !w.failures.is_empty())
            .count();
        if failed > 0 {
            parts.push(format!(
                "{failed} of {} worlds failed a check",
                self.worlds.len()
            ));
        }
        parts.join(", ")
    }
}

/// Runs the five worlds, as many at a time as there are cores, reporting each as it finishes;
/// keeps each one's saves at every tenth year's end under `saves` if given; and grades them.
pub fn run(
    content: &ContentRegistry,
    options: DashboardOptions,
    saves: Option<&Path>,
    report: &(dyn Fn(&WorldRun) + Sync),
) -> Dashboard {
    let started = Instant::now();
    let next = AtomicUsize::new(0);
    let results = Mutex::new(Vec::new());
    let workers = std::thread::available_parallelism()
        .map_or(1, |n| n.get())
        .min(WORLDS as usize)
        .max(1);
    std::thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| {
                loop {
                    let i = next.fetch_add(1, Ordering::Relaxed) as u64;
                    if i >= WORLDS {
                        break;
                    }
                    let world = run_one(content, options, i + 1, saves);
                    report(&world);
                    results
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .push(world);
                }
            });
        }
    });
    let mut worlds = results
        .into_inner()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    worlds.sort_by_key(|w| w.seed);
    let rows = grade(&worlds, options.years);
    Dashboard {
        version: env!("CARGO_PKG_VERSION"),
        content: content.fingerprint_hex(),
        mode: "the first month Detailed, then Accelerated at Max",
        preset: PRESET,
        size: options.size,
        years: options.years,
        seconds: started.elapsed().as_secs_f64(),
        worlds,
        rows,
    }
}

fn run_one(
    content: &ContentRegistry,
    options: DashboardOptions,
    seed: u64,
    saves: Option<&Path>,
) -> WorldRun {
    let started = Instant::now();
    // The content's regimes in turn, by seed, as the smoke takes them.
    let regimes = &content.catalog.regimes;
    let regime = regimes
        .get((seed.max(1) - 1) as usize % regimes.len().max(1))
        .map_or_else(
            || (String::new(), String::new()),
            |r| (r.id.clone(), r.name.clone()),
        );
    let mut world = WorldRun {
        seed,
        regime: regime.1,
        ..WorldRun::default()
    };
    let created = Sim::create(
        &NewWorld {
            name: format!("Dashboard {seed}"),
            seed,
            preset_id: PRESET.to_owned(),
            size_cells: options.size,
            band_size: 0,
            regime_id: regime.0,
        },
        content,
        &mut |_| {},
        &AtomicBool::new(false),
    );
    match created {
        Err(e) => world.failures.push(format!("generation failed: {e}")),
        Ok(mut sim) => {
            world
                .failures
                .extend(smoke::check_people(&mut sim, content));
            if world.failures.is_empty() {
                live(&mut sim, options.years, saves, &mut world);
            }
        }
    }
    world.seconds = started.elapsed().as_secs_f64();
    world
}

/// Lives the world its years a day at a time at Max, from the first midnight after its first
/// month, checking each year's end.
fn live(sim: &mut Sim, years: u32, saves: Option<&Path>, world: &mut WorldRun) {
    let to_max = sim
        .set_speed(SPEED_MAX)
        .map_err(|e| e.to_string())
        .and_then(|()| sim.advance_to_midnight().map_err(|e| e.to_string()));
    if let Err(e) = to_max {
        world.failures.push(format!("the clock stopped: {e}"));
        return;
    }
    let mut run = LongRun::begin(sim);
    world.founders = run.founders;
    let start = sim.now().minutes();
    let mut most = run.founders;
    let dir = saves.map(|d| d.join(format!("dashboard-{}", world.seed)));
    for y in 1..=years {
        if let Err(e) = run.live_to(sim, start + i64::from(y) * MINUTES_PER_YEAR) {
            world
                .failures
                .push(format!("the clock stopped in year {y}: {e}"));
            break;
        }
        world.failures.extend(run.year_end(sim, y));
        let living = sim.people().living();
        world.people.push(living);
        most = most.max(living);
        if living < KEEP && world.ended.is_none() {
            world.ended = Some(format!("fell to {living} in year {y}, from {most} at most"));
        }
        if y % 10 == 0
            && let Some(dir) = &dir
        {
            let kept = SaveDir::create(dir, SAVE_EXTENSION)
                .map_err(|e| e.to_string())
                .and_then(|saves| {
                    persist::save(sim, &saves, SaveKind::Manual, &format!("year {y}"))
                        .map_err(|e| e.to_string())
                });
            match kept {
                Ok(saved) => world.saves.push(saved.path),
                Err(e) => world.failures.push(format!("the save of year {y}: {e}")),
            }
        }
    }
    world.lived_building_months = run.lived_building_months;
    for e in &sim.people().chronicle {
        if e.kind == civ_agents::ChronicleKind::BuildingFailed {
            // An entry names its household's first member, when it had anyone living.
            if e.people.is_empty() {
                world.empty_failures += 1;
            } else {
                world.lived_failures += 1;
                world
                    .lived_failed
                    .push((e.at.date().year, format!("{}, {} killed", e.name, e.number)));
            }
        }
    }
    // The largest settlement's Gini of goods at each year's end.
    let mut by_year: BTreeMap<i64, (f64, u32)> = BTreeMap::new();
    for w in &sim.people().wealth_years {
        let s = &w.spread;
        let e = by_year.entry(w.year).or_insert((s.gini_goods, s.people));
        if s.people > e.1 {
            *e = (s.gini_goods, s.people);
        }
    }
    world.gini = by_year.into_iter().map(|(y, (g, p))| (y, g, p)).collect();
    world.labels = sim
        .people()
        .polities
        .iter()
        .map(|p| {
            let l = civ_sim::labels::label_of(sim, p);
            if l.modifiers.is_empty() {
                l.in_prose()
            } else {
                format!("{} ({})", l.in_prose(), l.modifiers.join("; "))
            }
        })
        .collect();
    for c in run.economy.grade(sim) {
        let seen = Some((c.grade, c.text.clone()));
        match c.name {
            "stocks" => world.stocks = seen,
            "asks" => world.asks = seen,
            "workshops" => world.workshops = seen,
            _ => {}
        }
    }
}

/// Grades the worlds' runs of `years` years against §4.7.
pub fn grade(worlds: &[WorldRun], years: u32) -> Vec<Row> {
    vec![
        population(worlds, years),
        food_prices(worlds),
        gini(worlds),
        firm_sizes(worlds),
        failures(worlds),
        Row::new(
            "Settlement sizes",
            "the spread of settlements' sizes",
            "right-skewed",
        )
        .graded(
            Grade::Gray,
            "one settlement a world until regions (M5)".to_owned(),
        ),
        Row::new("Crime and poverty", "crime against poverty", "correlated").graded(
            Grade::Gray,
            "no crime until councils and law (M4)".to_owned(),
        ),
        Row::new(
            "Epidemics",
            "waterborne cases against contaminated sources",
            "clustered",
        )
        .graded(Grade::Gray, "no disease until M6".to_owned()),
        regimes(worlds),
    ]
}

/// The labels the worlds' polities ended with (ADR-0013 §6). Every world keeps the one founding
/// custom, which nothing can yet amend or seize (M4c), so labels that differ would show offices
/// and evidence, not who rules: the row reports them and is not graded until then.
fn regimes(worlds: &[WorldRun]) -> Row {
    let seen: Vec<String> = worlds
        .iter()
        .filter(|w| !w.labels.is_empty())
        .map(|w| format!("world {}: {}", w.seed, w.labels.join(", ")))
        .collect();
    let why = "every world keeps its founding custom, which none can yet amend or seize (M4c), \
               so differing labels would show offices and evidence, not who rules";
    Row::new(
        "Regimes",
        "labels inferred from how worlds are run",
        "two seeds differ",
    )
    .graded(
        Grade::Gray,
        if seen.is_empty() {
            format!("no polity to label; {why}")
        } else {
            format!("{} / {why}", seen.join(" / "))
        },
    )
}

/// The most a band may have grown by the end of year `y`, as a multiple of its founders.
pub fn growth_allowed(y: u32) -> f64 {
    GROWTH_FLOOR.max(GROWTH_RATE.powi(y as i32))
}

fn population(worlds: &[WorldRun], years: u32) -> Row {
    let row = Row::new(
        "Population",
        "people at each year's end against the founders",
        "3 of 5 worlds keep 10 at the end; none above max(3, 1.05^y) times its founders",
    )
    .source("tuning; research 05-06 §5.2, 05-01");
    if worlds.is_empty() {
        return row.graded(Grade::Gray, "no worlds".to_owned());
    }
    let keeping = worlds.iter().filter(|w| w.living() >= KEEP).count();
    let mut grown: Option<(f64, u64, u32)> = None;
    let mut too_fast = Vec::new();
    for w in worlds {
        for (i, &n) in w.people.iter().enumerate() {
            let y = i as u32 + 1;
            let times = n as f64 / w.founders.max(1) as f64;
            if grown.is_none_or(|(t, ..)| times > t) {
                grown = Some((times, w.seed, y));
            }
            if times > growth_allowed(y) {
                too_fast.push(format!("world {} at {times:.1} times in year {y}", w.seed));
            }
        }
    }
    let mut text = format!(
        "{keeping} of {} worlds keep {KEEP} or more after {years} years",
        worlds.len()
    );
    if let Some((times, seed, y)) = grown {
        text.push_str(&format!(
            "; the most growth {times:.1} times the founders (world {seed}, year {y})"
        ));
    }
    for w in worlds {
        if let Some(e) = &w.ended {
            text.push_str(&format!("; world {} {e}", w.seed));
        }
    }
    if !too_fast.is_empty() {
        text.push_str(&format!("; grew too fast: {}", too_fast.join(", ")));
    }
    let grade = if keeping >= KEEPING_WORLDS.min(worlds.len()) && too_fast.is_empty() {
        Grade::Green
    } else {
        Grade::Red
    };
    row.graded(grade, text)
}

fn food_prices(worlds: &[WorldRun]) -> Row {
    let row = Row::new(
        "Food prices",
        "grain asked before the harvest against after, and food stocks after it",
        "in every world with two harvests or more to judge, asks over 2 log points higher before, \
         and stocks rising after one at least",
    )
    .source("tuning; research 16-01 §2.2 reports 17-61 log points");
    let judged: Vec<&WorldRun> = worlds
        .iter()
        .filter(|w| w.asks.as_ref().is_some_and(|(g, _)| *g != Grade::Gray))
        .collect();
    if judged.is_empty() {
        return row.graded(
            Grade::Gray,
            "no world had harvests enough to judge".to_owned(),
        );
    }
    let mut missed = Vec::new();
    let mut seen = Vec::new();
    for w in &judged {
        let asks_ok = w.asks.as_ref().is_some_and(|(g, _)| *g == Grade::Green);
        let stocks_ok = w
            .stocks
            .as_ref()
            .is_some_and(|(g, _)| matches!(g, Grade::Green | Grade::Amber));
        let words = [&w.asks, &w.stocks]
            .iter()
            .filter_map(|c| c.as_ref().map(|(_, t)| t.as_str()))
            .collect::<Vec<_>>()
            .join("; ");
        if asks_ok && stocks_ok {
            seen.push(format!("world {}: {words}", w.seed));
        } else {
            missed.push(format!("world {}: {words}", w.seed));
        }
    }
    if missed.is_empty() {
        row.graded(Grade::Green, seen.join(" / "))
    } else {
        row.graded(Grade::Red, format!("missed in {}", missed.join(" / ")))
    }
}

/// The median of a world's Gini of goods over [`GINI_YEARS`], counting the years its largest
/// settlement had [`KEEP`] people or more, with how many years that was; `None` when fewer than
/// [`GINI_MIN_YEARS`].
pub fn gini_median(world: &WorldRun) -> Option<(f64, usize)> {
    let mut v: Vec<f64> = world
        .gini
        .iter()
        .filter(|&&(y, _, people)| {
            (GINI_YEARS.0..=GINI_YEARS.1).contains(&y) && people as usize >= KEEP
        })
        .map(|&(_, g, _)| g)
        .filter(|g| g.is_finite())
        .collect();
    if v.len() < GINI_MIN_YEARS {
        return None;
    }
    v.sort_by(f64::total_cmp);
    let n = v.len();
    let median = if n % 2 == 1 {
        v[n / 2]
    } else {
        (v[n / 2 - 1] + v[n / 2]) / 2.0
    };
    Some((median, n))
}

fn gini(worlds: &[WorldRun]) -> Row {
    let row = Row::new(
        "Wealth Gini",
        "the median Gini of goods over years 26-50 of each settlement still lived in by 10",
        "0.3-0.75; amber from 0.1, as hoe farmers with land to spare may be",
    )
    .source("plan §4.7; research 08-14 §3.1-3.2, 16-01 §2.1; the floor of 0.1 is tuning");
    let judged: Vec<(u64, f64, usize)> = worlds
        .iter()
        .filter_map(|w| gini_median(w).map(|(g, n)| (w.seed, g, n)))
        .collect();
    if judged.is_empty() {
        return row.graded(
            Grade::Gray,
            format!(
                "no settlement was lived in by {KEEP} for {GINI_MIN_YEARS} of years {}-{}",
                GINI_YEARS.0, GINI_YEARS.1
            ),
        );
    }
    let grade_of = |g: f64| {
        if (GINI_BAND.0..=GINI_BAND.1).contains(&g) {
            Grade::Green
        } else if (GINI_FLOOR..GINI_BAND.0).contains(&g) {
            Grade::Amber
        } else {
            Grade::Red
        }
    };
    let grade = judged
        .iter()
        .map(|&(_, g, _)| grade_of(g))
        .max()
        .unwrap_or(Grade::Gray);
    let mut text = judged
        .iter()
        .map(|(seed, g, n)| format!("world {seed}: {g:.2} over {n} years"))
        .collect::<Vec<_>>()
        .join("; ");
    if grade == Grade::Amber {
        text.push_str(
            " (hoe farmers with land to spare may be this equal: research 08-14 §3.1 gives \
             horticultural populations 0.27 ± 0.03)",
        );
    }
    row.graded(grade, text)
}

fn firm_sizes(worlds: &[WorldRun]) -> Row {
    let row = Row::new(
        "Firm sizes",
        "each workshop's lifetime hours, owners' and hired",
        "in every world with 10 workshops or more, the mean above the median",
    )
    .source("research 16-01 §2.1; the least count is tuning");
    let judged: Vec<(u64, &Seen)> = worlds
        .iter()
        .filter_map(|w| w.workshops.as_ref().map(|c| (w.seed, c)))
        .filter(|(_, (g, _))| *g != Grade::Gray)
        .collect();
    if judged.is_empty() {
        return row.graded(
            Grade::Gray,
            "no world had workshops enough to judge".to_owned(),
        );
    }
    let text = judged
        .iter()
        .map(|(seed, (_, t))| format!("world {seed}: {t}"))
        .collect::<Vec<_>>()
        .join("; ");
    let grade = if judged.iter().all(|(_, (g, _))| *g == Grade::Green) {
        Grade::Green
    } else {
        Grade::Red
    };
    row.graded(grade, text)
}

/// Failures of buildings lived in per 1,000 years they stood lived in, over all the worlds, with
/// the failures and the building-years; `None` when no building stood lived in.
pub fn failure_rate(worlds: &[WorldRun]) -> Option<(f64, u32, f64)> {
    let failures: u32 = worlds.iter().map(|w| w.lived_failures).sum();
    let years: f64 = worlds
        .iter()
        .map(|w| w.lived_building_months as f64 / 12.0)
        .sum();
    (years > 0.0).then(|| (f64::from(failures) * 1000.0 / years, failures, years))
}

fn failures(worlds: &[WorldRun]) -> Row {
    let row = Row::new(
        "Structural failures",
        "failures of buildings lived in, per 1,000 years they stood lived in, all worlds together",
        "at most 2",
    )
    .source("tuning; research 11-06 §2.4's top scenario, 1, doubled for chance");
    let empty: u32 = worlds.iter().map(|w| w.empty_failures).sum();
    match failure_rate(worlds) {
        None => row.graded(Grade::Gray, "no building stood lived in".to_owned()),
        Some((rate, n, years)) => row.graded(
            if rate <= MAX_FAILURES_PER_1000 {
                Grade::Green
            } else {
                Grade::Red
            },
            format!(
                "{n} in {years:.0} building-years, {rate:.2} per 1,000; buildings nobody lived in \
                 gave way {empty} times besides"
            ),
        ),
    }
}

/// The header of the worlds' table.
pub fn header() -> String {
    format!(
        "{:<6} {:<15} {:>8} {:>8} {:>26}  how it went",
        "world", "regime", "seconds", "founders", "people at 10/20/30/40/50"
    )
}

/// One world's line in the table.
pub fn format_world(w: &WorldRun) -> String {
    let at = |y: usize| {
        w.people
            .get(y - 1)
            .map_or_else(|| "-".to_owned(), ToString::to_string)
    };
    let people = format!("{}/{}/{}/{}/{}", at(10), at(20), at(30), at(40), at(50));
    let mut how = Vec::new();
    if let Some(e) = &w.ended {
        how.push(e.clone());
    }
    if w.failures.is_empty() {
        how.push("checks passed".to_owned());
    } else {
        how.push(format!("FAILED: {}", w.failures.join("; ")));
    }
    format!(
        "{:<6} {:<15} {:>8.0} {:>8} {:>26}  {}",
        w.seed,
        w.regime,
        w.seconds,
        w.founders,
        people,
        how.join("; ")
    )
}

/// The rows, each with its threshold and source when it was graded.
pub fn format_rows(rows: &[Row]) -> String {
    rows.iter()
        .map(|r| {
            let mut s = format!("{:<20} {:<6} {}", r.name, r.grade.word(), r.text);
            if r.grade != Grade::Gray {
                s.push_str(&format!("\n{:<27} passes: {}", "", r.threshold));
                if !r.source.is_empty() {
                    s.push_str(&format!(" ({})", r.source));
                }
            }
            s
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world(seed: u64, people: &[usize]) -> WorldRun {
        WorldRun {
            seed,
            founders: 40,
            people: people.to_vec(),
            ..WorldRun::default()
        }
    }

    #[test]
    fn three_worlds_keeping_their_bands_pass_and_too_fast_a_growth_fails() {
        let mut worlds = vec![
            world(1, &[40, 45, 12]),
            world(2, &[40, 50, 10]),
            world(3, &[40, 52, 30]),
            world(4, &[40, 20, 2]),
            world(5, &[40, 30, 0]),
        ];
        assert_eq!(population(&worlds, 3).grade, Grade::Green);
        worlds[2].people = vec![40, 40, 9];
        assert_eq!(population(&worlds, 3).grade, Grade::Red, "two keep theirs");
        worlds[2].people = vec![40, 40, 30];
        // Three times the founders is allowed early on, 1.05^y later.
        assert!((growth_allowed(1) - 3.0).abs() < 1e-12);
        assert!((growth_allowed(50) - 1.05f64.powi(50)).abs() < 1e-9);
        worlds[0].people = vec![40, 121, 120];
        assert_eq!(population(&worlds, 3).grade, Grade::Red, "121 > 3 × 40");
    }

    #[test]
    fn the_gini_is_judged_over_years_lived_in_and_banded() {
        let mut w = world(1, &[40]);
        // Years 26-50 at 0.25, but only four with ten people or more: too few to judge.
        w.gini = (20..=50)
            .map(|y| (y, 0.25, if y <= 29 { 30 } else { 4 }))
            .collect();
        assert_eq!(gini_median(&w), None);
        assert_eq!(gini(&[w.clone()]).grade, Grade::Gray);
        w.gini = (20..=50).map(|y| (y, 0.25, 30)).collect();
        assert_eq!(gini_median(&w), Some((0.25, 25)));
        assert_eq!(gini(&[w.clone()]).grade, Grade::Amber, "hoe farmers");
        w.gini = (20..=50).map(|y| (y, 0.4, 30)).collect();
        assert_eq!(gini(&[w.clone()]).grade, Grade::Green);
        w.gini = (20..=50).map(|y| (y, 0.05, 30)).collect();
        assert_eq!(gini(&[w.clone()]).grade, Grade::Red, "no difference at all");
        w.gini = (20..=50).map(|y| (y, 0.8, 30)).collect();
        assert_eq!(gini(&[w]).grade, Grade::Red);
    }

    #[test]
    fn failures_are_counted_against_the_years_buildings_stood_lived_in() {
        let mut a = world(1, &[40]);
        a.lived_building_months = 12 * 600;
        a.lived_failures = 1;
        a.empty_failures = 20;
        let mut b = world(2, &[40]);
        b.lived_building_months = 12 * 400;
        // One in 1,000 building-years: within two; the empty huts' twenty do not count.
        let (rate, n, years) = failure_rate(&[a.clone(), b.clone()]).expect("stood");
        assert_eq!(n, 1);
        assert!((years - 1000.0).abs() < 1e-9 && (rate - 1.0).abs() < 1e-9);
        assert_eq!(failures(&[a.clone(), b.clone()]).grade, Grade::Green);
        a.lived_failures = 3;
        assert_eq!(failures(&[a, b]).grade, Grade::Red);
        assert_eq!(failures(&[world(3, &[0])]).grade, Grade::Gray);
    }

    #[test]
    fn rows_without_enough_to_judge_are_grey_say_why_and_never_pass() {
        let worlds = vec![world(1, &[40]), world(2, &[40])];
        let rows = grade(&worlds, 1);
        let grey: Vec<&str> = rows
            .iter()
            .filter(|r| r.grade == Grade::Gray)
            .map(|r| r.name)
            .collect();
        for name in [
            "Food prices",
            "Wealth Gini",
            "Firm sizes",
            "Structural failures",
            "Settlement sizes",
            "Crime and poverty",
            "Epidemics",
            "Regimes",
        ] {
            assert!(grey.contains(&name), "{name} grey: {grey:?}");
        }
        assert!(
            rows.iter()
                .filter(|r| r.grade == Grade::Gray)
                .all(|r| !r.text.is_empty())
        );
        let dashboard = Dashboard {
            version: "",
            content: String::new(),
            mode: "",
            preset: PRESET,
            size: 0,
            years: 1,
            seconds: 0.0,
            worlds,
            rows,
        };
        // Only population is graded, and both worlds keep their bands: a pass, but no grey row
        // counts as one.
        assert!(dashboard.passed());
        assert_eq!(dashboard.summary(), "1 passed, 8 not yet applicable");
    }

    #[test]
    fn prices_and_workshops_fail_when_a_judged_world_misses() {
        let mut a = world(1, &[40]);
        a.asks = Some((Grade::Green, "grain asked +4 log points more".to_owned()));
        a.stocks = Some((Grade::Green, "rose".to_owned()));
        let mut b = a.clone();
        b.seed = 2;
        assert_eq!(food_prices(&[a.clone(), b.clone()]).grade, Grade::Green);
        b.asks = Some((Grade::Amber, "grain asked +1 log points more".to_owned()));
        assert_eq!(food_prices(&[a.clone(), b]).grade, Grade::Red);
        a.workshops = Some((Grade::Amber, "mean = median".to_owned()));
        assert_eq!(firm_sizes(&[a]).grade, Grade::Red);
    }
}
