//! River hydraulics and floodplain storage (ADR-0022 §8; research 03-02 §1.2).
//!
//! Each reach has:
//! - Hydraulic geometry: width `w`, bed slope `S`, Manning roughness `n` (default 0.035).
//! - Bankfull discharge `Q_bf = 1.5 * Q_mean` and bankfull stage `d_bf = (n * Q_bf / (w * S^(1/2)))^(3/5)`.
//! - A floodplain compartment: all cells that drain along D8 flow paths into this reach before
//!   reaching any other reach.
//! - A precomputed elevation-storage curve from the Height Above Nearest Drainage (HAND) of its
//!   compartment cells, relating excess volume `V_excess = max(0, Q - Q_bf) * 86400` to flood
//!   stage `z_flood` above the channel bed.
//! - Per-cell flood depth `d_flood = max(0, z_flood - HAND)`.

use crate::{RECEIVER_OUTLET, RECEIVER_SINK, WATER_LAND, WorldMap};

/// Default Manning roughness coefficient for natural gravel/clean stream channels (s / m^(1/3)).
pub const DEFAULT_MANNING_N: f64 = 0.035;
/// Minimum bed gradient taken for flat sections or backwater (m/m).
pub const MIN_BED_SLOPE: f64 = 0.0005;
/// Bankfull discharge factor relative to mean annual discharge (research 03-02 §1.2).
pub const BANKFULL_FACTOR: f64 = 1.5;
/// Seconds in a day (for volume calculation from daily flow).
pub const SECONDS_PER_DAY: f64 = 86400.0;

/// Parameters for Manning hydraulics and bankfull thresholds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ManningParams {
    pub roughness_n: f64,
    pub min_slope: f64,
    pub bankfull_factor: f64,
}

impl Default for ManningParams {
    fn default() -> Self {
        ManningParams {
            roughness_n: DEFAULT_MANNING_N,
            min_slope: MIN_BED_SLOPE,
            bankfull_factor: BANKFULL_FACTOR,
        }
    }
}

/// A reach's hydraulic properties and its floodplain storage compartment.
#[derive(Clone, Debug, PartialEq)]
pub struct ReachHydraulics {
    pub reach_id: u32,
    /// Channel width, metres.
    pub width_m: f64,
    /// Bed slope S (m/m), clamped to `min_slope`.
    pub slope: f64,
    /// Manning roughness n.
    pub roughness_n: f64,
    /// Mean annual discharge, m³/s.
    pub mean_discharge_m3s: f64,
    /// Bankfull discharge, m³/s (`bankfull_factor * mean_discharge_m3s`).
    pub bankfull_discharge_m3s: f64,
    /// Bankfull stage in metres.
    pub bankfull_stage_m: f64,
    /// Flow transit time across the reach, seconds (`reach_length / velocity`).
    pub transit_time_s: f64,
    /// Compartment cells, sorted ascending by HAND.
    pub cells: Vec<u32>,
    /// Height above nearest drainage (metres) corresponding to `cells`.
    pub hands: Vec<f32>,
    /// Cumulative stored volume, m³, at each knot `hands[i]`.
    pub cumulative_v_m3: Vec<f64>,
}

impl ReachHydraulics {
    /// Channel stage (water depth in the channel, metres) for discharge `q_m3s` using Manning's equation.
    pub fn stage(&self, q_m3s: f64) -> f64 {
        if q_m3s <= 0.0 || self.width_m <= 0.0 || self.slope <= 0.0 {
            return 0.0;
        }
        let denom = self.width_m * self.slope.sqrt();
        if denom <= 0.0 {
            return 0.0;
        }
        let val = (self.roughness_n * q_m3s) / denom;
        val.max(0.0).powf(0.6)
    }

    /// Excess volume above bankfull for the reach at flow `q_m3s`, m³.
    pub fn excess_volume_m3(&self, q_m3s: f64) -> f64 {
        if q_m3s > self.bankfull_discharge_m3s {
            (q_m3s - self.bankfull_discharge_m3s) * self.transit_time_s
        } else {
            0.0
        }
    }

    /// Flood stage `z` (metres above channel bed) on the floodplain for excess volume `v_m3`.
    pub fn flood_stage_m(&self, v_m3: f64, cell_area_m2: f64) -> f64 {
        if v_m3 <= 0.0 || self.hands.is_empty() || cell_area_m2 <= 0.0 {
            return 0.0;
        }
        let n = self.hands.len();
        // If volume is beyond the highest knot, extrapolate with full compartment area.
        let max_v = self.cumulative_v_m3[n - 1];
        if v_m3 >= max_v {
            let total_area = n as f64 * cell_area_m2;
            return f64::from(self.hands[n - 1]) + (v_m3 - max_v) / total_area;
        }

        // Binary search for knot k where cumulative_v_m3[k] <= v_m3 < cumulative_v_m3[k + 1].
        let mut lo = 0usize;
        let mut hi = n - 1;
        while lo + 1 < hi {
            let mid = (lo + hi) / 2;
            if self.cumulative_v_m3[mid] <= v_m3 {
                lo = mid;
            } else {
                hi = mid;
            }
        }

        let base_h = f64::from(self.hands[lo]);
        let base_v = self.cumulative_v_m3[lo];
        let area = (lo + 1) as f64 * cell_area_m2;
        base_h + (v_m3 - base_v) / area
    }
}

/// Precomputed world-wide river hydraulics and floodplain storage curves.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WorldFloodplain {
    pub reaches: Vec<ReachHydraulics>,
    /// Which reach compartment each cell belongs to (`u32::MAX` if none).
    pub cell_reach: Vec<u32>,
    /// HAND for each cell in metres.
    pub cell_hand: Vec<f32>,
}

impl WorldFloodplain {
    /// Precomputes reach hydraulics and floodplain storage from a `WorldMap`.
    pub fn new(map: &WorldMap, params: &ManningParams) -> Self {
        let n = map.cell_count();
        let mut cell_reach = vec![u32::MAX; n];
        let mut cell_hand = vec![0.0f32; n];

        // 1. Build map of river cells to reach id.
        let mut reach_at = vec![u32::MAX; n];
        for reach in &map.reaches {
            for &c in &reach.cells {
                reach_at[c as usize] = reach.id;
            }
        }

        // 2. Trace D8 flow paths from every cell to find the first reach it drains into.
        let mut path: Vec<usize> = Vec::new();
        for start in 0..n {
            path.clear();
            let mut cur = start;
            let (target_reach, bed_elev) = loop {
                let r = reach_at[cur];
                if r != u32::MAX {
                    break (r, map.elevation[cur]);
                }
                let next = map.receiver_index(cur);
                let rec = map.receivers[cur];
                if rec == RECEIVER_OUTLET || rec == RECEIVER_SINK || next == cur {
                    break (u32::MAX, 0.0);
                }
                path.push(cur);
                cur = next;
                if path.len() > n {
                    break (u32::MAX, 0.0);
                }
            };

            if target_reach != u32::MAX {
                cell_reach[start] = target_reach;
                cell_hand[start] = (map.elevation[start] - bed_elev).max(0.0);
            }
        }

        // 3. Precompute ReachHydraulics for each reach.
        let cell_size = f64::from(map.cell_size_m);
        let cell_area = cell_size * cell_size;

        let reaches: Vec<ReachHydraulics> = map
            .reaches
            .iter()
            .map(|r| {
                let w = f64::from(r.width_m).max(1.0);
                let q_mean = f64::from(r.discharge_m3s);
                let q_bf = q_mean * params.bankfull_factor;
                let len_m = (r.cells.len() as f64) * cell_size;

                // Bed slope from start to end of reach.
                let slope = if r.cells.len() >= 2 {
                    let c_start = r.cells[0] as usize;
                    let c_end = r.cells[r.cells.len() - 1] as usize;
                    let dz = f64::from(map.elevation[c_start] - map.elevation[c_end]);
                    (dz / len_m).max(params.min_slope)
                } else {
                    params.min_slope
                };

                let denom = w * slope.sqrt();
                let bf_stage = if denom > 0.0 {
                    ((params.roughness_n * q_bf) / denom).max(0.0).powf(0.6)
                } else {
                    0.0
                };

                // Compartment cells draining into this reach.
                let mut comp_cells: Vec<(u32, f32)> = Vec::new();
                for (c, &cr) in cell_reach.iter().enumerate() {
                    if cr == r.id && map.water[c] == WATER_LAND {
                        comp_cells.push((c as u32, cell_hand[c]));
                    }
                }
                comp_cells.sort_by(|a, b| a.1.total_cmp(&b.1).then_with(|| a.0.cmp(&b.0)));

                let cells: Vec<u32> = comp_cells.iter().map(|p| p.0).collect();
                let hands: Vec<f32> = comp_cells.iter().map(|p| p.1).collect();

                // Compute cumulative volume knots.
                let mut cumulative_v_m3: Vec<f64> = Vec::with_capacity(hands.len());
                let mut cur_v = 0.0f64;
                for i in 0..hands.len() {
                    if i > 0 {
                        let dh = f64::from(hands[i] - hands[i - 1]);
                        let area = (i as f64) * cell_area;
                        cur_v += area * dh;
                    }
                    cumulative_v_m3.push(cur_v);
                }

                let v_bf = if w > 0.0 && bf_stage > 0.0 {
                    (q_bf / (w * bf_stage)).clamp(0.2, 5.0)
                } else {
                    1.0
                };
                let transit_time_s = (len_m / v_bf).max(10.0);

                ReachHydraulics {
                    reach_id: r.id,
                    width_m: w,
                    slope,
                    roughness_n: params.roughness_n,
                    mean_discharge_m3s: q_mean,
                    bankfull_discharge_m3s: q_bf,
                    bankfull_stage_m: bf_stage,
                    transit_time_s,
                    cells,
                    hands,
                    cumulative_v_m3,
                }
            })
            .collect();

        WorldFloodplain {
            reaches,
            cell_reach,
            cell_hand,
        }
    }

    /// Computes flood depths for inundated cells, returning `(cell, depth_m)` pairs.
    pub fn flood_depths(&self, map: &WorldMap, reach_discharges: &[f32]) -> Vec<(u32, f32)> {
        let cell_size = f64::from(map.cell_size_m);
        let cell_area = cell_size * cell_size;
        let mut inundated: Vec<(u32, f32)> = Vec::new();

        for rh in &self.reaches {
            let q = reach_discharges
                .get(rh.reach_id as usize)
                .copied()
                .map(f64::from)
                .unwrap_or(rh.mean_discharge_m3s);

            let excess_v = rh.excess_volume_m3(q);
            if excess_v <= 0.0 {
                continue;
            }

            let flood_z = rh.flood_stage_m(excess_v, cell_area);
            if flood_z <= 0.0 {
                continue;
            }

            for (&cell, &hand) in rh.cells.iter().zip(&rh.hands) {
                let h = f64::from(hand);
                if flood_z > h {
                    let depth = (flood_z - h) as f32;
                    if depth > 0.0 {
                        inundated.push((cell, depth));
                    }
                } else {
                    // Since cells and hands are sorted ascending, no further cells in this reach are flooded.
                    break;
                }
            }
        }

        inundated
    }

    /// Flood storage connected to a reach through ground and optional bank crests. Crests
    /// are metres above the same drainage datum as HAND, not absolute terrain elevations.
    /// An edge is crossed only above its crest. End runs remain possible through other edges.
    /// All excess volume is placed in connected cells; banks never delete floodwater.
    pub fn flood_depths_with_banks(
        &self,
        map: &WorldMap,
        reach_discharges: &[f32],
        banks: &[FloodBarrier],
    ) -> Vec<f32> {
        if banks.is_empty() {
            return self.flood_depth_raster(map, reach_discharges);
        }
        connected_depths(self, map, reach_discharges, banks)
    }

    /// Computes dense flood depths for all cells in `map`, metres.
    pub fn flood_depth_raster(&self, map: &WorldMap, reach_discharges: &[f32]) -> Vec<f32> {
        let n = map.cell_count();
        let mut raster = vec![0.0f32; n];
        for (cell, depth) in self.flood_depths(map, reach_discharges) {
            if let Some(d) = raster.get_mut(cell as usize) {
                *d = depth;
            }
        }
        raster
    }
}

/// A sub-grid bank crossing an edge between neighbouring cells. Below the crest there
/// is no passage across this edge; a breach is represented by removing the barrier.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FloodBarrier {
    pub a: u32,
    pub b: u32,
    pub crest_hand_m: f64,
}

fn connected_depths(
    floodplain: &WorldFloodplain,
    map: &WorldMap,
    discharges: &[f32],
    banks: &[FloodBarrier],
) -> Vec<f32> {
    use std::cmp::Reverse;
    use std::collections::{BTreeMap, BinaryHeap};
    let n = map.cell_count();
    let mut raster = vec![0.0; n];
    let area = f64::from(map.cell_size_m).powi(2);
    let mut edges = BTreeMap::<(u32, u32), f64>::new();
    for b in banks {
        let key = (b.a.min(b.b), b.a.max(b.b));
        edges
            .entry(key)
            .and_modify(|h| *h = h.max(b.crest_hand_m))
            .or_insert(b.crest_hand_m);
    }
    // A minimax path gives the least stage that can connect a cell to its channel.
    // Four-neighbour paths prevent slipping diagonally between touching bank edges.
    for rh in &floodplain.reaches {
        let q = discharges
            .get(rh.reach_id as usize)
            .copied()
            .map(f64::from)
            .unwrap_or(rh.mean_discharge_m3s);
        let volume = rh.excess_volume_m3(q);
        if volume <= 0.0 {
            continue;
        }
        if !banks.iter().any(|b| {
            floodplain.cell_reach.get(b.a as usize) == Some(&rh.reach_id)
                || floodplain.cell_reach.get(b.b as usize) == Some(&rh.reach_id)
        }) {
            let stage = rh.flood_stage_m(volume, area);
            for (&c, &hand) in rh.cells.iter().zip(&rh.hands) {
                raster[c as usize] = (stage - f64::from(hand)).max(0.0) as f32;
            }
            continue;
        }
        let mut entry = vec![f64::INFINITY; n];
        let mut queue = BinaryHeap::new();
        if let Some(reach) = map.reaches.iter().find(|r| r.id == rh.reach_id) {
            for &c in &reach.cells {
                entry[c as usize] = 0.0;
                queue.push(Reverse((0_u64, c)));
            }
        }
        while let Some(Reverse((bits, c))) = queue.pop() {
            let stage = f64::from_bits(bits);
            if stage > entry[c as usize] {
                continue;
            }
            let x = c % map.width;
            let y = c / map.width;
            let neighbours = [
                (x > 0).then(|| c - 1),
                (x + 1 < map.width).then(|| c + 1),
                (y > 0).then(|| c - map.width),
                (y + 1 < map.height).then(|| c + map.width),
            ];
            for to in neighbours.into_iter().flatten() {
                if floodplain.cell_reach[to as usize] != rh.reach_id {
                    continue;
                }
                let crest = edges.get(&(c.min(to), c.max(to))).copied().unwrap_or(0.0);
                let next = stage
                    .max(crest)
                    .max(f64::from(floodplain.cell_hand[to as usize]));
                if next < entry[to as usize] {
                    entry[to as usize] = next;
                    queue.push(Reverse((next.to_bits(), to)));
                }
            }
        }
        // Filling a newly connected basin can consume volume while stage stays at its
        // sill. At that knot, partial filling avoids manufacturing or discarding water.
        let mut cells: Vec<_> = rh.cells.iter().map(|&c| (entry[c as usize], c)).collect();
        cells.retain(|(h, _)| h.is_finite());
        cells.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        let mut left = volume;
        let mut level = 0.0_f64;
        let mut active = Vec::<u32>::new();
        let mut i = 0;
        while i < cells.len() && left > 0.0 {
            let sill = cells[i].0;
            let rise = (sill - level).max(0.0) * active.len() as f64 * area;
            if rise >= left && !active.is_empty() {
                level += left / (active.len() as f64 * area);
                left = 0.0;
                break;
            }
            left -= rise;
            level = sill;
            let first = i;
            while i < cells.len() && cells[i].0 == sill {
                i += 1;
            }
            let new = &cells[first..i];
            let fill: f64 = new
                .iter()
                .map(|&(_, c)| {
                    (level - f64::from(floodplain.cell_hand[c as usize])).max(0.0) * area
                })
                .sum();
            if fill > left {
                let share = left / fill;
                for &(_, c) in new {
                    raster[c as usize] = ((level - f64::from(floodplain.cell_hand[c as usize]))
                        .max(0.0)
                        * share) as f32;
                }
                left = 0.0;
                break;
            }
            left -= fill;
            active.extend(new.iter().map(|&(_, c)| c));
        }
        if left > 0.0 && !active.is_empty() {
            level += left / (active.len() as f64 * area);
        }
        for c in active {
            raster[c as usize] =
                (level - f64::from(floodplain.cell_hand[c as usize])).max(0.0) as f32;
        }
    }
    raster
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Climate, RiverReach, Terminus, WATER_RIVER};

    fn test_map(w: u32, h: u32) -> WorldMap {
        let n = (w * h) as usize;
        let mut elevation = vec![0.0f32; n];
        let mut receivers = vec![4u8; n]; // West (D8 code 4)
        let mut water = vec![WATER_LAND; n];

        // Column 0 is a river reach flowing south (D8 code 2).
        for y in 0..h {
            for x in 0..w {
                let idx = (y * w + x) as usize;
                elevation[idx] = 10.0 + x as f32 * 0.5 - y as f32 * 0.01;
                if x == 0 {
                    water[idx] = WATER_RIVER;
                    receivers[idx] = if y + 1 < h { 2 } else { RECEIVER_OUTLET };
                }
            }
        }

        let reach_cells: Vec<u32> = (0..h).map(|y| y * w).collect();
        let reach = RiverReach {
            id: 0,
            cells: reach_cells,
            downstream: None,
            terminus: Terminus::Outlet,
            order: 1,
            discharge_m3s: 2.0,
            width_m: 6.0,
            drainage_area_km2: 10.0,
        };

        WorldMap {
            width: w,
            height: h,
            cell_size_m: 10.0,
            sea_level_m: 0.0,
            elevation,
            receivers,
            water,
            lake_id: vec![0; n],
            lakes: Vec::new(),
            reaches: vec![reach],
            inflows: Vec::new(),
            climate: Climate {
                precipitation_mm_per_yr: 1000.0,
                evapotranspiration_mm_per_yr: 500.0,
                lake_evaporation_mm_per_yr: 600.0,
            },
            drainage_area_m2: vec![0.0; n],
        }
    }

    #[test]
    fn a_bank_redistributes_water_until_it_overtops_and_can_be_run_around() {
        let mut map = test_map(6, 4);
        // Flat floodplain beside a straight channel: no ground sill hides a bank effect.
        for y in 0..4 {
            for x in 1..6 {
                map.elevation[y * 6 + x] = map.elevation[y * 6];
            }
        }
        let fp = WorldFloodplain::new(&map, &ManningParams::default());
        let rh = &fp.reaches[0];
        let q = |volume: f64| [(rh.bankfull_discharge_m3s + volume / rh.transit_time_s) as f32];
        let bank: Vec<_> = (0..4)
            .map(|y| FloodBarrier {
                a: y * 6 + 1,
                b: y * 6 + 2,
                crest_hand_m: 2.0,
            })
            .collect();
        let depths = fp.flood_depths_with_banks(&map, &q(400.0), &bank);
        assert!((depths.iter().map(|d| f64::from(*d) * 100.0).sum::<f64>() - 400.0).abs() < 0.01);
        assert!(depths[1] > 0.9);
        assert_eq!(depths[2], 0.0);
        let overtopped = fp.flood_depths_with_banks(&map, &q(2000.0), &bank);
        assert!(overtopped[2] > 0.0);
        assert!(
            (overtopped
                .iter()
                .map(|d| f64::from(*d) * 100.0)
                .sum::<f64>()
                - 2000.0)
                .abs()
                < 0.01
        );
        let short = fp.flood_depths_with_banks(&map, &q(400.0), &bank[..3]);
        assert!(
            short[2] > 0.0,
            "an open end admits water behind a sound bank"
        );
        assert!((short.iter().map(|d| f64::from(*d) * 100.0).sum::<f64>() - 400.0).abs() < 0.01);
        assert_eq!(
            fp.flood_depths_with_banks(&map, &q(400.0), &[]),
            fp.flood_depth_raster(&map, &q(400.0))
        );
    }

    #[test]
    fn bankfull_discharge_and_stage_follow_manning() {
        let map = test_map(10, 10);
        let params = ManningParams::default();
        let fp = WorldFloodplain::new(&map, &params);

        assert_eq!(fp.reaches.len(), 1);
        let rh = &fp.reaches[0];
        assert_eq!(rh.mean_discharge_m3s, 2.0);
        assert_eq!(rh.bankfull_discharge_m3s, 3.0); // 1.5 * 2.0
        assert!(rh.bankfull_stage_m > 0.0);

        // Under bankfull flow: no excess volume.
        assert_eq!(rh.excess_volume_m3(2.5), 0.0);

        // Above bankfull flow: excess volume conserved.
        let q = 4.0;
        let excess_v = rh.excess_volume_m3(q);
        assert_eq!(excess_v, 1.0 * rh.transit_time_s);
    }

    #[test]
    fn storage_curve_conserves_flood_volume() {
        let map = test_map(10, 10);
        let params = ManningParams::default();
        let fp = WorldFloodplain::new(&map, &params);
        let rh = &fp.reaches[0];
        let cell_area = 100.0; // 10m x 10m

        // Choose an excess volume that floods some cells.
        let target_v = 5000.0;
        let z = rh.flood_stage_m(target_v, cell_area);
        assert!(z > 0.0);

        // Integrate volume over flooded compartment cells.
        let mut total_v = 0.0;
        for &h in &rh.hands {
            let h = f64::from(h);
            if z > h {
                total_v += (z - h) * cell_area;
            }
        }
        assert!(
            (total_v - target_v).abs() < 1e-6,
            "integrated volume {total_v} must match target volume {target_v}"
        );
    }

    #[test]
    fn flood_depths_identify_inundated_cells() {
        let map = test_map(10, 10);
        let params = ManningParams::default();
        let fp = WorldFloodplain::new(&map, &params);

        // Flow below bankfull: zero flooded cells.
        let depths = fp.flood_depths(&map, &[2.0]);
        assert!(depths.is_empty());

        // Flow above bankfull: produces flooded cells with positive depth.
        let depths = fp.flood_depths(&map, &[5.0]);
        assert!(!depths.is_empty());
        for &(cell, d) in &depths {
            assert!(d > 0.0);
            assert_eq!(map.water[cell as usize], WATER_LAND);
        }
    }
}
