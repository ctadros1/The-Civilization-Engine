//! Terrain measures derived from a map: slope, and height above the nearest drainage (HAND).
//!
//! Both are pure functions of the authoritative rasters, so they are rebuilt on load, never saved.

use crate::grid::{D8, D8_DIST};
use crate::{WATER_LAND, WorldMap};

/// Steepest slope from each cell to any neighbour, rise over run (0 for flat ground).
pub fn slopes(map: &WorldMap) -> Vec<f32> {
    (0..map.cell_count()).map(|i| slope_at(map, i)).collect()
}

/// [`slopes`] of one cell.
pub fn slope_at(map: &WorldMap, i: usize) -> f32 {
    let (w, h) = (map.width as usize, map.height as usize);
    let c = f64::from(map.cell_size_m);
    let (x, y) = ((i % w) as i64, (i / w) as i64);
    let z = f64::from(map.elevation[i]);
    let mut best = 0.0f64;
    for (&(dx, dy), dist) in D8.iter().zip(D8_DIST) {
        let (nx, ny) = (x + i64::from(dx), y + i64::from(dy));
        if nx < 0 || ny < 0 || nx >= w as i64 || ny >= h as i64 {
            continue;
        }
        let j = (ny * w as i64 + nx) as usize;
        let rise = (f64::from(map.elevation[j]) - z).abs();
        best = best.max(rise / (dist * c));
    }
    best as f32
}

/// [`height_above_drainage`] of one cell, by walking downstream from it.
pub fn hand_at(map: &WorldMap, start: usize, channel_area_m2: f32) -> f32 {
    let mut cur = start;
    for _ in 0..map.cell_count() {
        let is_drain = map.water[cur] != WATER_LAND
            || map
                .drainage_area_m2
                .get(cur)
                .is_some_and(|&a| a >= channel_area_m2);
        let next = map.receiver_index(cur);
        if is_drain || next == cur {
            break;
        }
        cur = next;
    }
    (map.elevation[start] - map.elevation[cur]).max(0.0)
}

/// Height of each cell above the drainage it flows to (HAND), metres: follow the D8 receivers
/// downstream to the first cell that is water or carries at least `channel_area_m2` of drainage,
/// and subtract its elevation. Water cells are 0. Cells that drain off the map or into a dry sink
/// without meeting a channel measure from that last cell.
pub fn height_above_drainage(map: &WorldMap, channel_area_m2: f32) -> Vec<f32> {
    let n = map.cell_count();
    let mut hand = vec![f32::NAN; n];
    let mut stack: Vec<usize> = Vec::new();
    for start in 0..n {
        if !hand[start].is_nan() {
            continue;
        }
        // Walk downstream until a cell with a known HAND or a drainage cell.
        let mut cur = start;
        loop {
            if !hand[cur].is_nan() {
                break;
            }
            let is_drain = map.water[cur] != WATER_LAND
                || map
                    .drainage_area_m2
                    .get(cur)
                    .is_some_and(|&a| a >= channel_area_m2);
            if is_drain {
                hand[cur] = 0.0;
                break;
            }
            let next = map.receiver_index(cur);
            if next == cur {
                // Outlet or sink on dry land: it is its own base.
                hand[cur] = 0.0;
                break;
            }
            stack.push(cur);
            cur = next;
        }
        // Unwind: every cell on the path measures from the base's drainage elevation.
        let base_level = map.elevation[cur] - hand[cur];
        while let Some(c) = stack.pop() {
            hand[c] = (map.elevation[c] - base_level).max(0.0);
        }
    }
    hand
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Climate, RECEIVER_OUTLET, WATER_RIVER};

    fn ramp(w: u32, h: u32) -> WorldMap {
        // Elevation rises 1 m per cell eastwards; everything drains west; column 0 is a river.
        let n = (w * h) as usize;
        let mut elevation = vec![0.0; n];
        let mut receivers = vec![4u8; n]; // D8 code 4 = west
        let mut water = vec![WATER_LAND; n];
        for i in 0..n {
            let x = i % w as usize;
            elevation[i] = 10.0 + x as f32;
            if x == 0 {
                receivers[i] = RECEIVER_OUTLET;
                water[i] = WATER_RIVER;
            }
        }
        WorldMap {
            width: w,
            height: h,
            cell_size_m: 8.0,
            sea_level_m: 0.0,
            elevation,
            receivers,
            water,
            lake_id: vec![0; n],
            lakes: Vec::new(),
            reaches: Vec::new(),
            inflows: Vec::new(),
            climate: Climate {
                precipitation_mm_per_yr: 800.0,
                evapotranspiration_mm_per_yr: 500.0,
                lake_evaporation_mm_per_yr: 900.0,
            },
            drainage_area_m2: vec![64.0; n],
        }
    }

    #[test]
    fn slope_is_rise_over_run() {
        let map = ramp(8, 4);
        let s = slopes(&map);
        // A diagonal step rises 1 m over 8·√2 m; a straight step rises 1 m over 8 m: steepest wins.
        assert!((s[3 * 8 + 4] - 1.0 / 8.0).abs() < 1e-6);
    }

    #[test]
    fn hand_measures_from_the_river() {
        let map = ramp(8, 4);
        let hand = height_above_drainage(&map, f32::MAX);
        for (i, &h) in hand.iter().enumerate() {
            let x = i % 8;
            assert!((h - x as f32).abs() < 1e-6, "cell {i}: {h}");
        }
        // A smaller channel threshold makes every cell a channel.
        let all = height_above_drainage(&map, 1.0);
        assert!(all.iter().all(|&h| h == 0.0));
    }

    #[test]
    fn one_cell_measures_as_the_whole_map_does() {
        let map = ramp(8, 4);
        let (s, hand) = (slopes(&map), height_above_drainage(&map, f32::MAX));
        for i in 0..map.cell_count() {
            assert_eq!(slope_at(&map, i), s[i]);
            assert_eq!(hand_at(&map, i, f32::MAX), hand[i]);
            assert_eq!(hand_at(&map, i, 1.0), 0.0);
        }
    }
}
