//! The earthworks panel (wire 1.20, M3b slice Q; ADR-0010 §2-3): every earthwork, what it is in
//! words, and the tiles of ground earthworks have changed, so a client knows what to read again.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use civ_land::Building;
use civ_land::earth::{DELTA_TILE, EarthKind, Earthwork};
use civ_schema::flatbuffers::FlatBufferBuilder;
use civ_schema::wire;

use super::people::{eldest_name, program_name};
use super::response;
use crate::Sim;

/// The building standing on earthwork `w`'s plot, if any.
fn building_on<'a>(sim: &'a Sim, w: &Earthwork) -> Option<&'a Building> {
    let plot = w.plot?;
    sim.land.buildings.iter().find(|b| b.plot == plot)
}

/// The earthworks' revision: changes whenever one is begun or advanced (a platform's share done,
/// a pit's or heap's earth), or the building on a levelled plot changes; 0 when the world has
/// none.
pub fn earthworks_rev(sim: &Sim) -> u64 {
    let works = &sim.land.earthworks;
    if works.is_empty() {
        return 0;
    }
    let mut hasher = DefaultHasher::new();
    for w in works {
        let building = building_on(sim, w).map_or(0, |b| b.id.get());
        (w.id.get(), w.done.to_bits(), w.cut_m3.to_bits(), building).hash(&mut hasher);
    }
    hasher.finish() | 1
}

/// A volume in words: "0.8 m³", "6.4 m³", "23 m³".
fn volume(m3: f64) -> String {
    if m3 < 10.0 {
        format!("{m3:.1} m³")
    } else {
        format!("{m3:.0} m³")
    }
}

/// What the pit or heap `w` is dug for, and what it is called, in words: ("clay", "pit"),
/// ("stone", "quarry"), from its deposit's good and the land profile's rule for it.
fn dug_for(sim: &Sim, w: &Earthwork) -> (String, String) {
    let body = w
        .deposit
        .and_then(|id| sim.land.deposits.iter().find(|d| d.id == id))
        .map(|d| usize::from(d.body.good));
    let good = body
        .and_then(|g| sim.rules.catalog.goods.get(g))
        .map_or_else(|| "earth".to_owned(), |g| g.name.to_lowercase());
    (good, working(sim, body))
}

/// What a working of a body of good `good` is called: the land profile's word for it, "pit" when
/// it gives none.
pub(crate) fn working(sim: &Sim, good: Option<usize>) -> String {
    good.and_then(|g| sim.rules.land.deposits.iter().find(|r| r.good == g))
        .map_or_else(|| "pit".to_owned(), |r| r.working.clone())
}

/// Building `b` as its owners call it: "Ada's hut", "a hut".
fn whose_building(sim: &Sim, b: &Building) -> String {
    let program = program_name(sim, &b.spec.program);
    match eldest_name(sim, b.household) {
        Some(name) => format!("{name}'s {program}"),
        None => format!("a {program}"),
    }
}

/// An earthwork in words: "the plot of Ada's hut, being levelled: 40% of 6.4 m³ cut and filled",
/// "Ada's household's clay pit, 1.2 m deep", "the spoil heap of a stone quarry, 3.1 m³", "the daub
/// pit of Ada's hut, 0.6 m deep".
fn words(sim: &Sim, w: &Earthwork, building: Option<&Building>) -> String {
    match w.kind {
        EarthKind::Platform => {}
        EarthKind::Pit if w.deposit.is_none() => {
            let of = building.map_or_else(
                || "a daub pit".to_owned(),
                |b| format!("the daub pit of {}", whose_building(sim, b)),
            );
            return format!("{of}, {:.1} m deep", w.depth_m());
        }
        EarthKind::Pit => {
            let whose = eldest_name(sim, w.household)
                .map_or_else(|| "a".to_owned(), |name| format!("{name}'s household's"));
            let (good, working) = dug_for(sim, w);
            return format!("{whose} {good} {working}, {:.1} m deep", w.depth_m());
        }
        EarthKind::Spoil => {
            let (good, working) = dug_for(sim, w);
            return format!(
                "the spoil heap of a {good} {working}, {}",
                volume(f64::from(w.cut_m3))
            );
        }
    }
    let what = building.map_or_else(
        || "a plot".to_owned(),
        |b| format!("the plot of {}", whose_building(sim, b)),
    );
    let earth = volume(f64::from(w.cut_m3));
    let done = f64::from(w.done);
    if done >= 1.0 {
        format!("{what}, levelled: {earth} cut and filled")
    } else if done <= 0.0 {
        format!("{what}, to be levelled: {earth} to cut and fill")
    } else {
        let percent = (done * 100.0).floor();
        format!("{what}, being levelled: {percent}% of {earth} cut and filled")
    }
}

/// What an earthwork is, as the wire numbers it.
fn kind(k: EarthKind) -> u8 {
    match k {
        EarthKind::Platform => 0,
        EarthKind::Pit => 1,
        EarthKind::Spoil => 2,
    }
}

/// Every earthwork, in the order begun, and every tile of ground they have changed.
pub fn earthworks_response(sim: &Sim) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let mut list = Vec::with_capacity(sim.land.earthworks.len());
    for w in &sim.land.earthworks {
        let building = building_on(sim, w);
        let text = fbb.create_string(&words(sim, w, building));
        list.push(wire::EarthworkInfo::create(
            &mut fbb,
            &wire::EarthworkInfoArgs {
                id: w.id.get(),
                kind: kind(w.kind),
                x: w.rect.x as f32 / 100.0,
                y: w.rect.y as f32 / 100.0,
                w: w.rect.w as f32 / 100.0,
                h: w.rect.h as f32 / 100.0,
                level_m: w.level_m() as f32,
                side_run: w.side_run() as f32,
                cut_m3: w.cut_m3,
                done: w.done,
                household: w.household.get(),
                plot: w.plot.map_or(0, |p| p.get()),
                building: building.map_or(0, |b| b.id.get()),
                words: Some(text),
                deposit: w.deposit.map_or(0, |d| d.get()),
            },
        ));
    }
    let list = fbb.create_vector(&list);
    let tiles: Vec<wire::GroundTileRev> = sim
        .land
        .ground
        .tiles()
        .map(|(index, t)| wire::GroundTileRev::new(index, t.rev))
        .collect();
    let tiles = fbb.create_vector(&tiles);
    let body = wire::Earthworks::create(
        &mut fbb,
        &wire::EarthworksArgs {
            rev: earthworks_rev(sim),
            works: Some(list),
            tile_cells: DELTA_TILE,
            tiles_x: sim.land.ground.tiles_x(),
            tiles: Some(tiles),
        },
    );
    response(fbb, wire::ResponseBody::Earthworks, body)
}
