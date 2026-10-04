//! Paths on the boundary (M1 slice F): the snapshot's paths revision and the response to a paths
//! query, the worn ground of every tile people have walked and the trails traced through it, as
//! last surveyed (ADR-0004 §4). Observers only draw them.

use civ_land::paths::TILE;
use civ_schema::flatbuffers::FlatBufferBuilder;
use civ_schema::wire;

use super::response;
use crate::Sim;

/// A number that changes whenever the paths are surveyed: monthly, and when a world is made or
/// loaded.
pub fn paths_rev(sim: &Sim) -> u64 {
    u64::from(sim.land.wear.rev())
}

/// A `Response` with the worn ground and the trails as last surveyed.
pub fn paths_response(sim: &Sim) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let wear = &sim.land.wear;
    let worn: Vec<_> = wear
        .surveyed_tiles()
        .map(|(index, cells, trail)| {
            let cells = fbb.create_vector(cells);
            let trail = fbb.create_vector(trail);
            wire::WornTile::create(
                &mut fbb,
                &wire::WornTileArgs {
                    index,
                    wear: Some(cells),
                    trail: Some(trail),
                },
            )
        })
        .collect();
    let worn = fbb.create_vector(&worn);
    let trails: Vec<_> = wear
        .trails()
        .iter()
        .map(|t| {
            let points: Vec<wire::Vec2> = t
                .points
                .iter()
                .map(|&(x, y)| wire::Vec2::new(x, y))
                .collect();
            let points = fbb.create_vector(&points);
            wire::TrailInfo::create(
                &mut fbb,
                &wire::TrailInfoArgs {
                    points: Some(points),
                    wear: t.wear,
                    length_m: t.length_m(),
                },
            )
        })
        .collect();
    let trails = fbb.create_vector(&trails);
    let body = wire::Paths::create(
        &mut fbb,
        &wire::PathsArgs {
            rev: paths_rev(sim),
            tiles_x: wear.tiles_x(),
            tile_cells: TILE,
            worn: Some(worn),
            trails: Some(trails),
        },
    );
    response(fbb, wire::ResponseBody::Paths, body)
}
