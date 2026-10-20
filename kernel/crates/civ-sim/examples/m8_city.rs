//! The M8 demo (plan §7, M8d slice DE; ADR-0031 §8): two settlements founded large on a river
//! valley, the one whose hearth lies nearer the river its port and the other inland, live as the
//! content has them. The observer acts only as ADR-0031 §8 asks, each act logged: on the first
//! day (`TEACH_DAY`) it introduces framing to the grown members of each settlement's largest households
//! (nobody comes to know framing unaided, the probe of slice CY found), and laying out by cord and
//! stake to the inland settlement's largest (`INTRODUCED`); on day `days - SHAKE_BEFORE_END` it
//! shakes the whole map once at `STRENGTH`, both settlements alike. Nothing else is set: who
//! builds a frame, a second storey or a third, which public buildings are proposed, raised or
//! stall, whether the inland gathering lays out a plan, and which rules follow the shaking are
//! the people's. If nothing of a step has come by its time box, the observer may nudge a prior
//! (`NUDGE`, logged as such), never an event; this run's nudges are listed in `NUDGES`.
//!
//! The demo also measures the scale gate (ADR-0028 §1): the wall time each simulated day takes at
//! Max, and so how many simulated years a real minute lives at the size reached.
//!
//! ```sh
//! cargo run --profile simcheck -p civ-sim --example m8_city -- <saves folder> [seed] [people] [days]
//! ```

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use civ_agents::polity::PolicyKind;
use civ_core::PermanentId;
use civ_grammar::Footprint;
use civ_sim::{NewWorld, SPEED_MAX, Sim, persist};
use commons_persist::{SaveDir, SaveKind};

/// People in each founding group, unless given.
const PEOPLE: u32 = 750;
/// Days lived, unless given: three years.
const DAYS: i64 = 3 * 360;
/// The day the observer introduces framing and laying out: the first, before anyone has begun a
/// home, as a people who brought framing with them would build.
const TEACH_DAY: i64 = 1;
/// How many of each settlement's largest households are taught framing.
const TAUGHT: usize = 4;
/// How long before the end the ground is shaken: half a year, for the gatherings to answer it.
const SHAKE_BEFORE_END: i64 = 180;
/// The shaking's lateral share of every building's weight (ADR-0031 §4: within 0.02 to 0.6): a
/// strong one, for a trial at 0.25 left every hut standing.
const STRENGTH: f64 = 0.4;
const DAY: i64 = 24 * 60;
const NAME: &str = "Two Rivers";
/// The priors this run nudged, in words; none unless a time box ran out.
const NUDGES: [&str; 0] = [];

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let Some(saves) = args.next().map(PathBuf::from) else {
        eprintln!("usage: m8_city <saves folder> [seed] [people] [days] | m8_city <save>");
        return ExitCode::FAILURE;
    };
    // Given a save, says again what its world came to.
    if saves.is_file() {
        return match reread(&saves) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("the save could not be read: {e}");
                ExitCode::FAILURE
            }
        };
    }
    let seed: u64 = args.next().and_then(|s| s.parse().ok()).unwrap_or(1);
    let people: u32 = args.next().and_then(|s| s.parse().ok()).unwrap_or(PEOPLE);
    let days: i64 = args.next().and_then(|s| s.parse().ok()).unwrap_or(DAYS);
    match make(&saves, seed, people, days) {
        // Each line was printed as it came, so a run cut short still says how far it got.
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("the demo failed: {e}");
            ExitCode::FAILURE
        }
    }
}

/// The demo's record: each line printed as it comes, and kept.
#[derive(Default)]
struct Log(Vec<String>);

impl Log {
    fn push(&mut self, line: String) {
        use std::io::Write;
        println!("{line}");
        let _ = std::io::stdout().flush();
        self.0.push(line);
    }

    fn extend(&mut self, lines: impl IntoIterator<Item = String>) {
        for line in lines {
            self.push(line);
        }
    }
}

/// The distance from `at` (metres) to the nearest river cell, metres.
fn to_river_m(sim: &Sim, at: (f32, f32)) -> f32 {
    let map = sim.map();
    let cell = map.cell_size_m;
    let mut best = f32::MAX;
    for (i, &w) in map.water.iter().enumerate() {
        if w != civ_world::WATER_RIVER {
            continue;
        }
        let (x, y) = (
            (i as u32 % map.width) as f32 * cell + cell / 2.0,
            (i as u32 / map.width) as f32 * cell + cell / 2.0,
        );
        best = best.min((x - at.0).hypot(y - at.1));
    }
    best
}

/// The grown members of the `n` largest households of `settlement`, with their households.
fn grown_of_largest(
    sim: &Sim,
    settlement: PermanentId,
    n: usize,
) -> Vec<(PermanentId, PermanentId)> {
    let now = sim.now();
    let pop = sim.people();
    let mut households: Vec<_> = pop
        .households
        .iter()
        .map(|(_, h)| h)
        .filter(|h| h.settlement == Some(settlement))
        .collect();
    households.sort_by_key(|h| (std::cmp::Reverse(h.members.len()), h.id));
    households
        .iter()
        .take(n)
        .flat_map(|h| {
            h.members
                .iter()
                .filter_map(|&m| pop.person(m))
                .filter(|p| p.age_years(now) >= 16.0)
                .map(|p| (p.id, h.id))
                .collect::<Vec<_>>()
        })
        .collect()
}

/// Teaches `technique` to the grown members of `settlement`'s `n` largest households.
fn teach(
    sim: &mut Sim,
    settlement: PermanentId,
    n: usize,
    technique: &str,
    words: &str,
) -> Result<Vec<String>, String> {
    let today = sim.now().day_index();
    let t = sim
        .rules()
        .catalog
        .technique_index(technique)
        .ok_or_else(|| format!("{technique} is not in the content"))?;
    let mut out = Vec::new();
    for (p, h) in grown_of_largest(sim, settlement, n) {
        // Someone who already knows it is not taught it again.
        if sim.people().person(p).is_some_and(|x| x.knows(t)) {
            out.push(format!(
                "day {today}: {} of household {h} already knows {words}",
                sim.people().name_of(p)
            ));
            continue;
        }
        sim.introduce_technique(p, technique, false)?;
        out.push(format!(
            "day {today}: INTRODUCED (observer): {words} to {} of household {h}",
            sim.people().name_of(p)
        ));
    }
    Ok(out)
}

/// Settlement `s`'s buildings, in words: how many of each program, how many frames by storeys,
/// and how many have a front.
fn buildings_words(sim: &Sim, s: PermanentId) -> Vec<String> {
    let pop = sim.people();
    let land = sim.land();
    let catalog = &sim.rules().catalog;
    let mut programs: BTreeMap<String, usize> = BTreeMap::new();
    let mut storeys: BTreeMap<u8, usize> = BTreeMap::new();
    let (mut frames, mut v3, mut fronted, mut public, mut unfinished) = (0, 0, 0, 0, 0);
    for b in land
        .buildings
        .iter()
        .filter(|b| b.state == civ_land::BuildingState::Standing)
        .filter(|b| pop.building_settlement(land, b) == Some(s))
    {
        let def = catalog
            .building_index(&b.spec.program)
            .and_then(|i| catalog.buildings.get(i));
        let name = def.map_or_else(|| b.spec.program.clone(), |d| d.name.clone());
        *programs.entry(name).or_default() += 1;
        if !b.finished() {
            unfinished += 1;
        }
        if b.polity.is_some() {
            public += 1;
        }
        if matches!(b.spec.footprint, Footprint::Rect { .. }) {
            frames += 1;
            *storeys.entry(b.spec.storeys).or_default() += 1;
            if b.spec.version >= 3 {
                v3 += 1;
            }
            if let Some(def) = def {
                let t = civ_agents::style::traits_of(&b.spec, def).traits();
                if t[civ_agents::params::Taste::ROOF_TRAITS..]
                    .iter()
                    .any(|&x| x > 0.0)
                {
                    fronted += 1;
                }
            }
        }
    }
    let list = programs
        .iter()
        .map(|(n, c)| format!("{c} {n}"))
        .collect::<Vec<_>>()
        .join(", ");
    let tall = storeys
        .iter()
        .map(|(s, c)| format!("{c} of {s} storey{}", if *s == 1 { "" } else { "s" }))
        .collect::<Vec<_>>()
        .join(", ");
    vec![
        format!("  standing: {list} ({unfinished} not finished, {public} raised by the polity)"),
        format!(
            "  frames: {frames} ({tall}); {v3} designed under version 3, {fronted} with a front"
        ),
    ]
}

/// Why households of `s` did or did not build a frame: how many know framing, how many hold a
/// frame, and how many of those that know it hold none.
fn framing_words(sim: &Sim, s: PermanentId) -> String {
    let pop = sim.people();
    let land = sim.land();
    let Some(t) = sim
        .rules()
        .catalog
        .technique_index("core:technique/jointed_frame")
    else {
        return "  framing: not in the content".to_owned();
    };
    let (mut know, mut built, mut know_none) = (0, 0, 0);
    for (_, h) in pop
        .households
        .iter()
        .filter(|(_, h)| h.settlement == Some(s))
    {
        let knows = h
            .members
            .iter()
            .filter_map(|&m| pop.person(m))
            .any(|p| p.knows(t));
        let has = land.buildings.iter().any(|b| {
            b.household == h.id
                && b.state == civ_land::BuildingState::Standing
                && matches!(b.spec.footprint, Footprint::Rect { .. })
        });
        know += usize::from(knows);
        built += usize::from(has);
        know_none += usize::from(knows && !has);
    }
    format!(
        "  framing: {know} households know it, {built} hold a frame, {know_none} know it and hold \
         none"
    )
}

/// Every law of the kinds the demo asks about, in words, those proposed from `since` marked.
fn laws_words(sim: &Sim, since: Option<i64>, names: &BTreeMap<PermanentId, String>) -> Vec<String> {
    let pop = sim.people();
    let policies = &sim.rules().catalog.policies;
    let mut out = Vec::new();
    for p in &pop.polities {
        out.push(format!(
            "  at {}:",
            names
                .get(&p.settlement)
                .map_or("a settlement", String::as_str)
        ));
        for l in p.laws.iter().filter(|l| {
            matches!(
                l.kind,
                PolicyKind::BuildingRule
                    | PolicyKind::RaiseBuilding
                    | PolicyKind::Plan
                    | PolicyKind::BuildWay
                    | PolicyKind::KeepWatch
            )
        }) {
            let (present, support, oppose) = l.counts();
            let after = since.is_some_and(|d| l.proposed.day_index() >= d);
            out.push(format!(
                "    {}{} proposed on day {} by {}: {:?} ({support} for, {oppose} against, \
                 {present} came)",
                if after { "[after the shaking] " } else { "" },
                policies[usize::from(l.policy)].name,
                l.proposed.day_index(),
                pop.name_of(l.sponsor),
                l.status,
            ));
        }
    }
    out
}

/// What the shaking did, in words, settlement by settlement.
fn shaking_words(sim: &Sim, names: &BTreeMap<PermanentId, String>) -> Vec<String> {
    let pop = sim.people();
    let mut out = Vec::new();
    for q in &pop.quakes.list {
        out.push(format!(
            "  shaking {} on day {} at {:.2} of each building's weight:",
            q.id,
            q.at.day_index(),
            q.strength
        ));
        let mut by: BTreeMap<Option<PermanentId>, [usize; 4]> = BTreeMap::new();
        for s in &q.shaken {
            let e = by.entry(s.settlement).or_default();
            match s.outcome {
                civ_agents::quake::Outcome::Stood => e[0] += 1,
                civ_agents::quake::Outcome::Cracked => e[1] += 1,
                civ_agents::quake::Outcome::Fell => e[2] += 1,
            }
            e[3] += usize::from(s.deaths);
        }
        for (s, [stood, cracked, fell, deaths]) in by {
            let name = s
                .and_then(|s| names.get(&s).cloned())
                .unwrap_or_else(|| "outside any settlement".to_owned());
            out.push(format!(
                "    {name}: {stood} stood, {cracked} cracked, {fell} fell, {deaths} died"
            ));
        }
    }
    out
}

/// What the world came to, in words: each settlement's residents, buildings, framing and style;
/// the buildings raised together; the laws on building; and what each shaking did.
fn report(
    sim: &Sim,
    settlements: &[(PermanentId, String, f32)],
    shaken_on: Option<i64>,
) -> Vec<String> {
    let names: BTreeMap<PermanentId, String> = settlements
        .iter()
        .map(|(id, n, _)| (*id, n.clone()))
        .collect();
    let mut out = Vec::new();
    for (id, name, _) in settlements {
        let residents = sim
            .people()
            .households
            .iter()
            .filter(|(_, h)| h.settlement == Some(*id))
            .map(|(_, h)| h.members.len())
            .sum::<usize>();
        out.push(format!("{name}: {residents} residents"));
        out.extend(buildings_words(sim, *id));
        out.push(framing_words(sim, *id));
        out.push(format!(
            "  style: {}",
            civ_sim::frames::people::style_clock_words(sim, *id)
        ));
    }
    out.push("public buildings:".to_owned());
    for p in &sim.people().polities {
        out.push(format!(
            "  at {}:",
            names
                .get(&p.settlement)
                .map_or("a settlement", String::as_str)
        ));
        for w in civ_sim::frames::government::project_words(sim, p) {
            out.push(format!("    {w}"));
        }
    }
    out.push("laws on building, raising, plans, ways and the watch:".to_owned());
    out.extend(laws_words(sim, shaken_on, &names));
    out.push("the shaking:".to_owned());
    out.extend(shaking_words(sim, &names));
    out
}

/// The world's two settlements, the port (nearer the river) first.
fn port_first(sim: &Sim) -> Vec<(PermanentId, String, f32)> {
    let mut settlements: Vec<(PermanentId, String, f32)> = sim
        .land()
        .settlements
        .iter()
        .map(|s| (s.id, s.name.clone(), to_river_m(sim, s.hearth_m)))
        .collect();
    settlements.sort_by(|a, b| a.2.total_cmp(&b.2));
    settlements
}

/// Reads the save at `path` and says what its world came to.
fn reread(path: &Path) -> Result<(), String> {
    let root = civ_content::find_content_root(Path::new(env!("CARGO_MANIFEST_DIR")))
        .ok_or("content/ is not above the crate")?;
    let content = civ_content::load(&root)
        .registry
        .ok_or("the content does not load: run `civ-host content validate`")?;
    let sim = persist::load(path, &content).map_err(|e| e.to_string())?;
    let shaken_on = sim.people().quakes.list.first().map(|q| q.at.day_index());
    let mut log = Log::default();
    log.push(format!(
        "{} on day {}: {} people",
        sim.meta().name,
        sim.now().day_index(),
        sim.people().people.len()
    ));
    log.extend(report(&sim, &port_first(&sim), shaken_on));
    Ok(())
}

/// Lives the demo's world: what happened, in words.
fn make(saves: &Path, seed: u64, people: u32, days: i64) -> Result<String, String> {
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
            seed,
            preset_id: "core:worldgen/river_valley".to_owned(),
            size_cells: 1024,
            band_size: people,
            neighbours: vec![people],
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
    sim.set_speed(SPEED_MAX).map_err(|e| e.to_string())?;
    sim.advance_to_midnight().map_err(|e| e.to_string())?;
    let mut log = Log::default();
    log.push(format!(
        "world (seed {seed}): two groups of {people} founded in {:.1} s",
        made.elapsed().as_secs_f64()
    ));
    // The port is the settlement whose hearth lies nearer the river.
    let settlements = port_first(&sim);
    if settlements.len() < 2 {
        return Err("fewer than two settlements were founded".to_owned());
    }
    let (port, inland) = (settlements[0].0, settlements[1].0);
    for (i, (_, n, d)) in settlements.iter().enumerate() {
        log.push(format!(
            "{n}: the {} ({d:.0} m from the river)",
            if i == 0 { "port" } else { "inland settlement" }
        ));
    }
    let world_dir = saves.join(persist::world_dir_name(sim.meta()));
    let dir = SaveDir::create(world_dir, civ_schema::SAVE_EXTENSION).map_err(|e| e.to_string())?;
    let shake_day = (days - SHAKE_BEFORE_END).max(TEACH_DAY + 1);
    let mut shaken_on = None;
    let mut walls: Vec<f64> = Vec::new();
    let mut day = 0;
    while day < days {
        let started = Instant::now();
        sim.advance_minutes(DAY).map_err(|e| e.to_string())?;
        walls.push(started.elapsed().as_secs_f64());
        day += 1;
        let today = sim.now().day_index();
        let living = sim.people().people.len();
        if day % 30 == 0 {
            let frames = sim
                .land()
                .buildings
                .iter()
                .filter(|b| matches!(b.spec.footprint, Footprint::Rect { .. }))
                .count();
            let tallest = sim
                .land()
                .buildings
                .iter()
                .map(|b| b.spec.storeys)
                .max()
                .unwrap_or(0);
            let recent = &walls[walls.len().saturating_sub(30)..];
            log.push(format!(
                "day {today}: {living} people, {} buildings, {frames} frames, the tallest of \
                 {tallest} storeys; {:.2} s a day lately",
                sim.land().buildings.len(),
                recent.iter().sum::<f64>() / recent.len().max(1) as f64
            ));
        }
        if living == 0 {
            log.push(format!("day {today}: the valley is empty"));
            break;
        }
        if day == TEACH_DAY {
            for s in [port, inland] {
                log.extend(teach(
                    &mut sim,
                    s,
                    TAUGHT,
                    "core:technique/jointed_frame",
                    "framing",
                )?);
            }
            log.extend(teach(
                &mut sim,
                inland,
                1,
                "core:technique/laying_out",
                "laying out by cord and stake",
            )?);
        }
        if day == shake_day {
            sim.shake(STRENGTH, None)?;
            log.push(format!(
                "day {today}: SHAKEN (observer): the whole map at {STRENGTH} of each building's \
                 weight"
            ));
            shaken_on = Some(today);
        }
    }
    for n in NUDGES {
        log.push(format!("NUDGE: {n}"));
    }
    log.push(format!(
        "lived {day} days, to day {}; {} people",
        sim.now().day_index(),
        sim.people().people.len()
    ));
    log.extend(report(&sim, &settlements, shaken_on));
    // The scale gate (ADR-0028 §1): wall time a simulated day at Max at the size reached.
    let mut sorted = walls.clone();
    sorted.sort_by(f64::total_cmp);
    let mean = walls.iter().sum::<f64>() / walls.len().max(1) as f64;
    let at = |q: f64| sorted[((sorted.len() as f64 - 1.0) * q).round() as usize];
    log.push(format!(
        "speed: {mean:.2} s a day on average (p50 {:.2} s, p95 {:.2} s), {:.3} simulated years a \
         real minute",
        at(0.5),
        at(0.95),
        60.0 / (mean * 360.0)
    ));
    let saved = persist::save(&mut sim, &dir, SaveKind::Manual, "After the years")
        .map_err(|e| e.to_string())?;
    log.push(format!("saved: {}", saved.path.display()));
    Ok(log.0.join("\n"))
}
