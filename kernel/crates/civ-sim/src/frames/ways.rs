//! The ways layer (wire 1.84, M8b slice CM; ADR-0029 §1, §9): every way people have worked on,
//! its stretches by their widths, surface, condition and keeper, and its derived rank.

use civ_land::ways::{Keeper, Way};
use civ_schema::flatbuffers::FlatBufferBuilder;
use civ_schema::wire;
use civ_world::nav::simplify_indices;

use super::people::eldest_name;
use super::response;
use crate::Sim;

/// The ways' revision: the land's, which changes whenever a way is begun or changed and at each
/// survey while there are any; 0 when the world has none.
pub fn ways_rev(sim: &Sim) -> u64 {
    if sim.land.ways.list.is_empty() {
        0
    } else {
        u64::from(sim.land.ways.revision())
    }
}

/// Who keeps a stretch, in words: "Ada's household", "Ashford", "" for nobody.
fn keeper(sim: &Sim, k: Keeper) -> String {
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

/// A way in words: "Street of 240 m, 3.0 m wide at most, about 12 trips a day; begun in year 2."
fn words(way: &Way, day: i64, length_m: f32) -> String {
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
        let text = fbb.create_string(&words(way, day, length_m));
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
            },
        ));
    }
    let list = fbb.create_vector(&list);
    let body = wire::Ways::create(
        &mut fbb,
        &wire::WaysArgs {
            rev: ways_rev(sim),
            ways: Some(list),
        },
    );
    response(fbb, wire::ResponseBody::Ways, body)
}
