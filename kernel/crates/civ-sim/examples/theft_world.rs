//! A village through a lean spell, for the M4b demo (plan §7: a theft from the act to its end).
//! Takings come only where some households are out of food while others have some, and a natural
//! run may have none: the dashboard's five valley worlds made no attempt in fifty years. Here a
//! band lives a week; then every other household, by id, loses the food in its store, as if rot or
//! fire took it, and the rest keep six days' food. From there nothing is set: who goes to take, who
//! is about, who sees and who is told, whether anyone proposes a law against taking and whether the
//! gathering passes it, what is brought before it, what it finds and what is paid are people's
//! choices. The world is lived a day at a time until a finding's obligations have all been
//! answered and settled, or for `MAX_DAYS`. Most lean spells end without one (in five runs of
//! seed 4, one did: guardianship turns most takers back), and each world draws its own identity,
//! so up to `TRIES` worlds are made and the first where a finding settled is saved, or the last if
//! none did; what it prints says which. Test scaffolding, as in `tests/crime.rs`: the observer
//! never edits people.
//!
//! ```sh
//! cargo run --release -p civ-sim --example theft_world -- <saves folder> [seed]
//! ```

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::AtomicBool;

use civ_agents::crime::{CaseStage, Outcome, Standing};
use civ_agents::params::GoodUse;
use civ_core::PermanentId;
use civ_sim::{NewWorld, Sim, persist};
use commons_persist::{SaveDir, SaveKind};

/// Days lived after the lean spell begins, at most.
const MAX_DAYS: u32 = 240;
/// Days the band lives before the lean spell.
const SETTLE_DAYS: i64 = 7;
/// Days of food the households that keep any keep.
const KEPT_DAYS: f64 = 6.0;
/// Worlds made at most.
const TRIES: u32 = 12;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let Some(saves) = args.next().map(PathBuf::from) else {
        eprintln!("usage: theft_world <saves folder> [seed]");
        return ExitCode::FAILURE;
    };
    let seed = args.next().and_then(|s| s.parse().ok()).unwrap_or(4);
    match make(&saves, seed) {
        Ok(text) => {
            println!("{text}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("theft_world: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Whether a case has been found and everything it imposed is settled: paid, refused, in arrears,
/// or owed by a household no more.
fn settled(sim: &Sim) -> bool {
    let order = &sim.people().order;
    order
        .cases
        .iter()
        .filter(|c| c.stage == CaseStage::Found)
        .any(|c| {
            let owed: Vec<_> = order
                .obligations
                .iter()
                .filter(|o| o.case == Some(c.id))
                .collect();
            !owed.is_empty() && owed.iter().all(|o| o.standing != Standing::Open)
        })
}

fn make(saves: &Path, seed: u64) -> Result<String, String> {
    let root = civ_content::find_content_root(Path::new(env!("CARGO_MANIFEST_DIR")))
        .ok_or("content/ is not above the crate")?;
    let content = civ_content::load(&root)
        .registry
        .ok_or("the content does not load: run `civ-host content validate`")?;
    let mut tries = 0;
    let (mut sim, days) = loop {
        tries += 1;
        let (sim, days) = lean_spell(&content, seed)?;
        if settled(&sim) || tries == TRIES {
            break (sim, days);
        }
    };
    let order = &sim.people().order;
    let stage = |s: CaseStage| order.cases.iter().filter(|c| c.stage == s).count();
    let summary = format!(
        "{} after {days} days of the lean spell (world {tries} of at most {TRIES}): {} attempts \
         to take, {} takings, {} seen; {} laws against taking passed; {} cases brought, {} \
         found, {} not found; {} of {} obligations a finding imposed paid in full",
        if settled(&sim) {
            "A finding settled"
        } else {
            "No finding settled"
        },
        order.incidents.len(),
        order
            .incidents
            .iter()
            .filter(|i| i.outcome == Outcome::Taken)
            .count(),
        order
            .incidents
            .iter()
            .filter(|i| !i.seen_by.is_empty())
            .count(),
        sim.people()
            .polities
            .iter()
            .flat_map(|p| &p.laws)
            .filter(|l| {
                l.kind == civ_agents::polity::PolicyKind::AgainstTaking
                    && l.outcome == Some(civ_agents::polity::Outcome::Passed)
            })
            .count(),
        order.cases.len(),
        stage(CaseStage::Found),
        stage(CaseStage::NotFound),
        order
            .obligations
            .iter()
            .filter(|o| o.case.is_some() && o.standing == Standing::Met)
            .count(),
        order
            .obligations
            .iter()
            .filter(|o| o.case.is_some())
            .count(),
    );
    let dir = SaveDir::create(
        saves.join(persist::world_dir_name(sim.meta())),
        civ_schema::SAVE_EXTENSION,
    )
    .map_err(|e| e.to_string())?;
    let saved = persist::save(&mut sim, &dir, SaveKind::Manual, "A lean spell")
        .map_err(|e| e.to_string())?;
    Ok(format!("{summary}\nsaved {}", saved.path.display()))
}

/// Makes a world of `seed`, sets its lean spell going and lives it until a finding settles or
/// `MAX_DAYS` pass; returns it with the days lived.
fn lean_spell(content: &civ_content::ContentRegistry, seed: u64) -> Result<(Sim, u32), String> {
    let mut sim = Sim::create(
        &NewWorld {
            name: "Lean Valley".to_owned(),
            seed,
            preset_id: "core:worldgen/river_valley".to_owned(),
            size_cells: 512,
            band_size: 0,
            neighbours: Vec::new(),
            neighbours_known: false,
            regime_id: String::new(),
        },
        content,
        &mut |_| {},
        &AtomicBool::new(false),
    )
    .map_err(|e| e.to_string())?;
    sim.advance_minutes(SETTLE_DAYS * 24 * 60)
        .map_err(|e| e.to_string())?;
    // The lean spell: every other household loses its food; the rest keep six days of it.
    let rules = sim.rules().clone();
    let goods = &rules.catalog.goods;
    let provisions = goods
        .iter()
        .position(|g| g.id == "core:good/provisions")
        .ok_or("no provisions")?;
    let pop = sim.people_mut_for_tests();
    let mut ids: Vec<PermanentId> = pop.households.iter().map(|(_, h)| h.id).collect();
    ids.sort_unstable();
    let hungry: Vec<PermanentId> = ids.iter().copied().step_by(2).collect();
    for (_, h) in pop.households.iter_mut() {
        for (kg, g) in h.stores.iter_mut().zip(goods) {
            if g.purpose == GoodUse::Food {
                *kg = 0.0;
            }
        }
        if !hungry.contains(&h.id) {
            let need = h.members.len() as f64 * rules.people.household.daily_kcal_per_person;
            h.stores[provisions] = KEPT_DAYS * need / goods[provisions].kcal_per_kg;
        }
    }
    let mut days = 0;
    while !settled(&sim) && days < MAX_DAYS {
        sim.advance_minutes(24 * 60).map_err(|e| e.to_string())?;
        days += 1;
    }
    Ok((sim, days))
}
