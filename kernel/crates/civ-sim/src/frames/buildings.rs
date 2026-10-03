//! Buildings on the boundary (M1 slice D): the snapshot's building revision and the response to a
//! buildings query. Geometry is expanded here from each building's saved design with the same
//! grammar the kernel uses for its facts (ADR-0004 §2); each building's state in words is rendered
//! here too. Observers only draw.

use std::f64::consts::TAU;

use civ_agents::build::{self, HomeWork};
use civ_agents::population;
use civ_grammar::{Expansion, PartKind, Stage, TURN, expand_hut};
use civ_land::Building;
use civ_schema::flatbuffers::FlatBufferBuilder;
use civ_schema::wire;

use super::response;
use crate::Sim;

/// A number that changes whenever ground is claimed for a building or work on one moves on (0 =
/// no buildings): per building, when its stage began plus one, its stage, and the minutes of work
/// done on that stage, summed.
pub fn buildings_rev(sim: &Sim) -> u64 {
    sim.land.buildings.iter().fold(0u64, |rev, b| {
        rev.wrapping_add(b.stage_since.minutes().max(0) as u64 + 1)
            .wrapping_add(u64::from(b.stage))
            .wrapping_add((f64::from(b.work_h) * 60.0).round().max(0.0) as u64)
    })
}

fn percent(x: f64) -> String {
    format!("{:.0}%", (x * 100.0).clamp(0.0, 100.0))
}

/// Share of the work of the stage under way that is done (1 once finished).
fn progress(b: &Building, work: Option<&HomeWork>) -> f64 {
    match work.and_then(HomeWork::needs) {
        Some(s) if s.labour_h > 0.0 => (f64::from(b.work_h) / s.labour_h).clamp(0.0, 1.0),
        Some(_) => 0.0,
        None => 1.0,
    }
}

/// A building's state in words: "walls going up, 40% done; waiting for timber".
pub fn status(sim: &Sim, b: &Building) -> String {
    let catalog = &sim.rules.catalog;
    let def = catalog
        .building_index(&b.spec.program)
        .map(|i| &catalog.buildings[i]);
    let work = def
        .and_then(|d| build::stage_needs(&b.spec, d))
        .map(|stages| HomeWork::of(b, stages));
    let Some(stage) = b.stage() else {
        return "finished".to_owned();
    };
    let (Some(def), Some(work)) = (def, work) else {
        return format!("unfinished: the {}", stage.name());
    };
    let done = progress(b, Some(&work));
    let mut text = match stage {
        Stage::Foundation if b.work_h <= 0.0 => "ground marked out".to_owned(),
        Stage::Foundation => format!("postholes being dug, {} done", percent(done)),
        Stage::Frame => format!("frame going up, {} done", percent(done)),
        Stage::Walls => format!("walls going up, {} done", percent(done)),
        Stage::Roof => format!("being thatched, {} done", percent(done)),
        Stage::Finish => format!("roofed; floor being laid, {} done", percent(done)),
    };
    // What the work is waiting for, if the household has none of a material it needs now.
    let stores = sim
        .people
        .household(b.household)
        .map(|h| population::stores_now(h, sim.now(), &sim.rules.people, &catalog.goods))
        .unwrap_or_default();
    let held = work.held_by_slot(def, &stores);
    if let Some(needs) = work.needs()
        && work.workable_h(&held) <= 1e-6
    {
        let mut missing: Vec<String> = Vec::new();
        for (slot, kg) in needs.materials_kg.iter().enumerate() {
            let name = def
                .materials
                .get(slot)
                .and_then(|&g| catalog.goods.get(g))
                .map(|g| g.name.to_lowercase());
            if *kg > 0.0
                && held.get(slot).copied().unwrap_or(0.0) <= 1e-6
                && let Some(name) = name
                && !missing.contains(&name)
            {
                missing.push(name);
            }
        }
        if !missing.is_empty() {
            text.push_str(&format!("; waiting for {}", missing.join(" and ")));
        }
    }
    text
}

fn vec2((x, y): (i32, i32)) -> wire::Vec2 {
    wire::Vec2::new(x as f32 / 100.0, y as f32 / 100.0)
}

/// A `Response` with every building, oldest first.
pub fn buildings_response(sim: &Sim) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let catalog = &sim.rules.catalog;
    let list: Vec<_> = sim
        .land
        .buildings
        .iter()
        .map(|b| {
            let def = catalog
                .building_index(&b.spec.program)
                .map(|i| &catalog.buildings[i]);
            let expansion: Option<Expansion> = def.and_then(|d| expand_hut(&b.spec, &d.rules).ok());
            let work = expansion
                .as_ref()
                .map(|e| HomeWork::of(b, e.stages.clone()));
            let civ_grammar::Footprint::Round { x, y, radius } = b.spec.footprint;
            let program = fbb.create_string(def.map_or("Building", |d| d.name.as_str()));
            let stage_name = fbb.create_string(b.stage().map_or("finished", Stage::name));
            let text = fbb.create_string(&status(sim, b));
            let outline: Vec<wire::Vec2> = expansion
                .as_ref()
                .map(|e| e.outline.iter().map(|&p| vec2(p)).collect())
                .unwrap_or_default();
            let outline = fbb.create_vector(&outline);
            let posts: Vec<wire::Vec2> = expansion
                .as_ref()
                .map(|e| {
                    e.parts
                        .iter()
                        .filter(|p| p.kind == PartKind::Post)
                        .map(|p| vec2((p.at[0], p.at[1])))
                        .collect()
                })
                .unwrap_or_default();
            let posts = fbb.create_vector(&posts);
            let (door, door_dir) = expansion.as_ref().map_or(((x, y), 0.0), |e| {
                ((e.door.0, e.door.1), f64::from(e.door.2) / TURN * TAU)
            });
            let plot = sim
                .land
                .plots
                .iter()
                .find(|p| p.id == b.plot)
                .map(|p| p.rect);
            let settlement = sim
                .people
                .household(b.household)
                .and_then(|h| h.settlement)
                .map_or(0, |s| s.get());
            wire::BuildingInfo::create(
                &mut fbb,
                &wire::BuildingInfoArgs {
                    id: b.id.get(),
                    household: b.household.get(),
                    settlement,
                    program: Some(program),
                    centre: Some(&vec2((x, y))),
                    radius_m: radius as f32 / 100.0,
                    roof_radius_m: expansion.as_ref().map_or(radius, |e| e.roof_radius_cm) as f32
                        / 100.0,
                    door: Some(&vec2(door)),
                    door_dir: door_dir as f32,
                    stage: b.stage,
                    stage_name: Some(stage_name),
                    progress: progress(b, work.as_ref()) as f32,
                    roofed: b.roofed(),
                    outline: Some(outline),
                    posts: Some(posts),
                    plot_min: plot.map(|r| vec2((r.x, r.y))).as_ref(),
                    plot_size: plot.map(|r| vec2((r.w, r.h))).as_ref(),
                    floor_m2: expansion.as_ref().map_or(0.0, |e| e.floor_area_m2) as f32,
                    sleeps: expansion.as_ref().map_or(0, |e| e.sleeping_places),
                    started_minute: b.started.minutes(),
                    status: Some(text),
                },
            )
        })
        .collect();
    let list = fbb.create_vector(&list);
    let buildings = wire::Buildings::create(
        &mut fbb,
        &wire::BuildingsArgs {
            rev: buildings_rev(sim),
            buildings: Some(list),
        },
    );
    response(
        fbb,
        wire::ResponseBody::Buildings,
        buildings.as_union_value(),
    )
}
