//! Final hydrology at simulation resolution (research 03-01 §3, research 03-02 §1.1).
//!
//! 1. **Ocean** is ground below sea level that is connected to the map edge.
//! 2. **Depressions** come from a priority-flood fill. Each one is classified:
//!    - shallow or tiny ones are terrain artifacts and are filled;
//!    - the rest hold water.
//! 3. A depression's **lake level** comes from its water balance. Inflow is the catchment's runoff
//!    plus rain on the lake; loss is evaporation from the lake surface.
//!    - If inflow can match evaporation at the spill level, the lake is full and drains out
//!      (open).
//!    - Otherwise it settles where the two balance and has no outlet (closed). A basin too dry for
//!      a lake becomes a terminal sink.
//!
//!    A topographic basin identifies possible storage, not guaranteed water (research 03-01 §3.3).
//! 4. **Routing**: the edge and ocean drain off the map, closed lakes are terminal, and everything
//!    else follows steepest descent on an epsilon-filled surface.
//!
//! Known simplification: a closed basin's level is computed for the depression as a whole. When
//! sub-basins inside it separate below that level, each becomes its own lake at the shared level.
//! Fill–Spill–Merge (Barnes et al. 2021) is the reference for doing this properly later.

use std::collections::VecDeque;

use crate::flood::{ocean_mask, priority_flood};
use crate::grid::{D8, direction_between, is_edge, neighbor};
use crate::routing::{Jitter, accumulate, stack_order, steepest_receivers};
use crate::{Lake, RECEIVER_OUTLET, RECEIVER_SINK, WATER_LAKE, WATER_OCEAN};

/// What the hydrology pass needs to know besides the terrain.
pub(crate) struct HydroInput<'a> {
    pub w: usize,
    pub h: usize,
    pub cell: f64,
    pub sea_level: f64,
    /// Runoff from land, m/yr.
    pub runoff: f64,
    /// Precipitation, m/yr.
    pub precipitation: f64,
    /// Evaporation from open water, m/yr.
    pub lake_evaporation: f64,
    pub min_lake_depth: f64,
    pub min_lake_area: f64,
    /// Upstream area entering cells from beyond the map, m².
    pub inflow: &'a [f64],
    /// Routing perturbation; the same for both routing passes.
    pub jitter: Jitter,
}

/// The finished water system.
pub(crate) struct HydroOutput {
    /// Bed elevation after artifact depressions were filled.
    pub bed: Vec<f64>,
    /// D8 code, [`RECEIVER_OUTLET`] or [`RECEIVER_SINK`] per cell.
    pub receivers: Vec<u8>,
    /// Receiver index per cell (a cell that is its own receiver drains off the map or is a sink).
    pub receiver_index: Vec<u32>,
    /// Water class per cell, before rivers are marked.
    pub water: Vec<u8>,
    /// Lake id per cell (0 = none).
    pub lake_id: Vec<u32>,
    pub lakes: Vec<Lake>,
    /// Drainage area per cell, m².
    pub area: Vec<f64>,
    /// Cells filled as artifacts (checked by tests; useful when tuning presets).
    #[cfg_attr(not(test), allow(dead_code))]
    pub filled_cells: usize,
}

struct Depression {
    cells: Vec<usize>,
    spill: f64,
    min: f64,
    catchment: f64,
}

fn label_depressions(z: &[f64], spill: &[f64], w: usize, h: usize) -> Vec<Depression> {
    let n = w * h;
    let mut seen = vec![false; n];
    let mut out = Vec::new();
    let mut queue = VecDeque::new();
    for start in 0..n {
        if seen[start] || spill[start] <= z[start] {
            continue;
        }
        seen[start] = true;
        queue.push_back(start);
        let mut cells = Vec::new();
        let mut min = f64::MAX;
        while let Some(c) = queue.pop_front() {
            cells.push(c);
            min = min.min(z[c]);
            for off in D8 {
                if let Some(nb) = neighbor(w, h, c, off)
                    && !seen[nb]
                    && spill[nb] > z[nb]
                {
                    seen[nb] = true;
                    queue.push_back(nb);
                }
            }
        }
        out.push(Depression {
            spill: spill[start],
            min,
            cells,
            catchment: 0.0,
        });
    }
    out
}

/// Builds lakes, receivers and drainage for a finished terrain.
pub(crate) fn finalize(mut z: Vec<f64>, input: &HydroInput) -> HydroOutput {
    let (w, h) = (input.w, input.h);
    let n = w * h;
    let cell_area = input.cell * input.cell;

    let ocean = ocean_mask(&z, w, h, input.sea_level);
    let outlets: Vec<bool> = (0..n).map(|i| ocean[i] || is_edge(w, h, i)).collect();

    // Depressions and their catchments.
    let spill = priority_flood(&z, w, h, &outlets, false);
    let mut depressions = label_depressions(&z, &spill, w, h);
    {
        let pre = priority_flood(&z, w, h, &outlets, true);
        let rcv = steepest_receivers(&pre, w, h, &outlets, input.jitter);
        let order = stack_order(&rcv);
        let area = accumulate(&order, &rcv, cell_area, input.inflow);
        let mut member = vec![u32::MAX; n];
        for (d, dep) in depressions.iter().enumerate() {
            for &c in &dep.cells {
                member[c] = d as u32;
            }
        }
        for (d, dep) in depressions.iter_mut().enumerate() {
            dep.catchment = dep
                .cells
                .iter()
                .filter(|&&c| member[rcv[c] as usize] != d as u32)
                .map(|&c| area[c])
                .sum();
        }
    }

    // Classify and level.
    let mut sink = vec![false; n];
    let mut surface = z.clone();
    let mut is_lake = vec![false; n];
    let mut lake_level = vec![f64::NAN; n];
    let mut lake_spill = vec![f64::NAN; n];
    let mut lake_closed = vec![false; n];
    let mut filled_cells = 0;
    let deficit = input.lake_evaporation - input.precipitation;
    for dep in &depressions {
        let depth = dep.spill - dep.min;
        let full_area = dep.cells.len() as f64 * cell_area;
        if depth < input.min_lake_depth || full_area < input.min_lake_area {
            for &c in &dep.cells {
                z[c] = dep.spill;
                surface[c] = dep.spill;
            }
            filled_cells += dep.cells.len();
            continue;
        }
        let target_area = if deficit <= 0.0 {
            f64::INFINITY
        } else {
            dep.catchment * input.runoff / (input.runoff + deficit)
        };
        if target_area >= full_area {
            for &c in &dep.cells {
                is_lake[c] = true;
                lake_level[c] = dep.spill;
                lake_spill[c] = dep.spill;
            }
            continue;
        }
        let mut by_height: Vec<usize> = dep.cells.clone();
        by_height.sort_by(|&a, &b| z[a].total_cmp(&z[b]));
        let k = (target_area / cell_area).ceil() as usize;
        if k == 0 || (k as f64) * cell_area < input.min_lake_area {
            // Too dry for standing water: the basin floor is a terminal sink.
            sink[by_height[0]] = true;
            continue;
        }
        let k = k.min(by_height.len() - 1);
        let level = 0.5 * (z[by_height[k - 1]] + z[by_height[k]]);
        for &c in &dep.cells {
            if z[c] < level {
                is_lake[c] = true;
                lake_level[c] = level;
                lake_spill[c] = dep.spill;
                lake_closed[c] = true;
                sink[c] = true;
                surface[c] = level;
            }
        }
    }

    // Final routing: map edge and ocean drain off the map, closed lakes and dry basins are sinks.
    let seeds: Vec<bool> = (0..n).map(|i| outlets[i] || sink[i]).collect();
    let routed = priority_flood(&surface, w, h, &seeds, true);
    let receiver_index = steepest_receivers(&routed, w, h, &seeds, input.jitter);
    let order = stack_order(&receiver_index);
    let area = accumulate(&order, &receiver_index, cell_area, input.inflow);
    let receivers: Vec<u8> = (0..n)
        .map(|i| {
            if outlets[i] {
                RECEIVER_OUTLET
            } else if sink[i] {
                RECEIVER_SINK
            } else {
                direction_between(w, i, receiver_index[i] as usize).unwrap_or(RECEIVER_SINK)
            }
        })
        .collect();

    // Lakes as connected water bodies.
    let mut lake_id = vec![0u32; n];
    let mut lakes = Vec::new();
    let mut queue = VecDeque::new();
    for start in 0..n {
        if !is_lake[start] || lake_id[start] != 0 {
            continue;
        }
        let id = lakes.len() as u32 + 1;
        lake_id[start] = id;
        queue.push_back(start);
        let mut cells = Vec::new();
        while let Some(c) = queue.pop_front() {
            cells.push(c);
            for off in D8 {
                if let Some(nb) = neighbor(w, h, c, off)
                    && is_lake[nb]
                    && lake_id[nb] == 0
                {
                    lake_id[nb] = id;
                    queue.push_back(nb);
                }
            }
        }
        let level = lake_level[start];
        let closed = lake_closed[start];
        let volume: f64 = cells
            .iter()
            .map(|&c| (level - z[c]).max(0.0) * cell_area)
            .sum();
        // An open lake drains through the lake cell whose receiver is outside it and carries the
        // most water; a closed lake gathers everything that reaches any of its cells.
        let (outlet, catchment) = if closed {
            (None, cells.iter().map(|&c| area[c]).sum::<f64>())
        } else {
            let exit = cells
                .iter()
                .copied()
                .filter(|&c| {
                    !is_lake[receiver_index[c] as usize] || receiver_index[c] as usize == c
                })
                .max_by(|&a, &b| area[a].total_cmp(&area[b]));
            match exit {
                Some(c) => {
                    let r = receiver_index[c] as usize;
                    (if r == c { None } else { Some(r as u32) }, area[c])
                }
                None => (None, 0.0),
            }
        };
        let centroid = cells.iter().fold((0.0, 0.0), |acc, &c| {
            (acc.0 + (c % w) as f64, acc.1 + (c / w) as f64)
        });
        let count = cells.len() as f64;
        lakes.push(Lake {
            id,
            level_m: level as f32,
            spill_m: lake_spill[start] as f32,
            closed,
            cell_count: cells.len() as u32,
            area_m2: count * cell_area,
            volume_m3: volume,
            catchment_m2: catchment,
            outlet_cell: outlet,
            centroid_m: (
                ((centroid.0 / count + 0.5) * input.cell) as f32,
                ((centroid.1 / count + 0.5) * input.cell) as f32,
            ),
        });
    }

    let water: Vec<u8> = (0..n)
        .map(|i| {
            if ocean[i] {
                WATER_OCEAN
            } else if is_lake[i] {
                WATER_LAKE
            } else {
                0
            }
        })
        .collect();

    HydroOutput {
        bed: z,
        receivers,
        receiver_index,
        water,
        lake_id,
        lakes,
        area,
        filled_cells,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        w: usize,
        h: usize,
        precipitation: f64,
        evaporation: f64,
        inflow: &[f64],
    ) -> HydroInput<'_> {
        HydroInput {
            w,
            h,
            cell: 10.0,
            sea_level: -1000.0,
            runoff: 0.3,
            precipitation,
            lake_evaporation: evaporation,
            min_lake_depth: 0.5,
            min_lake_area: 150.0,
            inflow,
            jitter: Jitter::NONE,
        }
    }

    /// A 21x21 bowl: a sloping plain toward the west edge with a 9x9 pit in the middle.
    fn bowl() -> (usize, usize, Vec<f64>) {
        let (w, h) = (21, 21);
        let z = (0..w * h)
            .map(|i| {
                let (x, y) = ((i % w) as f64, (i / w) as f64);
                let plain = 10.0 + x * 0.5;
                let r = ((x - 10.0).powi(2) + (y - 10.0).powi(2)).sqrt();
                if r < 4.5 {
                    plain - 6.0 + r * 0.5
                } else {
                    plain
                }
            })
            .collect();
        (w, h, z)
    }

    #[test]
    fn a_humid_basin_holds_an_open_lake_that_drains() {
        let (w, h, z) = bowl();
        let out = finalize(z, &input(w, h, 1.0, 0.8, &[]));
        assert_eq!(out.lakes.len(), 1);
        let lake = &out.lakes[0];
        assert!(!lake.closed);
        assert!(lake.outlet_cell.is_some(), "an open lake has an outlet");
        assert!((lake.level_m - lake.spill_m).abs() < 1e-4);
        // Everything still reaches the map edge.
        let order = stack_order(&out.receiver_index);
        assert_eq!(order.len(), w * h);
    }

    #[test]
    fn an_arid_basin_holds_a_smaller_closed_lake() {
        let (w, h, z) = bowl();
        let humid = finalize(z.clone(), &input(w, h, 1.0, 0.8, &[]));
        let arid = finalize(z, &input(w, h, 0.2, 2.0, &[]));
        assert_eq!(arid.lakes.len(), 1);
        let lake = &arid.lakes[0];
        assert!(lake.closed);
        assert!(lake.outlet_cell.is_none());
        assert!(lake.level_m < lake.spill_m);
        assert!(lake.area_m2 < humid.lakes[0].area_m2);
        for (i, &id) in arid.lake_id.iter().enumerate() {
            if id != 0 {
                assert_eq!(arid.receivers[i], RECEIVER_SINK);
            }
        }
    }

    #[test]
    fn shallow_depressions_are_filled_not_flooded() {
        let (w, h) = (9, 9);
        let mut z: Vec<f64> = (0..w * h).map(|i| 5.0 + (i % w) as f64 * 0.1).collect();
        z[4 * w + 4] -= 0.2; // a 0.2 m dimple
        let out = finalize(z, &input(w, h, 1.0, 0.8, &[]));
        assert!(out.lakes.is_empty());
        assert_eq!(out.filled_cells, 1);
    }

    #[test]
    fn drainage_area_is_conserved_including_inflows() {
        let (w, h, z) = bowl();
        let mut inflow = vec![0.0; w * h];
        inflow[w + 18] = 5_000.0;
        let out = finalize(z, &input(w, h, 0.2, 2.0, &inflow));
        let total: f64 = (0..w * h)
            .filter(|&i| out.receiver_index[i] as usize == i)
            .map(|i| out.area[i])
            .sum();
        let expected = (w * h) as f64 * 100.0 + 5_000.0;
        assert!(
            (total - expected).abs() < 1e-6 * expected,
            "{total} vs {expected}"
        );
    }
}
