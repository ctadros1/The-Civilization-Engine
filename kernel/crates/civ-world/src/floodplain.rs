//! Alluvial valley floors.
//!
//! Incision alone carves "sharp ridges separated by narrow grooves" (research 03-01 §1.1); real
//! river valleys have flat sedimentary floors, which is also where most usable land is. This is
//! an engineering approximation, not a sediment-transport model.
//!
//! It works like Height Above Nearest Drainage (Rennó et al. 2008). Every cell follows its own
//! flow path down to the first river that drains at least `floodplain_min_area_km2`. That river
//! is its reference, and the flow-path length is its distance from it. Cells within the river's
//! floodplain half-width (a multiple of its hydraulic-geometry channel width) are lowered toward
//! the river's depression-filled profile plus a gentle cross-valley slope. The edge blends
//! smoothly into the valley walls.
//!
//! Because each cell is lowered toward the channel it actually drains into, valley floors respect
//! watersheds and a tributary is never left standing above its trunk's floor.

use crate::SECONDS_PER_YEAR;
use crate::flood::priority_flood;
use crate::grid::step_length;
use crate::lem::Landscape;
use crate::params::TerrainParams;

/// Cross-valley slope of a floodplain (m/m): enough to drain toward the river.
const CROSS_SLOPE: f64 = 0.002;

/// Lowers ground beside large rivers into valley floors. Returns the number of cells lowered.
pub(crate) fn carve_floodplains(ls: &mut Landscape, params: &TerrainParams) -> usize {
    if params.floodplain_width_factor <= 0.0 {
        return 0;
    }
    let routing = ls.route();
    let area = &routing.area;
    let (w, h, cell) = (ls.w, ls.h, ls.cell);
    let n = w * h;
    let runoff = params.runoff_m_per_yr();
    let half_width = |a: f64| {
        let q = a * runoff / SECONDS_PER_YEAR;
        params.floodplain_width_factor * params.channel_width_coefficient * q.sqrt()
    };
    let min_area = params.floodplain_min_area_km2 * 1.0e6;
    // Valley floors follow the depression-filled profile of their river, which never rises
    // downstream; following the raw bed would widen every dip in the channel into a lake.
    let profile = priority_flood(&ls.z, w, h, &ls.base, false);

    // Walk the drainage tree from the outlets upward, so every receiver is resolved before the
    // cells that drain into it.
    let mut source = vec![u32::MAX; n];
    let mut dist = vec![0.0f64; n];
    for &i in &routing.order {
        let i = i as usize;
        if !ls.base[i] && area[i] >= min_area && half_width(area[i]) >= cell {
            source[i] = i as u32;
            continue;
        }
        let r = routing.receivers[i] as usize;
        if r == i || source[r] == u32::MAX {
            continue;
        }
        let s = source[r] as usize;
        let d = dist[r] + step_length(w, i, r, cell);
        if d <= half_width(area[s]) {
            source[i] = s as u32;
            dist[i] = d;
        }
    }

    let mut lowered = 0;
    for i in 0..n {
        let s = source[i];
        if s == u32::MAX || ls.base[i] || s as usize == i {
            continue;
        }
        let s = s as usize;
        let target = profile[s] + CROSS_SLOPE * dist[i];
        if ls.z[i] > target {
            let t = ((dist[i] / half_width(area[s]) - 0.55) / 0.45).clamp(0.0, 1.0);
            let blend = 1.0 - t * t * (3.0 - 2.0 * t);
            ls.z[i] -= (ls.z[i] - target) * blend;
            lowered += 1;
        }
    }
    lowered
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::is_edge;

    /// A V-shaped valley draining south with a large inflow at its head.
    fn valley() -> Landscape {
        let (w, h) = (41, 60);
        let cell = 10.0;
        let z: Vec<f64> = (0..w * h)
            .map(|i| {
                let (x, y) = ((i % w) as f64, (i / w) as f64);
                100.0 - y * 0.5 + (x - 20.0).abs() * 2.0
            })
            .collect();
        let mut inflow = vec![0.0; w * h];
        inflow[w + 20] = 50.0e6; // 50 km² enters at the head of the valley
        Landscape {
            w,
            h,
            cell,
            origin: (0.0, 0.0),
            z,
            uplift: vec![0.0; w * h],
            erodibility: vec![0.0; w * h],
            base: (0..w * h).map(|i| is_edge(w, h, i)).collect(),
            inflow,
            routing_seed: 0,
            steps: 0,
            jitter: 0.0,
            endorheic_depth: 0.0,
        }
    }

    #[test]
    fn a_large_river_gets_a_flat_floor_and_walls_stay() {
        let mut ls = valley();
        let before = ls.z.clone();
        let params = TerrainParams::default();
        let lowered = carve_floodplains(&mut ls, &params);
        assert!(lowered > 0);
        let (w, row) = (ls.w, 30);
        let centre = ls.z[row * w + 20];
        // Near the river the floor is within a metre of the channel; far up the wall nothing moved.
        assert!((ls.z[row * w + 22] - centre).abs() < 1.0);
        assert_eq!(ls.z[row * w + 1], before[row * w + 1]);
        // Nothing was raised.
        assert!(ls.z.iter().zip(&before).all(|(a, b)| a <= b));
    }

    #[test]
    fn disabled_floodplains_change_nothing() {
        let mut ls = valley();
        let before = ls.z.clone();
        let params = TerrainParams {
            floodplain_width_factor: 0.0,
            ..TerrainParams::default()
        };
        assert_eq!(carve_floodplains(&mut ls, &params), 0);
        assert_eq!(ls.z, before);
    }
}
