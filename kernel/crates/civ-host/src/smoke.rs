//! Smoke seeds (plan §4.7): a few worlds per preset, checked against fixed thresholds. No
//! statistics; any failure blocks the milestone.
//!
//! The checks cover terrain and water, the save round trip, and a month of the founding band's
//! life: they settle, keep water and firewood at home, still have food, break ground and sow
//! their first field, and nobody is ever stuck between events.
//!
//! With `years`, each world then lives on for years (the M1 sanity run, plan §4.7), checked at
//! every year's end: nobody stuck, no population or land problems, every good held accounted for
//! by what came in and went out (ADR-0006 §3), no population explosion,
//! households under a roof from the second year, and a first trail worn out of the settlement
//! within the first. A band may fail and leave its valley, as foundings did (research 05-06 §5.2);
//! across the worlds at least half the bands must still live where they settled. Runs differ from
//! one to the next (determinism is not a goal), so the thresholds leave room for chance.
//!
//! Every year's end also checks that the techniques every founder brings and children learn in
//! upbringing are known by nearly everyone old enough for them (ADR-0008 §4); a technique lost
//! where people still live, and every find, is noted.
//!
//! The worlds take the content's property regimes in turn by seed, and a long run checks their
//! economies too ([`crate::economy`]): land claims, fields and wealth measures at each year's end,
//! and at the run's end, graded, whether food stocks rise with the harvest, whether grain is asked
//! more for before it than after, how unequal goods are and how workshop sizes spread.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use civ_content::ContentRegistry;
use civ_core::time::{MINUTES_PER_DAY, SimTime};
use civ_schema::SAVE_EXTENSION;
use civ_sim::{NewWorld, Sim, persist};
use civ_world::MapStats;
use commons_persist::{SaveDir, SaveKind};

use crate::economy::{Check, Economy, Grade};

/// Least elevation range, metres.
pub const MIN_RELIEF_M: f32 = 20.0;
/// Least share of the map that is not ocean.
pub const MIN_NOT_OCEAN: f32 = 0.5;
/// Least share of dry land with slopes under 5 %.
pub const MIN_GENTLE: f32 = 0.05;
/// Least total river length, km.
pub const MIN_RIVER_KM: f32 = 0.5;
/// In-game days the founding band lives.
pub const DAYS: i64 = 30;
/// Most people a band may grow to, as a multiple of the founding band, in the long run.
pub const MAX_GROWTH: f64 = 3.0;
/// People a band must keep to count as still living where it settled, in the long run.
pub const MIN_ALIVE: usize = 10;
/// Share of the households, from the second year on, that must live under a roof.
pub const MIN_ROOFED: f64 = 0.8;
/// Least share of the living old enough for a technique every founder brings and children learn
/// in upbringing who know it, at each year's end (ADR-0008 §4).
pub const MIN_UPBRINGING: f64 = 0.95;

/// What to run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SmokeOptions {
    /// Cells per side of each world.
    pub size: u32,
    /// Seeds per preset, starting at 1.
    pub seeds: u64,
    /// Years each world lives on after its first month (0: the first month only).
    pub years: u32,
}

/// One world's result.
#[derive(Clone, Debug)]
pub struct SmokeResult {
    /// Preset id.
    pub preset: String,
    /// Seed.
    pub seed: u64,
    /// Time to generate, save, load and run.
    pub elapsed: Duration,
    /// The world's figures, when it was generated.
    pub stats: Option<MapStats>,
    /// Every threshold it missed; empty means it passed.
    pub failures: Vec<String>,
    /// People living at the end, when it lived on for years.
    pub living: Option<usize>,
    /// What else happened that a reader should know (a band that left its valley).
    pub notes: Vec<String>,
    /// The property regime it lived under.
    pub regime: String,
    /// How its economy was graded, when it lived on for years.
    pub economy: Vec<Check>,
}

fn check_world(stats: &MapStats) -> Vec<String> {
    let mut failures = Vec::new();
    let relief = stats.max_elevation_m - stats.min_elevation_m;
    if relief < MIN_RELIEF_M {
        failures.push(format!("relief {relief:.1} m < {MIN_RELIEF_M} m"));
    }
    let not_ocean = 1.0 - stats.ocean_fraction;
    if not_ocean < MIN_NOT_OCEAN {
        failures.push(format!(
            "land {:.0}% < {:.0}%",
            not_ocean * 100.0,
            MIN_NOT_OCEAN * 100.0
        ));
    }
    if stats.gentle_land_fraction < MIN_GENTLE {
        failures.push(format!(
            "gentle land {:.1}% < {:.0}%",
            stats.gentle_land_fraction * 100.0,
            MIN_GENTLE * 100.0
        ));
    }
    if stats.river_length_km < MIN_RIVER_KM {
        failures.push(format!(
            "rivers {:.2} km < {MIN_RIVER_KM} km",
            stats.river_length_km
        ));
    }
    failures
}

/// Saves, reloads and saves again; the reloaded map must equal the original and the second
/// save's raw section digests must equal the first's (ADR-0002 §6).
fn check_round_trip(sim: &mut Sim, content: &ContentRegistry) -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|e| format!("no temp folder: {e}"))?;
    let first_dir =
        SaveDir::create(temp.path().join("a"), SAVE_EXTENSION).map_err(|e| e.to_string())?;
    let second_dir =
        SaveDir::create(temp.path().join("b"), SAVE_EXTENSION).map_err(|e| e.to_string())?;
    let first = persist::save(sim, &first_dir, SaveKind::Manual, "smoke")
        .map_err(|e| format!("save failed: {e}"))?;
    let mut loaded =
        persist::load(&first.path, content).map_err(|e| format!("load failed: {e}"))?;
    if **loaded.map() != **sim.map() {
        return Err("the reloaded map differs".to_owned());
    }
    let second = persist::save(&mut loaded, &second_dir, SaveKind::Manual, "smoke")
        .map_err(|e| format!("second save failed: {e}"))?;
    let digests = |chunks: &[commons_persist::ChunkInfo]| {
        chunks
            .iter()
            .map(|c| (c.tag, c.index, c.raw_digest))
            .collect::<Vec<_>>()
    };
    if digests(&first.chunks) != digests(&second.chunks) {
        return Err("a section changed in the save → load → save round trip".to_owned());
    }
    Ok(())
}

/// Lives the founding band's first month and checks what must always hold.
fn check_people(sim: &mut Sim, content: &ContentRegistry) -> Vec<String> {
    let mut failures = Vec::new();
    if let Some(why) = sim.founding_problem() {
        return vec![format!("the founding band could not settle: {why}")];
    }
    let band = content.people.params.band.default_size as usize;
    if sim.people().living() != band {
        failures.push(format!(
            "{} people arrived instead of {band}",
            sim.people().living()
        ));
    }
    match sim.advance_minutes(DAYS * MINUTES_PER_DAY) {
        Ok(advance) if i64::from(advance.days) == DAYS => {}
        Ok(advance) => failures.push(format!(
            "the clock crossed {} day boundaries in {DAYS} days",
            advance.days
        )),
        Err(e) => failures.push(format!("the clock stopped: {e}")),
    }
    let now = sim.now();
    let stuck = sim
        .people()
        .people
        .iter()
        .filter(|(_, p)| p.act.step_ends < now)
        .count();
    if stuck > 0 {
        failures.push(format!("{stuck} people are stuck between events"));
    }
    let problems = sim
        .people()
        .problems(sim.ids().peek_next(), sim.rules().catalog.activities.len());
    if let Some(first) = problems.first() {
        failures.push(format!(
            "{} population problems, first: {first}",
            problems.len()
        ));
    }
    let homeless = sim
        .people()
        .households
        .iter()
        .filter(|(_, h)| {
            let cell = civ_agents::population::cell_of(sim.map(), h.home);
            !sim.nav().walkable(cell)
        })
        .count();
    if homeless > 0 {
        failures.push(format!("{homeless} homes stand where nobody can walk"));
    }
    let per_person = content.people.params.household.water_l_per_person_day;
    let dry = sim
        .people()
        .households
        .iter()
        .filter(|(_, h)| h.water_at_time(now, h.members.len() as f64 * per_person) <= 0.0)
        .count();
    if dry > 0 {
        failures.push(format!("{dry} households have no water"));
    }
    let rules = sim.rules();
    let goods = &rules.catalog.goods;
    let cold = sim
        .people()
        .households
        .iter()
        .filter(|(_, h)| {
            let stores = civ_agents::population::stores_now(h, now, &rules.people, goods);
            civ_agents::person::fuel_kg(&stores, goods) <= 0.0
        })
        .count();
    if cold * 2 > sim.people().households.len() {
        failures.push(format!(
            "{cold} of {} households have no firewood",
            sim.people().households.len()
        ));
    }
    // The band arrives with stores for months: a month in, no settlement may be short.
    for s in &sim.land().settlements {
        if s.food_short {
            failures.push(format!("{} ran short of food within {DAYS} days", s.name));
        }
    }
    // Arriving on the first of March, they break ground and sow once the window opens.
    let sown = sim
        .land()
        .fields
        .iter()
        .filter(|f| f.stage == civ_land::FieldStage::Sown)
        .count();
    if sown == 0 {
        failures.push(format!(
            "no field sown within {DAYS} days ({} marked out)",
            sim.land().fields.len()
        ));
    }
    // In their first weeks most households claim ground for their homes and begin their huts,
    // on ground nobody else has claimed. A household short of hands may put its fields first and
    // build after sowing.
    let land = sim.land();
    let households = sim.people().households.len();
    let begun = sim
        .people()
        .households
        .iter()
        .filter(|(_, h)| land.buildings.iter().any(|b| b.household == h.id))
        .count();
    if begun * 2 < households {
        failures.push(format!(
            "only {begun} of {households} households began a home within {DAYS} days"
        ));
    }
    let crowded = land
        .plots
        .iter()
        .enumerate()
        .filter(|(i, p)| {
            land.plots[i + 1..].iter().any(|q| p.rect.near(&q.rect, 0))
                || land.fields.iter().any(|f| p.rect.near(&f.rect, 0))
        })
        .count();
    if crowded > 0 {
        failures.push(format!("{crowded} plots overlap another plot or a field"));
    }
    let problems = land.problems(
        sim.map(),
        sim.rules().land.habitats.len(),
        sim.ids().peek_next(),
    );
    if let Some(first) = problems.first() {
        failures.push(format!("{} land problems, first: {first}", problems.len()));
    }
    failures
}

/// Lives a world on for `years` after its first month, checking it at every year's end: one
/// year, two years and so on from the founding.
fn check_years(sim: &mut Sim, years: u32, result: &mut SmokeResult) {
    let founders = sim.people().living().max(1);
    let year = civ_core::time::MINUTES_PER_YEAR;
    // The band has lived its first month (`check_people`).
    let founded = sim.now().minutes() - DAYS * MINUTES_PER_DAY;
    // The books at the start: every good held is accounted for from here on (ADR-0006 §3).
    let books = (sim.people().goods_held(), sim.people().flows());
    let mut economy = Economy::default();
    for y in 1..=years {
        let end = founded + i64::from(y) * year;
        // Month by month, sampling the economy as each begins.
        loop {
            let d = sim.now().date();
            let (ny, nm) = if d.month == 12 {
                (d.year + 1, 1)
            } else {
                (d.year, d.month + 1)
            };
            let next = SimTime::from_date(ny, nm, 1, 0, 0).map_or(end, |t| t.minutes());
            let to = next.min(end);
            if let Err(e) = sim.advance_minutes(to - sim.now().minutes()) {
                result
                    .failures
                    .push(format!("the clock stopped in year {y}: {e}"));
                return;
            }
            if to == end {
                break;
            }
            economy.sample(sim);
        }
        result.failures.extend(economy.year_end(sim, y));
        let now = sim.now();
        let living = sim.people().living();
        let stuck = sim
            .people()
            .people
            .iter()
            .filter(|(_, p)| p.act.step_ends < now)
            .count();
        if stuck > 0 {
            result
                .failures
                .push(format!("{stuck} people stuck between events in year {y}"));
        }
        let problems = sim
            .people()
            .problems(sim.ids().peek_next(), sim.rules().catalog.activities.len());
        if let Some(first) = problems.first() {
            result.failures.push(format!(
                "{} population problems in year {y}, first: {first}",
                problems.len()
            ));
        }
        let land = sim.land();
        let problems = land.problems(
            sim.map(),
            sim.rules().land.habitats.len(),
            sim.ids().peek_next(),
        );
        if let Some(first) = problems.first() {
            result.failures.push(format!(
                "{} land problems in year {y}, first: {first}",
                problems.len()
            ));
        }
        let goods = &sim.rules().catalog.goods;
        let gaps = civ_agents::population::unaccounted(
            goods.len(),
            (&books.0, &books.1),
            (&sim.people().goods_held(), &sim.people().flows()),
        );
        if let Some(&(g, kg)) = gaps.first() {
            result.failures.push(format!(
                "{} goods unaccounted for in year {y}, first: {} {kg:+.3}",
                gaps.len(),
                goods[g].name
            ));
        }
        if living as f64 > MAX_GROWTH * founders as f64 {
            result
                .failures
                .push(format!("{living} people in year {y}: the band exploded"));
        }
        let households: Vec<_> = sim
            .people()
            .households
            .iter()
            .filter(|(_, h)| !h.members.is_empty())
            .map(|(_, h)| h.id)
            .collect();
        // Under a roof of their own home: a store or a workshop is no home.
        let catalog = &sim.rules().catalog;
        let roofed = households
            .iter()
            .filter(|&&h| {
                land.buildings
                    .iter()
                    .any(|b| b.household == h && b.roofed() && catalog.is_dwelling(&b.spec.program))
            })
            .count();
        if y >= 2 && (roofed as f64) < MIN_ROOFED * households.len() as f64 {
            result.failures.push(format!(
                "only {roofed} of {} households under a roof in year {y}",
                households.len()
            ));
        }
        if let Some(failure) = check_knowledge(sim, y) {
            result.failures.push(failure);
        }
        if y == 1
            && living > 0
            && !sim
                .people()
                .chronicle
                .iter()
                .any(|e| e.kind == civ_agents::ChronicleKind::FirstTrail)
        {
            result
                .failures
                .push("no trail worn out of the settlement in its first year".to_owned());
        }
        if living < MIN_ALIVE && !result.notes.iter().any(|n| n.starts_with("the band")) {
            result
                .notes
                .push(format!("the band was down to {living} in year {y}"));
        }
        result.living = Some(living);
    }
    result.economy = economy.grade(sim);
    for c in result.economy.iter().filter(|c| c.grade == Grade::Red) {
        result.failures.push(format!("{}: {}", c.name, c.text));
    }
    result.notes.extend(lost_where_people_live(sim));
    result.notes.extend(finds(sim));
    result.notes.extend(condition(sim));
}

/// How the world's buildings stand at the end (ADR-0009 §4), in words: "14 buildings, 3 showing
/// wear, 1 damaged, no ruin; 6 groups mended". `None` without buildings.
fn condition(sim: &Sim) -> Option<String> {
    use civ_land::{BuildingState, GroupState};
    let buildings = &sim.land().buildings;
    if buildings.is_empty() {
        return None;
    }
    let count = |state: BuildingState| buildings.iter().filter(|b| b.state == state).count();
    let showing = buildings
        .iter()
        .filter(|b| {
            b.state == BuildingState::Standing
                && b.condition.iter().any(|c| c.state != GroupState::Sound)
        })
        .count();
    let mended: usize = buildings
        .iter()
        .map(|b| {
            b.condition
                .iter()
                .filter(|c| c.repaired > c.installed)
                .count()
        })
        .sum();
    let ruins = match count(BuildingState::Ruin) {
        0 => "no ruin".to_owned(),
        1 => "1 ruin".to_owned(),
        n => format!("{n} ruins"),
    };
    Some(format!(
        "{} buildings, {showing} showing wear, {} damaged, {ruins}; {mended} groups mended",
        buildings.len(),
        count(BuildingState::Damaged)
    ))
}

/// What people worked out (ADR-0008 §3), in words: "found drying and smoking in year 4".
fn finds(sim: &Sim) -> Vec<String> {
    sim.people()
        .chronicle
        .iter()
        .filter(|e| e.kind == civ_agents::ChronicleKind::TechniqueFound)
        .map(|e| {
            format!(
                "found {} in year {}",
                e.name.to_lowercase(),
                e.at.date().year
            )
        })
        .collect()
}

/// Whether the techniques every founder brings and children learn in upbringing are known by
/// at least [`MIN_UPBRINGING`] of the living old enough for them: the least known one if not.
fn check_knowledge(sim: &Sim, y: u32) -> Option<String> {
    let (catalog, params) = (&sim.rules().catalog, &sim.rules().people);
    let now = sim.now();
    let mut worst: Option<(f64, usize, usize, &str)> = None;
    for &(t, share) in &params.knowledge.founders {
        let Some(def) = catalog.techniques.get(t) else {
            continue;
        };
        let age = civ_agents::knowledge::knowing_age(catalog, t, params.family.independent_age);
        let (Some(age), true) = (age, share >= 1.0 && def.upbringing) else {
            continue;
        };
        let (old_enough, knowing) = sim
            .people()
            .people
            .iter()
            .filter(|(_, p)| p.age_years(now) >= age)
            .fold((0, 0), |(n, k), (_, p)| {
                (n + 1, k + usize::from(p.knows(t)))
            });
        if old_enough == 0 {
            continue;
        }
        let coverage = knowing as f64 / old_enough as f64;
        if worst.is_none_or(|(c, ..)| coverage < c) {
            worst = Some((coverage, knowing, old_enough, &def.name));
        }
    }
    let (coverage, knowing, old_enough, name) = worst?;
    (coverage < MIN_UPBRINGING).then(|| {
        format!(
            "only {knowing} of the {old_enough} old enough know {} in year {y}",
            name.to_lowercase()
        )
    })
}

/// Techniques lost in a settlement where people still live, in words.
fn lost_where_people_live(sim: &Sim) -> Vec<String> {
    let people = sim.people();
    let lived_in: std::collections::BTreeSet<_> = people
        .households
        .iter()
        .filter(|(_, h)| !h.members.is_empty())
        .filter_map(|(_, h)| h.settlement)
        .collect();
    let catalog = &sim.rules().catalog;
    people
        .knowledge
        .iter()
        .filter(|e| {
            e.kind == civ_agents::knowledge::KnowledgeEventKind::Lost
                && lived_in.contains(&e.settlement)
        })
        .map(|e| {
            let name = catalog
                .techniques
                .get(usize::from(e.technique))
                .map_or("a technique", |d| d.name.as_str());
            format!("{} lost in year {}", name.to_lowercase(), e.at.date().year)
        })
        .collect()
}

/// Runs every preset with seeds `1..=seeds`, a few worlds at a time, reporting each result as it
/// finishes. The results come back in preset and seed order.
pub fn run(
    content: &ContentRegistry,
    options: SmokeOptions,
    report: &(dyn Fn(&SmokeResult) + Sync),
) -> Vec<SmokeResult> {
    let jobs: Vec<(String, u64)> = content
        .presets
        .iter()
        .flat_map(|p| (1..=options.seeds).map(move |seed| (p.id.clone(), seed)))
        .collect();
    let next = AtomicUsize::new(0);
    let results = Mutex::new(Vec::new());
    let workers = std::thread::available_parallelism()
        .map_or(1, |n| n.get())
        .min(jobs.len())
        .max(1);
    std::thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| {
                loop {
                    let i = next.fetch_add(1, Ordering::Relaxed);
                    let Some((preset, seed)) = jobs.get(i) else {
                        break;
                    };
                    let result = run_one(content, options, preset, *seed);
                    report(&result);
                    results
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .push((i, result));
                }
            });
        }
    });
    let mut results = results
        .into_inner()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    results.sort_by_key(|(i, _)| *i);
    results.into_iter().map(|(_, r)| r).collect()
}

fn run_one(
    content: &ContentRegistry,
    options: SmokeOptions,
    preset: &str,
    seed: u64,
) -> SmokeResult {
    let started = Instant::now();
    // The content's regimes in turn, by seed.
    let regimes = &content.catalog.regimes;
    let regime = regimes
        .get((seed.max(1) - 1) as usize % regimes.len().max(1))
        .map_or_else(
            || (String::new(), String::new()),
            |r| (r.id.clone(), r.name.clone()),
        );
    let created = Sim::create(
        &NewWorld {
            name: format!("Smoke {seed}"),
            seed,
            preset_id: preset.to_owned(),
            size_cells: options.size,
            band_size: 0,
            regime_id: regime.0,
        },
        content,
        &mut |_| {},
        &AtomicBool::new(false),
    );
    let mut result = SmokeResult {
        preset: preset.to_owned(),
        seed,
        elapsed: Duration::ZERO,
        stats: None,
        failures: Vec::new(),
        living: None,
        notes: Vec::new(),
        regime: regime.1,
        economy: Vec::new(),
    };
    match created {
        Err(e) => result.failures.push(format!("generation failed: {e}")),
        Ok(mut sim) => {
            result.stats = Some(*sim.stats());
            result.failures.extend(check_world(sim.stats()));
            if let Err(e) = check_round_trip(&mut sim, content) {
                result.failures.push(e);
            }
            result.failures.extend(check_people(&mut sim, content));
            if options.years > 0 && result.failures.is_empty() {
                check_years(&mut sim, options.years, &mut result);
            }
        }
    }
    result.elapsed = started.elapsed();
    result
}

/// Whether enough bands still live where they settled after a long run: at least half.
pub fn enough_alive(results: &[SmokeResult]) -> bool {
    let long: Vec<_> = results.iter().filter_map(|r| r.living).collect();
    long.is_empty() || 2 * long.iter().filter(|&&n| n >= MIN_ALIVE).count() >= long.len()
}

/// One line of the report table.
pub fn format_result(result: &SmokeResult) -> String {
    let figures = result.stats.map_or_else(String::new, |s| {
        format!(
            "{:>6.0} m {:>5.0}% {:>6.0}% {:>7} {:>9.1} {:>5}",
            s.max_elevation_m - s.min_elevation_m,
            (1.0 - s.ocean_fraction) * 100.0,
            s.gentle_land_fraction * 100.0,
            s.reaches,
            s.river_length_km,
            s.lakes
        )
    });
    let mut verdict = if result.failures.is_empty() {
        "pass".to_owned()
    } else {
        format!("FAIL: {}", result.failures.join("; "))
    };
    if let Some(living) = result.living {
        verdict = format!("{living:>3} people  {verdict}");
    }
    if !result.notes.is_empty() {
        verdict = format!("{verdict} ({})", result.notes.join("; "));
    }
    let mut line = format!(
        "{:<28} {:>4} {:>6.1} s {figures}  {verdict}",
        result.preset,
        result.seed,
        result.elapsed.as_secs_f64()
    );
    if !result.economy.is_empty() {
        let checks: Vec<String> = result
            .economy
            .iter()
            .map(|c| format!("{} ({})", c.text, c.grade.word()))
            .collect();
        line.push_str(&format!(
            "\n    {}: {}",
            result.regime.to_lowercase(),
            checks.join("; ")
        ));
    }
    line
}

/// How the worlds' economies were graded, check by check: how many came out at each grade.
pub fn economy_summary(results: &[SmokeResult]) -> Option<String> {
    let names: Vec<&str> = results
        .iter()
        .flat_map(|r| r.economy.iter().map(|c| c.name))
        .fold(Vec::new(), |mut v, n| {
            if !v.contains(&n) {
                v.push(n);
            }
            v
        });
    if names.is_empty() {
        return None;
    }
    let parts: Vec<String> = names
        .iter()
        .map(|&name| {
            let grades: Vec<String> = [Grade::Green, Grade::Amber, Grade::Gray, Grade::Red]
                .iter()
                .filter_map(|&g| {
                    let n = results
                        .iter()
                        .flat_map(|r| &r.economy)
                        .filter(|c| c.name == name && c.grade == g)
                        .count();
                    (n > 0).then(|| format!("{n} {}", g.word()))
                })
                .collect();
            format!("{name} {}", grades.join(", "))
        })
        .collect();
    Some(format!("economy: {}", parts.join("; ")))
}

/// The report table's header.
pub fn header() -> String {
    format!(
        "{:<28} {:>4} {:>8} {:>8} {:>6} {:>7} {:>7} {:>9} {:>5}  result",
        "preset", "seed", "time", "relief", "land", "gentle", "reaches", "river km", "lakes"
    )
}
