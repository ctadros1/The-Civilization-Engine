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
//! - **Regimes** (M4c slice AJ): whether two worlds' polities ended under labels that differ in
//!   who may decide or who leads, by the label's principal name, which its reasons name (the M4
//!   observer brief: "differ" only where the why names a structural difference). Fewer than two is
//!   amber, reported and never forced; the classifier does not read the property regime.
//! - **Crime and poverty** (M4b slice AD): over all the worlds' attempts to take, where each
//!   taker's household stood among its settlement's by food when they came, graded by direction
//!   only: more of their neighbours held more than held less, once there are [`MIN_ATTEMPTS`].
//!   Takings, those seen and those brought as cases (recorded crime, which is not crime: research
//!   12-04 §4.B) are shown beside the attempts and not graded.
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
/// Least attempts to take, all worlds together, for the crime row to be graded (tuning: at 30,
/// a mean share of one half has a standard error of about 0.05).
pub const MIN_ATTEMPTS: u32 = 30;

/// Moves between settlements per 100 residents a year outside which the moves row is amber: the
/// sensitivity range 05-06 §5.4 proposes around its starting point of 2. A design choice of the
/// report's, an output benchmark and not a quota, so the row is never red.
pub const MOVES_BAND: (f64, f64) = (0.5, 10.0);

/// What to run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DashboardOptions {
    /// Cells per side of each world.
    pub size: u32,
    /// Years each world lives after its first month: [`YEARS`] for the dashboard itself, fewer
    /// only for a quick look, which grades the rows over what it saw.
    pub years: u32,
    /// Founding groups each world is made with, each a band of the content's default size placed
    /// together with the others (ADR-0018 §6): [`GROUPS`] from M5a; 1 is the dashboard of M3c to
    /// M4.
    pub groups: u32,
}

/// Founding groups a dashboard world is made with (M5a slice AK).
pub const GROUPS: u32 = 2;

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
    /// Each polity's regime at the end, by its label's principal name without the person it names
    /// ("Council community — led by its proposer"), with the reason its body is what it is.
    pub regimes: Vec<(String, String)>,
    /// The same at each tenth year's end: (year, each polity's principal name).
    pub regimes_by_decade: Vec<(u32, Vec<String>)>,
    /// Its attempts to take, by where the taker's household stood by food.
    pub crime: CrimeSeen,
    /// Each settlement at the end: its name and its residents (ADR-0018).
    pub settlements: Vec<(String, u32)>,
    /// Where a settlement's accounts did not balance in some year (ADR-0018 §3's hard check).
    pub accounting: Vec<String>,
    /// Its settlements' accounts over the run, summed.
    pub moves: smoke::Moves,
    /// Visits between its settlements over the run, the hours visitors spent at the hearths they
    /// went to, and marriages between them (M5a slice AM).
    pub contacts: (u32, f64, u32),
    /// People who came to live in another of its settlements over the run, by why (slice AN).
    pub between: Between,
}

/// People who came to live in one settlement from another, by why: the moves 05-06 §5.4 counts,
/// less forced displacement, and the exiles it leaves out.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Between {
    /// With their household, by its choice or in an emergency.
    pub moved: u32,
    /// With their household and others, to found a settlement (M5a slice AO).
    pub founded: u32,
    /// On marrying someone there.
    pub married: u32,
    /// Taken in by kin there.
    pub taken_in: u32,
    /// Sent away by a finding: forced, so not counted with the others.
    pub exiled: u32,
}

impl Between {
    /// The residence histories' moves from one settlement to another.
    pub fn of(sim: &Sim) -> Between {
        use civ_agents::ResidenceWhy;
        let mut out = Between::default();
        for r in sim.people().records.values() {
            for pair in r.residence.windows(2) {
                let (Some(a), Some(b)) = (pair[0].settlement, pair[1].settlement) else {
                    continue;
                };
                if a == b {
                    continue;
                }
                match pair[1].why {
                    ResidenceWhy::Moved => out.moved += 1,
                    ResidenceWhy::Founded => out.founded += 1,
                    ResidenceWhy::Married => out.married += 1,
                    ResidenceWhy::TakenIn => out.taken_in += 1,
                    ResidenceWhy::Exiled => out.exiled += 1,
                    _ => {}
                }
            }
        }
        out
    }

    /// The moves not forced.
    pub fn peaceful(&self) -> u32 {
        self.moved + self.founded + self.married + self.taken_in
    }
}

impl WorldRun {
    /// People at the end of the run.
    pub fn living(&self) -> usize {
        self.people.last().copied().unwrap_or(0)
    }
}

/// Attempts to take of one kind, counted with where each taker's household stood among its
/// settlement's by food: the sum of the shares of its neighbours that held more
/// ([`civ_agents::crime::Incident::richer`]).
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
pub struct Ranked {
    /// Attempts whose household could be compared with its neighbours.
    pub n: u32,
    /// The sum of the shares of their neighbours that held more.
    pub richer: f64,
}

impl Ranked {
    fn add(&mut self, richer: f32) {
        if richer.is_finite() {
            self.n += 1;
            self.richer += f64::from(richer);
        }
    }

    fn join(&mut self, other: Ranked) {
        self.n += other.n;
        self.richer += other.richer;
    }

    /// The mean share of their neighbours that held more: about one half if takers came from any
    /// household alike, more if from the poorer.
    pub fn mean(&self) -> Option<f64> {
        (self.n > 0).then(|| self.richer / f64::from(self.n))
    }
}

/// A world's attempts to take, the truth, with what came to be known of them beside it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
pub struct CrimeSeen {
    /// Every attempt.
    pub attempts: Ranked,
    /// Those that carried food off.
    pub takings: Ranked,
    /// Those someone saw.
    pub seen: Ranked,
    /// Those brought before a gathering.
    pub cases: Ranked,
}

impl CrimeSeen {
    /// The tally of `order`'s incidents.
    pub fn of(order: &civ_agents::crime::Order) -> CrimeSeen {
        let mut c = CrimeSeen::default();
        for i in &order.incidents {
            c.attempts.add(i.richer);
            if i.outcome == civ_agents::crime::Outcome::Taken {
                c.takings.add(i.richer);
            }
            if !i.seen_by.is_empty() {
                c.seen.add(i.richer);
            }
            if order.cases.iter().any(|k| k.incident == i.id) {
                c.cases.add(i.richer);
            }
        }
        c
    }

    fn join(&mut self, other: &CrimeSeen) {
        self.attempts.join(other.attempts);
        self.takings.join(other.takings);
        self.seen.join(other.seen);
        self.cases.join(other.cases);
    }

    /// In words: "34 attempts, 0.71 of their neighbours richer on average; 12 takings (0.68), 8
    /// seen (0.70), 3 brought as cases (0.80)".
    pub fn words(&self) -> String {
        let one = |r: &Ranked, one: &str, many: &str, none: &str| {
            let what = if r.n == 1 { one } else { many };
            match r.mean() {
                Some(m) => format!("{} {what} ({m:.2})", r.n),
                None => none.to_owned(),
            }
        };
        let attempts = match self.attempts.mean() {
            Some(m) => format!(
                "{} {}, {m:.2} of the takers' neighbours richer on average",
                self.attempts.n,
                if self.attempts.n == 1 {
                    "attempt"
                } else {
                    "attempts"
                }
            ),
            None => "no attempts".to_owned(),
        };
        format!(
            "{attempts}; {}, {}, {}",
            one(&self.takings, "taking", "takings", "no takings"),
            one(&self.seen, "seen", "seen", "none seen"),
            one(
                &self.cases,
                "brought as a case",
                "brought as cases",
                "none brought as cases"
            ),
        )
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
            neighbours: vec![0; options.groups.saturating_sub(1) as usize],
            neighbours_known: false,
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
        if y % 10 == 0 {
            world.regimes_by_decade.push((
                y,
                regimes_of(sim).into_iter().map(|(name, _)| name).collect(),
            ));
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
    world.crime = CrimeSeen::of(&sim.people().order);
    world.settlements = sim
        .land()
        .settlements
        .iter()
        .map(|s| (s.name.clone(), sim.people().residents(s.id)))
        .collect();
    world.accounting = run.accounting.clone();
    world.moves = run.moves;
    world.contacts = sim
        .people()
        .contacts
        .years
        .values()
        .fold((0, 0.0, 0), |(v, h, m), c| {
            (v + c.visits, h + c.minutes as f64 / 60.0, m + c.marriages)
        });
    world.between = Between::of(sim);
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
    world.regimes = regimes_of(sim);
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
        settlement_sizes(worlds),
        contacts(worlds, years),
        moves(worlds),
        accounting(worlds),
        crime(worlds),
        Row::new(
            "Epidemics",
            "waterborne cases against contaminated sources",
            "clustered",
        )
        .graded(Grade::Gray, "no disease until M6".to_owned()),
        regimes(worlds),
    ]
}

/// Settlements' sizes at the end, reported and not graded: 10-01 §4 and 10-02 §5 warn against
/// fitting a distribution to a few settlements, so the row stays grey until there are many (M9).
fn settlement_sizes(worlds: &[WorldRun]) -> Row {
    let row = Row::new(
        "Settlement sizes",
        "the spread of settlements' sizes",
        "right-skewed",
    )
    .source("10-01 §4; 10-02 §5");
    let seen: Vec<String> = worlds
        .iter()
        .filter(|w| !w.settlements.is_empty())
        .map(|w| {
            let sizes: Vec<String> = w
                .settlements
                .iter()
                .map(|(name, n)| format!("{name} {n}"))
                .collect();
            format!("world {}: {}", w.seed, sizes.join(", "))
        })
        .collect();
    let text = if seen.is_empty() {
        "no world lived".to_owned()
    } else {
        format!(
            "reported, not graded until there are many (M9): {}",
            seen.join("; ")
        )
    };
    row.graded(Grade::Gray, text)
}

/// Contacts between settlements, reported and not graded (M5a slice AM): the research gives no
/// rate of visiting between neighbouring villages to hold them to (05-06 §5.4 grades moves; see
/// [`moves`]).
fn contacts(worlds: &[WorldRun], years: u32) -> Row {
    let row = Row::new(
        "Contacts",
        "visits and marriages between settlements a year",
        "reported",
    )
    .source("ADR-0018 §4");
    let years = f64::from(years.max(1));
    let seen: Vec<String> = worlds
        .iter()
        .filter(|w| w.settlements.len() > 1)
        .map(|w| {
            let (visits, hours, marriages) = w.contacts;
            format!(
                "world {}: {:.1} visits a year ({:.0} hours at others' hearths), {:.1} marriages",
                w.seed,
                f64::from(visits) / years,
                hours / years,
                f64::from(marriages) / years
            )
        })
        .collect();
    let text = if seen.is_empty() {
        "no world of several settlements lived".to_owned()
    } else {
        format!("reported, not graded: {}", seen.join("; "))
    };
    row.graded(Grade::Gray, text)
}

/// Moves between settlements per 100 residents a year (M5a slice AN), against 05-06 §5.4's
/// sensitivity range: amber outside it and never red, since the report offers it as a benchmark to
/// look at and not a target. Exiles are left out, as forced displacement; moves in an emergency
/// cannot yet be told from chosen ones and are counted. Turnover — everyone who came or went,
/// off the map too — is shown beside the net change, since a steady size can hide much of it
/// (05-06 §4). Grey for a world of one settlement.
fn moves(worlds: &[WorldRun]) -> Row {
    let row = Row::new(
        "Moves",
        "moves between settlements per 100 residents a year",
        "0.5–10",
    )
    .source("05-06 §4, §5.4");
    let judged: Vec<(&WorldRun, f64)> = worlds
        .iter()
        .filter(|w| w.settlements.len() > 1)
        .filter_map(|w| {
            // Residents' years: each year's end counted for that year.
            let lived: f64 = w.people.iter().map(|&n| n as f64).sum();
            (lived > 0.0).then(|| (w, 100.0 * f64::from(w.between.peaceful()) / lived))
        })
        .collect();
    if judged.is_empty() {
        return row.graded(
            Grade::Gray,
            "no world of several settlements lived".to_owned(),
        );
    }
    let seen: Vec<String> = judged
        .iter()
        .map(|&(w, rate)| {
            let lived: f64 = w.people.iter().map(|&n| n as f64).sum();
            let turnover = 100.0 * f64::from(w.moves.arrivals + w.moves.departures) / lived;
            let net = 100.0 * (w.living() as f64 - w.founders as f64) / lived;
            let b = &w.between;
            format!(
                "world {}: {rate:.1} ({} with their households, {} founding, {} on marrying, {} \
                 taken in; {} exiled not counted); turnover {turnover:.1}, net {net:+.1}",
                w.seed, b.moved, b.founded, b.married, b.taken_in, b.exiled
            )
        })
        .collect();
    let outside = judged
        .iter()
        .any(|&(_, rate)| rate < MOVES_BAND.0 || rate > MOVES_BAND.1);
    row.graded(
        if outside { Grade::Amber } else { Grade::Green },
        seen.join("; "),
    )
}

/// Every settlement's accounts balance every year (ADR-0018 §3): a hard check. Shown with the
/// moves the runs saw.
fn accounting(worlds: &[WorldRun]) -> Row {
    let row = Row::new(
        "Population accounting",
        "each settlement's births − deaths + arrivals − departures against its change in residents, \
         every year",
        "every one balances",
    )
    .source("05-06 §1.1; ADR-0018 §3");
    let lived: Vec<&WorldRun> = worlds.iter().filter(|w| !w.people.is_empty()).collect();
    if lived.is_empty() {
        return row.graded(Grade::Gray, "no world lived".to_owned());
    }
    let mut sum = smoke::Moves::default();
    for w in &lived {
        sum.births += w.moves.births;
        sum.deaths += w.moves.deaths;
        sum.arrivals += w.moves.arrivals;
        sum.from_off_map += w.moves.from_off_map;
        sum.departures += w.moves.departures;
        sum.off_map += w.moves.off_map;
    }
    let between = sum.departures - sum.off_map;
    let moves = format!(
        "{} born, {} died, {} came from off the map, {} left it, {between} moved between \
         settlements",
        sum.births, sum.deaths, sum.from_off_map, sum.off_map
    );
    match lived
        .iter()
        .find_map(|w| w.accounting.first().map(|f| (w.seed, f)))
    {
        None => row.graded(
            Grade::Green,
            format!(
                "every settlement of {} worlds, every year; {moves}",
                lived.len()
            ),
        ),
        Some((seed, first)) => row.graded(Grade::Red, format!("world {seed}: {first}; {moves}")),
    }
}

/// Each polity of `sim`'s regime now: its label's principal name without the person it names
/// ("Council community — big-man leadership"), and the first of its reasons, which says who may
/// decide (ADR-0013 §6; the classifier is frozen for M4's demo, research 16-03 §2.2).
fn regimes_of(sim: &Sim) -> Vec<(String, String)> {
    sim.people()
        .polities
        .iter()
        .map(|p| {
            let l = civ_sim::labels::label_of(sim, p);
            let name = l.name.split(" (").next().unwrap_or(&l.name).to_owned();
            (name, l.why.first().cloned().unwrap_or_default())
        })
        .collect()
}

/// The regimes the worlds' polities ended under (M4c slice AJ; plan §4.7: "two seeds differ").
/// Two worlds differ only where their labels' principal names do, and those name who may decide
/// and who leads, as their reasons say. Fewer than two regimes is amber: reported, never forced
/// (research 16-03 §2.2: no reference rate of divergence is known, and none is tuned toward).
fn regimes(worlds: &[WorldRun]) -> Row {
    let row = Row::new(
        "Regimes",
        "each world's polity at the end, by its label's principal name",
        "two worlds differ in who may decide or who leads",
    )
    .source("the M4 observer brief; ADR-0013 §6");
    let labelled: Vec<&WorldRun> = worlds.iter().filter(|w| !w.regimes.is_empty()).collect();
    let seen: Vec<String> = labelled
        .iter()
        .map(|w| {
            let mut s = format!(
                "world {}: {}",
                w.seed,
                w.regimes
                    .iter()
                    .map(|(name, why)| format!("{} ({})", name.to_lowercase(), why))
                    .collect::<Vec<_>>()
                    .join("; ")
            );
            let changes: Vec<String> = w
                .regimes_by_decade
                .windows(2)
                .filter(|p| p[0].1 != p[1].1)
                .map(|p| format!("by year {}: {}", p[1].0, p[1].1.join(", ").to_lowercase()))
                .collect();
            if !changes.is_empty() {
                s.push_str(&format!(" [{}]", changes.join("; ")));
            }
            s
        })
        .collect();
    if labelled.len() < 2 {
        return row.graded(
            Grade::Gray,
            format!(
                "fewer than two worlds with a polity to label{}",
                if seen.is_empty() {
                    String::new()
                } else {
                    format!(": {}", seen.join(" / "))
                }
            ),
        );
    }
    let mut kinds: Vec<&str> = labelled
        .iter()
        .flat_map(|w| w.regimes.iter().map(|(name, _)| name.as_str()))
        .collect();
    kinds.sort_unstable();
    kinds.dedup();
    let note = "the classifier does not read the property regime";
    if kinds.len() >= 2 {
        row.graded(
            Grade::Green,
            format!("{} regimes: {} / {note}", kinds.len(), seen.join(" / ")),
        )
    } else {
        row.graded(
            Grade::Amber,
            format!(
                "every world ended as a {}: fewer than two regimes arose, reported and not forced: \
                 {} / {note}",
                kinds.first().map_or("polity", |k| k).to_lowercase(),
                seen.join(" / ")
            ),
        )
    }
}

/// Crime against poverty (plan §4.7; M4b slice AD): every attempt to take, all worlds together,
/// by the share of the taker's settlement's other households that held more food when they came,
/// graded by its direction only. What was recorded is shown beside it, ungraded: better detection
/// raises recorded crime (research 12-04 §4.B).
fn crime(worlds: &[WorldRun]) -> Row {
    let row = Row::new(
        "Crime and poverty",
        "every attempt to take, by the share of the taker's neighbours holding more food, \
         all worlds together; takings, those seen and cases beside",
        "with 30 attempts or more, more than half of the neighbours richer on average",
    )
    .source("plan §4.7; research 04-09 §5.10 (attempts as well as completions); 30 is tuning");
    let mut all = CrimeSeen::default();
    for w in worlds {
        all.join(&w.crime);
    }
    let words = all.words();
    match all.attempts.mean() {
        Some(m) if all.attempts.n >= MIN_ATTEMPTS => {
            row.graded(if m > 0.5 { Grade::Green } else { Grade::Red }, words)
        }
        _ => row.graded(
            Grade::Gray,
            format!("too few attempts to take to judge, {MIN_ATTEMPTS} needed: {words}"),
        ),
    }
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
    fn regimes_differ_only_by_who_decides_or_leads_and_one_regime_is_reported_not_failed() {
        let mut a = world(1, &[40]);
        let mut b = world(2, &[40]);
        let council = (
            "Council community".to_owned(),
            "All its adults may come and decide.".to_owned(),
        );
        a.regimes = vec![council.clone()];
        b.regimes = vec![council.clone()];
        let one = regimes(&[a.clone(), b.clone()]);
        assert_eq!(one.grade, Grade::Amber, "{}", one.text);
        assert!(one.text.contains("fewer than two regimes arose"));
        b.regimes = vec![(
            "Oligarchy".to_owned(),
            "Only 20 % of its adults may decide.".to_owned(),
        )];
        b.regimes_by_decade = vec![
            (10, vec!["Council community".to_owned()]),
            (20, vec!["Oligarchy".to_owned()]),
        ];
        let two = regimes(&[a.clone(), b]);
        assert_eq!(two.grade, Grade::Green, "{}", two.text);
        assert!(two.text.contains("by year 20: oligarchy"), "{}", two.text);
        assert_eq!(regimes(&[a]).grade, Grade::Gray, "one world cannot differ");
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
            "Contacts",
            "Moves",
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
        // Only population and the accounts are graded, both worlds keep their bands and no
        // settlement's accounts failed: a pass, but no grey row counts as one.
        assert!(dashboard.passed());
        assert_eq!(dashboard.summary(), "2 passed, 10 not yet applicable");
    }

    #[test]
    fn crime_is_graded_by_its_direction_over_attempts_and_needs_enough() {
        let ranked = |n: u32, mean: f64| Ranked {
            n,
            richer: f64::from(n) * mean,
        };
        let mut a = world(1, &[40]);
        a.crime.attempts = ranked(20, 0.7);
        a.crime.takings = ranked(5, 0.8);
        // Twenty attempts: too few, and the row says why.
        let row = crime(&[a.clone()]);
        assert_eq!(row.grade, Grade::Gray);
        assert!(row.text.starts_with("too few"), "{}", row.text);
        // Two worlds' together are enough; the takers came from the poorer.
        let mut b = a.clone();
        b.seed = 2;
        b.crime.attempts = ranked(10, 0.6);
        let row = crime(&[a.clone(), b.clone()]);
        assert_eq!(row.grade, Grade::Green, "{}", row.text);
        assert!(
            row.text
                .starts_with("30 attempts, 0.67 of the takers' neighbours richer"),
            "{}",
            row.text
        );
        assert!(row.text.contains("10 takings (0.80)"), "{}", row.text);
        assert!(
            row.text.contains("none seen, none brought as cases"),
            "{}",
            row.text
        );
        // From the richer, or from any household alike: the wrong direction.
        b.crime.attempts = ranked(40, 0.2);
        assert_eq!(crime(&[a.clone(), b.clone()]).grade, Grade::Red);
        a.crime.attempts = ranked(30, 0.5);
        assert_eq!(crime(&[a]).grade, Grade::Red, "one half is no direction");
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

    #[test]
    fn moves_are_graded_per_100_residents_a_year_leaving_exiles_out_and_never_red() {
        // Ten years of 100 residents: 1,000 residents' years.
        let mut w = world(1, &[100; 10]);
        w.founders = 100;
        w.between = Between {
            moved: 8,
            founded: 4,
            married: 6,
            taken_in: 2,
            exiled: 40,
        };
        assert_eq!(moves(&[w.clone()]).grade, Grade::Gray, "one settlement");
        w.settlements = vec![("A".to_owned(), 50), ("B".to_owned(), 50)];
        let row = moves(&[w.clone()]);
        assert_eq!(row.grade, Grade::Green, "{}", row.text);
        assert!(row.text.contains("world 1: 2.0 "), "{}", row.text);
        assert!(row.text.contains("40 exiled not counted"), "{}", row.text);
        w.between = Between {
            moved: 3,
            exiled: 40,
            ..Between::default()
        };
        assert_eq!(moves(&[w.clone()]).grade, Grade::Amber, "0.3 is too few");
        w.between.moved = 150;
        assert_eq!(moves(&[w]).grade, Grade::Amber, "15 is too many, never red");
    }
}
