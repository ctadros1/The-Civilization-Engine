//! The ways layer (wire 1.84, M8b slice CM; ADR-0029 §1, §9): every way people have worked on,
//! its stretches by their widths, surface, condition and keeper, and its derived rank.

use civ_land::gas::{Cause, Main, StreetLamp};
use civ_land::ways::{Keeper, Way};
use civ_schema::flatbuffers::FlatBufferBuilder;
use civ_schema::wire;
use civ_world::nav::simplify_indices;

use super::people::eldest_name;
use super::response;
use crate::Sim;

/// The ways' revision: the land's, which changes whenever a way is begun or changed and at each
/// survey while there are any, mixed with what each line's record shows (its stations,
/// timetable, fare and tallies, M10b slice DX) and what the gas mains and street lamps show (how
/// much of each main is laid, each connection's last evening and whether its burners are lit,
/// each lamp and whether it burns now, M10b slice DY); 0 when the world has none of them.
pub fn ways_rev(sim: &Sim) -> u64 {
    let gas = &sim.land.gas;
    let no_gas = gas.mains.is_empty() && gas.lamps.is_empty();
    if sim.land.ways.list.is_empty() && no_gas {
        return 0;
    }
    let base = u64::from(sim.land.ways.revision());
    if sim.land.runs.lines.is_empty() && no_gas {
        return base;
    }
    // FNV-1a over the lines' records, which the ways' revision does not follow.
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut mix = |v: u64| {
        for b in v.to_le_bytes() {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
    };
    mix(base);
    for l in &sim.land.runs.lines {
        mix(l.way.get());
        mix(u64::from(l.stations));
        mix(u64::from(l.runs_a_day));
        mix(u64::from(l.fare_kg.to_bits()));
        mix(u64::from(l.riders));
        mix(u64::from(l.freight_kg.to_bits()));
        mix(l.freight_since as u64);
    }
    let now = sim.now().minutes();
    for m in &gas.mains {
        mix(m.id.get());
        mix(u64::from(m.laid_m.to_bits()));
        mix(u64::from(m.broken));
    }
    for c in &gas.connections {
        mix(c.building.get());
        mix(c.evening as u64);
        mix(u64::from(c.cause.code()));
        mix(u64::from(c.cut_off));
        mix(u64::from(c.lit_from <= now && now < c.lit_until));
    }
    for l in &gas.lamps {
        mix(u64::from(l.cell));
        mix(u64::from(now < l.lit_until));
    }
    h.max(1)
}

/// "22:40", the time of day at minute `minute`.
pub(super) fn clock_at(minute: i64) -> String {
    let m = minute.rem_euclid(civ_core::time::MINUTES_PER_DAY);
    format!("{:02}:{:02}", m / 60, m % 60)
}

/// The good the piped burners burn, lower case ("coal gas"), or "gas".
pub(super) fn gas_name(sim: &Sim) -> String {
    let goods = &sim.rules.catalog.goods;
    goods
        .iter()
        .find_map(|d| d.light.filter(|l| l.piped).and_then(|l| l.fuel))
        .and_then(|g| goods.get(g))
        .map_or_else(|| "gas".to_owned(), |d| d.name.to_lowercase())
}

/// A gas main in words (wire 1.103, M10b slice DY, fifth part): "Gas main of 120 m from Ada's
/// household's gasworks, 40% laid." Once laid: "Gas main of 120 m from Ada's household's gasworks,
/// laid, carrying up to 6.0 kg of coal gas an hour; 2 homes connected, 1 lit now."
fn main_words(sim: &Sim, m: &Main, homes: u32, lit: u32) -> String {
    let mut out = format!(
        "Gas main of {:.0} m from {}'s gasworks",
        m.length_m,
        keeper(sim, Keeper::Household(m.works)).trim_end_matches("'s household")
    );
    if !m.whole() {
        let laid = if m.length_m > 0.0 {
            (m.laid_m / m.length_m).clamp(0.0, 1.0)
        } else {
            0.0
        };
        out.push_str(&format!(", {:.0}% laid.", (laid * 100.0).floor()));
        return out;
    }
    out.push_str(&format!(
        ", laid, carrying up to {:.1} kg of {} an hour",
        m.capacity_kg_h,
        gas_name(sim)
    ));
    if m.broken {
        out.push_str("; broken");
    }
    let homes = match homes {
        0 => "no homes connected".to_owned(),
        1 => "1 home connected".to_owned(),
        n => format!("{n} homes connected"),
    };
    out.push_str(&format!("; {homes}"));
    if lit > 0 {
        out.push_str(&format!(", {lit} lit now"));
    }
    out.push('.');
    out
}

/// A street lamp in words (wire 1.103): "Street lamp of Ashford, set by its law to light the
/// streets; burning until 05:42." Or "; dark now", or "; not yet lit".
fn lamp_words(sim: &Sim, l: &StreetLamp, now: i64) -> String {
    let whose = keeper(sim, Keeper::Polity(l.polity));
    let state = if now < l.lit_until {
        format!("burning until {}", clock_at(l.lit_until))
    } else if l.lit_until < 0 {
        "not yet lit".to_owned()
    } else {
        "dark now".to_owned()
    };
    format!("Street lamp of {whose}, set by its law to light the streets; {state}.")
}

/// What a line's record shows (M10b slice DX): whether its ends are stations, its runs a day, its
/// fare, kg, and since its last review its rides and the goods brought a day, kg.
fn line_facts(sim: &Sim, way: &Way) -> (bool, u8, f32, u32, f32) {
    let Some(l) = sim.land.runs.line(way.id) else {
        return (false, 0, 0.0, 0, 0.0);
    };
    let days = (sim.now().day_index() - l.freight_since).max(1) as f32;
    (
        l.stations,
        l.runs_a_day,
        l.fare_kg,
        l.riders,
        l.freight_kg / days,
    )
}

/// A line's stations, timetable and terms in words (M10b slice DX): " Stations at both ends; 2
/// trains a day; a ride costs 0.6 kg of coal; 14 rides and 380 kg brought a day since its
/// review." Nothing for a line whose stations are not open.
fn line_words(
    sim: &Sim,
    (stations, runs, fare, riders, freight): (bool, u8, f32, u32, f32),
) -> String {
    if !stations {
        return String::new();
    }
    let fuel = sim
        .rules
        .catalog
        .vehicles
        .iter()
        .find_map(|v| v.engine)
        .and_then(|e| sim.rules.catalog.goods.get(e.fuel))
        .map_or_else(|| "fuel".to_owned(), |g| g.name.to_lowercase());
    let trains = if runs == 1 {
        "1 train a day".to_owned()
    } else {
        format!("{runs} trains a day")
    };
    let fare = if fare > 0.0 {
        format!("a ride costs {fare:.1} kg of {fuel}")
    } else {
        "no fare posted yet".to_owned()
    };
    let rides = if riders == 1 {
        "1 ride".to_owned()
    } else {
        format!("{riders} rides")
    };
    format!(
        " Stations at both ends; {trains}; {fare}; {rides} and {freight:.0} kg brought a day since its review."
    )
}

/// Who keeps a stretch, in words: "Ada's household", "Ashford", "" for nobody.
pub(super) fn keeper(sim: &Sim, k: Keeper) -> String {
    match k {
        Keeper::Nobody => String::new(),
        Keeper::Household(h) => eldest_name(sim, h)
            .map_or_else(|| "a household".to_owned(), |n| format!("{n}'s household")),
        Keeper::Polity(p) => sim
            .people
            .polities
            .iter()
            .find(|x| x.id == p)
            .and_then(|x| sim.land.settlements.iter().find(|s| s.id == x.settlement))
            .map_or_else(|| "a polity".to_owned(), |s| s.name.clone()),
    }
}

/// A line of track's gauge, metres, its bed's steepest grade either way, rise over run, and the
/// share of it laid, 0-1; `None` for a way with no track. `width` cells to a map row, `cell_m`
/// metres a cell.
fn track_facts(way: &Way, width: u32, cell_m: f64) -> Option<(f32, f32, f32)> {
    let t = way.track.as_ref()?;
    let cells: Vec<u32> = way
        .segments
        .iter()
        .flat_map(|s| s.cells.iter().copied())
        .collect();
    let runs = civ_land::rail::runs(&cells, width, cell_m);
    let ruling = civ_land::rail::steepest_rise(&t.bed, &runs, false)
        .max(civ_land::rail::steepest_rise(&t.bed, &runs, true)) as f32;
    let from = civ_agents::population::LAID_FROM_M;
    let (done, whole) = way.segments.iter().fold((0.0f32, 0.0f32), |(d, w), s| {
        let n = s.cells.len() as f32;
        (
            d + n * (s.built_m - from).max(0.0),
            w + n * (s.corridor_m - from).max(0.0),
        )
    });
    let laid = if whole > 0.0 {
        (done / whole).clamp(0.0, 1.0)
    } else {
        1.0
    };
    Some((t.gauge_m, ruling, laid))
}

/// A way in words: "Street of 240 m, 3.0 m wide at most, about 12 trips a day; begun in year 2."
/// A line of track: "Wagonway of 240 m, gauge 1.0 m, graded to 1 in 120 at its steepest, a third
/// laid; begun in year 2."
fn words(way: &Way, day: i64, length_m: f32, track: Option<(f32, f32, f32)>) -> String {
    if let Some((gauge, ruling, laid)) = track {
        let mut out = format!("Wagonway of {length_m:.0} m, gauge {gauge:.1} m");
        if ruling > 1e-4 {
            out.push_str(&format!(
                ", graded to 1 in {:.0} at its steepest",
                1.0 / ruling
            ));
        } else {
            out.push_str(", level");
        }
        if laid >= 1.0 - 1e-3 {
            out.push_str(", laid");
        } else {
            out.push_str(&format!(", {:.0}% laid", (laid * 100.0).floor()));
        }
        out.push_str(&format!(
            "; begun in year {}.",
            super::crossings::year_of(way.begun)
        ));
        return out;
    }
    let rank = way.rank.name();
    let mut out = format!(
        "{}{} of {:.0} m, {:.1} m wide at most",
        rank[..1].to_uppercase(),
        &rank[1..],
        length_m,
        way.usable_m()
    );
    let trips = way.trips_a_day(day);
    if trips >= 0.5 {
        out.push_str(&format!(", about {trips:.0} trips a day"));
    }
    out.push_str(&format!(
        "; begun in year {}.",
        super::crossings::year_of(way.begun)
    ));
    out
}

/// Every way, in the order begun.
pub fn ways_response(sim: &Sim) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let map = &sim.map;
    let day = sim.now().day_index();
    let paths = &sim.rules.land.paths;
    let mut list = Vec::with_capacity(sim.land.ways.list.len());
    for way in &sim.land.ways.list {
        let track = track_facts(way, map.width, f64::from(map.cell_size_m));
        let mut length_m = 0.0f32;
        let mut segments = Vec::with_capacity(way.segments.len());
        for s in &way.segments {
            let centres: Vec<(f32, f32)> = s
                .cells
                .iter()
                .map(|&c| civ_agents::population::cell_centre(map, c as usize))
                .collect();
            length_m += centres
                .windows(2)
                .map(|w| (w[1].0 - w[0].0).hypot(w[1].1 - w[0].1))
                .sum::<f32>();
            let points: Vec<f32> = simplify_indices(&centres, map.cell_size_m)
                .into_iter()
                .flat_map(|i| [centres[i].0, centres[i].1])
                .collect();
            let points = fbb.create_vector(&points);
            let surface = fbb.create_string(s.surface.name());
            let keeper = fbb.create_string(&keeper(sim, s.keeper));
            segments.push(wire::WaySegmentInfo::create(
                &mut fbb,
                &wire::WaySegmentInfoArgs {
                    points: Some(points),
                    corridor_m: s.corridor_m,
                    built_m: s.built_m,
                    usable_m: s.usable_m,
                    surface: Some(surface),
                    condition: s.condition(day, paths),
                    grade: s.grade(&map.elevation, map.width, map.cell_size_m),
                    keeper: Some(keeper),
                },
            ));
        }
        let segments = fbb.create_vector(&segments);
        let rank_name = fbb.create_string(way.rank.name());
        let line = line_facts(sim, way);
        let mut said = words(way, day, length_m, track);
        said.push_str(&line_words(sim, line));
        let text = fbb.create_string(&said);
        let (gauge_m, ruling, laid) = track.unwrap_or((0.0, 0.0, 1.0));
        let (stations, runs_a_day, fare_kg, riders, freight_kg_day) = line;
        list.push(wire::WayInfo::create(
            &mut fbb,
            &wire::WayInfoArgs {
                id: way.id.get(),
                rank: way.rank.code(),
                rank_name: Some(rank_name),
                modes: way.modes,
                trips_a_day: way.trips_a_day(day),
                segments: Some(segments),
                words: Some(text),
                gauge_m,
                ruling,
                laid,
                stations,
                runs_a_day,
                fare_kg,
                riders,
                freight_kg_day,
            },
        ));
    }
    let list = fbb.create_vector(&list);
    // The gas mains and street lamps (M10b slice DY, fifth part).
    let gas = &sim.land.gas;
    let now = sim.now().minutes();
    let mut mains = Vec::with_capacity(gas.mains.len());
    for m in &gas.mains {
        let centres: Vec<(f32, f32)> = m
            .cells
            .iter()
            .map(|&c| civ_agents::population::cell_centre(map, c as usize))
            .collect();
        let points: Vec<f32> = simplify_indices(&centres, map.cell_size_m)
            .into_iter()
            .flat_map(|i| [centres[i].0, centres[i].1])
            .collect();
        let points = fbb.create_vector(&points);
        let on: Vec<_> = gas.connections.iter().filter(|c| c.main == m.id).collect();
        let homes = on.len() as u32;
        let lit = on
            .iter()
            .filter(|c| c.cause == Cause::Served && c.lit_from <= now && now < c.lit_until)
            .count() as u32;
        let text = fbb.create_string(&main_words(sim, m, homes, lit));
        let laid = if m.length_m > 0.0 {
            (m.laid_m / m.length_m).clamp(0.0, 1.0)
        } else {
            1.0
        };
        mains.push(wire::GasMainInfo::create(
            &mut fbb,
            &wire::GasMainInfoArgs {
                id: m.id.get(),
                works: m.works.get(),
                points: Some(points),
                length_m: m.length_m,
                laid,
                capacity_kg_h: m.capacity_kg_h,
                broken: m.broken,
                homes,
                lit,
                words: Some(text),
            },
        ));
    }
    let mains = fbb.create_vector(&mains);
    let mut lamps = Vec::with_capacity(gas.lamps.len());
    for l in &gas.lamps {
        let (x, y) = civ_agents::population::cell_centre(map, l.cell as usize);
        let text = fbb.create_string(&lamp_words(sim, l, now));
        lamps.push(wire::StreetLampInfo::create(
            &mut fbb,
            &wire::StreetLampInfoArgs {
                x,
                y,
                lit: now < l.lit_until,
                words: Some(text),
            },
        ));
    }
    let lamps = fbb.create_vector(&lamps);
    let body = wire::Ways::create(
        &mut fbb,
        &wire::WaysArgs {
            rev: ways_rev(sim),
            ways: Some(list),
            mains: Some(mains),
            lamps: Some(lamps),
        },
    );
    response(fbb, wire::ResponseBody::Ways, body)
}
