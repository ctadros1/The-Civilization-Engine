//! A village and a disease brought to it, for the M6a demo (plan §7: the observer gives cholera to
//! one person; whether and where it spreads is the water's and people's). A band of 100 lives
//! `SETTLE_DAYS`, so that households have dug wells and draw at them. Then the observer brings
//! cholera to one person, chosen by the observer's sight of the truth: someone whose home's heap
//! lies within reach of, and no lower than, an open well that the most other households drew at in
//! the past 30 days (`NUDGE`: the observer's choice of whom, logged as such; the plague tool targets
//! a person, never a source, ADR-0021 §8), on the first wet day after (`NUDGE`: the observer's
//! choice of when; slice AZ's outbreaks grew where rain washed a heap into a well, and a first try
//! that brought it at the end of a dry summer spread to nobody). From there nothing is set: whether they fall ill and
//! shed, whether rain washes their heap into the well or it soaks through the ground to it, who
//! draws and drinks there, who falls ill, who suspects the well, whether anyone proposes closing it
//! and whether the gathering agrees are the water's and people's. The world lives `AFTER_DAYS`
//! after that and is saved. Most introductions go no further than a home or two (M6a slice AZ
//! measured six of twenty reaching 14 to 43 people), and each world draws its own identity, so up
//! to `TRIES` worlds are made and the first whose outbreak had `MIN_WATER_CASES` cases by water is
//! saved, or the last if none did; what it prints says which. The observer never edits people.
//!
//! ```sh
//! cargo run --release -p civ-sim --example cholera_world -- <saves folder> [seed]
//! ```

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::AtomicBool;

use civ_agents::polity::PolicyKind;
use civ_agents::population::cell_of;
use civ_agents::sickness::Acquired;
use civ_agents::uses::Place;
use civ_core::PermanentId;
use civ_sim::frames::incidents;
use civ_sim::{NewWorld, SPEED_MAX, Sim, persist};
use commons_persist::{SaveDir, SaveKind};

/// Days the band lives before the disease is brought: wells are dug in the first year or two.
const SETTLE_DAYS: i64 = 540;
/// Days lived after it is brought.
const AFTER_DAYS: i64 = 90;
/// What the soil could not hold on a day for it to count as wet, mm.
const WET_MM: f64 = 5.0;
/// Days waited at most for a wet day.
const WAIT_DAYS: i64 = 120;
/// Cases by water an outbreak needs for the world to be kept.
const MIN_WATER_CASES: u32 = 5;
/// Worlds made at most.
const TRIES: u32 = 6;
const DAY: i64 = 24 * 60;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let Some(saves) = args.next().map(PathBuf::from) else {
        eprintln!("usage: cholera_world <saves folder> [seed]");
        return ExitCode::FAILURE;
    };
    let seed = args.next().and_then(|s| s.parse().ok()).unwrap_or(1);
    match make(&saves, seed) {
        Ok(text) => {
            println!("{text}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("cholera_world: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Whom the observer brings it to: the household whose home's heap lies within reach of, and no
/// lower than, the open well the most other households drew at in the past 30 days, nearest
/// first; one of its people, the well, how far and how much higher, and how many drew there.
struct Target {
    person: PermanentId,
    household: PermanentId,
    well: PermanentId,
    metres: f64,
    above_m: f64,
    drawers: usize,
}

fn target(sim: &Sim) -> Option<Target> {
    let params = &sim.rules().land.contamination;
    let reach = params.runoff_radius_m.min(params.link_radius_m);
    let today = sim.now().day_index();
    let pop = sim.people();
    let homes: Vec<_> = pop
        .households
        .iter()
        .map(|(_, x)| x)
        .filter(|x| !x.members.is_empty())
        .collect();
    let mut best: Option<Target> = None;
    for w in sim.land().wells.list.iter().filter(|w| w.is_open()) {
        let drawers = homes
            .iter()
            .filter(|x| {
                pop.uses.households.get(&x.id).is_some_and(|list| {
                    list.iter()
                        .any(|u| u.place == Place::Source(w.cell) && u.days_worked(today, 30) > 0)
                })
            })
            .count();
        let (wx, wy) = w.rect.centre_m();
        for x in &homes {
            let metres = f64::from(wx - x.home.0).hypot(f64::from(wy - x.home.1));
            let ground = sim.map().elevation[cell_of(sim.map(), x.home)];
            let above_m = f64::from(ground) - f64::from(w.ground_m);
            if metres > reach || above_m < 0.0 {
                continue;
            }
            let better = best
                .as_ref()
                .is_none_or(|b| (drawers, -metres) > (b.drawers, -b.metres));
            if better {
                best = Some(Target {
                    person: x.members[0],
                    household: x.id,
                    well: w.id,
                    metres,
                    above_m,
                    drawers,
                });
            }
        }
    }
    best
}

/// A world made, settled, given the disease and lived on: what happened, in a line, and the
/// cases it took in water.
fn one(content: &civ_content::ContentRegistry, seed: u64) -> Result<(Sim, String, u32), String> {
    let mut sim = Sim::create(
        &NewWorld {
            name: "Wellfield".to_owned(),
            seed,
            preset_id: "core:worldgen/river_valley".to_owned(),
            size_cells: 768,
            band_size: 100,
            neighbours: Vec::new(),
            neighbours_known: false,
            regime_id: String::new(),
        },
        content,
        &mut |_| {},
        &AtomicBool::new(false),
    )
    .map_err(|e| e.to_string())?;
    sim.set_speed(SPEED_MAX).map_err(|e| e.to_string())?;
    sim.advance_to_midnight().map_err(|e| e.to_string())?;
    sim.advance_minutes(SETTLE_DAYS * DAY)
        .map_err(|e| e.to_string())?;
    let mut waited = 0;
    while sim.land().water.last_surplus_mm < WET_MM && waited < WAIT_DAYS {
        sim.advance_minutes(DAY).map_err(|e| e.to_string())?;
        waited += 1;
    }
    let wet = sim.land().water.last_surplus_mm;
    let open = sim.land().wells.list.iter().filter(|w| w.is_open()).count();
    let Some(t) = target(&sim) else {
        return Ok((
            sim,
            format!("{open} wells open; no heap lies above one: nobody to bring it to"),
            0,
        ));
    };
    let who = sim.people().name_of(t.person);
    sim.plague(t.person, "core:disease/cholera")?;
    let brought = sim.now().day_index();
    sim.advance_minutes(AFTER_DAYS * DAY)
        .map_err(|e| e.to_string())?;
    let pop = sim.people();
    let outbreak = pop
        .sickness
        .outbreaks()
        .iter()
        .find(|o| o.began >= brought)
        .map(|o| o.id);
    let (water, words) = match outbreak.and_then(|id| incidents::clustering(&sim, id)) {
        None => (0, "no outbreak".to_owned()),
        Some(c) => {
            let split = c.source.map_or_else(String::new, |s| {
                format!(
                    "; {} of {} households that drew it at {} had a case ({} cases), {} of {} of \
                     the rest ({} cases)",
                    s.drew.with_case,
                    s.drew.households,
                    s.name,
                    s.drew.cases,
                    s.rest.with_case,
                    s.rest.households,
                    s.rest.cases
                )
            });
            (
                c.water_cases,
                format!(
                    "{}: {} took it, {} in water{split}",
                    c.name, c.cases, c.water_cases
                ),
            )
        }
    };
    let ill = pop
        .sickness
        .episodes()
        .iter()
        .filter(|e| e.symptomatic() && !matches!(e.acquired, Acquired::Observer { .. }))
        .count();
    let w = sim.land().wells.get(t.well).copied();
    let suspects = w.map_or(0, |w| {
        pop.word
            .suspicions
            .iter()
            .filter(|s| s.source == Place::Source(w.cell))
            .count()
    });
    let closures = pop
        .polities
        .iter()
        .flat_map(|p| p.laws.iter().map(move |l| (p, l)))
        .filter(|(p, l)| {
            l.kind == PolicyKind::CloseSource
                && w.is_some_and(|w| p.closure_of(l.id) == Some(Place::Source(w.cell)))
        })
        .map(|(_, l)| format!("{:?}", l.outcome))
        .collect::<Vec<_>>();
    let line = format!(
        "{open} wells open after {} days. NUDGE: the observer brought cholera, on a day the soil \
         could not hold {wet:.1} mm, to {who} (household {}), whose home's heap lies {:.0} m from \
         and {:.1} m above well {}, at which {} households drew in the past 30 days. After \
         {AFTER_DAYS} days: {words}; {ill} others fell ill; {suspects} people suspect the well; \
         laws to close it: {}",
        SETTLE_DAYS + waited,
        t.household.get(),
        t.metres,
        t.above_m,
        t.well.get(),
        t.drawers,
        if closures.is_empty() {
            "none".to_owned()
        } else {
            closures.join(", ")
        }
    );
    Ok((sim, line, water))
}

fn make(saves: &Path, seed: u64) -> Result<String, String> {
    let root = civ_content::find_content_root(Path::new(env!("CARGO_MANIFEST_DIR")))
        .ok_or("content/ is not above the crate")?;
    let content = civ_content::load(&root)
        .registry
        .ok_or("the content does not load: run `civ-host content validate`")?;
    let mut lines = Vec::new();
    let mut tries = 0;
    let (mut sim, water) = loop {
        tries += 1;
        let (sim, line, water) = one(&content, seed)?;
        lines.push(format!("world {tries}: {line}"));
        if water >= MIN_WATER_CASES || tries == TRIES {
            break (sim, water);
        }
    };
    let kept = if water >= MIN_WATER_CASES {
        format!("kept world {tries}: its outbreak had {water} cases by water")
    } else {
        format!("no outbreak had {MIN_WATER_CASES} cases by water in {TRIES} worlds; kept the last")
    };
    let dir = SaveDir::create(
        saves.join(persist::world_dir_name(sim.meta())),
        civ_schema::SAVE_EXTENSION,
    )
    .map_err(|e| e.to_string())?;
    let saved = persist::save(&mut sim, &dir, SaveKind::Manual, "Cholera brought")
        .map_err(|e| e.to_string())?;
    Ok(format!(
        "{}\n{kept}\nsaved {}",
        lines.join("\n"),
        saved.path.display()
    ))
}
