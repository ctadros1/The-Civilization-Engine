//! Buildings on the boundary (M1 slice D): the snapshot's building revision and the response to a
//! buildings query. Geometry is expanded here from each building's saved design with the same
//! grammar the kernel uses for its facts (ADR-0004 §2); each building's state in words is rendered
//! here too. Observers only draw.

use std::collections::HashMap;
use std::f64::consts::TAU;

use civ_agents::build::{self, HomeWork};
use civ_agents::condition;
use civ_agents::params::BuildingDef;
use civ_agents::person::Keeping;
use civ_agents::population;
use civ_core::PermanentId;
use civ_grammar::{
    Expansion, Footprint, GroupKind, PartKind, ProgramRules, Stage, TURN, expand, frame_params,
};
use civ_land::{Building, BuildingState, GroupState};
use civ_schema::flatbuffers::FlatBufferBuilder;
use civ_schema::wire;

use super::response;
use crate::Sim;

/// A number that changes whenever ground is claimed for a building, work on one moves on, a
/// workshop changes firm or a building's condition changes (0 = no buildings): per building, when
/// its stage began plus one, its stage, the minutes of work done on that stage, its firm, its
/// state, its groups' loss in ten-thousandths and states, and the minutes of upkeep done, summed.
pub fn buildings_rev(sim: &Sim) -> u64 {
    sim.land.buildings.iter().fold(0u64, |rev, b| {
        let condition = b.condition.iter().fold(0u64, |sum, c| {
            sum.wrapping_add((f64::from(c.loss) * 1e4).round().max(0.0) as u64)
                .wrapping_add(group_state(c.state) as u64)
        });
        rev.wrapping_add(b.stage_since.minutes().max(0) as u64 + 1)
            .wrapping_add(u64::from(b.stage))
            .wrapping_add((f64::from(b.work_h) * 60.0).round().max(0.0) as u64)
            .wrapping_add(b.firm.map_or(0, |f| f.get()))
            .wrapping_add(building_state(b.state) as u64)
            .wrapping_add(condition)
            .wrapping_add(b.repair.map_or(0, |r| {
                (f64::from(r.work_h) * 60.0).round().max(0.0) as u64 + 1
            }))
    })
}

fn group_state(s: GroupState) -> u8 {
    match s {
        GroupState::Sound => 0,
        GroupState::Symptom => 1,
        GroupState::Failed => 2,
    }
}

fn building_state(s: BuildingState) -> u8 {
    match s {
        BuildingState::Standing => 0,
        BuildingState::Damaged => 1,
        BuildingState::Ruin => 2,
    }
}

/// The upkeep under way on building `b` (of expansion `e`) in words: "mending the covering, 40%
/// done"; empty for none.
fn upkeep(b: &Building, e: Option<&Expansion>) -> String {
    let (Some(r), Some(e)) = (b.repair, e) else {
        return String::new();
    };
    let kind = GroupKind::of_group(r.group).map_or("building", GroupKind::name);
    let done = condition::mend_needs(e, r.group, f64::from(r.share)).map_or(0.0, |n| {
        if n.labour_h > 0.0 {
            f64::from(r.work_h) / n.labour_h
        } else {
            1.0
        }
    });
    format!("mending the {kind}, {} done", percent(done))
}

/// How building `b` of program `def` was built, in words, and the building its household's taste
/// followed (M3b slice R): "roof pitched 49°, walls 1.9 m to the eaves, after Bo's hut"; for a
/// frame building its storeys and how far its eaves reach: "roof pitched 50°, storeys 2.1 m to
/// the eaves, eaves 0.6 m out"; and where the building followed stands if in another settlement
/// (M5b slice AR): "after Bo's hut at Westford". Empty when the content lacks its program.
pub fn style_words(sim: &Sim, b: &Building, def: Option<&BuildingDef>) -> String {
    let Some(def) = def else {
        return String::new();
    };
    let t = civ_agents::style::traits_of(&b.spec, def);
    let mut words = vec![format!("roof pitched {:.0}°", t.pitch_centideg / 100.0)];
    let walls = if b.spec.storeys > 1 {
        "storeys"
    } else {
        "walls"
    };
    words.push(format!("{walls} {:.1} m to the eaves", t.eave_cm / 100.0));
    if matches!(def.rules, ProgramRules::Frame(_)) {
        words.push(format!("eaves {:.1} m out", t.overhang_cm / 100.0));
    }
    if let Some(from) = b.style_from {
        // Where the building followed stands, if not where this one does (M5b slice AR).
        let here = sim.people.household(b.household).and_then(|h| h.settlement);
        words.push(format!(
            "after {}",
            super::people::building_name_from(sim, from, here)
        ));
    }
    words.join(", ")
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
        return if b.standing() { "finished" } else { "a ruin" }.to_owned();
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

/// What each household keeps under its roofs now: the room its buildings give by kind
/// ([`civ_grammar::storage`]) and the kilograms of each good in each kind of room.
type Kept = HashMap<PermanentId, ([f64; 3], [Vec<f64>; 3])>;

fn kept_by_household(sim: &Sim) -> Kept {
    let catalog = &sim.rules.catalog;
    let mut room: HashMap<PermanentId, [f64; 3]> = HashMap::new();
    for b in &sim.land.buildings {
        let r = population::room_of(b, catalog);
        let e = room.entry(b.household).or_insert([0.0; 3]);
        for (e, r) in e.iter_mut().zip(r) {
            *e += r;
        }
    }
    room.into_iter()
        .map(|(h, room)| {
            let fill = sim.people.household(h).map_or_else(
                || [Vec::new(), Vec::new(), Vec::new()],
                |x| {
                    let stores =
                        population::stores_now(x, sim.now(), &sim.rules.people, &catalog.goods);
                    Keeping::fill(&stores, &catalog.goods, room)
                },
            );
            (h, (room, fill))
        })
        .collect()
}

/// A mass in words: "850 kg", "1.2 t".
fn mass(kg: f64) -> String {
    if kg < 1_000.0 {
        format!("{:.0} kg", kg.max(0.0))
    } else {
        format!("{:.1} t", kg / 1_000.0)
    }
}

/// What building `b` holds of its household's goods, kilograms by kind of room, and in words:
/// "loft over 1 bay: 1.2 t of 1.9 t, mostly grain; floor: 2.1 t of 3.8 t, mostly provisions".
/// Each kind of room's goods are shared among the household's buildings by the room each gives.
fn stored(sim: &Sim, b: &Building, kept: &Kept) -> ([f64; 3], String) {
    use civ_grammar::storage;
    let catalog = &sim.rules.catalog;
    let mine = population::room_of(b, catalog);
    let Some((room, fill)) = kept.get(&b.household) else {
        return ([0.0; 3], String::new());
    };
    let mut kg = [0.0; 3];
    let mut words = Vec::new();
    for kind in 0..storage::KINDS {
        if !(mine[kind] > 0.0 && mine[kind].is_finite() && room[kind] > 0.0) {
            continue;
        }
        let part = mine[kind] / room[kind];
        let goods = fill[kind].iter().map(|g| g * part);
        kg[kind] = goods.clone().sum();
        let mostly = goods
            .enumerate()
            .filter(|(_, g)| *g > 0.0)
            .max_by(|a, b| a.1.total_cmp(&b.1).then(b.0.cmp(&a.0)))
            .and_then(|(g, _)| catalog.goods.get(g))
            .map(|g| format!(", mostly {}", g.name.to_lowercase()))
            .unwrap_or_default();
        let label = match kind {
            storage::RAISED => "raised floor".to_owned(),
            storage::LOFT => {
                let n = b.spec.params[frame_params::LOFT_BAYS].count_ones();
                format!("loft over {n} {}", if n == 1 { "bay" } else { "bays" })
            }
            _ => "floor".to_owned(),
        };
        words.push(format!(
            "{label}: {} of {}{mostly}",
            mass(kg[kind]),
            mass(mine[kind])
        ));
    }
    (kg, words.join("; "))
}

fn vec2((x, y): (i32, i32)) -> wire::Vec2 {
    wire::Vec2::new(x as f32 / 100.0, y as f32 / 100.0)
}

/// A `Response` with every building, oldest first.
pub fn buildings_response(sim: &Sim) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let catalog = &sim.rules.catalog;
    let kept = kept_by_household(sim);
    let list: Vec<_> = sim
        .land
        .buildings
        .iter()
        .map(|b| {
            let def = catalog
                .building_index(&b.spec.program)
                .map(|i| &catalog.buildings[i]);
            let expansion: Option<Expansion> = def.and_then(|d| expand(&b.spec, &d.rules).ok());
            let work = expansion
                .as_ref()
                .map(|e| HomeWork::of(b, e.stages.clone()));
            let (x, y) = b.spec.footprint.centre();
            // A hut's wall line, or the circle round a frame's (what older observers draw).
            let (radius, size, angle, bays, lofts) = match b.spec.footprint {
                Footprint::Round { radius, .. } => (radius, (2 * radius, 2 * radius), 0, 1, 0),
                Footprint::Rect {
                    length,
                    width,
                    angle,
                    ..
                } => {
                    let half = (f64::from(length) / 2.0).hypot(f64::from(width) / 2.0);
                    let p = &b.spec.params;
                    (
                        half.round() as i32,
                        (length, width),
                        angle,
                        p[frame_params::BAYS],
                        p[frame_params::LOFT_BAYS],
                    )
                }
            };
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
            let roof_outline: Vec<wire::Vec2> = expansion
                .as_ref()
                .map(|e| e.roof_outline.iter().map(|&p| vec2(p)).collect())
                .unwrap_or_default();
            let roof_outline = fbb.create_vector(&roof_outline);
            let ridge: Vec<wire::Vec2> = expansion
                .as_ref()
                .and_then(|e| e.ridge)
                .map(|r| r.iter().map(|&p| vec2(p)).collect())
                .unwrap_or_default();
            let ridge = fbb.create_vector(&ridge);
            let floor_by_use: Vec<f32> = expansion
                .as_ref()
                .map(|e| e.floor_by_use.iter().map(|&a| a as f32).collect())
                .unwrap_or_default();
            let floor_by_use = fbb.create_vector(&floor_by_use);
            let storage_kg: Vec<f32> = expansion
                .as_ref()
                .map(|e| e.storage_kg.iter().map(|&kg| kg as f32).collect())
                .unwrap_or_default();
            let storage_kg = fbb.create_vector(&storage_kg);
            let (stored_kg, stored_words) = stored(sim, b, &kept);
            let stored_kg = fbb.create_vector(&stored_kg.map(|kg| kg as f32));
            let stored_words = fbb.create_string(&stored_words);
            let firm_name = b
                .firm
                .map(|f| super::people::firm_name(sim, f))
                .unwrap_or_default();
            let firm_name = fbb.create_string(&firm_name);
            let upkeep_words = fbb.create_string(&upkeep(b, expansion.as_ref()));
            let style = fbb.create_string(&style_words(sim, b, def));
            let symptoms = fbb.create_string(
                &def.map_or_else(String::new, |d| condition::symptoms(b, &d.upkeep)),
            );
            let leak = def.map_or(0.0, |d| condition::leak(b, &d.upkeep)) as f32;
            let mut groups = Vec::with_capacity(b.condition.len());
            for c in &b.condition {
                let kind =
                    fbb.create_string(GroupKind::of_group(c.group).map_or("", GroupKind::name));
                groups.push(wire::GroupInfo::create(
                    &mut fbb,
                    &wire::GroupInfoArgs {
                        id: c.group,
                        kind: Some(kind),
                        quality: c.quality,
                        loss: c.loss,
                        state: wire::GroupState(group_state(c.state)),
                        installed_minute: c.installed.minutes(),
                        repaired_minute: c.repaired.minutes(),
                    },
                ));
            }
            let groups = fbb.create_vector(&groups);
            let grammar = fbb.create_string(def.map_or("", |d| d.grammar().name()));
            let purpose = fbb.create_string(def.map_or("", |d| d.use_.name()));
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
                    grammar: Some(grammar),
                    purpose: Some(purpose),
                    size: Some(&vec2(size)),
                    angle: (f64::from(angle) / TURN * TAU) as f32,
                    storeys: b.spec.storeys,
                    bays: u8::try_from(bays).unwrap_or(0),
                    loft_bays: u16::try_from(lofts).unwrap_or(0),
                    roof_outline: Some(roof_outline),
                    ridge: Some(ridge),
                    apex_m: expansion.as_ref().map_or(0, |e| e.apex_cm) as f32 / 100.0,
                    floor_by_use: Some(floor_by_use),
                    storage_kg: Some(storage_kg),
                    work_places: expansion.as_ref().map_or(0, |e| e.work_places),
                    stored_kg: Some(stored_kg),
                    stored: Some(stored_words),
                    firm: b.firm.map_or(0, |f| f.get()),
                    firm_name: Some(firm_name),
                    state: wire::BuildingState(building_state(b.state)),
                    symptoms: Some(symptoms),
                    leak,
                    groups: Some(groups),
                    upkeep: Some(upkeep_words),
                    style: Some(style),
                    style_from: b.style_from.map_or(0, |f| f.get()),
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
