//! A village and the council it may take, for the M7a demo (plan §7, M7 design §9: "each part's
//! demo shows what its world reached, nudging only priors"). A village of `PEOPLE` (the largest
//! founding band) lives `QUIET_DAYS` as the content has them. Nothing is ever set: no law is
//! proposed, no custom amended, nobody stood or polled by the observer. If no gathering has taken
//! its custom to a council by then, the observer nudges a prior (`NUDGE`, logged as such): what an
//! evening at the gathering costs one who comes (`elections.w_evening`), the price of the evenings
//! a council would save a household (ADR-0024 §6). Whether anyone proposes a council, whether it
//! passes, who stands, who polls and who is seated is people's. The world lives at most `MAX_DAYS`;
//! it is saved when a council is first seated ("A council is seated") and at the end ("After the
//! elections"), `AFTER_DAYS` after the first council sat, if one did. The nudged prior lives only
//! in this run: a loaded world takes its content's. Up to `TRIES` worlds are lived, each of the
//! next seed and its own name: the first where a council sat is kept, else the first where a custom
//! was amended to one, else the last; the others' saves are removed.
//!
//! ```sh
//! cargo run --release -p civ-sim --example council -- <saves folder> [seed]
//! ```

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::AtomicBool;

use civ_agents::polity::{LawStatus, PolicyKind};
use civ_agents::seats::ElectionStage;
use civ_sim::{NewWorld, SPEED_MAX, Sim, persist};
use commons_persist::{SaveDir, SaveKind};

/// People in the village.
const PEOPLE: u32 = 120;
/// Days lived as the content has them before any nudge.
const QUIET_DAYS: i64 = 180;
/// Days lived at most, nudged or not.
const MAX_DAYS: i64 = 720;
/// Days lived after the first council sat.
const AFTER_DAYS: i64 = 120;
/// The nudge: what an evening at the gathering costs one who comes (the content's 0.25; the
/// custom's own pull to the gathering, `polity.attend_base`, is 1.0).
const NUDGED_EVENING: f64 = 1.0;
const DAY: i64 = 24 * 60;
const NAME: &str = "Moothill";
/// Worlds lived at most.
const TRIES: u64 = 2;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let Some(saves) = args.next().map(PathBuf::from) else {
        eprintln!("usage: council <saves folder> [seed]");
        return ExitCode::FAILURE;
    };
    let seed = args.next().and_then(|s| s.parse().ok()).unwrap_or(1);
    match make_tries(&saves, seed) {
        Ok(text) => {
            println!("{text}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("council: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Every amendment of the custom in words, with its outcome.
fn amendments_words(sim: &Sim) -> Vec<String> {
    let pop = sim.people();
    let mut out = Vec::new();
    for p in &pop.polities {
        for l in p.laws.iter().filter(|l| l.kind == PolicyKind::AmendBody) {
            let (present, support, oppose) = l.counts();
            out.push(format!(
                "{} proposed on day {} by {}: {} — {:?} ({support} for, {oppose} against, \
                 {present} came)",
                if l.status == LawStatus::InForce {
                    "in force"
                } else {
                    "an amendment"
                },
                l.proposed.day_index(),
                pop.name_of(l.sponsor),
                l.body.map_or_else(|| "?".to_owned(), |b| b.words()),
                l.status,
            ));
        }
    }
    out
}

/// Each polity's elections, in the observer's words.
fn elections_words(sim: &Sim) -> Vec<String> {
    let mut out = Vec::new();
    for pi in 0..sim.people().polities.len() {
        for e in civ_sim::frames::government::election_words(sim, pi)
            .iter()
            .rev()
        {
            out.push(format!("{}; {}; {}", e.heading, e.stage, e.turnout));
            for a in &e.alliances {
                out.push(format!("    {a}"));
            }
            for c in &e.candidates {
                out.push(format!(
                    "    {}{}: {} names{}{}{}",
                    c.name,
                    if c.district.is_empty() {
                        String::new()
                    } else {
                        format!(" ({})", c.district)
                    },
                    c.votes,
                    if c.seated { ", seated" } else { "" },
                    if c.backed.is_empty() {
                        String::new()
                    } else {
                        format!("; {}", c.backed)
                    },
                    if c.told.is_empty() {
                        String::new()
                    } else {
                        format!("; {}", c.told)
                    },
                ));
            }
        }
    }
    out
}

/// Lives up to `TRIES` worlds from `seed` on and keeps the one that went furthest: a council
/// seated (2), a custom amended to a council (1), or none (0).
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
            "no custom was taken to a council",
            "a custom was taken to a council, and no council sat",
            "a council sat"
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
            neighbours: Vec::new(),
            neighbours_known: false,
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
    let mut nudged = false;
    let (mut amended_on, mut seated_on) = (None, None);
    let mut day = 0;
    while day < MAX_DAYS {
        sim.advance_minutes(DAY).map_err(|e| e.to_string())?;
        day += 1;
        let today = sim.now().day_index();
        let pop = sim.people();
        let council = pop.polities.iter().any(|p| p.body.seats.is_some());
        if amended_on.is_none() && council {
            log.push(format!("day {today}: the custom is a council"));
            amended_on = Some(day);
        }
        let seated = pop.polities.iter().any(|p| {
            p.elections
                .iter()
                .any(|e| e.stage == ElectionStage::Installed)
        });
        if seated_on.is_none() && seated {
            let saved = persist::save(&mut sim, &dir, SaveKind::Manual, "A council is seated")
                .map_err(|e| e.to_string())?;
            log.push(format!(
                "day {today}: a council is seated; saved {}",
                saved.path.display()
            ));
            seated_on = Some(day);
            continue;
        }
        if !nudged && day >= QUIET_DAYS && amended_on.is_none() {
            let amendments = pop
                .polities
                .iter()
                .flat_map(|p| &p.laws)
                .filter(|l| l.kind == PolicyKind::AmendBody)
                .count();
            let rules = sim
                .rules_mut_for_tests()
                .ok_or("the rules are shared: no nudge can be made")?;
            let was = rules.people.elections.w_evening;
            rules.people.elections.w_evening = NUDGED_EVENING;
            log.push(format!(
                "day {today}: NUDGE (what an evening at the gathering costs one who comes): \
                 {amendments} amendments proposed in {QUIET_DAYS} days and no council; \
                 elections.w_evening {was} -> {NUDGED_EVENING}"
            ));
            nudged = true;
            continue;
        }
        if seated_on.is_some_and(|d| day - d >= AFTER_DAYS) {
            break;
        }
    }
    log.push(format!(
        "lived {day} days, to day {}; {} people",
        sim.now().day_index(),
        sim.people().people.len()
    ));
    log.extend(amendments_words(&sim).into_iter().map(|w| format!("  {w}")));
    log.extend(elections_words(&sim).into_iter().map(|w| format!("  {w}")));
    for p in &sim.people().polities {
        let label = civ_sim::labels::label_of(&sim, p);
        log.push(format!("  label: {} ({})", label.name, p.body.words()));
    }
    let saved = persist::save(&mut sim, &dir, SaveKind::Manual, "After the elections")
        .map_err(|e| e.to_string())?;
    log.push(format!("saved {}", saved.path.display()));
    let reached = if seated_on.is_some() {
        2
    } else {
        u8::from(amended_on.is_some())
    };
    Ok((log.join("\n"), reached, world_dir))
}
