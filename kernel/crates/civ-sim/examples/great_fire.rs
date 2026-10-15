//! A village, a great fire and what followed, for the M6b demo (plan §7: "a great fire, the fire
//! tool's if no lived world shows one in a time-boxed week, and what followed"). A band of 100
//! lives `SETTLE_DAYS`. If a fire of its own took `GREAT` buildings or more in that time, that is
//! the great fire and nothing is set. Otherwise the observer sets one alight (`NUDGE`s, logged as
//! such): where, the thatched building with the most others within 4 m of its plot, the densest
//! part of the village (the observer's sight of the truth); and when, at noon of the first day
//! whose fuel is as dry as `DRY` and whose wind blows at `WINDY` or more (research 12-03 §2.2: the
//! full-scale test burned in 4.2-6.9 m/s), waiting at most `WAIT_DAYS`. From there nothing is set:
//! whether it spreads, who notices and fights it, what it takes, who remembers it, whether anyone
//! proposes a rule for building, whether the gathering agrees and whether builders keep it are
//! the fire's, the weather's and people's. The world lives `AFTER_DAYS` after it and is saved. Each
//! world draws its own identity, so up to `TRIES` worlds are made and the first whose fire took
//! `GREAT` buildings is kept, or the last if none did; what it prints says which.
//!
//! ```sh
//! cargo run --release -p civ-sim --example great_fire -- <saves folder> [seed]
//! ```

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::AtomicBool;

use civ_agents::polity::PolicyKind;
use civ_core::PermanentId;
use civ_sim::{NewWorld, SPEED_MAX, Sim, persist};
use commons_persist::{SaveDir, SaveKind};

/// Days the band lives before a fire is set, if none of its own was great: into its second
/// summer, when dry days come.
const SETTLE_DAYS: i64 = 450;
/// Days lived after the fire.
const AFTER_DAYS: i64 = 365;
/// Buildings a fire must take to be called great.
const GREAT: usize = 4;
/// Fuel at least this dry, as a share of its dry mass, counts as a dry day (the land profile:
/// on this valley's weather a summer day's middle is about 0.14).
const DRY: f32 = 0.13;
/// A wind at least this strong, m/s, counts as windy.
const WINDY: f32 = 4.0;
/// Days waited at most for a dry, windy day.
const WAIT_DAYS: i64 = 180;
/// Worlds made at most.
const TRIES: u32 = 4;
/// How near another building a roof counts as close, metres (the roofs rule's reach).
const CLOSE_M: f64 = 4.0;
const DAY: i64 = 24 * 60;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let Some(saves) = args.next().map(PathBuf::from) else {
        eprintln!("usage: great_fire <saves folder> [seed]");
        return ExitCode::FAILURE;
    };
    let seed = args.next().and_then(|s| s.parse().ok()).unwrap_or(1);
    match make(&saves, seed) {
        Ok(text) => {
            println!("{text}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("great_fire: {e}");
            ExitCode::FAILURE
        }
    }
}

/// The fires since day `since`, the largest first: (its first fire's building, the day it began,
/// buildings it took). A conflagration counts as one, of all its fires.
fn fires_by_size(sim: &Sim, since: i64) -> Vec<(PermanentId, i64, usize)> {
    let fires = &sim.people().fires;
    let mut out: Vec<(PermanentId, i64, usize)> = fires
        .episodes
        .iter()
        .filter(|e| e.began.day_index() >= since)
        .filter_map(|e| {
            let n = fires
                .list
                .iter()
                .filter(|f| f.episode == Some(e.id))
                .count();
            let first = fires.get(e.first)?;
            Some((first.building, e.began.day_index(), n))
        })
        .collect();
    out.extend(
        fires
            .list
            .iter()
            .filter(|f| f.episode.is_none() && f.began.day_index() >= since)
            .map(|f| (f.building, f.began.day_index(), 1)),
    );
    out.sort_by_key(|x| std::cmp::Reverse(x.2));
    out
}

/// The standing, roofed, thatched building with the most other standing buildings within
/// `CLOSE_M` of its plot, and how many; ties to the lower id.
fn densest(sim: &Sim) -> Option<(PermanentId, usize)> {
    let land = sim.land();
    let rects: Vec<(PermanentId, civ_land::RectCm)> = land
        .buildings
        .iter()
        .filter(|b| b.standing())
        .filter_map(|b| {
            land.plots
                .iter()
                .find(|p| p.id == b.plot)
                .map(|p| (b.id, p.rect))
        })
        .collect();
    land.buildings
        .iter()
        .filter(|b| b.standing() && b.roofed() && !civ_agents::rules::roof_unburnt(&b.spec))
        .filter_map(|b| {
            let own = rects.iter().find(|r| r.0 == b.id)?.1;
            let n = rects
                .iter()
                .filter(|r| r.0 != b.id && civ_agents::rules::gap_m(&own, &r.1) < CLOSE_M)
                .count();
            Some((b.id, n))
        })
        .max_by(|a, b| a.1.cmp(&b.1).then(b.0.cmp(&a.0)))
}

/// What followed the fire of day `fire_day`, in words: who remembers it, the rules for building
/// proposed and what became of them, and the buildings begun since.
fn followed(sim: &Sim, fire_day: i64) -> String {
    let pop = sim.people();
    let households: Vec<_> = pop
        .households
        .iter()
        .filter(|(_, x)| !x.members.is_empty())
        .map(|(_, x)| x)
        .collect();
    let remember = households
        .iter()
        .filter(|x| !x.fire_losses.is_empty())
        .count();
    let catalog = &sim.rules().catalog;
    let mut rules = Vec::new();
    for p in &pop.polities {
        for l in p.laws.iter().filter(|l| l.kind == PolicyKind::BuildingRule) {
            let def = catalog.policies.get(usize::from(l.policy));
            let (_, support, oppose) = l.counts();
            let c = l.compliance;
            rules.push(format!(
                "{} proposed by {} against {} on day {} after the fire: {:?} ({support} for, \
                 {oppose} against); begun under it and bound by it: {} kept, {} broken knowingly, \
                 {} unaware",
                def.map_or("?", |d| d.id.as_str()),
                pop.name_of(l.sponsor),
                l.issue.name(),
                l.proposed.day_index() - fire_day,
                l.status,
                c.complied,
                c.broken,
                c.broken_unaware,
            ));
        }
    }
    let begun: Vec<_> = sim
        .land()
        .buildings
        .iter()
        .filter(|b| b.started.day_index() > fire_day)
        .collect();
    let tiled = begun
        .iter()
        .filter(|b| civ_agents::rules::roof_unburnt(&b.spec))
        .count();
    let walled = begun
        .iter()
        .filter(|b| {
            catalog
                .building_index(&b.spec.program)
                .and_then(|i| catalog.buildings.get(i))
                .is_some_and(|d| civ_agents::rules::wall_kind(&b.spec, &d.rules) > 0)
        })
        .count();
    // The buildings the fire took, as they stand now: mended in place, or not.
    let burned: Vec<String> = pop
        .fires
        .list
        .iter()
        .filter(|f| f.began.day_index() >= fire_day)
        .map(|f| {
            sim.land()
                .buildings
                .iter()
                .find(|b| b.id == f.building)
                .map_or_else(
                    || format!("{} gone", f.building.get()),
                    |b| {
                        format!(
                            "{} {:?}{}",
                            b.id.get(),
                            b.state,
                            if b.roofed() { ", roofed" } else { ", unroofed" }
                        )
                    },
                )
        })
        .collect();
    let firing = catalog.technique_index("core:technique/firing");
    let fire_tile = firing.map_or(0, |t| pop.people.iter().filter(|(_, p)| p.knows(t)).count());
    format!(
        "the buildings it took now: {}; {remember} of {} households remember a fire; {fire_tile} \
         people know firing; {} \
         buildings begun since the fire, {tiled} with tile roofs and {walled} with walls of stone \
         or brick; rules for building: {}",
        burned.join(", "),
        households.len(),
        begun.len(),
        if rules.is_empty() {
            "none proposed".to_owned()
        } else {
            rules.join("; ")
        }
    )
}

/// The people living.
fn living(sim: &Sim) -> usize {
    sim.people().people.iter().count()
}

fn one(content: &civ_content::ContentRegistry, seed: u64) -> Result<(Sim, String, usize), String> {
    let mut sim = Sim::create(
        &NewWorld {
            name: "Thatchford".to_owned(),
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
    let lived = fires_by_size(&sim, 0);
    let (fire_day, how) = match lived.first().filter(|f| f.2 >= GREAT) {
        Some(&(_, day, n)) => (
            day,
            format!("a fire of the village's own, on day {day}, took {n} buildings: no NUDGE"),
        ),
        None => {
            let mut waited = 0;
            while waited < WAIT_DAYS {
                let today = sim.land().weather.today;
                if today.fuel_moisture <= DRY && today.wind_ms >= WINDY {
                    break;
                }
                sim.advance_minutes(DAY).map_err(|e| e.to_string())?;
                waited += 1;
            }
            let today = sim.land().weather.today;
            let Some((target, close)) = densest(&sim) else {
                return Ok((
                    sim,
                    "no thatched building stands: nothing to set alight".into(),
                    0,
                ));
            };
            let whose = sim
                .land()
                .buildings
                .iter()
                .find(|b| b.id == target)
                .and_then(|b| sim.people().household(b.household))
                .and_then(|x| x.members.first().copied())
                .map_or_else(|| "someone".to_owned(), |p| sim.people().name_of(p));
            // Noon of the day, when most are awake and about.
            sim.advance_minutes(12 * 60).map_err(|e| e.to_string())?;
            sim.set_fire(target)?;
            let day = sim.now().day_index();
            (
                day,
                format!(
                    "{} fires of its own in {SETTLE_DAYS} days, none great. NUDGE (where): the \
                     observer set alight building {} of {whose}'s household, the thatched building \
                     with the most others within {CLOSE_M:.0} m ({close}); NUDGE (when): noon of \
                     day {day}, after {waited} more days, fuel at {:.2} of its dry mass and a \
                     wind of {:.1} m/s from {:.0}°",
                    lived.len(),
                    target.get(),
                    today.fuel_moisture,
                    today.wind_ms,
                    today.wind_from_deg
                ),
            )
        }
    };
    let people_then = living(&sim);
    // The fire's course: three days.
    sim.advance_minutes(3 * DAY).map_err(|e| e.to_string())?;
    let biggest = fires_by_size(&sim, fire_day).first().map_or(0, |f| f.2);
    let deaths: usize = sim
        .people()
        .fires
        .list
        .iter()
        .filter(|f| f.began.day_index() >= fire_day)
        .map(|f| f.deaths.len())
        .sum();
    let after_fire = format!("it took {biggest} buildings and {deaths} lives");
    sim.advance_minutes(AFTER_DAYS * DAY)
        .map_err(|e| e.to_string())?;
    let (people_now, households_now) = (
        living(&sim),
        sim.people()
            .households
            .iter()
            .filter(|(_, x)| !x.members.is_empty())
            .count(),
    );
    let line = format!(
        "{how}; {after_fire}. After {AFTER_DAYS} days ({people_then} people at the fire, \
         {people_now} in {households_now} households now): {}",
        followed(&sim, fire_day)
    );
    Ok((sim, line, biggest))
}

fn make(saves: &Path, seed: u64) -> Result<String, String> {
    let root = civ_content::find_content_root(Path::new(env!("CARGO_MANIFEST_DIR")))
        .ok_or("content/ is not above the crate")?;
    let content = civ_content::load(&root)
        .registry
        .ok_or("the content does not load: run `civ-host content validate`")?;
    let mut lines = Vec::new();
    let mut tries = 0;
    let (mut sim, biggest) = loop {
        tries += 1;
        let (sim, line, biggest) = one(&content, seed)?;
        lines.push(format!("world {tries}: {line}"));
        if biggest >= GREAT || tries == TRIES {
            break (sim, biggest);
        }
    };
    let kept = if biggest >= GREAT {
        format!("kept world {tries}: its fire took {biggest} buildings")
    } else {
        format!("no fire took {GREAT} buildings in {TRIES} worlds; kept the last")
    };
    let dir = SaveDir::create(
        saves.join(persist::world_dir_name(sim.meta())),
        civ_schema::SAVE_EXTENSION,
    )
    .map_err(|e| e.to_string())?;
    let saved = persist::save(&mut sim, &dir, SaveKind::Manual, "After the great fire")
        .map_err(|e| e.to_string())?;
    Ok(format!(
        "{}\n{kept}\nsaved {}",
        lines.join("\n"),
        saved.path.display()
    ))
}
