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
/// state, its groups' loss in ten-thousandths and states, and the minutes of upkeep done, summed,
/// over each gas connection's last evening and whether its burners are lit (wire 1.103).
pub fn buildings_rev(sim: &Sim) -> u64 {
    // A building's fire changes how it stands by the minute while anything burns (wire 1.68).
    let fires = &sim.people.fires;
    let burning = if fires.any_burning() {
        sim.now().minutes().max(0) as u64
    } else {
        0
    };
    let fire = (fires.list.len() as u64)
        .wrapping_mul(31)
        .wrapping_add(fires.list.iter().filter(|f| f.ended.is_some()).count() as u64)
        .wrapping_add(burning);
    // A building's gas (wire 1.103): each connection's last evening, its cause and account.
    let now = sim.now().minutes();
    let gas = sim.land.gas.connections.iter().fold(fire, |rev, c| {
        rev.wrapping_mul(31)
            .wrapping_add(c.evening as u64)
            .wrapping_add(u64::from(c.cause.code()) << 8)
            .wrapping_add(u64::from(c.cut_off) << 12)
            .wrapping_add(u64::from(c.lit_from <= now && now < c.lit_until) << 13)
    });
    sim.land.buildings.iter().fold(gas, |rev, b| {
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
/// the eaves, eaves 0.6 m out", and from version 3 its front; and where the building followed stands if in another settlement
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
        // Its front (M8d slice DD): ", 30 % of the walls open, 1.5 h of ornament a square
        // metre, lime-washed, set out about the middle".
        let front = super::people::front_words(&t.traits().map(f64::from));
        words.extend(
            front
                .split(", ")
                .filter(|w| !w.is_empty())
                .map(str::to_owned),
        );
        // What version 3 lets it be built with (M8d slice CY).
        words.extend(civ_agents::style::system_words(&b.spec));
    }
    if let Some(from) = b.style_from {
        // Where the building followed stands, if not where this one does (M5b slice AR).
        let here = sim.people.household(b.household).and_then(|h| h.settlement);
        let mut after = format!(
            "after {}",
            super::people::building_name_from(sim, from, here)
        );
        // And the first building elsewhere its chain of followed buildings goes back to, if the
        // one it followed stands here (M5b slice AR; the M5 diffusion brief §1.7).
        if let Some(first) = sim.people.crossing_of(&sim.land, b).filter(|&x| x != from) {
            after.push_str(&format!(
                ", which goes back to {}",
                super::people::building_name_from(sim, first, here)
            ));
        }
        words.push(after);
    }
    // The rules for building it was begun under and keeps, derived from its start and the
    // laws' history (M6b slice BH; ADR-0022 §12).
    words.extend(rules_kept_by(sim, b, def));
    words.join(", ")
}

/// The rules for building in force where building `b` stands when it was begun that it keeps,
/// in words: "its roof kept Ada's rule that a new roof within 4 m of another building must not
/// burn". A roof rule speaks only where the roof does not burn, a walls rule where the walls are
/// of a kind it allows, and a gap rule where the building stands its gap from every other.
fn rules_kept_by(sim: &Sim, b: &Building, def: &BuildingDef) -> Vec<String> {
    use civ_agents::polity::{PolicyKind, RuleTemplate};
    let Some(settlement) = sim.people.household(b.household).and_then(|x| x.settlement) else {
        return Vec::new();
    };
    let Some(polity) = sim
        .people
        .polities
        .iter()
        .find(|p| p.settlement == settlement)
    else {
        return Vec::new();
    };
    let policies = &sim.rules.catalog.policies;
    let rect = sim
        .land
        .plots
        .iter()
        .find(|p| p.id == b.plot)
        .map(|p| p.rect);
    let plots = |o: &Building| {
        sim.land
            .plots
            .iter()
            .find(|p| p.id == o.plot)
            .map(|p| p.rect)
    };
    let apart = |m: f32| {
        rect.is_some_and(|r| {
            !civ_agents::rules::near_another(
                &r,
                f64::from(m),
                &sim.land.buildings,
                Some(b.id),
                &plots,
            )
        })
    };
    polity
        .in_force(policies, PolicyKind::BuildingRule)
        .filter(|l| l.decided.is_some_and(|t| t <= b.started))
        .filter_map(|l| {
            let rule = policies.get(usize::from(l.policy))?.rule.as_ref()?;
            let (what, kept) = match rule.template {
                RuleTemplate::RoofsUnburnt { .. } => {
                    ("its roof kept", civ_agents::rules::roof_unburnt(&b.spec))
                }
                RuleTemplate::WallsOfKinds { kinds } => (
                    "its walls kept",
                    kinds & (1 << civ_agents::rules::wall_kind(&b.spec, &def.rules)) != 0,
                ),
                RuleTemplate::LeastGap { gap_m } => ("it stands apart, keeping", apart(gap_m)),
                // A home, apart from others' homes or from open wells (M8c slice CW).
                RuleTemplate::FireFromHomes { from_m } => (
                    "it stands apart from others' homes, keeping",
                    def.use_ == civ_land::PlotUse::Dwelling
                        && rect.is_some_and(|r| {
                            !civ_agents::rules::near_others_homes(
                                &r,
                                f64::from(from_m),
                                &sim.land.buildings,
                                b.household,
                                &plots,
                                &|o| sim.rules.catalog.is_dwelling(&o.spec.program),
                            )
                        }),
                ),
                RuleTemplate::MiddenFromWells { from_m } => (
                    "it and its midden stand apart from the wells, keeping",
                    def.use_ == civ_land::PlotUse::Dwelling
                        && rect.is_some_and(|r| {
                            !civ_agents::rules::near_open_well(
                                &r,
                                f64::from(from_m),
                                &sim.land.wells,
                            )
                        }),
                ),
                // Walls of a kind a height rule names, no higher than it allows; mass walls
                // above a storey, tied (M8d slice DB).
                RuleTemplate::MostStoreys { storeys, kinds } => (
                    "it rises low, keeping",
                    kinds & (1 << civ_agents::rules::wall_kind(&b.spec, &def.rules)) != 0
                        && b.spec.storeys <= storeys,
                ),
                RuleTemplate::TiedWalls => (
                    "its walls are tied, keeping",
                    civ_agents::rules::wall_kind(&b.spec, &def.rules) > 0
                        && b.spec.storeys > 1
                        && civ_agents::rules::tied(&b.spec),
                ),
            };
            kept.then(|| {
                format!(
                    "{what} {}'s rule that {}",
                    sim.people.name_of(l.sponsor),
                    rule.template.words()
                )
            })
        })
        .collect()
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
    // A building raised together has its materials made and carried among its hours (M8d slice
    // DC): it waits on nothing but hands.
    if b.polity.is_some() {
        return text;
    }
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
            let (fire, fire_id, fire_words) = super::incidents::building_fire(sim, b);
            let fire_words = fbb.create_string(&fire_words);
            let site = sim
                .land
                .plots
                .iter()
                .find(|p| p.id == b.plot)
                .map(|p| {
                    let why = p
                        .receipt
                        .map(|r| site_words(&r, sim.rules.people.build.site_years));
                    // On a plan's parcel (M8c slice CU): which, and how it was given.
                    let given = p.allotted.map(|a| {
                        format!(
                            "on parcel {} of the plan, given {}",
                            a.parcel + 1,
                            a.rule.words()
                        )
                    });
                    match (why, given) {
                        (Some(w), Some(g)) => format!("{w}; {g}"),
                        (Some(w), None) => w,
                        (None, Some(g)) => g,
                        (None, None) => String::new(),
                    }
                })
                .unwrap_or_default();
            let site = fbb.create_string(&site);
            let gas = fbb.create_string(&gas_words(sim, b));
            let records = fbb.create_string(&records_words(sim, b));
            let current = fbb.create_string(&wire_words(sim, b));
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
                    fire,
                    fire_id,
                    fire_words: Some(fire_words),
                    site: Some(site),
                    gas: Some(gas),
                    records: Some(records),
                    wire: Some(current),
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

/// Building `b`'s gas in words (wire 1.103, M10b slice DY, fifth part): connected to a main, its
/// burners and how its last evening went ("Gas: 1 burner on Ada's main; lit until 22:40"; "not
/// lit on its last evening: the main could not carry its burners"; "cut off: its account went
/// unpaid"); the first building of a household holding a gas holder, the gasworks ("Gasworks:
/// a retort and a gas holder with 12 kg of coal gas of 40; feeds 1 main, 3 homes connected").
/// Empty for neither.
/// The records kept in building `b`, each with its writer (wire 1.106, M10c slice EC): "Records:
/// laying out by cord and stake, by Nell"; empty for none.
pub fn records_words(sim: &Sim, b: &Building) -> String {
    let people = sim.people();
    let techniques = &sim.rules().catalog.techniques;
    let kept: Vec<String> = people
        .writings
        .iter()
        .filter(|r| r.building == Some(b.id))
        .map(|r| {
            let what = techniques
                .get(usize::from(r.technique))
                .map_or_else(|| "a craft".to_owned(), |d| d.name.to_lowercase());
            format!("{what}, by {}", people.name_of(r.maker))
        })
        .collect();
    if kept.is_empty() {
        String::new()
    } else {
        format!("Records: {}", kept.join("; "))
    }
}

pub(crate) fn gas_words(sim: &Sim, b: &Building) -> String {
    use civ_land::gas::Cause;
    let gas = &sim.land.gas;
    let now = sim.now().minutes();
    let mut said = Vec::new();
    if let Some(c) = gas.connections.iter().find(|c| c.building == b.id) {
        let burners = if c.burners == 1 {
            "1 burner".to_owned()
        } else {
            format!("{} burners", c.burners)
        };
        let main = gas.mains.iter().find(|m| m.id == c.main);
        let works = main.map_or_else(
            || "a gasworks".to_owned(),
            |m| super::ways::keeper(sim, civ_land::ways::Keeper::Household(m.works)),
        );
        let works = works.trim_end_matches("'s household");
        let state = if c.cut_off {
            "cut off: its account went unpaid".to_owned()
        } else if main.is_some_and(|m| !m.whole()) {
            "waiting for the main to be laid".to_owned()
        } else if c.evening < 0 {
            "not yet lit".to_owned()
        } else {
            match c.cause {
                Cause::Served if c.lit_from <= now && now < c.lit_until => {
                    format!("lit until {}", super::ways::clock_at(c.lit_until))
                }
                Cause::Served => "lit on its last evening".to_owned(),
                Cause::NoGas => {
                    "not lit on its last evening: the works had too little gas".to_owned()
                }
                Cause::Overloaded => {
                    "not lit on its last evening: the main could not carry its burners".to_owned()
                }
                Cause::Broken => "not lit on its last evening: the main is broken".to_owned(),
                Cause::CutOff => "cut off: its account went unpaid".to_owned(),
            }
        };
        said.push(format!("Gas: {burners} on {works}'s main; {state}"));
    }
    let is_first = sim
        .land
        .buildings
        .iter()
        .find(|o| o.household == b.household)
        .is_some_and(|o| o.id == b.id);
    let catalog = &sim.rules.catalog;
    if is_first && let Some(x) = sim.people.household(b.household) {
        let held = |g: usize| x.stores.get(g).copied().unwrap_or(0.0);
        let whole = |g: usize| held(g) >= 1.0 - 1e-9;
        let holder = catalog.goods.iter().enumerate().find_map(|(h, d)| {
            let keeps = d.holder?;
            whole(h).then_some((h, keeps.good?, keeps.kg))
        });
        if let Some((h, g, kg)) = holder {
            let mut parts: Vec<String> = catalog
                .recipes
                .iter()
                .find(|r| r.outputs.iter().any(|&(o, _)| o == g) && !r.tools.is_empty())
                .filter(|r| r.tools.iter().all(|&t| whole(t)))
                .map(|r| {
                    r.tools
                        .iter()
                        .map(|&t| format!("a {}", catalog.goods[t].name.to_lowercase()))
                        .collect()
                })
                .unwrap_or_default();
            parts.push(format!(
                "a {} with {:.0} kg of {} of {:.0}",
                catalog.goods[h].name.to_lowercase(),
                held(g),
                catalog.goods[g].name.to_lowercase(),
                kg
            ));
            let mut out = format!("Gasworks: {}", parts.join(" and "));
            let mains: Vec<_> = gas
                .mains
                .iter()
                .filter(|m| m.works == b.household)
                .collect();
            if !mains.is_empty() {
                let homes = gas
                    .connections
                    .iter()
                    .filter(|c| mains.iter().any(|m| m.id == c.main))
                    .count();
                let feeds = if mains.len() == 1 {
                    "1 main".to_owned()
                } else {
                    format!("{} mains", mains.len())
                };
                let homes = if homes == 1 {
                    "1 home connected".to_owned()
                } else {
                    format!("{homes} homes connected")
                };
                out.push_str(&format!("; feeds {feeds}, {homes}"));
            }
            said.push(out);
        }
    }
    said.join(". ")
}

/// Building `b`'s current in words (wire 1.107, M11a, fifth part): hooked up to a line, what it asks
/// and how its last evening went ("Current: 0.2 kW from Ada's station; lit until 22:40"), and for a
/// household's first building, the station it keeps ("Station: a dynamo beside its own water
/// installation; feeds 2 lines, 3 homes hooked up"); empty for neither.
pub(crate) fn wire_words(sim: &Sim, b: &Building) -> String {
    use civ_land::wire::Cause;
    let wires = &sim.land.wires;
    let now = sim.now().minutes();
    let mut said = Vec::new();
    if let Some(k) = wires.hookups.iter().find(|k| k.building == b.id) {
        let line = wires.lines.iter().find(|l| l.id == k.line);
        let station = line.map_or_else(
            || "a station".to_owned(),
            |l| {
                let s = super::ways::keeper(sim, civ_land::ways::Keeper::Household(l.station));
                format!("{}'s station", s.trim_end_matches("'s household"))
            },
        );
        let state = if k.cut_off {
            "cut off: its account went unpaid".to_owned()
        } else if line.is_some_and(|l| !l.whole()) {
            "waiting for the line to be strung".to_owned()
        } else if k.lit_until < 0 && k.cause == Cause::Served {
            "not yet lit".to_owned()
        } else {
            match k.cause {
                Cause::Served if now < k.lit_until => {
                    format!("lit until {}", super::ways::clock_at(k.lit_until))
                }
                Cause::Served => "lit on its last evening".to_owned(),
                Cause::StationShort => {
                    "not lit on its last evening: the station gave too little".to_owned()
                }
                Cause::LineFull => {
                    "not lit on its last evening: a line could not pass it".to_owned()
                }
                Cause::LineDown => {
                    "not lit on its last evening: a line is down or unstrung".to_owned()
                }
                Cause::CutOff => "cut off: its account went unpaid".to_owned(),
            }
        };
        // Its service record (ADR-0036 §4): of the evenings it asked, those it was served.
        let (asked, served) = k.record_of(30);
        let record = if asked > 0 {
            let evenings = if asked == 1 { "evening" } else { "evenings" };
            format!("; served {served} of {asked} {evenings} asked in the last 30")
        } else {
            String::new()
        };
        said.push(format!(
            "Current: {:.1} kW from {station}; {state}{record}",
            k.ask_kw
        ));
    }
    let is_first = sim
        .land
        .buildings
        .iter()
        .find(|o| o.household == b.household)
        .is_some_and(|o| o.id == b.id);
    let catalog = &sim.rules.catalog;
    if is_first && let Some(x) = sim.people.household(b.household) {
        let dynamos = catalog
            .goods
            .iter()
            .enumerate()
            .filter(|(g, d)| {
                d.power
                    .is_some_and(|p| p.source == civ_agents::params::PowerSource::Shaft)
                    && x.stores.get(*g).copied().unwrap_or(0.0) >= 1.0 - 1e-9
            })
            .map(|(g, _)| x.stores[g].floor() as u32)
            .sum::<u32>();
        let owns = sim.land.installations.iter().any(|s| s.owner == x.id);
        // An engine that burns a named fuel turns them too (M11b, first part).
        let engine = catalog.goods.iter().enumerate().find(|(g, d)| {
            d.power.is_some_and(|p| {
                p.source == civ_agents::params::PowerSource::Fuel && p.fuel.is_some()
            }) && x.stores.get(*g).copied().unwrap_or(0.0) >= 1.0 - 1e-9
        });
        if dynamos > 0 && (owns || engine.is_some()) {
            let beside = match (owns, engine) {
                (true, Some((_, e))) => format!(
                    "its own water installation and its {}",
                    e.name.to_lowercase()
                ),
                (false, Some((_, e))) => format!("its {}", e.name.to_lowercase()),
                _ => "its own water installation".to_owned(),
            };
            let mut out = if dynamos == 1 {
                format!("Station: a dynamo beside {beside}")
            } else {
                format!("Station: {dynamos} dynamos beside {beside}")
            };
            let lines: Vec<_> = wires
                .lines
                .iter()
                .filter(|l| l.station == b.household)
                .collect();
            if !lines.is_empty() {
                let homes = wires
                    .hookups
                    .iter()
                    .filter(|k| lines.iter().any(|l| l.id == k.line))
                    .count();
                let feeds = if lines.len() == 1 {
                    "1 line".to_owned()
                } else {
                    format!("{} lines", lines.len())
                };
                let homes = if homes == 1 {
                    "1 home hooked up".to_owned()
                } else {
                    format!("{homes} homes hooked up")
                };
                out.push_str(&format!("; feeds {feeds}, {homes}"));
            }
            said.push(out);
        }
    }
    said.join(". ")
}

/// Why a household chose a plot, from its receipt (M8c slice CS, third part): over how many others
/// and by how many hours of walking and levelling over its reckoning's `years`.
pub(crate) fn site_words(r: &civ_land::SiteReceipt, years: f64) -> String {
    let others = r.weighed.saturating_sub(1);
    if others == 0 {
        return "built where its household first looked for ground".to_owned();
    }
    let less = f64::from(r.next_hours - r.hours).max(0.0);
    let years = if (years - years.round()).abs() < 1e-9 {
        match years.round() as i64 {
            1 => "a year".to_owned(),
            n => format!("{n} years"),
        }
    } else {
        format!("{years:.1} years")
    };
    let over = match others {
        1 => "over 1 other".to_owned(),
        n => format!("over {n} others"),
    };
    if less < 0.5 {
        format!("chosen {over}, about as good as the next best over {years}")
    } else {
        format!(
            "chosen {over}: {less:.0} hours less walking and levelling over {years} than the \
             next best"
        )
    }
}

#[cfg(test)]
mod site_tests {
    #[test]
    fn a_plot_s_receipt_reads_as_why_it_was_chosen() {
        let r = |weighed, hours, next_hours| civ_land::SiteReceipt {
            weighed,
            hours,
            next_hours,
        };
        assert_eq!(
            super::site_words(&r(1, 40.0, 40.0), 5.0),
            "built where its household first looked for ground"
        );
        assert_eq!(
            super::site_words(&r(6, 400.0, 434.2), 5.0),
            "chosen over 5 others: 34 hours less walking and levelling over 5 years than the next \
             best"
        );
        assert_eq!(
            super::site_words(&r(2, 400.0, 400.1), 1.0),
            "chosen over 1 other, about as good as the next best over a year"
        );
    }
}
