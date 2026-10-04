//! River reaches extracted from the drainage network (research 03-01 §3.4).
//!
//! A channel begins where drainage area passes the preset's threshold. Reaches run between channel
//! heads, confluences, lake inlets and outlets, and the map's outlets. Order is Strahler's.
//! Discharge is mean annual flow from the catchment water balance, `Q = A·runoff` (research 03-02
//! §1.1). Width follows hydraulic geometry, `w = a·Q^0.5`; the coefficient is a tuning starting
//! point, not a calibration.

use crate::{
    Lake, RECEIVER_OUTLET, RECEIVER_SINK, RiverReach, SECONDS_PER_YEAR, Terminus, WATER_LAKE,
    WATER_OCEAN, WATER_RIVER,
};

/// Inputs to reach extraction.
pub(crate) struct RiverInput<'a> {
    pub w: usize,
    pub receivers: &'a [u8],
    pub receiver_index: &'a [u32],
    pub area: &'a [f64],
    pub lake_id: &'a [u32],
    pub lakes: &'a [Lake],
    pub threshold_area: f64,
    pub runoff: f64,
    pub width_coefficient: f64,
}

/// Extracts reaches and marks channel cells as rivers in `water`.
pub(crate) fn extract(input: &RiverInput, water: &mut [u8]) -> Vec<RiverReach> {
    let n = input.receivers.len();
    let channel: Vec<bool> = (0..n)
        .map(|i| water[i] == 0 && input.area[i] >= input.threshold_area)
        .collect();

    let mut channel_donors = vec![0u8; n];
    let mut lake_fed = vec![false; n];
    for i in 0..n {
        let r = input.receiver_index[i] as usize;
        if r == i {
            continue;
        }
        if channel[i] {
            channel_donors[r] = channel_donors[r].saturating_add(1);
        }
        if water[i] == WATER_LAKE && channel[r] {
            lake_fed[r] = true;
        }
    }

    let mut starts: Vec<usize> = (0..n)
        .filter(|&i| channel[i] && (channel_donors[i] != 1 || lake_fed[i]))
        .collect();
    // Drainage area grows strictly downstream, so ascending area is a topological order.
    starts.sort_by(|&a, &b| input.area[a].total_cmp(&input.area[b]).then(a.cmp(&b)));
    let mut reach_at = vec![u32::MAX; n];
    for (id, &s) in starts.iter().enumerate() {
        reach_at[s] = id as u32;
    }

    let mut reaches: Vec<RiverReach> = Vec::with_capacity(starts.len());
    for (id, &start) in starts.iter().enumerate() {
        let mut cells = vec![start as u32];
        let mut cur = start;
        let (terminus, downstream) = loop {
            match input.receivers[cur] {
                RECEIVER_OUTLET => break (Terminus::Outlet, None),
                RECEIVER_SINK => break (Terminus::Sink, None),
                _ => {}
            }
            let r = input.receiver_index[cur] as usize;
            if water[r] == WATER_LAKE {
                cells.push(r as u32);
                break (Terminus::Lake(input.lake_id[r]), None);
            }
            if water[r] == WATER_OCEAN {
                cells.push(r as u32);
                break (Terminus::Outlet, None);
            }
            if reach_at[r] != u32::MAX {
                cells.push(r as u32);
                break (Terminus::Junction, Some(reach_at[r]));
            }
            if !channel[r] {
                // Cannot happen with strictly growing area; end the reach rather than wander.
                break (Terminus::Outlet, None);
            }
            cells.push(r as u32);
            cur = r;
        };
        // Discharge where the reach leaves land (the last channel cell).
        let last_land = cells
            .iter()
            .rev()
            .map(|&c| c as usize)
            .find(|&c| channel[c])
            .unwrap_or(start);
        let area = input.area[last_land];
        let discharge = area * input.runoff / SECONDS_PER_YEAR;
        reaches.push(RiverReach {
            id: id as u32,
            cells,
            downstream,
            terminus,
            order: 1,
            discharge_m3s: discharge as f32,
            width_m: (input.width_coefficient * discharge.sqrt()) as f32,
            drainage_area_km2: (area / 1.0e6) as f32,
        });
    }

    // Strahler order, upstream first. A lake passes on the highest order that enters it.
    let mut upstream: Vec<Vec<u32>> = vec![Vec::new(); reaches.len()];
    let mut lake_outflow_reach = vec![u32::MAX; input.lakes.len() + 1];
    for lake in input.lakes {
        if let Some(cell) = lake.outlet_cell
            && reach_at[cell as usize] != u32::MAX
        {
            lake_outflow_reach[lake.id as usize] = reach_at[cell as usize];
        }
    }
    for reach in &reaches {
        let target = match reach.terminus {
            Terminus::Junction => reach.downstream,
            Terminus::Lake(lake) => match lake_outflow_reach.get(lake as usize) {
                Some(&r) if r != u32::MAX => Some(r),
                _ => None,
            },
            _ => None,
        };
        if let Some(t) = target {
            upstream[t as usize].push(reach.id);
        }
    }
    for id in 0..reaches.len() {
        let orders: Vec<u8> = upstream[id]
            .iter()
            .map(|&u| reaches[u as usize].order)
            .collect();
        let order = match orders.iter().copied().max() {
            None => 1,
            Some(max) => {
                let joined = matches!(reaches[id].cells.first(), Some(&c) if channel_donors[c as usize] >= 2);
                if joined && orders.iter().filter(|&&o| o == max).count() >= 2 {
                    max.saturating_add(1)
                } else {
                    max
                }
            }
        };
        reaches[id].order = order;
    }

    for (i, w) in water.iter_mut().enumerate() {
        if channel[i] {
            *w = WATER_RIVER;
        }
    }
    let _ = input.w;
    reaches
}

/// Douglas–Peucker simplification of a polyline, keeping both ends.
pub fn simplify(points: &[(f32, f32)], tolerance: f32) -> Vec<(f32, f32)> {
    if points.len() <= 2 {
        return points.to_vec();
    }
    let mut keep = vec![false; points.len()];
    keep[0] = true;
    keep[points.len() - 1] = true;
    let mut stack = vec![(0usize, points.len() - 1)];
    while let Some((a, b)) = stack.pop() {
        if b <= a + 1 {
            continue;
        }
        let (ax, ay) = points[a];
        let (bx, by) = points[b];
        let (dx, dy) = (bx - ax, by - ay);
        let len = (dx * dx + dy * dy).sqrt();
        let mut worst = (0.0f32, a);
        for (k, &(px, py)) in points.iter().enumerate().take(b).skip(a + 1) {
            let d = if len > 0.0 {
                ((px - ax) * dy - (py - ay) * dx).abs() / len
            } else {
                ((px - ax).powi(2) + (py - ay).powi(2)).sqrt()
            };
            if d > worst.0 {
                worst = (d, k);
            }
        }
        if worst.0 > tolerance {
            keep[worst.1] = true;
            stack.push((a, worst.1));
            stack.push((worst.1, b));
        }
    }
    points
        .iter()
        .zip(keep)
        .filter_map(|(&p, k)| k.then_some(p))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simplification_keeps_corners_and_drops_collinear_points() {
        let line: Vec<(f32, f32)> = (0..10)
            .map(|i| (i as f32, 0.0))
            .chain((1..10).map(|i| (9.0, i as f32)))
            .collect();
        let s = simplify(&line, 0.1);
        assert_eq!(s, vec![(0.0, 0.0), (9.0, 0.0), (9.0, 9.0)]);
    }

    #[test]
    fn a_confluence_raises_strahler_order() {
        // Two first-order streams (cells 0 and 2) join at cell 4, which flows to outlet cell 5.
        //   0 -> 1 -> 4 -> 5(outlet)
        //   2 -> 3 -> 4
        let receivers_index: Vec<u32> = vec![1, 4, 3, 4, 5, 5];
        let receivers: Vec<u8> = vec![0, 0, 0, 0, 0, RECEIVER_OUTLET];
        let area = vec![1.0, 2.0, 1.0, 2.0, 5.0, 6.0];
        let mut water = vec![0u8; 6];
        let input = RiverInput {
            w: 6,
            receivers: &receivers,
            receiver_index: &receivers_index,
            area: &area,
            lake_id: &[0; 6],
            lakes: &[],
            threshold_area: 1.0,
            runoff: 1.0,
            width_coefficient: 4.0,
        };
        let reaches = extract(&input, &mut water);
        assert_eq!(reaches.len(), 3);
        let main = reaches.iter().find(|r| r.cells[0] == 4).expect("trunk");
        assert_eq!(main.order, 2);
        assert_eq!(main.terminus, Terminus::Outlet);
        let tribs: Vec<&RiverReach> = reaches.iter().filter(|r| r.cells[0] != 4).collect();
        assert!(
            tribs
                .iter()
                .all(|r| r.order == 1 && r.downstream == Some(main.id))
        );
        assert!(water.iter().all(|&w| w == WATER_RIVER));
    }
}
