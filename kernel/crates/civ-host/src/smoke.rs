//! Smoke seeds (plan §4.7): a few worlds per preset, checked against fixed thresholds. No
//! statistics; any failure blocks the milestone.
//!
//! The checks cover terrain and water, the save round trip, and a month of the founding band's
//! life: they settle, keep water at home, and nobody is ever stuck between events. Checks on a
//! population over years arrive with births and deaths (plan §7, M1).

use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

use civ_content::ContentRegistry;
use civ_core::time::MINUTES_PER_DAY;
use civ_schema::SAVE_EXTENSION;
use civ_sim::{NewWorld, Sim, persist};
use civ_world::MapStats;
use commons_persist::{SaveDir, SaveKind};

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

/// What to run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SmokeOptions {
    /// Cells per side of each world.
    pub size: u32,
    /// Seeds per preset, starting at 1.
    pub seeds: u64,
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
    failures
}

/// Runs every preset with seeds `1..=seeds`, reporting each result as it finishes.
pub fn run(
    content: &ContentRegistry,
    options: SmokeOptions,
    report: &mut dyn FnMut(&SmokeResult),
) -> Vec<SmokeResult> {
    let mut results = Vec::new();
    for preset in &content.presets {
        for seed in 1..=options.seeds {
            let started = Instant::now();
            let created = Sim::create(
                &NewWorld {
                    name: format!("Smoke {seed}"),
                    seed,
                    preset_id: preset.id.clone(),
                    size_cells: options.size,
                    band_size: 0,
                },
                content,
                &mut |_| {},
                &AtomicBool::new(false),
            );
            let mut result = SmokeResult {
                preset: preset.id.clone(),
                seed,
                elapsed: Duration::ZERO,
                stats: None,
                failures: Vec::new(),
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
                }
            }
            result.elapsed = started.elapsed();
            report(&result);
            results.push(result);
        }
    }
    results
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
    let verdict = if result.failures.is_empty() {
        "pass".to_owned()
    } else {
        format!("FAIL: {}", result.failures.join("; "))
    };
    format!(
        "{:<28} {:>4} {:>6.1} s {figures}  {verdict}",
        result.preset,
        result.seed,
        result.elapsed.as_secs_f64()
    )
}

/// The report table's header.
pub fn header() -> String {
    format!(
        "{:<28} {:>4} {:>8} {:>8} {:>6} {:>7} {:>7} {:>9} {:>5}  result",
        "preset", "seed", "time", "relief", "land", "gentle", "reaches", "river km", "lakes"
    )
}
