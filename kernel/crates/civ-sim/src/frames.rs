//! Boundary payloads built from a world (ADR-0001): the world and clock tables of a `Snapshot`,
//! and the `Response` bodies of raster and hydrography queries. People, settlements, trips and
//! the chronicle are in [`people`]; fields in [`fields`]; markets in [`markets`]; workshops in
//! [`firms`]; property regimes and wealth in [`wealth`].
//!
//! Rasters are served at power-of-two downsampling levels, so a client can show a whole 2048² map
//! without moving 16 MB. Level `L` has `ceil(side / 2^L)` cells per side, each summarising the
//! `2^L × 2^L` block of map cells it covers:
//!
//! | Layer | Format | Summary of a block |
//! |---|---|---|
//! | Elevation | U16; `value = raw · scale + offset` over the map's elevation range | mean |
//!
//! Elevation is the ground in use: the generated bed plus what earthworks have done to it (wire
//! 1.20; ADR-0010 §3). The other layers stay as generated.
//! | Water | U8 water class | most common class (ties go to the wetter class) |
//! | DrainageArea | F32, m² | maximum |
//! | LakeId | U32 | most common id (ties go to the higher id) |

use std::fmt;

use civ_land::earth::GroundDelta;
use civ_schema::flatbuffers::{FlatBufferBuilder, WIPOffset};
use civ_schema::wire;
use civ_world::{MapStats, WorldMap};

use crate::Sim;

pub mod buildings;
pub mod crossings;
pub mod deposits;
pub mod earthworks;
pub mod fields;
pub mod firms;
pub mod government;
pub mod knowledge;
pub mod markets;
pub mod order;
pub mod paths;
pub mod people;
pub mod standing;
pub mod water;
pub mod wealth;
pub mod weather;
pub mod word;

/// Largest region one raster query may ask for, in cells of its level.
pub const MAX_QUERY_CELLS: u64 = 1024 * 1024;
/// Coarsest downsampling level.
pub const MAX_LEVEL: u8 = 12;
/// Largest polyline simplification tolerance, metres.
pub const MAX_TOLERANCE_M: f32 = 1000.0;

/// A raster query, as decoded from `GetRaster`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RasterQuery {
    /// Which raster.
    pub layer: wire::RasterLayer,
    /// Downsampling level.
    pub level: u8,
    /// Region, in cells of the level.
    pub x0: u32,
    /// Region, in cells of the level.
    pub y0: u32,
    /// Region, in cells of the level.
    pub width: u32,
    /// Region, in cells of the level.
    pub height: u32,
}

impl RasterQuery {
    /// The query a `GetRaster` table asks.
    pub fn from_wire(q: &wire::GetRaster<'_>) -> Self {
        RasterQuery {
            layer: q.layer(),
            level: q.level(),
            x0: q.x0(),
            y0: q.y0(),
            width: q.width(),
            height: q.height(),
        }
    }
}

/// Why a query could not be answered.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QueryError(pub String);

impl fmt::Display for QueryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for QueryError {}

/// The `WorldInfo` table of a snapshot.
pub fn world_info<'a>(
    fbb: &mut FlatBufferBuilder<'a>,
    sim: &Sim,
) -> WIPOffset<wire::WorldInfo<'a>> {
    let (meta, map, stats) = (sim.meta(), sim.map(), sim.stats());
    let world_id = fbb.create_string(&meta.world_id_hex());
    let name = fbb.create_string(&meta.name);
    let preset_id = fbb.create_string(&meta.preset_id);
    let regime_id = fbb.create_string(&sim.regime().id);
    let regime_name = fbb.create_string(&sim.regime().name);
    wire::WorldInfo::create(
        fbb,
        &wire::WorldInfoArgs {
            world_id: Some(world_id),
            name: Some(name),
            seed: meta.seed,
            preset_id: Some(preset_id),
            width: map.width,
            height: map.height,
            cell_size_m: map.cell_size_m,
            sea_level_m: map.sea_level_m,
            min_elevation_m: stats.min_elevation_m,
            max_elevation_m: stats.max_elevation_m,
            generator_version: meta.generator_version,
            lakes: stats.lakes,
            reaches: stats.reaches,
            river_length_km: stats.river_length_km,
            max_discharge_m3s: stats.max_discharge_m3s,
            gentle_land_fraction: stats.gentle_land_fraction,
            land_fraction: stats.land_fraction,
            ocean_fraction: stats.ocean_fraction,
            generation: sim.generation(),
            content_changed: sim.content_changed(),
            created_unix_ms: meta.created_unix_ms,
            regime_id: Some(regime_id),
            regime_name: Some(regime_name),
        },
    )
}

/// The `Clock` table of a snapshot.
pub fn clock<'a>(fbb: &mut FlatBufferBuilder<'a>, sim: &Sim) -> WIPOffset<wire::Clock<'a>> {
    let date = sim.date();
    let season = fbb.create_string(date.season().name());
    let weather = weather::day_weather(fbb, sim);
    wire::Clock::create(
        fbb,
        &wire::ClockArgs {
            minute: sim.now().minutes(),
            year: date.year,
            month: date.month,
            day: date.day,
            hour: date.hour,
            minute_of_hour: date.minute,
            season: Some(season),
            paused: sim.paused(),
            speed: sim.speed(),
            mode: match sim.mode() {
                crate::Mode::Detailed => wire::ClockMode::Detailed,
                crate::Mode::Accelerated => wire::ClockMode::Accelerated,
            },
            weather: Some(weather),
        },
    )
}

/// Cells per side of the map at a downsampling level.
pub fn level_size(map: &WorldMap, level: u8) -> (u32, u32) {
    let block = 1u32 << level.min(MAX_LEVEL);
    (map.width.div_ceil(block), map.height.div_ceil(block))
}

/// A `Response` holding a raster region, clipped to the map. `stats` gives the elevation range
/// that elevation is quantised over, so tiles of one map share a scale; elevation adds `ground`,
/// what earthworks have done to the generated bed.
pub fn raster_response(
    map: &WorldMap,
    ground: &GroundDelta,
    stats: &MapStats,
    query: &RasterQuery,
) -> Result<Vec<u8>, QueryError> {
    if query.level > MAX_LEVEL {
        return Err(QueryError(format!(
            "level {} is coarser than the coarsest, {MAX_LEVEL}",
            query.level
        )));
    }
    let (lw, lh) = level_size(map, query.level);
    if query.width == 0 || query.height == 0 || query.x0 >= lw || query.y0 >= lh {
        return Err(QueryError(format!(
            "region {}×{} at ({}, {}) is outside the {lw}×{lh} map at level {}",
            query.width, query.height, query.x0, query.y0, query.level
        )));
    }
    let (w, h) = (
        query.width.min(lw - query.x0),
        query.height.min(lh - query.y0),
    );
    if u64::from(w) * u64::from(h) > MAX_QUERY_CELLS {
        return Err(QueryError(format!(
            "region {w}×{h} is larger than {MAX_QUERY_CELLS} cells; ask for tiles"
        )));
    }

    let block = 1usize << query.level;
    let (mw, mh) = (map.width as usize, map.height as usize);
    // The map cells under output cell (x, y) of the region.
    let cells = |x: u32, y: u32| {
        let bx = (query.x0 + x) as usize * block;
        let by = (query.y0 + y) as usize * block;
        (bx..(bx + block).min(mw), by..(by + block).min(mh))
    };
    let mut data = Vec::new();
    let (format, scale, offset) = match query.layer {
        wire::RasterLayer::Elevation => {
            let offset = stats.min_elevation_m;
            let range = stats.max_elevation_m - stats.min_elevation_m;
            let scale = if range > 0.0 { range / 65535.0 } else { 1.0 };
            data.reserve(w as usize * h as usize * 2);
            // The changes under the region, in map cells, if earthworks have touched it.
            let (cx0, cy0) = (query.x0 as usize * block, query.y0 as usize * block);
            let (cw, ch) = (
                ((query.x0 + w) as usize * block).min(mw) - cx0,
                ((query.y0 + h) as usize * block).min(mh) - cy0,
            );
            let changes = ground.region((cx0 as u32, cy0 as u32), (cw as u32, ch as u32));
            for y in 0..h {
                for x in 0..w {
                    let (xs, ys) = cells(x, y);
                    let count = (xs.len() * ys.len()) as f64;
                    let mut sum = 0.0f64;
                    for yy in ys {
                        for &z in &map.elevation[yy * mw + xs.start..yy * mw + xs.end] {
                            sum += f64::from(z);
                        }
                        if let Some(dz) = &changes {
                            let row = (yy - cy0) * cw;
                            for &d in &dz[row + xs.start - cx0..row + xs.end - cx0] {
                                sum += f64::from(d);
                            }
                        }
                    }
                    let mean = (sum / count) as f32;
                    let raw = ((mean - offset) / scale).round().clamp(0.0, 65535.0) as u16;
                    data.extend_from_slice(&raw.to_le_bytes());
                }
            }
            (wire::RasterFormat::U16, scale, offset)
        }
        wire::RasterLayer::Water => {
            data.reserve(w as usize * h as usize);
            for y in 0..h {
                for x in 0..w {
                    let (xs, ys) = cells(x, y);
                    let mut counts = [0u32; 4];
                    for yy in ys {
                        for &c in &map.water[yy * mw + xs.start..yy * mw + xs.end] {
                            counts[usize::from(c.min(3))] += 1;
                        }
                    }
                    let mut best = 0u8;
                    for class in 1..4u8 {
                        if counts[usize::from(class)] >= counts[usize::from(best)] {
                            best = class;
                        }
                    }
                    data.push(best);
                }
            }
            (wire::RasterFormat::U8, 1.0, 0.0)
        }
        wire::RasterLayer::DrainageArea => {
            data.reserve(w as usize * h as usize * 4);
            for y in 0..h {
                for x in 0..w {
                    let (xs, ys) = cells(x, y);
                    let mut max = 0.0f32;
                    for yy in ys {
                        for &a in &map.drainage_area_m2[yy * mw + xs.start..yy * mw + xs.end] {
                            max = max.max(a);
                        }
                    }
                    data.extend_from_slice(&max.to_le_bytes());
                }
            }
            (wire::RasterFormat::F32, 1.0, 0.0)
        }
        wire::RasterLayer::LakeId => {
            data.reserve(w as usize * h as usize * 4);
            let mut counts: Vec<(u32, u32)> = Vec::new();
            for y in 0..h {
                for x in 0..w {
                    let (xs, ys) = cells(x, y);
                    counts.clear();
                    for yy in ys {
                        for &id in &map.lake_id[yy * mw + xs.start..yy * mw + xs.end] {
                            match counts.iter_mut().find(|(v, _)| *v == id) {
                                Some((_, n)) => *n += 1,
                                None => counts.push((id, 1)),
                            }
                        }
                    }
                    counts.sort_unstable();
                    let mut best = (0u32, 0u32);
                    for &(id, n) in &counts {
                        if n >= best.1 {
                            best = (id, n);
                        }
                    }
                    data.extend_from_slice(&best.0.to_le_bytes());
                }
            }
            (wire::RasterFormat::U32, 1.0, 0.0)
        }
        other => {
            return Err(QueryError(format!(
                "raster layer {} does not exist",
                other.0
            )));
        }
    };

    let mut fbb = FlatBufferBuilder::with_capacity(data.len() + 256);
    let data = fbb.create_vector(&data);
    let tile = wire::RasterTile::create(
        &mut fbb,
        &wire::RasterTileArgs {
            layer: query.layer,
            level: query.level,
            x0: query.x0,
            y0: query.y0,
            width: w,
            height: h,
            format,
            scale,
            offset,
            data: Some(data),
            full_width: lw,
            full_height: lh,
        },
    );
    Ok(response(fbb, wire::ResponseBody::RasterTile, tile))
}

/// A `Response` holding the rivers and lakes. Reach polylines run downstream through cell
/// centres, in metres from the map's north-west corner, simplified to within `tolerance_m`.
pub fn hydrography_response(map: &WorldMap, tolerance_m: f32) -> Vec<u8> {
    let tolerance = if tolerance_m.is_finite() {
        tolerance_m.clamp(0.0, MAX_TOLERANCE_M)
    } else {
        0.0
    };
    let mut fbb = FlatBufferBuilder::new();
    let reaches: Vec<_> = map
        .reaches
        .iter()
        .map(|reach| {
            let points: Vec<wire::Vec2> = map
                .reach_polyline(reach, tolerance)
                .into_iter()
                .map(|(x, y)| wire::Vec2::new(x, y))
                .collect();
            let points = fbb.create_vector(&points);
            wire::Reach::create(
                &mut fbb,
                &wire::ReachArgs {
                    id: reach.id,
                    order: reach.order,
                    discharge_m3s: reach.discharge_m3s,
                    width_m: reach.width_m,
                    downstream: reach
                        .downstream
                        .and_then(|d| i32::try_from(d).ok())
                        .unwrap_or(-1),
                    points: Some(points),
                },
            )
        })
        .collect();
    let reaches = fbb.create_vector(&reaches);
    let lakes: Vec<_> = map
        .lakes
        .iter()
        .map(|lake| {
            let centroid = wire::Vec2::new(lake.centroid_m.0, lake.centroid_m.1);
            wire::LakeInfo::create(
                &mut fbb,
                &wire::LakeInfoArgs {
                    id: lake.id,
                    level_m: lake.level_m,
                    area_m2: lake.area_m2 as f32,
                    closed: lake.closed,
                    centroid: Some(&centroid),
                },
            )
        })
        .collect();
    let lakes = fbb.create_vector(&lakes);
    let body = wire::Hydrography::create(
        &mut fbb,
        &wire::HydrographyArgs {
            reaches: Some(reaches),
            lakes: Some(lakes),
        },
    );
    response(fbb, wire::ResponseBody::Hydrography, body)
}

/// Finishes a `Response` around a body table.
fn response<T>(
    mut fbb: FlatBufferBuilder<'_>,
    body_type: wire::ResponseBody,
    body: WIPOffset<T>,
) -> Vec<u8> {
    let root = wire::Response::create(
        &mut fbb,
        &wire::ResponseArgs {
            body_type,
            body: Some(body.as_union_value()),
        },
    );
    fbb.finish(root, None);
    fbb.finished_data().to_vec()
}
