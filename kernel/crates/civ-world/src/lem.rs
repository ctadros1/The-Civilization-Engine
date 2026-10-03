//! Landscape evolution: rock uplift, stream-power incision, hillslope diffusion and talus
//! relaxation (research 03-01 §1.1 and §1.5).
//!
//! Incision uses the implicit downstream-to-upstream update of Braun & Willett (2013) with n = 1:
//!
//! ```text
//! z_i ← (z_i + U_i·dt + c_i·z_r) / (1 + c_i),   c_i = K_i·A_i^m·dt / ℓ_i
//! ```
//!
//! where `r` is the receiver, already updated because cells are processed in stack order. Routing
//! uses a priority-flood epsilon surface, so water crosses depressions instead of stopping in them,
//! while the physical surface keeps its shape.

use rayon::prelude::*;

use civ_core::rng::{Rng64, key};

use crate::flood::priority_flood;
use crate::grid::{D4, D8, D8_DIST, neighbor, step_length};
use crate::noise::Noise;
use crate::params::TerrainParams;
use crate::routing::{Jitter, accumulate, stack_order, steepest_receivers};

/// Purposes for keyed random streams and noise layers. Changing a value changes every world.
pub(crate) mod purpose {
    pub const SIDES: u64 = 1;
    pub const RELIEF: u64 = 2;
    pub const RANGES: u64 = 3;
    pub const WARP: u64 = 4;
    pub const BASINS: u64 = 5;
    pub const ROCK: u64 = 6;
    pub const JITTER: u64 = 7;
    pub const ROUTING: u64 = 8;
    pub const TRUNK: u64 = 9;
    pub const DETAIL: u64 = 10;
}

/// A raster landscape and the fields that drive its evolution.
pub(crate) struct Landscape {
    pub w: usize,
    pub h: usize,
    /// Cell size, metres.
    pub cell: f64,
    /// Position of the grid's north-west corner in context coordinates, metres. Noise is sampled
    /// in context coordinates so every resolution sees the same fields.
    pub origin: (f64, f64),
    /// Elevation, metres.
    pub z: Vec<f64>,
    /// Rock uplift, metres per year.
    pub uplift: Vec<f64>,
    /// Stream-power erodibility.
    pub erodibility: Vec<f64>,
    /// Cells held at base level; they are the outlets.
    pub base: Vec<bool>,
    /// Upstream area entering a cell from outside the grid, m².
    pub inflow: Vec<f64>,
    /// Seed of the routing perturbation; combined with `steps` so each pass differs.
    pub routing_seed: u64,
    /// Erosion steps taken so far.
    pub steps: u64,
    /// Strength of the routing perturbation.
    pub jitter: f64,
}

/// Receivers, order and drainage area of one routing pass.
pub(crate) struct Routing {
    pub receivers: Vec<u32>,
    pub order: Vec<u32>,
    pub area: Vec<f64>,
}

/// Settings for one erosion pass.
pub(crate) struct Erosion {
    pub dt: f64,
    pub area_exponent: f64,
    /// Drainage area below which a cell is hillslope, m²: uplift only, no incision.
    pub channel_area: f64,
    pub diffusivity: f64,
    pub talus: f64,
}

impl Erosion {
    /// The erosion settings for a time step `dt` under `params`.
    pub fn new(params: &TerrainParams, dt: f64) -> Self {
        Erosion {
            dt,
            area_exponent: params.area_exponent,
            channel_area: params.channel_initiation_km2 * 1.0e6,
            diffusivity: params.diffusivity_m2_per_yr,
            talus: params.talus_slope,
        }
    }
}

#[inline]
fn smoothstep(edge0: f64, edge1: f64, x: f64) -> f64 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// The noise fields that shape a world, sampled in context coordinates (metres).
pub(crate) struct Fields {
    relief: Noise,
    ranges: Noise,
    warp: Noise,
    basins: Noise,
    rock: Noise,
    jitter: Noise,
    range_scale_m: f64,
}

impl Fields {
    pub fn new(seed: u64, params: &TerrainParams) -> Self {
        Fields {
            relief: Noise::new(key(&[seed, purpose::RELIEF])),
            ranges: Noise::new(key(&[seed, purpose::RANGES])),
            warp: Noise::new(key(&[seed, purpose::WARP])),
            basins: Noise::new(key(&[seed, purpose::BASINS])),
            rock: Noise::new(key(&[seed, purpose::ROCK])),
            jitter: Noise::new(key(&[seed, purpose::JITTER])),
            range_scale_m: params.range_scale_km * 1000.0,
        }
    }

    /// Erodibility at a point: rock of varying resistance.
    pub fn erodibility(&self, params: &TerrainParams, x: f64, y: f64) -> f64 {
        let s = self.range_scale_m * 0.4;
        let v = self.rock.fbm(x / s, y / s, 3);
        params.erodibility * (1.0 + params.erodibility_variation * v).max(0.1)
    }
}

/// Which context edges are held at base level: 0 north, 1 east, 2 south, 3 west.
pub(crate) fn outlet_sides(seed: u64, count: u32) -> Vec<u8> {
    let mut rng = Rng64::from_key(&[seed, purpose::SIDES]);
    let first = rng.below(4) as u8;
    let mut sides = vec![first];
    if count >= 2 {
        sides.push((first + 1) % 4);
    }
    sides
}

fn distance_to_sides(x: f64, y: f64, width_m: f64, height_m: f64, sides: &[u8]) -> f64 {
    sides
        .iter()
        .map(|&s| match s {
            0 => y,
            1 => width_m - x,
            2 => height_m - y,
            _ => x,
        })
        .fold(f64::MAX, f64::min)
        .max(0.0)
}

fn on_side(w: usize, h: usize, i: usize, sides: &[u8]) -> bool {
    let (x, y) = (i % w, i / w);
    sides.iter().any(|&s| match s {
        0 => y == 0,
        1 => x + 1 == w,
        2 => y + 1 == h,
        _ => x == 0,
    })
}

/// The coarse watershed context: initial surface, uplift and erodibility fields, and base-level
/// edges chosen by the seed.
pub(crate) fn build_context(
    seed: u64,
    params: &TerrainParams,
    fields: &Fields,
    w: usize,
    h: usize,
    cell: f64,
) -> Landscape {
    let sides = outlet_sides(seed, params.outlet_edges);
    let width_m = w as f64 * cell;
    let height_m = h as f64 * cell;
    let extent = width_m.min(height_m);
    let s = fields.range_scale_m;
    let umax = params.uplift_mm_per_yr / 1000.0;

    let cells: Vec<(f64, f64, f64, bool)> = (0..w * h)
        .into_par_iter()
        .map(|i| {
            let x = (i % w) as f64 * cell + cell * 0.5;
            let y = (i / w) as f64 * cell + cell * 0.5;
            let base = on_side(w, h, i, &sides);
            let d = distance_to_sides(x, y, width_m, height_m, &sides);
            let t = (d / extent).min(1.0);
            if base {
                return (
                    params.base_level_m,
                    0.0,
                    fields.erodibility(params, x, y),
                    true,
                );
            }
            let relief = fields.relief.fbm(x / (s * 0.7), y / (s * 0.7), 4);
            let jitter = fields.jitter.sample(x / cell * 0.73, y / cell * 0.73);
            let z0 = params.base_level_m
                + params.regional_slope * d
                + params.initial_relief_m * relief * smoothstep(0.0, 0.1, t)
                + 0.05 * jitter;

            let wx = x / s + 0.6 * fields.warp.fbm(x / (s * 1.3), y / (s * 1.3), 3);
            let wy = y / s
                + 0.6
                    * fields
                        .warp
                        .fbm(x / (s * 1.3) + 31.4, y / (s * 1.3) - 27.1, 3);
            let ridge = fields.ranges.ridged(wx, wy, 5);
            let mountain = ridge * ridge.sqrt();
            let far = 0.35 + 0.65 * t;
            let taper = smoothstep(0.0, 0.06, t);
            let lowland = params.lowland_uplift_fraction;
            let mut u = umax * (lowland + (1.0 - lowland) * mountain * far) * taper;
            if params.basin_strength > 0.0 {
                let b = fields.basins.fbm(x / (s * 0.45), y / (s * 0.45), 3);
                if b > 0.3 {
                    u -= params.basin_strength * umax * (b - 0.3) * 2.5 * taper;
                }
            }
            (z0, u, fields.erodibility(params, x, y), false)
        })
        .collect();

    let mut inflow = vec![0.0; w * h];
    if params.trunk_inflow_km2 > 0.0 {
        // The trunk river enters on the edge opposite the (first) base-level edge, somewhere in
        // its middle half.
        let mut rng = Rng64::from_key(&[seed, purpose::TRUNK]);
        let upstream = (sides[0] + 2) % 4;
        let mut along = |len: usize| (len as f64 * rng.range_f64(0.25, 0.75)) as usize;
        let cell_index = match upstream {
            0 => along(w),
            1 => along(h) * w + (w - 1),
            2 => (h - 1) * w + along(w),
            _ => along(h) * w,
        };
        inflow[cell_index] = params.trunk_inflow_km2 * 1.0e6;
    }

    Landscape {
        w,
        h,
        cell,
        origin: (0.0, 0.0),
        z: cells.iter().map(|c| c.0).collect(),
        uplift: cells.iter().map(|c| c.1).collect(),
        erodibility: cells.iter().map(|c| c.2).collect(),
        base: cells.iter().map(|c| c.3).collect(),
        inflow,
        routing_seed: key(&[seed, purpose::ROUTING]),
        steps: 0,
        jitter: params.routing_jitter,
    }
}

impl Landscape {
    /// The routing perturbation for the current step.
    pub fn jitter(&self) -> Jitter {
        Jitter {
            seed: key(&[self.routing_seed, self.steps]),
            amount: self.jitter,
        }
    }

    /// Routes flow over the current surface.
    pub fn route(&self) -> Routing {
        let surface = priority_flood(&self.z, self.w, self.h, &self.base, true);
        let receivers = steepest_receivers(&surface, self.w, self.h, &self.base, self.jitter());
        let order = stack_order(&receivers);
        let area = accumulate(&order, &receivers, self.cell * self.cell, &self.inflow);
        Routing {
            receivers,
            order,
            area,
        }
    }

    /// One full erosion step. Returns the routing it used.
    pub fn step(&mut self, e: &Erosion, scratch: &mut Vec<f64>) -> Routing {
        let routing = self.route();
        self.stream_power(&routing, e);
        self.diffuse(e, scratch);
        self.relax_talus(e, scratch);
        self.steps += 1;
        routing
    }

    fn stream_power(&mut self, routing: &Routing, e: &Erosion) {
        let sqrt_area = (e.area_exponent - 0.5).abs() < 1e-12;
        for &i in &routing.order {
            let i = i as usize;
            let r = routing.receivers[i] as usize;
            if r == i || self.base[i] {
                continue;
            }
            let a = routing.area[i];
            if a < e.channel_area {
                // Hillslope: material leaves by diffusion and talus, not by incision.
                self.z[i] += self.uplift[i] * e.dt;
                continue;
            }
            let a_m = if sqrt_area {
                a.sqrt()
            } else {
                a.powf(e.area_exponent)
            };
            let c = self.erodibility[i] * a_m * e.dt / step_length(self.w, i, r, self.cell);
            self.z[i] = (self.z[i] + self.uplift[i] * e.dt + c * self.z[r]) / (1.0 + c);
        }
    }

    /// Explicit linear diffusion, sub-stepped for stability; base cells are fixed and grid edges
    /// are no-flux.
    fn diffuse(&mut self, e: &Erosion, scratch: &mut Vec<f64>) {
        if e.diffusivity <= 0.0 {
            return;
        }
        let alpha_total = e.diffusivity * e.dt / (self.cell * self.cell);
        let substeps = (alpha_total / 0.2).ceil().max(1.0) as usize;
        let alpha = alpha_total / substeps as f64;
        let (w, h) = (self.w, self.h);
        scratch.resize(w * h, 0.0);
        for _ in 0..substeps {
            let z = &self.z;
            let base = &self.base;
            scratch.par_chunks_mut(w).enumerate().for_each(|(y, row)| {
                for (x, out) in row.iter_mut().enumerate() {
                    let i = y * w + x;
                    if base[i] {
                        *out = z[i];
                        continue;
                    }
                    let mut lap = 0.0;
                    for off in D4 {
                        if let Some(nb) = neighbor(w, h, i, off) {
                            lap += z[nb] - z[i];
                        }
                    }
                    *out = z[i] + alpha * lap;
                }
            });
            std::mem::swap(&mut self.z, scratch);
        }
    }

    /// Moves material down slopes steeper than the talus slope. Pairwise symmetric, so it
    /// conserves material except at fixed base cells.
    fn relax_talus(&mut self, e: &Erosion, scratch: &mut Vec<f64>) {
        let (w, h, cell) = (self.w, self.h, self.cell);
        scratch.resize(w * h, 0.0);
        let z = &self.z;
        let base = &self.base;
        let rate = 0.1;
        scratch.par_chunks_mut(w).enumerate().for_each(|(y, row)| {
            for (x, out) in row.iter_mut().enumerate() {
                let i = y * w + x;
                if base[i] {
                    *out = z[i];
                    continue;
                }
                let mut delta = 0.0;
                for (d, &off) in D8.iter().enumerate() {
                    if let Some(nb) = neighbor(w, h, i, off) {
                        if base[nb] {
                            continue;
                        }
                        let limit = e.talus * D8_DIST[d] * cell;
                        let diff = z[i] - z[nb];
                        if diff > limit {
                            delta -= rate * (diff - limit) * 0.5;
                        } else if -diff > limit {
                            delta += rate * (-diff - limit) * 0.5;
                        }
                    }
                }
                *out = z[i] + delta;
            }
        });
        std::mem::swap(&mut self.z, scratch);
    }
}

/// Upstream area entering the rectangle `(x0, y0, w, h)` of the context from outside it. Returned
/// as points in metres relative to the rectangle's north-west corner, with the area in m².
pub(crate) fn boundary_inflows(
    ls: &Landscape,
    routing: &Routing,
    rect: (usize, usize, usize, usize),
) -> Vec<(f64, f64, f64)> {
    let (x0, y0, rw, rh) = rect;
    let inside = |i: usize| {
        let (x, y) = (i % ls.w, i / ls.w);
        x >= x0 && x < x0 + rw && y >= y0 && y < y0 + rh
    };
    let mut entries: Vec<(usize, f64)> = Vec::new();
    for i in 0..ls.w * ls.h {
        let r = routing.receivers[i] as usize;
        if r != i && !inside(i) && inside(r) {
            entries.push((r, routing.area[i]));
        }
    }
    entries.sort_by_key(|e| e.0);
    let mut merged: Vec<(usize, f64)> = Vec::new();
    for (cell, area) in entries {
        match merged.last_mut() {
            Some(last) if last.0 == cell => last.1 += area,
            _ => merged.push((cell, area)),
        }
    }
    let min_area = 4.0 * ls.cell * ls.cell;
    merged
        .into_iter()
        .filter(|&(_, a)| a >= min_area)
        .map(|(cell, area)| {
            let (x, y) = (cell % ls.w, cell / ls.w);
            (
                ((x - x0) as f64 + 0.5) * ls.cell,
                ((y - y0) as f64 + 0.5) * ls.cell,
                area,
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn small_context(seed: u64) -> Landscape {
        let params = TerrainParams::default();
        let fields = Fields::new(seed, &params);
        build_context(seed, &params, &fields, 48, 48, 200.0)
    }

    #[test]
    fn erosion_creates_relief_and_keeps_base_level() {
        let params = TerrainParams::default();
        let mut ls = small_context(5);
        let e = Erosion::new(&params, params.coarse_dt_years);
        let mut scratch = Vec::new();
        for _ in 0..40 {
            ls.step(&e, &mut scratch);
        }
        let max = ls.z.iter().copied().fold(f64::MIN, f64::max);
        assert!(
            max > params.base_level_m + 20.0,
            "uplift raised the land: {max}"
        );
        for i in 0..ls.w * ls.h {
            assert!(ls.z[i].is_finite());
            if ls.base[i] {
                assert_eq!(ls.z[i], params.base_level_m);
            }
        }
        let routing = ls.route();
        assert_eq!(
            routing.order.len(),
            ls.w * ls.h,
            "everything drains to base level"
        );
    }

    #[test]
    fn outlet_sides_are_seeded_and_adjacent() {
        for seed in 0..20 {
            let two = outlet_sides(seed, 2);
            assert_eq!(two.len(), 2);
            assert_eq!((two[0] + 1) % 4, two[1]);
            assert_eq!(outlet_sides(seed, 1), outlet_sides(seed, 1));
        }
    }

    #[test]
    fn inflows_are_found_where_flow_enters_a_rectangle() {
        // Flow runs west to east along a row; a rectangle in the middle receives it.
        let (w, h) = (10, 3);
        let ls = Landscape {
            w,
            h,
            cell: 1.0,
            origin: (0.0, 0.0),
            z: vec![0.0; w * h],
            uplift: vec![0.0; w * h],
            erodibility: vec![0.0; w * h],
            base: vec![false; w * h],
            inflow: vec![0.0; w * h],
            routing_seed: 0,
            steps: 0,
            jitter: 0.0,
        };
        let receivers: Vec<u32> = (0..w * h)
            .map(|i| {
                if i % w == w - 1 {
                    i as u32
                } else {
                    i as u32 + 1
                }
            })
            .collect();
        let order = stack_order(&receivers);
        let area = accumulate(&order, &receivers, 1.0, &[]);
        let routing = Routing {
            receivers,
            order,
            area,
        };
        let inflows = boundary_inflows(&ls, &routing, (4, 0, 3, 3));
        assert_eq!(inflows.len(), 3, "one per row");
        for (x, _, a) in inflows {
            assert_eq!(x, 0.5);
            assert_eq!(a, 4.0, "cells 0-3 of the row");
        }
    }
}
