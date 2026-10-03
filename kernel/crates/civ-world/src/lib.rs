//! World generation: terrain, lakes and rivers from a seed (plan §6, research 03-01 and 03-02).
//!
//! The pipeline (research 03-01 §5.2):
//!
//! 1. **Watershed context.** A coarse grid (8× the simulation cell) covering about twice the map's
//!    extent evolves under rock uplift, implicit stream-power incision, hillslope diffusion and
//!    talus relaxation. Mountains and valleys form together with their drainage, and rivers may
//!    enter the map from outside it.
//! 2. **Refinement.** The map's part of the context is refined three times, to 4×, 2× and 1× the
//!    simulation cell. Each level adds relief-scaled detail and runs a few erosion passes, with
//!    upstream area from the context injected where rivers enter.
//! 3. **Water.**
//!    - Ocean is connected ground below sea level.
//!    - Depressions are filled when they are artifacts, or hold lakes whose level comes from the
//!      water balance; closed lakes are terminal.
//!    - Every cell drains along a D8 receiver to the map edge, the ocean or a sink.
//! 4. **Rivers.** Reaches start where drainage area passes a threshold, with Strahler order,
//!    mean annual discharge `Q = A·runoff` and hydraulic-geometry width.
//!
//! A seed and a request always produce the same world on a given build of this crate. Different
//! platforms may differ in the last bits wherever `powf` is used (only when the area exponent
//! is not 0.5). Changing anything that alters what a seed produces must bump
//! [`GENERATOR_VERSION`].
//!
//! Generation is a pure function of its inputs. The kernel never edits terrain afterwards except
//! through people's earthworks (plan §5.1), which arrive in M3.

#![forbid(unsafe_code)]

use std::sync::atomic::{AtomicBool, Ordering};

mod flood;
mod floodplain;
pub mod grid;
mod hydro;
mod lem;
pub mod noise;
pub mod params;
mod refine;
pub mod rivers;
mod routing;
mod validate;

pub use params::TerrainParams;
pub use validate::validate;

/// Version of the generation algorithm. A world records it; bump it whenever a seed would produce
/// a different world.
pub const GENERATOR_VERSION: u32 = 1;
/// Number of times the coarse context is refined; the coarse cell is `2^REFINEMENT_LEVELS` times
/// the simulation cell.
pub const REFINEMENT_LEVELS: u32 = 3;
/// Smallest supported map side, in cells.
pub const MIN_CELLS: u32 = 64;
/// Largest supported map side, in cells.
pub const MAX_CELLS: u32 = 4096;

/// Receiver code: drains off the map (an edge cell or the ocean).
pub const RECEIVER_OUTLET: u8 = 8;
/// Receiver code: terminal, the floor or water of a closed basin.
pub const RECEIVER_SINK: u8 = 9;

/// Water class: dry land.
pub const WATER_LAND: u8 = 0;
/// Water class: a river channel.
pub const WATER_RIVER: u8 = 1;
/// Water class: a lake.
pub const WATER_LAKE: u8 = 2;
/// Water class: the ocean.
pub const WATER_OCEAN: u8 = 3;

/// Seconds in a simulated (365-day) year.
pub const SECONDS_PER_YEAR: f64 = 365.0 * 86_400.0;

/// A body of standing water.
#[derive(Clone, Debug, PartialEq)]
pub struct Lake {
    /// 1-based id, matching the lake-id raster.
    pub id: u32,
    /// Water-surface elevation, metres.
    pub level_m: f32,
    /// Elevation at which the basin would overflow, metres.
    pub spill_m: f32,
    /// Whether the lake has no outlet (evaporation balances inflow below the spill level).
    pub closed: bool,
    /// Cells covered.
    pub cell_count: u32,
    /// Surface area, m².
    pub area_m2: f64,
    /// Stored volume, m³.
    pub volume_m3: f64,
    /// Area draining into the lake, m².
    pub catchment_m2: f64,
    /// The first cell downstream of an open lake.
    pub outlet_cell: Option<u32>,
    /// Centre of the lake surface, metres from the map's north-west corner.
    pub centroid_m: (f32, f32),
}

/// Where a river reach ends.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Terminus {
    /// At a confluence; the reach named by `downstream` continues from there.
    Junction,
    /// In the lake with this id.
    Lake(u32),
    /// Off the map or into the ocean.
    Outlet,
    /// In a closed basin with no standing water.
    Sink,
}

/// A stretch of river between confluences, lakes and outlets.
#[derive(Clone, Debug, PartialEq)]
pub struct RiverReach {
    /// Position in [`WorldMap::reaches`].
    pub id: u32,
    /// Cells along the reach, downstream order. The last cell may be the junction, lake or ocean
    /// cell it ends in.
    pub cells: Vec<u32>,
    /// The reach that continues from this one's last cell, for junctions.
    pub downstream: Option<u32>,
    /// How the reach ends.
    pub terminus: Terminus,
    /// Strahler order.
    pub order: u8,
    /// Mean annual discharge where the reach leaves land, m³/s.
    pub discharge_m3s: f32,
    /// Channel width from hydraulic geometry, metres.
    pub width_m: f32,
    /// Drainage area where the reach leaves land, km².
    pub drainage_area_km2: f32,
}

/// Upstream area entering the map at a cell from beyond its edge.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Inflow {
    /// Cell where the water enters.
    pub cell: u32,
    /// Area draining into the map there, m².
    pub area_m2: f64,
}

/// The regional water balance a map was generated with.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Climate {
    /// Mean annual precipitation, mm.
    pub precipitation_mm_per_yr: f32,
    /// Mean annual evapotranspiration from land, mm.
    pub evapotranspiration_mm_per_yr: f32,
    /// Mean annual evaporation from open water, mm.
    pub lake_evaporation_mm_per_yr: f32,
}

impl Climate {
    /// Runoff from land, m per year.
    pub fn runoff_m_per_yr(&self) -> f64 {
        (f64::from(self.precipitation_mm_per_yr - self.evapotranspiration_mm_per_yr) / 1000.0)
            .max(0.0)
    }
}

/// The physical world: terrain, water and drainage at simulation resolution.
#[derive(Clone, Debug, PartialEq)]
pub struct WorldMap {
    /// Cells west–east.
    pub width: u32,
    /// Cells north–south.
    pub height: u32,
    /// Cell size, metres.
    pub cell_size_m: f32,
    /// Sea level, metres.
    pub sea_level_m: f32,
    /// Bed elevation per cell, metres. Authoritative.
    pub elevation: Vec<f32>,
    /// Receiver per cell: D8 code, [`RECEIVER_OUTLET`] or [`RECEIVER_SINK`]. Authoritative.
    pub receivers: Vec<u8>,
    /// Water class per cell. Authoritative.
    pub water: Vec<u8>,
    /// Lake id per cell (0 = none). Authoritative.
    pub lake_id: Vec<u32>,
    /// Lakes, by id − 1. Authoritative.
    pub lakes: Vec<Lake>,
    /// River reaches. Authoritative.
    pub reaches: Vec<RiverReach>,
    /// Water entering from beyond the map. Authoritative.
    pub inflows: Vec<Inflow>,
    /// Regional water balance. Authoritative.
    pub climate: Climate,
    /// Drainage area per cell, m². Derived: rebuilt by [`WorldMap::rebuild_derived`].
    pub drainage_area_m2: Vec<f32>,
}

/// Summary figures for a map: for the observer, smoke checks and plausibility review.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MapStats {
    /// Lowest elevation, metres.
    pub min_elevation_m: f32,
    /// Highest elevation, metres.
    pub max_elevation_m: f32,
    /// Mean elevation of non-ocean cells, metres.
    pub mean_land_elevation_m: f32,
    /// Fraction of cells that are dry land.
    pub land_fraction: f32,
    /// Fraction of cells that are ocean.
    pub ocean_fraction: f32,
    /// Fraction of cells that are lake.
    pub lake_fraction: f32,
    /// Fraction of cells that are river channel.
    pub river_fraction: f32,
    /// Number of lakes.
    pub lakes: u32,
    /// Lakes without an outlet.
    pub closed_lakes: u32,
    /// Total lake area, km².
    pub lake_area_km2: f32,
    /// Number of river reaches.
    pub reaches: u32,
    /// Total channel length, km.
    pub river_length_km: f32,
    /// Highest Strahler order.
    pub max_order: u8,
    /// Largest reach discharge, m³/s.
    pub max_discharge_m3s: f32,
    /// Fraction of land with a slope under 5 % (easily usable ground).
    pub gentle_land_fraction: f32,
}

impl WorldMap {
    /// Number of cells.
    pub fn cell_count(&self) -> usize {
        self.width as usize * self.height as usize
    }

    /// Size of the map in metres.
    pub fn extent_m(&self) -> (f64, f64) {
        let c = f64::from(self.cell_size_m);
        (f64::from(self.width) * c, f64::from(self.height) * c)
    }

    /// The cell that `i` drains into; `i` itself for outlets and sinks.
    pub fn receiver_index(&self, i: usize) -> usize {
        let code = self.receivers[i];
        if code < 8 {
            grid::neighbor(
                self.width as usize,
                self.height as usize,
                i,
                grid::D8[code as usize],
            )
            .unwrap_or(i)
        } else {
            i
        }
    }

    /// Recomputes derived rasters from the authoritative ones (after loading).
    pub fn rebuild_derived(&mut self) {
        let n = self.cell_count();
        let rcv: Vec<u32> = (0..n).map(|i| self.receiver_index(i) as u32).collect();
        let order = routing::stack_order(&rcv);
        let mut extra = vec![0.0; n];
        for inflow in &self.inflows {
            if let Some(e) = extra.get_mut(inflow.cell as usize) {
                *e += inflow.area_m2;
            }
        }
        let cell_area = f64::from(self.cell_size_m) * f64::from(self.cell_size_m);
        let area = routing::accumulate(&order, &rcv, cell_area, &extra);
        self.drainage_area_m2 = area.iter().map(|&a| a as f32).collect();
    }

    /// Cell centres along a reach, in metres, simplified to within `tolerance_m`.
    pub fn reach_polyline(&self, reach: &RiverReach, tolerance_m: f32) -> Vec<(f32, f32)> {
        let w = self.width as usize;
        let c = self.cell_size_m;
        let points: Vec<(f32, f32)> = reach
            .cells
            .iter()
            .map(|&i| {
                let i = i as usize;
                (((i % w) as f32 + 0.5) * c, ((i / w) as f32 + 0.5) * c)
            })
            .collect();
        rivers::simplify(&points, tolerance_m)
    }

    /// Summary figures.
    pub fn stats(&self) -> MapStats {
        let n = self.cell_count();
        let (w, h) = (self.width as usize, self.height as usize);
        let c = f64::from(self.cell_size_m);
        let mut min = f32::MAX;
        let mut max = f32::MIN;
        let mut land_sum = 0.0f64;
        let mut counts = [0usize; 4];
        let mut gentle = 0usize;
        let mut dry = 0usize;
        for i in 0..n {
            let z = self.elevation[i];
            min = min.min(z);
            max = max.max(z);
            let class = (self.water[i] as usize).min(3);
            counts[class] += 1;
            if class != WATER_OCEAN as usize {
                land_sum += f64::from(z);
            }
            if class == WATER_LAND as usize {
                dry += 1;
                let (x, y) = (i % w, i / w);
                let zx = |xx: usize| f64::from(self.elevation[y * w + xx]);
                let zy = |yy: usize| f64::from(self.elevation[yy * w + x]);
                let gx = (zx((x + 1).min(w - 1)) - zx(x.saturating_sub(1))) / (2.0 * c);
                let gy = (zy((y + 1).min(h - 1)) - zy(y.saturating_sub(1))) / (2.0 * c);
                if (gx * gx + gy * gy).sqrt() < 0.05 {
                    gentle += 1;
                }
            }
        }
        let not_ocean = n - counts[WATER_OCEAN as usize];
        let river_length: f64 = self
            .reaches
            .iter()
            .map(|r| {
                r.cells
                    .windows(2)
                    .map(|p| grid::step_length(w, p[0] as usize, p[1] as usize, c))
                    .sum::<f64>()
            })
            .sum();
        MapStats {
            min_elevation_m: min,
            max_elevation_m: max,
            mean_land_elevation_m: if not_ocean > 0 {
                (land_sum / not_ocean as f64) as f32
            } else {
                0.0
            },
            land_fraction: counts[0] as f32 / n as f32,
            ocean_fraction: counts[3] as f32 / n as f32,
            lake_fraction: counts[2] as f32 / n as f32,
            river_fraction: counts[1] as f32 / n as f32,
            lakes: self.lakes.len() as u32,
            closed_lakes: self.lakes.iter().filter(|l| l.closed).count() as u32,
            lake_area_km2: (self.lakes.iter().map(|l| l.area_m2).sum::<f64>() / 1.0e6) as f32,
            reaches: self.reaches.len() as u32,
            river_length_km: (river_length / 1000.0) as f32,
            max_order: self.reaches.iter().map(|r| r.order).max().unwrap_or(0),
            max_discharge_m3s: self
                .reaches
                .iter()
                .map(|r| r.discharge_m3s)
                .fold(0.0, f32::max),
            gentle_land_fraction: if dry > 0 {
                gentle as f32 / dry as f32
            } else {
                0.0
            },
        }
    }
}

/// What to generate.
#[derive(Clone, Debug, PartialEq)]
pub struct GenerateRequest {
    /// Seed: the only source of variation between worlds made from the same parameters.
    pub seed: u64,
    /// Cells west–east; a multiple of 8 within [`MIN_CELLS`]–[`MAX_CELLS`].
    pub width_cells: u32,
    /// Cells north–south; a multiple of 8 within [`MIN_CELLS`]–[`MAX_CELLS`].
    pub height_cells: u32,
    /// Simulation cell size, metres (the plan's default is 8).
    pub cell_size_m: f64,
    /// Landscape parameters, usually from a content preset.
    pub params: TerrainParams,
}

impl GenerateRequest {
    /// Checks sizes and parameters.
    pub fn validate(&self) -> Result<(), GenError> {
        let factor = 1u32 << REFINEMENT_LEVELS;
        for (name, v) in [("width", self.width_cells), ("height", self.height_cells)] {
            if !(MIN_CELLS..=MAX_CELLS).contains(&v) || v % factor != 0 {
                return Err(GenError::InvalidRequest(format!(
                    "{name} must be a multiple of {factor} between {MIN_CELLS} and {MAX_CELLS} cells, got {v}"
                )));
            }
        }
        if !(self.cell_size_m.is_finite() && (1.0..=100.0).contains(&self.cell_size_m)) {
            return Err(GenError::InvalidRequest(
                "cell size must be between 1 and 100 metres".to_owned(),
            ));
        }
        self.params.validate().map_err(GenError::InvalidRequest)
    }
}

/// Generation progress, for loading screens.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Progress {
    /// What is happening, in plain words.
    pub stage: &'static str,
    /// Overall completion, 0–1.
    pub fraction: f32,
}

/// Why generation stopped.
#[derive(Clone, Debug, PartialEq)]
pub enum GenError {
    /// The request or its parameters are out of range.
    InvalidRequest(String),
    /// The cancel flag was raised.
    Cancelled,
    /// The generated world broke an invariant: a generator bug.
    Invariant(Vec<String>),
}

impl std::fmt::Display for GenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GenError::InvalidRequest(why) => write!(f, "invalid world request: {why}"),
            GenError::Cancelled => f.write_str("world generation was cancelled"),
            GenError::Invariant(problems) => write!(
                f,
                "generated world failed {} invariant check(s): {}",
                problems.len(),
                problems
                    .iter()
                    .take(5)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
        }
    }
}

impl std::error::Error for GenError {}

/// Generates a world. Deterministic for a given request on a given build.
pub fn generate(
    request: &GenerateRequest,
    progress: &mut dyn FnMut(Progress),
    cancel: &AtomicBool,
) -> Result<WorldMap, GenError> {
    request.validate()?;
    let p = &request.params;
    let factor = 1usize << REFINEMENT_LEVELS;
    let coarse_cell = request.cell_size_m * factor as f64;
    let (pw, ph) = (
        request.width_cells as usize / factor,
        request.height_cells as usize / factor,
    );
    let context_side = |playable: usize| {
        let mut side = ((playable as f64 * p.context_factor).round() as usize).max(playable + 4);
        if (side - playable) % 2 == 1 {
            side += 1;
        }
        side
    };
    let (cw, ch) = (context_side(pw), context_side(ph));
    // Centre the map in its watershed, then slide it toward the base-level edges by
    // `window_offset` (a coastal map needs its coast on the map).
    let (mut x0, mut y0) = ((cw - pw) as f64 / 2.0, (ch - ph) as f64 / 2.0);
    for side in lem::outlet_sides(request.seed, p.outlet_edges) {
        let (mx, my) = ((cw - pw) as f64 / 2.0, (ch - ph) as f64 / 2.0);
        match side {
            0 => y0 -= my * p.window_offset,
            1 => x0 += mx * p.window_offset,
            2 => y0 += my * p.window_offset,
            _ => x0 -= mx * p.window_offset,
        }
    }
    let rect = (
        (x0.round() as usize).min(cw - pw),
        (y0.round() as usize).min(ch - ph),
        pw,
        ph,
    );
    let check = || {
        if cancel.load(Ordering::Relaxed) {
            Err(GenError::Cancelled)
        } else {
            Ok(())
        }
    };

    progress(Progress {
        stage: "Raising the land",
        fraction: 0.0,
    });
    let fields = lem::Fields::new(request.seed, p);
    let mut context = lem::build_context(request.seed, p, &fields, cw, ch, coarse_cell);
    let erosion = lem::Erosion::new(p, p.coarse_dt_years);
    let mut scratch = Vec::new();
    for it in 0..p.coarse_iterations {
        check()?;
        context.step(&erosion, &mut scratch);
        if it % 5 == 0 {
            progress(Progress {
                stage: "Eroding mountains and valleys",
                fraction: 0.02 + 0.5 * it as f32 / p.coarse_iterations as f32,
            });
        }
    }
    let routing = context.route();
    let inflow_points = lem::boundary_inflows(&context, &routing, rect);
    let mut land = refine::crop(&context, rect);
    drop(context);

    const REFINE_STAGES: [&str; 3] = [
        "Refining the terrain (coarse)",
        "Refining the terrain (medium)",
        "Refining the terrain (fine)",
    ];
    for (level, stage) in REFINE_STAGES.iter().enumerate() {
        check()?;
        progress(Progress {
            stage,
            fraction: 0.55 + 0.11 * level as f32,
        });
        refine::refine_level(&mut land, level, request.seed, p, &fields, &inflow_points);
    }
    debug_assert_eq!(land.w, request.width_cells as usize);
    debug_assert_eq!(land.h, request.height_cells as usize);

    check()?;
    progress(Progress {
        stage: "Laying down valley floors",
        fraction: 0.86,
    });
    floodplain::carve_floodplains(&mut land, p);

    check()?;
    progress(Progress {
        stage: "Filling basins and placing lakes",
        fraction: 0.88,
    });
    let inflows: Vec<Inflow> = land
        .inflow
        .iter()
        .enumerate()
        .filter(|&(_, &a)| a > 0.0)
        .map(|(i, &a)| Inflow {
            cell: i as u32,
            area_m2: a,
        })
        .collect();
    let bed = std::mem::take(&mut land.z);
    let hydro = hydro::finalize(
        bed,
        &hydro::HydroInput {
            w: land.w,
            h: land.h,
            cell: land.cell,
            sea_level: p.sea_level_m,
            runoff: p.runoff_m_per_yr(),
            precipitation: p.precipitation_mm_per_yr / 1000.0,
            lake_evaporation: p.lake_evaporation_mm_per_yr / 1000.0,
            min_lake_depth: p.min_lake_depth_m,
            min_lake_area: p.min_lake_area_m2,
            inflow: &land.inflow,
            jitter: routing::Jitter {
                seed: civ_core::rng::key(&[land.routing_seed, u64::MAX]),
                amount: land.jitter,
            },
        },
    );

    check()?;
    progress(Progress {
        stage: "Tracing rivers",
        fraction: 0.94,
    });
    let mut water = hydro.water;
    let reaches = rivers::extract(
        &rivers::RiverInput {
            w: land.w,
            receivers: &hydro.receivers,
            receiver_index: &hydro.receiver_index,
            area: &hydro.area,
            lake_id: &hydro.lake_id,
            lakes: &hydro.lakes,
            threshold_area: p.channel_area_km2 * 1.0e6,
            runoff: p.runoff_m_per_yr(),
            width_coefficient: p.channel_width_coefficient,
        },
        &mut water,
    );

    let map = WorldMap {
        width: request.width_cells,
        height: request.height_cells,
        cell_size_m: request.cell_size_m as f32,
        sea_level_m: p.sea_level_m as f32,
        elevation: hydro.bed.iter().map(|&z| z as f32).collect(),
        receivers: hydro.receivers,
        water,
        lake_id: hydro.lake_id,
        lakes: hydro.lakes,
        reaches,
        inflows,
        climate: Climate {
            precipitation_mm_per_yr: p.precipitation_mm_per_yr as f32,
            evapotranspiration_mm_per_yr: p.evapotranspiration_mm_per_yr as f32,
            lake_evaporation_mm_per_yr: p.lake_evaporation_mm_per_yr as f32,
        },
        drainage_area_m2: hydro.area.iter().map(|&a| a as f32).collect(),
    };

    progress(Progress {
        stage: "Checking the water",
        fraction: 0.98,
    });
    let problems = validate(&map);
    if !problems.is_empty() {
        return Err(GenError::Invariant(problems));
    }
    progress(Progress {
        stage: "Done",
        fraction: 1.0,
    });
    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 2 km map with a short coarse run: small enough for debug-build tests. The channel
    /// threshold is lowered so that local drainage alone yields rivers on so small a map.
    fn small(seed: u64) -> GenerateRequest {
        GenerateRequest {
            seed,
            width_cells: 256,
            height_cells: 256,
            cell_size_m: 8.0,
            params: TerrainParams {
                coarse_iterations: 40,
                channel_area_km2: 0.1,
                ..TerrainParams::default()
            },
        }
    }

    fn run(request: &GenerateRequest) -> WorldMap {
        generate(request, &mut |_| {}, &AtomicBool::new(false)).expect("generates")
    }

    #[test]
    fn a_small_world_is_valid_and_has_rivers() {
        let map = run(&small(11));
        assert!(validate(&map).is_empty());
        let stats = map.stats();
        assert!(
            stats.max_elevation_m > stats.min_elevation_m + 5.0,
            "{stats:?}"
        );
        assert!(stats.reaches > 0, "{stats:?}");
        assert!(stats.river_length_km > 0.0);
    }

    #[test]
    fn the_same_seed_gives_the_same_world_and_another_seed_does_not() {
        let a = run(&small(3));
        let b = run(&small(3));
        assert_eq!(a, b);
        let c = run(&small(4));
        assert_ne!(a.elevation, c.elevation);
    }

    #[test]
    fn derived_rasters_rebuild_exactly() {
        let map = run(&small(8));
        let mut reloaded = map.clone();
        reloaded.drainage_area_m2.clear();
        reloaded.rebuild_derived();
        assert_eq!(reloaded.drainage_area_m2, map.drainage_area_m2);
    }

    #[test]
    fn a_coast_puts_ocean_on_the_map() {
        let mut request = small(21);
        request.params.base_level_m = -40.0;
        request.params.regional_slope = 0.002;
        request.params.window_offset = 1.0;
        let map = run(&request);
        let stats = map.stats();
        assert!(stats.ocean_fraction > 0.0, "{stats:?}");
        assert!(validate(&map).is_empty());
    }

    #[test]
    fn cancellation_stops_generation() {
        let cancel = AtomicBool::new(true);
        assert_eq!(
            generate(&small(1), &mut |_| {}, &cancel),
            Err(GenError::Cancelled)
        );
    }

    #[test]
    fn bad_requests_are_refused() {
        let mut r = small(1);
        r.width_cells = 100; // not a multiple of 8
        assert!(matches!(r.validate(), Err(GenError::InvalidRequest(_))));
    }

    #[test]
    fn validation_catches_a_receiver_cycle() {
        let mut map = run(&small(2));
        // Point two interior neighbours at each other.
        let w = map.width as usize;
        let a = 40 * w + 40;
        map.receivers[a] = 0; // east
        map.receivers[a + 1] = 4; // west
        let problems = validate(&map);
        assert!(problems.iter().any(|p| p.contains("cycle")), "{problems:?}");
    }
}
