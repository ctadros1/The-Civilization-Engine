//! The water panel (wire 1.61, M6a slice AY; ADR-0021 §1-§3): the wells, the springs flowing
//! today, where people drew at a river's or lake's edge today, and, for the inspector, where a
//! household drew its water today, in words.

use std::collections::BTreeMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use civ_agents::population::cell_centre;
use civ_agents::uses::Place;
use civ_core::PermanentId;
use civ_land::wells::{Well, WellState};
use civ_schema::flatbuffers::FlatBufferBuilder;
use civ_schema::wire;

use super::crossings::year_of;
use super::people::eldest_name;
use super::response;
use crate::Sim;

/// The water's revision: changes whenever a well is begun, dug deeper, opens, rots, is relined,
/// given up or falls in, and with each day lived while the world has wells or ground where a
/// spring may rise; 0 when it has neither.
pub fn water_rev(sim: &Sim) -> u64 {
    let wells = &sim.land.wells.list;
    if wells.is_empty() && !sim.land.water.aquifer.seep.iter().any(Option::is_some) {
        return 0;
    }
    let mut hasher = DefaultHasher::new();
    sim.now().day_index().hash(&mut hasher);
    for w in wells {
        let (state, at) = w.state.code();
        (
            w.id.get(),
            state,
            at.minutes(),
            w.depth_m.to_bits(),
            w.target_m.to_bits(),
            w.work_h.to_bits(),
            w.loss.to_bits(),
            w.mend_h.to_bits(),
            w.household.get(),
        )
            .hash(&mut hasher);
    }
    hasher.finish() | 1
}

/// The water standing in an open well's shaft now and how far below its mouth its surface is,
/// metres; `(0, 0)` for a well not open.
fn standing(sim: &Sim, w: &Well) -> (f64, f64) {
    let Some(def) = sim.rules.catalog.wells.get(w.system) else {
        return (0.0, 0.0);
    };
    if !w.is_open() {
        return (0.0, 0.0);
    }
    let (head, rate) = sim.land.ground_water(
        w.cell as usize,
        sim.map.width,
        def.radius_m,
        def.influence_m,
    );
    let level = w.level_now(sim.now(), head, rate);
    (
        (level - w.floor_m()).max(0.0),
        (f64::from(w.ground_m) - level).max(0.0),
    )
}

/// A share in words: "a tenth", "a quarter", "40%".
fn share(x: f32) -> String {
    match (f64::from(x) * 100.0).round() as i64 {
        10 => "a tenth".to_owned(),
        25 => "a quarter".to_owned(),
        50 => "half".to_owned(),
        n => format!("{n}%"),
    }
}

/// A well in words: "Ada's household's timber-lined well, being dug: 2.0 of 4.5 m", "..., open
/// since year 2: 4.1 m deep, 1.8 m of water standing 2.3 m down; rot has taken a tenth of its
/// lining", "..., given up dry at 10.0 m in year 3", "..., fell in in year 9".
fn words(sim: &Sim, w: &Well, (water_m, below_m): (f64, f64)) -> String {
    let name = sim
        .rules
        .catalog
        .wells
        .get(w.system)
        .map_or_else(|| "well".to_owned(), |d| d.name.to_lowercase());
    let whose = eldest_name(sim, w.household)
        .map_or_else(|| "a".to_owned(), |n| format!("{n}'s household's"));
    let what = format!("{whose} {name}");
    match w.state {
        WellState::Digging => format!("{what}, being dug: {:.1} of {:.1} m", w.depth_m, w.target_m),
        WellState::Open { since } => {
            let rot = if w.loss < 0.05 {
                "its lining is sound".to_owned()
            } else {
                format!("rot has taken {} of its lining", share(w.loss))
            };
            let mend = if w.mend_h > 0.0 {
                format!("; being relined, {:.0} hours of work left", w.mend_h)
            } else {
                String::new()
            };
            format!(
                "{what}, open since year {}: {:.1} m deep, {water_m:.1} m of water standing \
                 {below_m:.1} m down; {rot}{mend}",
                year_of(since.day_index()),
                w.depth_m
            )
        }
        WellState::GivenUp { on } => format!(
            "{what}, given up dry at {:.1} m in year {}",
            w.depth_m,
            year_of(on.day_index())
        ),
        WellState::FellIn { on } => {
            format!("{what}, fell in in year {}", year_of(on.day_index()))
        }
    }
}

/// Whether terrain cell `cell` is where a spring may rise: the lowest ground of a patch off the
/// water.
fn spring_cell(sim: &Sim, cell: u32) -> bool {
    let aq = &sim.land.water.aquifer;
    let p = sim.land.patches.of_cell(cell as usize, sim.map.width);
    aq.stage.get(p).is_some_and(Option::is_none)
        && aq
            .seep
            .get(p)
            .copied()
            .flatten()
            .is_some_and(|(c, _)| c == cell)
}

/// Every well, in the order begun; every spring flowing today with what was drawn there today;
/// the river and lake edges drawn at today; and today's flow.
pub fn water_response(sim: &Sim) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let map = &sim.map;
    let mut wells = Vec::with_capacity(sim.land.wells.list.len());
    for w in &sim.land.wells.list {
        let def = sim.rules.catalog.wells.get(w.system);
        let water = standing(sim, w);
        let text = fbb.create_string(&words(sim, w, water));
        let system = fbb.create_string(def.map_or("", |d| d.name.as_str()));
        let litres = water.0 * def.map_or(0.0, |d| d.area_m2()) * 1000.0;
        let fouled = fbb.create_string(&super::sickness::fouled_words(sim, w.id, litres));
        let (x, y) = w.rect.centre_m();
        let state = w.state.code().0;
        wells.push(wire::WellInfo::create(
            &mut fbb,
            &wire::WellInfoArgs {
                id: w.id.get(),
                system: Some(system),
                x,
                y,
                radius_m: def.map_or(0.0, |d| d.dig_radius_m as f32),
                state,
                depth_m: w.depth_m,
                target_m: w.target_m,
                water_m: water.0 as f32,
                below_m: water.1 as f32,
                quality: w.quality,
                loss: w.loss,
                household: w.household.get(),
                words: Some(text),
                fouled: Some(fouled),
            },
        ));
    }
    let wells = fbb.create_vector(&wells);
    let day = sim.now().day_index();
    let params = &sim.rules.land.water;
    let springs: Vec<_> = (0..sim.land.water.aquifer.len())
        .filter_map(|p| sim.land.water.spring_at(params, p))
        .map(|s| {
            let (x, y) = cell_centre(map, s.cell as usize);
            wire::SpringInfo::new(
                x,
                y,
                s.flow_m3_day as f32,
                sim.people.spring_draws.drawn(day, s.patch) as f32,
            )
        })
        .collect();
    let springs = fbb.create_vector(&springs);
    let wells_at: Vec<u32> = sim.land.wells.list.iter().map(|w| w.cell).collect();
    let mut banks: BTreeMap<u32, u32> = BTreeMap::new();
    for w in &sim.people.uses.today {
        if let Place::Source(cell) = w.place
            && w.day == day
            && !wells_at.contains(&cell)
            && !spring_cell(sim, cell)
        {
            *banks.entry(cell).or_default() += 1;
        }
    }
    let banks: Vec<_> = banks
        .into_iter()
        .map(|(cell, trips)| {
            let (x, y) = cell_centre(map, cell as usize);
            wire::BankDraw::new(x, y, trips)
        })
        .collect();
    let banks = fbb.create_vector(&banks);
    let body = wire::Water::create(
        &mut fbb,
        &wire::WaterArgs {
            rev: water_rev(sim),
            wells: Some(wells),
            springs: Some(springs),
            flow: sim.land.flow_factor(&sim.rules.land) as f32,
            banks: Some(banks),
        },
    );
    response(fbb, wire::ResponseBody::Water, body)
}

/// A count of times in words.
fn times(n: u32) -> String {
    match n {
        1 => "once".to_owned(),
        2 => "twice".to_owned(),
        n => format!("{n} times"),
    }
}

/// Where household `household` went for water today, in words (ADR-0021 §1): "Their household
/// went for water 6 times today: 4 times to its own well, twice to the water's edge; each of them
/// uses 20 L a day, and it holds 85 L"; "Their household has not gone for water yet today; ...".
pub fn water_words(sim: &Sim, household: PermanentId) -> String {
    let Some(x) = sim.people.household(household) else {
        return String::new();
    };
    let day = sim.now().day_index();
    let mut whither: Vec<(String, u32)> = Vec::new();
    for w in &sim.people.uses.today {
        let Place::Source(cell) = w.place else {
            continue;
        };
        if w.household != household || w.day != day {
            continue;
        }
        let at = sim.land.wells.list.iter().filter(|v| v.cell == cell);
        let mut own = None;
        let mut other = None;
        for v in at {
            if v.household == household {
                own = Some(v);
            } else if other.is_none() {
                other = Some(v);
            }
        }
        let place = match (own, other) {
            (Some(_), _) => "its own well".to_owned(),
            (None, Some(v)) => eldest_name(sim, v.household).map_or_else(
                || "a well".to_owned(),
                |n| format!("{n}'s household's well"),
            ),
            (None, None) if spring_cell(sim, cell) => "a spring".to_owned(),
            (None, None) => "the water's edge".to_owned(),
        };
        match whither.iter_mut().find(|(p, _)| *p == place) {
            Some((_, n)) => *n += 1,
            None => whither.push((place, 1)),
        }
    }
    let use_l = x.water_use(&sim.rules.people.household);
    let held = x.water_at_time(sim.now(), x.members.len() as f64 * use_l);
    let tail = format!("each of them uses {use_l:.0} L a day, and it holds {held:.0} L");
    if whither.is_empty() {
        return format!("Their household has not gone for water yet today; {tail}");
    }
    whither.sort_by_key(|w| std::cmp::Reverse(w.1));
    let total: u32 = whither.iter().map(|w| w.1).sum();
    let parts = if whither.len() == 1 {
        format!("to {}", whither[0].0)
    } else {
        whither
            .iter()
            .map(|(place, n)| format!("{} to {place}", times(*n)))
            .collect::<Vec<_>>()
            .join(", ")
    };
    format!(
        "Their household went for water {} today: {parts}; {tail}",
        times(total)
    )
}
