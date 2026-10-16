//! Two villages, a siege and what each gathering made of it, for the M6c demo (plan §7: "a
//! siege, and what each gathering made of it. After a week with none, nudge a prior (a raider's
//! valuation of a store they saw, or how long members stay before their harvest), never an
//! event, and log a `NUDGE:`"). Two villages of `PEOPLE`, founded apart and known to each other,
//! live `QUIET_DAYS` as the content has them. Nothing is ever set: no band is called, no law
//! proposed, no line raised and no hold kept by the observer. If no band camped by a line or held
//! a place in that time, the observer nudges a prior (`NUDGE`, logged as such): how many points a
//! day of food in a store someone saw is worth to them (`bands.w_spoils`). If a band then camps or
//! holds and its people go home before `bands.hold_days`, the observer nudges a second prior: what
//! each day away weighs against staying (`bands.w_camp_day`). Whether anyone calls a band, whether
//! a gathering sends one, raises a line, takes a village in, seeks terms or none of these, is
//! people's. The world lives at most `MAX_DAYS`; it is saved the first day a band camps by a line
//! or holds a place ("During the siege") and at the end ("After the siege"), `AFTER_DAYS` after a
//! hold was kept or a polity took another in, if one did. The nudged priors live only in this run:
//! a loaded world takes its content's. Whether a siege comes at all is people's, so up to `TRIES`
//! worlds are lived, each of the next seed and its own name: the first where a hold was kept or a
//! polity took another in is kept, else the first with a siege, else the last; the others' saves
//! are removed, so the folder holds one world.
//!
//! ```sh
//! cargo run --release -p civ-sim --example siege -- <saves folder> [seed]
//! ```

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::AtomicBool;

use civ_agents::polity::{LawStatus, PolicyKind};
use civ_agents::war::BandStage;
use civ_sim::{NewWorld, SPEED_MAX, Sim, persist};
use commons_persist::{SaveDir, SaveKind};

/// People in each village.
const PEOPLE: u32 = 100;
/// Days lived as the content has them before any nudge.
const QUIET_DAYS: i64 = 120;
/// Days lived at most, nudged or not.
const MAX_DAYS: i64 = 720;
/// Days lived after a hold was kept or a polity took another in.
const AFTER_DAYS: i64 = 60;
/// The first nudge: points for each day of a household's food a load would bring home (the
/// content's 0.5; a band's walk costs about 7 points, its spoils 1-3, slice BJ's finding).
const NUDGED_SPOILS: f64 = 2.0;
/// The second nudge: points against staying for each day camped or holding (the content's 1.0).
const NUDGED_CAMP_DAY: f64 = 0.25;
const DAY: i64 = 24 * 60;
const NAME: &str = "Siegeford";
/// Worlds lived at most.
const TRIES: u64 = 3;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let Some(saves) = args.next().map(PathBuf::from) else {
        eprintln!("usage: siege <saves folder> [seed]");
        return ExitCode::FAILURE;
    };
    let seed = args.next().and_then(|s| s.parse().ok()).unwrap_or(1);
    match make_tries(&saves, seed) {
        Ok(text) => {
            println!("{text}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("siege: {e}");
            ExitCode::FAILURE
        }
    }
}

/// The name of settlement `s`.
fn place(sim: &Sim, s: civ_core::PermanentId) -> String {
    sim.land()
        .settlements
        .iter()
        .find(|x| x.id == s)
        .map_or_else(|| "?".to_owned(), |x| x.name.clone())
}

/// Whether a band is camped by a line or holding a place now.
fn besieging(sim: &Sim) -> bool {
    sim.people()
        .order
        .bands
        .iter()
        .any(|b| matches!(b.stage, BandStage::Camped | BandStage::Holding))
}

/// Every band in words: who called it, from where, whether a law sent it, how far it went.
fn bands_words(sim: &Sim) -> Vec<String> {
    let pop = sim.people();
    pop.order
        .bands
        .iter()
        .map(|b| {
            let to = pop
                .household(b.target)
                .and_then(|x| x.settlement)
                .map_or_else(|| "?".to_owned(), |s| place(sim, s));
            let fight = b
                .clash
                .and_then(|c| pop.order.clashes.iter().find(|e| e.id == c))
                .map_or_else(
                    || "no encounter".to_owned(),
                    |e| format!("an encounter of {} minutes, {:?}", e.minutes, e.ended),
                );
            format!(
                "band {} of {} from {} to {to} on day {}{}: {} joined, {} came, {fight}, \
                 {:.0} kcal taken, {:?}{}",
                b.id,
                pop.name_of(b.organizer),
                place(sim, b.settlement),
                b.called / DAY,
                if b.law.is_some() { ", sent by law" } else { "" },
                b.members.len(),
                b.arrived.len(),
                b.taken_kcal,
                b.stage,
                b.held
                    .map_or_else(String::new, |d| format!(", held from day {d}")),
            )
        })
        .collect()
}

/// Every law of a war policy kind in words, with its outcome.
fn laws_words(sim: &Sim) -> Vec<String> {
    let pop = sim.people();
    let catalog = &sim.rules().catalog;
    let mut out = Vec::new();
    for p in &pop.polities {
        for l in &p.laws {
            if !matches!(
                l.kind,
                PolicyKind::SendBand
                    | PolicyKind::AgainstRaiding
                    | PolicyKind::Enclose
                    | PolicyKind::Annex
                    | PolicyKind::Agreement
            ) {
                continue;
            }
            let (present, support, oppose) = l.counts();
            out.push(format!(
                "{} at {}: {} proposed by {} on day {} against {}: {:?} ({support} for, {oppose} \
                 against, {present} came)",
                if l.status == LawStatus::InForce {
                    "in force"
                } else {
                    "law"
                },
                place(sim, p.settlement),
                catalog
                    .policies
                    .get(usize::from(l.policy))
                    .map_or("?", |d| d.id.as_str()),
                pop.name_of(l.sponsor),
                l.proposed.day_index(),
                l.issue.name(),
                l.status,
            ));
        }
    }
    out
}

/// Each polity's settlements and end, in words.
fn polities_words(sim: &Sim) -> Vec<String> {
    sim.people()
        .polities
        .iter()
        .map(|p| {
            let seats: Vec<String> = p.settlements().iter().map(|&s| place(sim, s)).collect();
            format!(
                "the polity seated at {} holds {}{}",
                place(sim, p.settlement),
                seats.join(" and "),
                p.ended.map_or_else(String::new, |e| format!(
                    "; its deciding closed on day {} ({:?})",
                    e.day, e.how
                ))
            )
        })
        .collect()
}

/// Lives up to `TRIES` worlds from `seed` on and keeps the one that went furthest: a hold kept or
/// a polity taken in (2), a siege (1), or none (0).
fn make_tries(saves: &Path, seed: u64) -> Result<String, String> {
    let mut kept: Option<(u8, PathBuf, String)> = None;
    for k in 0..TRIES {
        let name = if k == 0 {
            NAME.to_owned()
        } else {
            format!("{NAME} {}", k + 1)
        };
        let (text, reached, dir) = make(saves, seed + k, &name)?;
        let text = format!("world {} (seed {}):\n{text}\nname: {name}", k + 1, seed + k);
        match &kept {
            Some((best, _, _)) if *best >= reached => {
                std::fs::remove_dir_all(&dir).map_err(|e| e.to_string())?;
            }
            _ => {
                if let Some((_, old, _)) = kept.take() {
                    std::fs::remove_dir_all(&old).map_err(|e| e.to_string())?;
                }
                kept = Some((reached, dir, text));
            }
        }
        if kept.as_ref().is_some_and(|k| k.0 == 2) {
            break;
        }
    }
    let (reached, _, text) = kept.ok_or("no world was lived")?;
    Ok(format!(
        "{text}\nkept this world: {}",
        [
            "no siege came",
            "a siege came",
            "a hold was kept or a polity took another in"
        ][usize::from(reached)]
    ))
}

/// Lives one world named `name` from `seed`: what happened, in words; how far it went (see
/// `make_tries`); and its saves folder.
fn make(saves: &Path, seed: u64, name: &str) -> Result<(String, u8, PathBuf), String> {
    let root = civ_content::find_content_root(Path::new(env!("CARGO_MANIFEST_DIR")))
        .ok_or("content/ is not above the crate")?;
    let content = civ_content::load(&root)
        .registry
        .ok_or("the content does not load: run `civ-host content validate`")?;
    let mut sim = Sim::create(
        &NewWorld {
            name: name.to_owned(),
            seed,
            preset_id: "core:worldgen/river_valley".to_owned(),
            size_cells: 768,
            band_size: PEOPLE,
            neighbours: vec![PEOPLE],
            neighbours_known: true,
            regime_id: String::new(),
        },
        &content,
        &mut |_| {},
        &AtomicBool::new(false),
    )
    .map_err(|e| e.to_string())?;
    sim.set_speed(SPEED_MAX).map_err(|e| e.to_string())?;
    sim.advance_to_midnight().map_err(|e| e.to_string())?;
    let world_dir = saves.join(persist::world_dir_name(sim.meta()));
    let dir = SaveDir::create(world_dir.clone(), civ_schema::SAVE_EXTENSION)
        .map_err(|e| e.to_string())?;
    let mut log = Vec::new();
    let (mut spoils_nudged, mut camp_nudged) = (false, false);
    let mut siege_saved = None;
    // The bands seen camped by a line or holding a place, by number.
    let mut besiegers: Vec<u32> = Vec::new();
    let mut ended_on = None;
    let mut day = 0;
    while day < MAX_DAYS {
        sim.advance_minutes(DAY).map_err(|e| e.to_string())?;
        day += 1;
        // The world's own day, as bands and laws name it.
        let today = sim.now().day_index();
        let pop = sim.people();
        for b in &pop.order.bands {
            if matches!(b.stage, BandStage::Camped | BandStage::Holding)
                && !besiegers.contains(&b.id)
            {
                besiegers.push(b.id);
            }
        }
        if siege_saved.is_none() && besieging(&sim) {
            let saved = persist::save(&mut sim, &dir, SaveKind::Manual, "During the siege")
                .map_err(|e| e.to_string())?;
            log.push(format!(
                "day {today}: a band camps by a line or holds a place; saved {}",
                saved.path.display()
            ));
            siege_saved = Some(day);
            continue;
        }
        if !spoils_nudged && day >= QUIET_DAYS && siege_saved.is_none() {
            let bands = pop.order.bands.len();
            let rules = sim
                .rules_mut_for_tests()
                .ok_or("the rules are shared: no nudge can be made")?;
            let was = rules.people.bands.w_spoils;
            rules.people.bands.w_spoils = NUDGED_SPOILS;
            log.push(format!(
                "day {today}: NUDGE (a raider's valuation of a store they saw): {bands} bands called \
                 in {QUIET_DAYS} days and none camped or held; bands.w_spoils {was} -> \
                 {NUDGED_SPOILS}"
            ));
            spoils_nudged = true;
            continue;
        }
        // A band seen camped or holding went home before it held: how long members stay.
        let gave_up = pop
            .order
            .bands
            .iter()
            .any(|b| besiegers.contains(&b.id) && b.stage.over() && b.held.is_none());
        if !camp_nudged && gave_up {
            let rules = sim
                .rules_mut_for_tests()
                .ok_or("the rules are shared: no nudge can be made")?;
            let was = rules.people.bands.w_camp_day;
            rules.people.bands.w_camp_day = NUDGED_CAMP_DAY;
            log.push(format!(
                "day {today}: NUDGE (how long members stay): a band camped or holding went home \
                 before it held; bands.w_camp_day {was} -> {NUDGED_CAMP_DAY}"
            ));
            camp_nudged = true;
            continue;
        }
        let held = pop.order.bands.iter().any(|b| b.held.is_some());
        let joined = pop.polities.iter().any(|p| p.ended.is_some());
        if ended_on.is_none() && (held || joined) {
            log.push(format!(
                "day {today}: {}",
                if joined {
                    "a polity took another in"
                } else {
                    "a hold was kept"
                }
            ));
            ended_on = Some(day);
        }
        if ended_on.is_some_and(|d| day - d >= AFTER_DAYS) {
            break;
        }
    }
    log.push(format!(
        "lived {day} days, to day {}",
        sim.now().day_index()
    ));
    log.extend(bands_words(&sim).into_iter().map(|w| format!("  {w}")));
    log.extend(laws_words(&sim).into_iter().map(|w| format!("  {w}")));
    log.extend(polities_words(&sim).into_iter().map(|w| format!("  {w}")));
    let agreements = sim.people().agreements.list.len();
    let encounters = sim.people().order.clashes.len();
    log.push(format!(
        "  {encounters} encounters, {agreements} agreements sought, {} enclosures",
        sim.land().enclosures.list.len()
    ));
    let saved = persist::save(&mut sim, &dir, SaveKind::Manual, "After the siege")
        .map_err(|e| e.to_string())?;
    log.push(format!("saved {}", saved.path.display()));
    let reached = if ended_on.is_some() {
        2
    } else {
        u8::from(siege_saved.is_some())
    };
    Ok((log.join("\n"), reached, world_dir))
}
