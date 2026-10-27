//! The M10 demo (plan §7, M10 slice EL): a world of several settlements, some with a fall or coal
//! and some without, lived ahead at Max until whoever will has built a mill or a works, laid a
//! line, lit a street, kept a school or issued notes; what was built where, by whom and why, and
//! what the others did instead. Whether any settlement industrialises is the people's.
//!
//! The observer acts only as the demo asks, each act logged: on the first day, in every other
//! settlement (the first, the third, ...), the eldest of its largest household comes to know the
//! M10 vocabulary by the god tool (ADR-0008 §6, `INTRODUCED`); the others are left as founded, the
//! control. Resolution is by focus on the first settlement, as M9's demo has it. Nothing else is
//! set and nothing is nudged; a nudge, if one were made, is logged as `NUDGE:`.
//!
//! Each year it prints, settlement by settlement, what of the M10 vocabulary is in use there: who
//! knows any of it, mills and engines, workshops open, lines laid, gas mains and street lamps,
//! schools and the other M10 laws in force, notes out and records written. At the end it prints
//! the chronicle's entries of the M10 vocabulary with their words, saves, loads, and says whether
//! the control settlements took any of it up.
//!
//! ```sh
//! cargo run --release -p civ-sim --example m10_world -- <saves folder> [groups] [people] [years] [size]
//! ```

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use civ_agents::history::ChronicleKind;
use civ_agents::polity::{LawStatus, PolicyKind};
use civ_core::PermanentId;
use civ_core::time::DAYS_PER_YEAR;
use civ_sim::{NewWorld, SPEED_MAX, Sim, persist};
use commons_persist::{SaveDir, SaveKind};

/// Founding groups, unless given: three that learn and three that do not.
const GROUPS: u32 = 6;
/// People in each founding group, unless given.
const PEOPLE: u32 = 120;
/// Years lived, unless given: the time box.
const YEARS: i64 = 15;
/// The map's side in cells, unless given.
const SIZE: u32 = 2048;
const DAY: i64 = 24 * 60;
const NAME: &str = "Wheels and Notes";

/// The M10 vocabulary the observer introduces (plan §7, M10): power and the works, rail and gas
/// light, care and letters. Notes need no technique of their own but writing.
const VOCABULARY: [&str; 19] = [
    "core:technique/water_wheel",
    "core:technique/water_hammer",
    "core:technique/coal_burning",
    "core:technique/coking",
    "core:technique/bloomery",
    "core:technique/forging",
    "core:technique/blast_furnace",
    "core:technique/atmospheric_engine",
    "core:technique/wooden_railway",
    "core:technique/locomotive",
    "core:technique/lamps",
    "core:technique/gas_making",
    "core:technique/reckoning",
    "core:technique/reading",
    "core:technique/writing",
    "core:technique/variolation",
    "core:technique/vaccination",
    "core:technique/clean_hands",
    "core:technique/birth_attending",
];

/// The law kinds M10 brought (plan §7, M10a-M10d).
const LAWS: [PolicyKind; 12] = [
    PolicyKind::BuildLine,
    PolicyKind::LightStreets,
    PolicyKind::KeepSchool,
    PolicyKind::Licence,
    PolicyKind::KeepApart,
    PolicyKind::ClearMiddens,
    PolicyKind::BearerHeard,
    PolicyKind::IssueNotes,
    PolicyKind::LegalTender,
    PolicyKind::SuspendRedemption,
    PolicyKind::AgainstCounterfeiting,
    PolicyKind::PayWork,
];

/// The chronicle's entries of the M10 vocabulary printed at the end.
const KINDS: [ChronicleKind; 2] = [ChronicleKind::LineLaid, ChronicleKind::RecordWritten];

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let Some(saves) = args.next().map(PathBuf::from) else {
        eprintln!("usage: m10_world <saves folder> [groups] [people] [years] [size]");
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

fn settlement_name(sim: &Sim, s: PermanentId) -> String {
    sim.land()
        .settlements
        .iter()
        .find(|x| x.id == s)
        .map_or_else(|| "another settlement".to_owned(), |x| x.name.clone())
}

/// The settlement of owner `id`: a household's, a firm's, or a polity's seat.
fn home_of(sim: &Sim, id: PermanentId) -> Option<PermanentId> {
    let pop = sim.people();
    if let Some(h) = pop.household(id) {
        return h.settlement;
    }
    if let Some(f) = pop.firms.iter().find(|f| f.id == id) {
        return f.settlement;
    }
    pop.polities
        .iter()
        .find(|p| p.id == id)
        .map(|p| p.settlement)
}

/// What of the M10 vocabulary settlement `s` uses now, in words, and whether it uses any.
fn in_use(sim: &Sim, s: PermanentId, known: &[usize]) -> (String, bool) {
    let pop = sim.people();
    let land = sim.land();
    let residents: Vec<PermanentId> = pop
        .households
        .iter()
        .filter(|(_, h)| h.settlement == Some(s))
        .flat_map(|(_, h)| h.members.clone())
        .collect();
    let knowers = residents
        .iter()
        .filter(|&&m| {
            pop.person(m)
                .is_some_and(|p| known.iter().any(|&t| p.knows(t)))
        })
        .count();
    let here = |id: PermanentId| home_of(sim, id) == Some(s);
    let mills = land.installations.iter().filter(|i| here(i.owner)).count();
    let engines = land.engines.iter().filter(|e| here(e.owner)).count();
    let workshops = pop
        .firms
        .iter()
        .filter(|f| f.is_open() && f.settlement == Some(s))
        .count();
    let lines = pop
        .chronicle
        .iter()
        .filter(|e| e.kind == ChronicleKind::LineLaid && e.settlement == Some(s))
        .count();
    let mains = land
        .gas
        .mains
        .iter()
        .filter(|m| here(m.works) && m.whole())
        .count();
    let lamps = land.gas.lamps.iter().filter(|l| here(l.polity)).count();
    let laws: Vec<&str> = pop
        .polities
        .iter()
        .filter(|p| p.settlement == s)
        .flat_map(|p| p.laws.iter())
        .filter(|l| l.status == LawStatus::InForce && LAWS.contains(&l.kind))
        .map(|l| l.kind.name())
        .collect();
    let notes: f64 = pop
        .notes
        .issues
        .iter()
        .filter(|c| here(c.issuer))
        .map(|c| c.count)
        .sum::<f64>()
        // An empty sum is -0, which would read "-0 notes".
        + 0.0;
    let records = pop
        .chronicle
        .iter()
        .filter(|e| e.kind == ChronicleKind::RecordWritten && e.settlement == Some(s))
        .count();
    let any =
        mills + engines + lines + mains + lamps + records > 0 || !laws.is_empty() || notes > 1e-6;
    (
        format!(
            "{} people, {knowers} knowing any of it; {mills} mills, {engines} engines, \
             {workshops} workshops open, {lines} lines laid, {mains} gas mains, {lamps} street \
             lamps; laws: {}; {notes:.0} notes out; {records} records written",
            residents.len(),
            if laws.is_empty() {
                "none of M10's".to_owned()
            } else {
                laws.join(", ")
            }
        ),
        any,
    )
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
        [10; 16],
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
    sim.set_resolution_by_focus(true);
    sim.set_focus(Some(first));
    // The observer: the M10 vocabulary to one person of every other settlement.
    let known: Vec<usize> = VOCABULARY
        .iter()
        .filter_map(|t| sim.rules().catalog.technique_index(t))
        .collect();
    let mut taught: BTreeSet<PermanentId> = BTreeSet::new();
    for (i, &s) in settlements.iter().enumerate() {
        let name = settlement_name(&sim, s);
        if i % 2 == 1 {
            say(format!("{name}: the control, as founded"));
            continue;
        }
        let Some(p) = eldest_of_largest(&sim, s) else {
            continue;
        };
        for t in VOCABULARY {
            sim.introduce_technique(p, t, false)?;
        }
        taught.insert(s);
        say(format!(
            "day {}: INTRODUCED (observer): {} of {name} comes to know the M10 vocabulary ({} \
             techniques)",
            sim.now().day_index(),
            sim.people().name_of(p),
            VOCABULARY.len()
        ));
    }
    let started = Instant::now();
    sim.advance_to_midnight().map_err(|e| e.to_string())?;
    say(format!(
        "the founding day lived to midnight in {:.1} s",
        started.elapsed().as_secs_f64()
    ));
    let world_dir = saves.join(persist::world_dir_name(sim.meta()));
    let dir = SaveDir::create(world_dir, civ_schema::SAVE_EXTENSION).map_err(|e| e.to_string())?;
    let days = years * DAYS_PER_YEAR;
    let mut walls: Vec<f64> = Vec::with_capacity(days as usize);
    for day in 1..=days {
        let t = Instant::now();
        sim.advance_minutes(DAY).map_err(|e| e.to_string())?;
        walls.push(t.elapsed().as_secs_f64());
        if sim.people().living() == 0 {
            say(format!("day {}: the land is empty", sim.now().day_index()));
            break;
        }
        if day % DAYS_PER_YEAR == 0 {
            let recent = &walls[walls.len().saturating_sub(DAYS_PER_YEAR as usize)..];
            say(format!(
                "year {}: {} people in {} settlements; {:.2} s a day",
                day / DAYS_PER_YEAR,
                sim.people().living(),
                sim.land().settlements.len(),
                recent.iter().sum::<f64>() / recent.len().max(1) as f64
            ));
            for s in &sim.land().settlements {
                let (words, _) = in_use(&sim, s.id, &known);
                let mark = if taught.contains(&s.id) {
                    "taught"
                } else {
                    "control"
                };
                say(format!("  {} ({mark}): {words}", s.name));
            }
        }
    }
    say(format!(
        "lived {} days, to day {}; {} people",
        walls.len(),
        sim.now().day_index(),
        sim.people().living()
    ));
    // The M10 laws put to a gathering, whoever proposed them and how they stand.
    say("the M10 laws proposed:".to_owned());
    let mut proposed = 0usize;
    for p in &sim.people().polities {
        for l in p.laws.iter().filter(|l| LAWS.contains(&l.kind)) {
            say(format!(
                "  day {} at {}: {} proposed by {}: {:?}",
                l.proposed.day_index(),
                settlement_name(&sim, p.settlement),
                l.kind.name(),
                sim.people().name_of(l.sponsor),
                l.status
            ));
            proposed += 1;
        }
    }
    if proposed == 0 {
        say("  none".to_owned());
    }
    // What was built where, by whom and why: the chronicle's own words.
    say("the chronicle's entries of the M10 vocabulary:".to_owned());
    let mut printed = 0usize;
    for e in sim
        .people()
        .chronicle
        .iter()
        .filter(|e| KINDS.contains(&e.kind))
    {
        let place = e
            .settlement
            .map_or_else(String::new, |s| settlement_name(&sim, s));
        say(format!(
            "  day {} at {place}: {:?}: {}",
            e.at.day_index(),
            e.kind,
            e.name
        ));
        printed += 1;
    }
    if printed == 0 {
        say("  none".to_owned());
    }
    // The control: whether any of it reached where nobody was taught (plan §7, M10: a world
    // without them lives as before), as people may carry it there.
    for s in &sim.land().settlements {
        if taught.contains(&s.id) {
            continue;
        }
        let (words, any) = in_use(&sim, s.id, &known);
        say(format!(
            "control {}: {}: {words}",
            s.name,
            if any { "took some up" } else { "as before" }
        ));
    }
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
    say(format!(
        "saved ({:.1} MB) and loaded: {} people against {}",
        bytes as f64 / 1e6,
        loaded.people().living(),
        sim.people().living()
    ));
    Ok(())
}
