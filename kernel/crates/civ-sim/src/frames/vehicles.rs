//! The vehicles layer (wire 1.86, M8b slices CO and CP; ADR-0029 §3, §9): every boat and cart,
//! and (wire 1.100, M10b slice DV) every wagon on its line, whose it is, where it stands or who of its household has it out, how far it is made, its wear
//! and the goods aboard. It reads the world and changes nothing.

use civ_agents::person::Step;
use civ_core::PermanentId;
use civ_land::boats::Boat;
use civ_schema::flatbuffers::FlatBufferBuilder;
use civ_schema::wire;

use super::people::eldest_name;
use super::response;
use crate::Sim;

/// A vehicle's mode on the wire: 0 a boat, 1 a cart, 2 a wagon.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Boat = 0,
    Cart = 1,
    Wagon = 2,
}

const MODES: [Mode; 3] = [Mode::Boat, Mode::Cart, Mode::Wagon];

impl Mode {
    /// The land's store of vehicles of this mode.
    fn list(self, land: &civ_land::Land) -> &[Boat] {
        match self {
            Mode::Boat => &land.boats.list,
            Mode::Cart => &land.carts.list,
            Mode::Wagon => &land.wagons.list,
        }
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
    for c in &sim.people.consignments {
        mix(c.id.get());
        mix(c.boat.get());
        mix(c.stores.iter().sum::<f64>().to_bits());
    }
    h.max(1)
}

/// A vehicle in words: "Ada's household's dugout, made in year 2, a tenth worn; out with Bo, 40 kg
/// aboard", "Ada's household's handcart, being made: 12 of 40 hours' work done".
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

/// Every boat, then every cart, then every wagon, each in the order begun.
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
            let (x, y) = if (v.at as usize) < sim.map.water.len() {
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
                    mode: mode as u8,
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
