//! The M9 demo (plan §7, M9c slice DN; ADR-0032 §8): a world of many founding groups, one
//! settlement followed in full detail and the rest household-resolved by the observer's focus,
//! lived ahead at Max for years with the climate forced, and every person still one record the
//! observer can open. The observer acts only as the demo asks, each act logged: it turns
//! resolution by focus on and follows the eldest of the first settlement's largest household (a
//! new one of that settlement when they die, `FOLLOWED`), and forces the climate on the first day
//! (`FORCED`). Nothing else is set and nothing is nudged.
//!
//! The demo also measures the wall time each simulated day takes at Max (ADR-0032 §8), and at
//! the end opens people of household-resolved settlements as the inspector does, saves the
//! world, loads it and checks that everyone is still there.
//!
//! ```sh
//! cargo run --release -p civ-sim --example m9_world -- <saves folder> [groups] [people] [years] [size]
//! ```

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use civ_core::PermanentId;
use civ_core::time::DAYS_PER_YEAR;
use civ_sim::{NewWorld, SPEED_MAX, Sim, persist};
use commons_persist::{SaveDir, SaveKind};

/// Founding groups, unless given: the gate's ten settlements.
const GROUPS: u32 = 10;
/// People in each founding group, unless given.
const PEOPLE: u32 = 500;
/// Years lived, unless given: two decades.
const YEARS: i64 = 20;
/// The map's side in cells, unless given.
const SIZE: u32 = 2048;
/// The forcing (ADR-0032 §9): 3 °C warmer and wet days a fifth drier, reached over twenty years.
const OFFSET_C: f64 = 3.0;
const WET_FACTOR: f64 = 0.8;
const RAMP_YEARS: u32 = 20;
/// How many people of household-resolved settlements are opened at the end.
const OPENED: usize = 3;
const DAY: i64 = 24 * 60;
const NAME: &str = "Many Hearths";

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let Some(saves) = args.next().map(PathBuf::from) else {
        eprintln!("usage: m9_world <saves folder> [groups] [people] [years] [size]");
        return ExitCode::FAILURE;
    };
    let groups: u32 = args.next().and_then(|s| s.parse().ok()).unwrap_or(GROUPS);
    let people: u32 = args.next().and_then(|s| s.parse().ok()).unwrap_or(PEOPLE);
    let years: i64 = args.next().and_then(|s| s.parse().ok()).unwrap_or(YEARS);
    let size: u32 = args.next().and_then(|s| s.parse().ok()).unwrap_or(SIZE);
    match make(&saves, groups, people, years, size) {
        // Each line was printed as it came, so a run cut short still says how far it got.
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("the demo failed: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Prints a line of the demo's record as it comes.
fn say(line: String) {
    use std::io::Write;
    println!("{line}");
    let _ = std::io::stdout().flush();
}

/// The eldest living member of settlement `s`'s largest household.
fn eldest_of_largest(sim: &Sim, s: PermanentId) -> Option<PermanentId> {
    let now = sim.now();
    let pop = sim.people();
    pop.households
        .iter()
        .map(|(_, h)| h)
        .filter(|h| h.settlement == Some(s) && !h.members.is_empty())
        .max_by_key(|h| (h.members.len(), std::cmp::Reverse(h.id)))
        .and_then(|h| {
            h.members
                .iter()
                .filter_map(|&m| pop.person(m))
                .max_by(|a, b| a.age_years(now).total_cmp(&b.age_years(now)))
                .map(|p| p.id)
        })
}

/// The settlement person `p` lives in.
fn settlement_of(sim: &Sim, p: PermanentId) -> Option<PermanentId> {
    let pop = sim.people();
    pop.household(pop.person(p)?.household)?.settlement
}

fn settlement_name(sim: &Sim, s: PermanentId) -> String {
    sim.land()
        .settlements
        .iter()
        .find(|x| x.id == s)
        .map_or_else(|| "another settlement".to_owned(), |x| x.name.clone())
}

/// Person `id` as the inspector opens them: who they are, where they live and, in a
/// household-resolved settlement, the block of their day the clock stands in.
fn opened(sim: &Sim, id: PermanentId) -> String {
    let pop = sim.people();
    let Some(p) = pop.person(id) else {
        return format!("person {id} is no longer living");
    };
    let now = sim.now();
    let catalog = &sim.rules().catalog;
    let parents: Vec<String> = [p.mother, p.father]
        .into_iter()
        .flatten()
        .map(|x| pop.name_of(x))
        .collect();
    let known = (0..catalog.techniques.len())
        .filter(|&t| p.knows(t))
        .count();
    let place = settlement_of(sim, id).map_or_else(String::new, |s| settlement_name(sim, s));
    let day = match pop.day_block(id, now) {
        Some(b) => {
            let doing = catalog
                .activities
                .get(usize::from(b.def))
                .map_or("busy", |d| d.doing.as_str());
            format!(
                "their day lived by household: {}{doing} until {:02}:{:02}",
                if b.asleep { "asleep, " } else { "" },
                b.until / 60,
                b.until % 60
            )
        }
        None => civ_sim::frames::people::doing(sim, p),
    };
    format!(
        "{} (person {id}), {:.0}, of {} at {place}; child of {}; knows {known} techniques; {day}",
        pop.name_of(id),
        p.age_years(now),
        civ_sim::frames::people::household_name(sim, p.household),
        if parents.is_empty() {
            "founders unrecorded".to_owned()
        } else {
            parents.join(" and ")
        },
    )
}

/// The year's chronicle, as counts of each kind of entry a skip can stop at.
fn year_counts(sim: &Sim, after_seq: u64) -> String {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for e in sim
        .people()
        .chronicle
        .iter()
        .rev()
        .take_while(|e| e.seq > after_seq)
        .filter(|e| civ_sim::skip::mark_of(e.kind) != 0)
    {
        *counts.entry(format!("{:?}", e.kind)).or_default() += 1;
    }
    if counts.is_empty() {
        return "nothing a skip stops at".to_owned();
    }
    counts
        .iter()
        .map(|(k, n)| format!("{n} {k}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn make(saves: &Path, groups: u32, people: u32, years: i64, size: u32) -> Result<(), String> {
    let root = civ_content::find_content_root(Path::new(env!("CARGO_MANIFEST_DIR")))
        .ok_or("content/ is not above the crate")?;
    let mut content = civ_content::load(&root)
        .registry
        .ok_or("the content does not load: run `civ-host content validate`")?;
    // Founded large: the content's band limits widened for this run only, as the bench does.
    let band = &mut content.people.params.band;
    band.max_size = band.max_size.max(people);
    band.min_size = band.min_size.min(people);
    let made = Instant::now();
    let mut sim = Sim::create_for_tests(
        &NewWorld {
            name: NAME.to_owned(),
            seed: 1,
            preset_id: content.default_preset().id.clone(),
            size_cells: size,
            band_size: people,
            neighbours: vec![people; groups.saturating_sub(1) as usize],
            neighbours_known: true,
            regime_id: String::new(),
        },
        &content,
        // A fixed identity, so the demo lives the same life every run.
        [7; 16],
    )
    .map_err(|e| e.to_string())?;
    if let Some(problem) = sim.founding_problem() {
        return Err(format!("the founding groups could not settle: {problem}"));
    }
    let settlements: Vec<PermanentId> = sim.land().settlements.iter().map(|s| s.id).collect();
    say(format!(
        "world (seed 1, {size} cells): {} people in {} settlements, founded in {:.1} s",
        sim.people().living(),
        settlements.len(),
        made.elapsed().as_secs_f64()
    ));
    let first = *settlements.first().ok_or("nothing was founded")?;
    sim.set_speed(SPEED_MAX).map_err(|e| e.to_string())?;
    // The observer: resolution by focus, following one person of the first settlement.
    sim.set_resolution_by_focus(true);
    sim.set_focus(Some(first));
    let mut followed = eldest_of_largest(&sim, first).ok_or("the first settlement is empty")?;
    sim.set_follow(Some(followed));
    say(format!(
        "day {}: FOLLOWED (observer): {}",
        sim.now().day_index(),
        opened(&sim, followed)
    ));
    let reached = sim.force_climate(OFFSET_C, WET_FACTOR, RAMP_YEARS)?;
    say(format!(
        "day {}: FORCED (observer): the climate {OFFSET_C:+} °C and wet days' rain ×{WET_FACTOR}, \
         over {RAMP_YEARS} years ({reached:?})",
        sim.now().day_index()
    ));
    let started = Instant::now();
    sim.advance_to_midnight().map_err(|e| e.to_string())?;
    say(format!(
        "the founding day lived in full detail to midnight in {:.1} s",
        started.elapsed().as_secs_f64()
    ));
    let world_dir = saves.join(persist::world_dir_name(sim.meta()));
    let dir = SaveDir::create(world_dir, civ_schema::SAVE_EXTENSION).map_err(|e| e.to_string())?;
    let days = years * DAYS_PER_YEAR;
    let mut walls: Vec<f64> = Vec::with_capacity(days as usize);
    let (mut year_c, mut year_mm, mut year_days) = (0.0f64, 0.0f64, 0u32);
    let mut seq = sim.chronicle_head();
    for day in 1..=days {
        let t = Instant::now();
        sim.advance_minutes(DAY).map_err(|e| e.to_string())?;
        walls.push(t.elapsed().as_secs_f64());
        let today = sim.now().day_index();
        let w = &sim.land().weather.today;
        year_c += f64::from(w.mean_c);
        year_mm += f64::from(w.precip_mm);
        year_days += 1;
        if sim.people().living() == 0 {
            say(format!("day {today}: the land is empty"));
            break;
        }
        // The followed person died or left the first settlement: the observer follows the eldest
        // of its largest household now.
        if settlement_of(&sim, followed) != Some(first)
            && let Some(next) = eldest_of_largest(&sim, first)
        {
            followed = next;
            sim.set_follow(Some(followed));
            say(format!(
                "day {today}: FOLLOWED (observer): {}",
                opened(&sim, followed)
            ));
        }
        if day % DAYS_PER_YEAR == 0 {
            let recent = &walls[walls.len().saturating_sub(DAYS_PER_YEAR as usize)..];
            let resolved = sim.household_resolved().len();
            say(format!(
                "year {}: {} people in {} households; {resolved} of {} settlements \
                 household-resolved; {} buildings, {} fields; the year {:.1} °C on average, {:.0} \
                 mm of rain and snow; chronicle: {}; {:.2} s a day",
                day / DAYS_PER_YEAR,
                sim.people().living(),
                sim.people()
                    .households
                    .iter()
                    .filter(|(_, h)| !h.members.is_empty())
                    .count(),
                sim.land().settlements.len(),
                sim.land().buildings.len(),
                sim.land().fields.len(),
                year_c / f64::from(year_days.max(1)),
                year_mm,
                year_counts(&sim, seq),
                recent.iter().sum::<f64>() / recent.len().max(1) as f64
            ));
            say(format!("  followed: {}", opened(&sim, followed)));
            (year_c, year_mm, year_days) = (0.0, 0.0, 0);
            seq = sim.chronicle_head();
        }
    }
    say(format!(
        "lived {} days, to day {}; {} people",
        walls.len(),
        sim.now().day_index(),
        sim.people().living()
    ));
    for s in &sim.land().settlements {
        let residents = sim
            .people()
            .households
            .iter()
            .filter(|(_, h)| h.settlement == Some(s.id))
            .map(|(_, h)| h.members.len())
            .sum::<usize>();
        say(format!(
            "{}: {residents} residents, {}",
            s.name,
            if sim.household_resolved().contains(&s.id) {
                "household-resolved"
            } else {
                "Detailed"
            }
        ));
    }
    // Every person still one record: some of the household-resolved settlements, opened.
    let resolved: Vec<PermanentId> = sim.household_resolved().iter().copied().collect();
    let mut opened_ids = Vec::new();
    for s in resolved.iter().take(OPENED) {
        if let Some(p) = eldest_of_largest(&sim, *s) {
            say(format!("opened: {}", opened(&sim, p)));
            opened_ids.push(p);
        }
    }
    // The wall time a simulated day at Max (ADR-0032 §8).
    let mut sorted = walls.clone();
    sorted.sort_by(f64::total_cmp);
    let mean = walls.iter().sum::<f64>() / walls.len().max(1) as f64;
    let at = |q: f64| sorted[((sorted.len() as f64 - 1.0) * q).round() as usize];
    say(format!(
        "speed: {mean:.2} s a day on average (p50 {:.2} s, p95 {:.2} s), {:.3} simulated years a \
         real minute",
        at(0.5),
        at(0.95),
        60.0 / (mean * DAYS_PER_YEAR as f64)
    ));
    let saved = persist::save(&mut sim, &dir, SaveKind::Manual, "After the years")
        .map_err(|e| e.to_string())?;
    let bytes = std::fs::metadata(&saved.path).map_or(0, |m| m.len());
    let loaded = persist::load(&saved.path, &content).map_err(|e| e.to_string())?;
    let same = opened_ids.iter().all(|&p| {
        loaded.people().person(p).map(|x| &x.given) == sim.people().person(p).map(|x| &x.given)
    });
    say(format!(
        "saved ({:.1} MB) and loaded: {} people against {}; the people opened {}",
        bytes as f64 / 1e6,
        loaded.people().living(),
        sim.people().living(),
        if same {
            "are all there"
        } else {
            "are not all there"
        }
    ));
    Ok(())
}
