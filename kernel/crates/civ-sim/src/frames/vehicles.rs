//! The vehicles layer (wire 1.86, M8b slices CO and CP; ADR-0029 §3, §9): every boat and cart,
//! and (wire 1.100, M10b slice DV) every wagon on its line, whose it is, where it stands or who of
//! its household has it out, how far it is made, its wear and the goods aboard; and (wire 1.101,
//! M10b slice DW) every engine on rails, an engine or wagon on a run drawn where the run has it
//! along its line, the run in its words. It reads the world and changes nothing.

use civ_agents::person::Step;
use civ_core::PermanentId;
use civ_land::boats::Boat;
use civ_land::trains::{Run, RunState};
use civ_schema::flatbuffers::FlatBufferBuilder;
use civ_schema::wire;

use super::people::eldest_name;
use super::response;
use crate::Sim;

/// A vehicle's mode on the wire: 0 a boat, 1 a cart, 2 a wagon, 3 an engine on rails.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Boat = 0,
    Cart = 1,
    Wagon = 2,
    Engine = 3,
}

/// The gap between an engine and its first wagon on a run, and between wagons, metres: about a
/// wagon's length.
const CAR_GAP_M: f64 = 4.0;

const MODES: [Mode; 3] = [Mode::Boat, Mode::Cart, Mode::Wagon];

impl Mode {
    /// The land's store of vehicles of this mode.
    fn list(self, land: &civ_land::Land) -> &[Boat] {
        match self {
            Mode::Boat => &land.boats.list,
            Mode::Cart => &land.carts.list,
            Mode::Wagon | Mode::Engine => &land.wagons.list,
        }
    }

    /// The mode a vehicle of this store shows as: a rail vehicle with an engine is an engine.
    fn of(self, sim: &Sim, v: &Boat) -> Mode {
        let engine = sim
            .rules
            .catalog
            .vehicles
            .get(usize::from(v.vehicle))
            .is_some_and(|d| d.engine.is_some());
        if self == Mode::Wagon && engine {
            Mode::Engine
        } else {
            self
        }
    }
}

/// The run vehicle `id` is on, if any, and its place in the train: 0 the engine, then its wagons
/// in order.
fn on_run(sim: &Sim, id: PermanentId) -> Option<(&Run, usize)> {
    sim.land.runs.list.iter().find_map(|r| {
        if r.engine == id {
            Some((r, 0))
        } else {
            r.cars.iter().position(|&c| c == id).map(|k| (r, k + 1))
        }
    })
}

/// Where the `k`-th of run `run`'s train is now, metres from the map's north-west corner: the
/// engine at the share of its leg's section it has run (at the section's start while it waits for
/// it, at the line's end once it has arrived), each wagon `CAR_GAP_M` further behind, along the
/// line's cells.
fn run_place(sim: &Sim, run: &Run, k: usize) -> Option<(f32, f32)> {
    let way = sim.land.ways.list.iter().find(|w| w.id == run.way)?;
    let cells: Vec<u32> = way
        .segments
        .iter()
        .flat_map(|s| s.cells.iter().copied())
        .collect();
    let points: Vec<(f64, f64)> = cells
        .iter()
        .map(|&c| {
            let (x, y) = civ_agents::population::cell_centre(&sim.map, c as usize);
            (f64::from(x), f64::from(y))
        })
        .collect();
    let mut along = vec![0.0f64; points.len()];
    for i in 1..points.len() {
        let (a, b) = (points[i - 1], points[i]);
        along[i] = along[i - 1] + (b.0 - a.0).hypot(b.1 - a.1);
    }
    let total = *along.last()?;
    let sections = civ_land::trains::sections(sim.land.runs.loops_of(run.way), cells.len());
    // How far along the line, first cell to last, the engine is.
    let leg = run.legs.get(usize::from(run.leg));
    let section = |s: u16| sections.get(usize::from(s)).copied();
    let s = match run.state {
        RunState::Arrived { .. } => {
            if run.back {
                0.0
            } else {
                total
            }
        }
        RunState::Waiting { .. } => {
            let (a, b) = section(leg?.0)?;
            if run.back { along[b] } else { along[a] }
        }
        RunState::Running { until } => {
            let &(sec, minutes) = leg?;
            let (a, b) = section(sec)?;
            let left = (until - sim.now().minutes()) as f64 / f64::from(minutes.max(1));
            let done = (1.0 - left).clamp(0.0, 1.0);
            if run.back {
                along[b] - done * (along[b] - along[a])
            } else {
                along[a] + done * (along[b] - along[a])
            }
        }
    };
    // The train trails behind the way it runs.
    let behind = CAR_GAP_M * k as f64;
    let s = if run.back { s + behind } else { s - behind }.clamp(0.0, total);
    let i = along
        .partition_point(|&d| d < s)
        .clamp(1, points.len().max(2) - 1);
    if points.len() < 2 {
        return points.first().map(|&(x, y)| (x as f32, y as f32));
    }
    let (a, b) = (points[i - 1], points[i]);
    let span = (along[i] - along[i - 1]).max(1e-9);
    let t = ((s - along[i - 1]) / span).clamp(0.0, 1.0);
    Some((
        (a.0 + t * (b.0 - a.0)) as f32,
        (a.1 + t * (b.1 - a.1)) as f32,
    ))
}

/// A run on the wire: 1 waiting, 2 running, 3 arrived.
fn run_code(run: &Run) -> u8 {
    match run.state {
        RunState::Waiting { .. } => 1,
        RunState::Running { .. } => 2,
        RunState::Arrived { .. } => 3,
    }
}

/// A run in words, as its engine tells it: "running home with 4 wagons, 380 kg aboard", "out
/// light to the pit with a wagon", "waiting at the loop for the section ahead", "arrived home".
/// Loaded it is bound home, light it is bound for the pit.
fn run_words(run: &Run) -> String {
    let home = run.load_kg >= 0.5;
    let wagons = match run.cars.len() {
        0 => "no wagons".to_owned(),
        1 => "a wagon".to_owned(),
        n => format!("{n} wagons"),
    };
    let bound = if home {
        format!("home with {wagons}, {:.0} kg aboard", run.load_kg)
    } else {
        format!("out light to the pit with {wagons}")
    };
    match run.state {
        RunState::Running { .. } => format!("running {bound}"),
        RunState::Waiting { .. } if run.leg == 0 => {
            format!("waiting to set off {bound}, the line ahead held")
        }
        RunState::Waiting { .. } => format!("waiting at the loop for the section ahead, {bound}"),
        RunState::Arrived { .. } if home => "arrived home".to_owned(),
        RunState::Arrived { .. } => "arrived at the pit".to_owned(),
    }
}

/// Who of `owner` has a vehicle standing at cell `at` out now: someone of the household whose step
/// in progress is a paddle (a boat) or a haul (a cart) begun from that cell. A wagon is pushed
/// within a session's digging, never out on a step of its own.
fn out_with(sim: &Sim, owner: PermanentId, at: u32, mode: Mode) -> Option<PermanentId> {
    if mode == Mode::Wagon {
        return None;
    }
    let boat = mode == Mode::Boat;
    let map = &sim.map;
    let household = sim.people.household(owner)?;
    household.members.iter().copied().find(|&m| {
        sim.people.person(m).is_some_and(|p| {
            let going = match p.act.steps.get(usize::from(p.act.step)) {
                Some(Step::Paddle { .. }) => boat,
                Some(Step::Haul { .. }) => !boat,
                _ => false,
            };
            going
                && p.trip.as_ref().is_some_and(|t| {
                    t.points
                        .first()
                        .is_some_and(|&q| civ_agents::population::cell_of(map, q) as u32 == at)
                })
        })
    })
}

/// The goods aboard vehicle `id`, kg: what its consignments in transit hold.
fn load_kg(sim: &Sim, id: PermanentId) -> f64 {
    sim.people
        .consignments
        .iter()
        .filter(|c| c.boat == id)
        .map(|c| c.stores.iter().sum::<f64>())
        .sum()
}

/// The vehicles' revision: 0 while the world has never had a boat or a cart; else it changes
/// whenever one is begun, worked on, made, moved or gone, taken out or brought in, or the goods
/// aboard one change.
pub fn vehicles_rev(sim: &Sim) -> u64 {
    let land = &sim.land;
    if land.boats.list.is_empty() && land.carts.list.is_empty() && land.wagons.list.is_empty() {
        return 0;
    }
    // FNV-1a over what the layer shows that the land's revisions do not.
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut mix = |v: u64| {
        for b in v.to_le_bytes() {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
    };
    mix(u64::from(land.boats.revision()));
    mix(u64::from(land.carts.revision()));
    mix(u64::from(land.wagons.revision()));
    for (list, mode) in MODES.map(|m| (m.list(land), m)) {
        for v in list {
            mix(v.id.get());
            mix(u64::from(v.at));
            mix(u64::from(v.work_h.to_bits()));
            mix(out_with(sim, v.owner, v.at, mode).map_or(0, PermanentId::get));
        }
    }
    // A run moves its train along the line as the clock goes (wire 1.101).
    mix(u64::from(land.runs.revision()));
    for r in &land.runs.list {
        mix(r.id.get());
        mix(u64::from(run_code(r)));
        if let Some((x, y)) = run_place(sim, r, 0) {
            mix(u64::from(x.to_bits()));
            mix(u64::from(y.to_bits()));
        }
    }
    for c in &sim.people.consignments {
        mix(c.id.get());
        mix(c.boat.get());
        mix(c.stores.iter().sum::<f64>().to_bits());
    }
    h.max(1)
}

/// A vehicle in words: "Ada's household's dugout, made in year 2, a tenth worn; out with Bo, 40 kg
/// aboard", "Ada's household's handcart, being made: 12 of 40 hours' work done", "Ada's
/// household's locomotive, made in year 3; running home with 4 wagons, 380 kg aboard".
fn words(
    sim: &Sim,
    v: &Boat,
    name: &str,
    owner: &str,
    out: Option<PermanentId>,
    load: f64,
) -> String {
    let mut text = format!("{owner}'s {}", name.to_lowercase());
    if v.made() {
        text.push_str(&format!(
            ", made in year {}",
            super::crossings::year_of(v.made)
        ));
        let worn = (v.wear * 10.0).round() as i32;
        if worn >= 1 {
            text.push_str(&format!(", {} worn", tenths(worn)));
        }
    } else {
        text.push_str(&format!(
            ", being made: {:.0} of {:.0} hours' work done",
            v.work_h.floor(),
            v.labour_h
        ));
    }
    if let Some((run, k)) = on_run(sim, v.id) {
        if k == 0 {
            text.push_str(&format!("; {}", run_words(run)));
        } else {
            text.push_str(&format!("; in a train {}", run_words(run)));
            if load >= 0.5 {
                text.push_str(&format!(", {load:.0} kg on it"));
            }
        }
        return text;
    }
    if let Some(p) = out.and_then(|p| sim.people.person(p)) {
        text.push_str(&format!("; out with {}", p.given));
        if load >= 0.5 {
            text.push_str(&format!(", {load:.0} kg aboard"));
        }
    } else if load >= 0.5 {
        text.push_str(&format!("; {load:.0} kg aboard"));
    }
    text
}

/// Tenths in words: "a tenth", "three tenths", "half", "all".
fn tenths(n: i32) -> String {
    const WORDS: [&str; 9] = [
        "a tenth",
        "two tenths",
        "three tenths",
        "four tenths",
        "half",
        "six tenths",
        "seven tenths",
        "eight tenths",
        "nine tenths",
    ];
    usize::try_from(n - 1)
        .ok()
        .and_then(|i| WORDS.get(i))
        .map_or_else(|| "all".to_owned(), |w| (*w).to_owned())
}

/// Every boat, then every cart, then every wagon and engine on rails, each in the order begun.
pub fn vehicles_response(sim: &Sim) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let land = &sim.land;
    let mut list =
        Vec::with_capacity(land.boats.list.len() + land.carts.list.len() + land.wagons.list.len());
    for (vehicles, mode) in MODES.map(|m| (m.list(land), m)) {
        for v in vehicles {
            let name = sim
                .rules
                .catalog
                .vehicles
                .get(usize::from(v.vehicle))
                .map_or_else(|| "Vehicle".to_owned(), |d| d.name.clone());
            let owner = eldest_name(sim, v.owner)
                .map_or_else(|| "a household".to_owned(), |n| format!("{n}'s household"));
            let out = out_with(sim, v.owner, v.at, mode);
            let load = load_kg(sim, v.id);
            let run = on_run(sim, v.id);
            let placed = run.and_then(|(r, k)| run_place(sim, r, k));
            let (x, y) = if let Some(at) = placed {
                at
            } else if (v.at as usize) < sim.map.water.len() {
                civ_agents::population::cell_centre(&sim.map, v.at as usize)
            } else {
                (0.0, 0.0)
            };
            let made = if v.labour_h > 0.0 {
                (v.work_h / v.labour_h).clamp(0.0, 1.0)
            } else {
                1.0
            };
            let text = fbb.create_string(&words(sim, v, &name, &owner, out, load));
            let name = fbb.create_string(&name);
            let owner = fbb.create_string(&owner);
            list.push(wire::VehicleInfo::create(
                &mut fbb,
                &wire::VehicleInfoArgs {
                    id: v.id.get(),
                    mode: mode.of(sim, v) as u8,
                    name: Some(name),
                    owner: v.owner.get(),
                    owner_name: Some(owner),
                    x,
                    y,
                    made,
                    wear: v.wear.clamp(0.0, 1.0),
                    out_with: out.map_or(0, PermanentId::get),
                    load_kg: load as f32,
                    words: Some(text),
                    run: run.map_or(0, |(r, _)| run_code(r)),
                },
            ));
        }
    }
    let list = fbb.create_vector(&list);
    let body = wire::Vehicles::create(
        &mut fbb,
        &wire::VehiclesArgs {
            rev: vehicles_rev(sim),
            vehicles: Some(list),
        },
    );
    response(fbb, wire::ResponseBody::Vehicles, body)
}
