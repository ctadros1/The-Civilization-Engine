//! World invariants (research 03-01 §5.7). A violation is a generator or loader bug, never
//! something to repair quietly.

use crate::grid::{D8, is_edge, neighbor};
use crate::routing::{accumulate, stack_order};
use crate::{RECEIVER_OUTLET, RECEIVER_SINK, Terminus, WATER_LAKE, WATER_OCEAN, WorldMap};

/// Checks a map's structural and hydrological invariants. Returns a description of each problem;
/// an empty list means the map is sound.
pub fn validate(map: &WorldMap) -> Vec<String> {
    let mut problems = Vec::new();
    let (w, h) = (map.width as usize, map.height as usize);
    let n = w * h;
    let sizes = [
        ("elevation", map.elevation.len()),
        ("receivers", map.receivers.len()),
        ("water", map.water.len()),
        ("lake_id", map.lake_id.len()),
    ];
    for (name, len) in sizes {
        if len != n {
            problems.push(format!("{name} has {len} cells, expected {n}"));
        }
    }
    if !problems.is_empty() {
        return problems;
    }
    if !(map.cell_size_m.is_finite() && map.cell_size_m > 0.0) {
        problems.push("cell size is not a positive number".to_owned());
        return problems;
    }

    let bad_elevations = map.elevation.iter().filter(|v| !v.is_finite()).count();
    if bad_elevations > 0 {
        problems.push(format!("{bad_elevations} elevations are not finite"));
    }

    let mut receiver_index = vec![0u32; n];
    for (i, slot) in receiver_index.iter_mut().enumerate() {
        let code = map.receivers[i];
        *slot = match code {
            RECEIVER_OUTLET | RECEIVER_SINK => i as u32,
            0..=7 => match neighbor(w, h, i, D8[code as usize]) {
                Some(nb) => nb as u32,
                None => {
                    problems.push(format!(
                        "cell {i} drains off the grid without being an outlet"
                    ));
                    i as u32
                }
            },
            _ => {
                problems.push(format!("cell {i} has invalid receiver code {code}"));
                i as u32
            }
        };
        if map.water[i] == WATER_OCEAN && code != RECEIVER_OUTLET {
            problems.push(format!("ocean cell {i} is not an outlet"));
        }
        if code == RECEIVER_OUTLET && !is_edge(w, h, i) && map.water[i] != WATER_OCEAN {
            problems.push(format!(
                "cell {i} drains off the map but is neither edge nor ocean"
            ));
        }
        if map.water[i] > WATER_OCEAN {
            problems.push(format!("cell {i} has invalid water class {}", map.water[i]));
        }
        if map.water[i] == WATER_OCEAN && map.elevation[i] >= map.sea_level_m {
            problems.push(format!("ocean cell {i} is above sea level"));
        }
    }

    let order = stack_order(&receiver_index);
    if order.len() != n {
        problems.push(format!(
            "{} cells never reach an outlet or sink (receiver cycle)",
            n - order.len()
        ));
    } else {
        let cell_area = f64::from(map.cell_size_m) * f64::from(map.cell_size_m);
        let mut extra = vec![0.0; n];
        let mut inflow_total = 0.0;
        for inflow in &map.inflows {
            match extra.get_mut(inflow.cell as usize) {
                Some(e) => {
                    *e += inflow.area_m2;
                    inflow_total += inflow.area_m2;
                }
                None => problems.push(format!("inflow at cell {} is off the map", inflow.cell)),
            }
        }
        let area = accumulate(&order, &receiver_index, cell_area, &extra);
        let terminal: f64 = (0..n)
            .filter(|&i| receiver_index[i] as usize == i)
            .map(|i| area[i])
            .sum();
        let expected = n as f64 * cell_area + inflow_total;
        if (terminal - expected).abs() > expected * 1e-9 {
            problems.push(format!(
                "drainage is not conserved: {terminal} m² reach outlets, expected {expected}"
            ));
        }
    }

    // Lakes.
    let mut counts = vec![0u32; map.lakes.len() + 1];
    for i in 0..n {
        let id = map.lake_id[i] as usize;
        if id == 0 {
            if map.water[i] == WATER_LAKE {
                problems.push(format!("lake cell {i} has no lake id"));
            }
            continue;
        }
        if id > map.lakes.len() {
            problems.push(format!("cell {i} names lake {id}, which does not exist"));
            continue;
        }
        counts[id] += 1;
        if map.water[i] != WATER_LAKE {
            problems.push(format!("cell {i} has a lake id but is not lake water"));
        }
        let lake = &map.lakes[id - 1];
        if map.elevation[i] > lake.level_m + 1e-3 {
            problems.push(format!("lake {id} cell {i} rises above the water surface"));
        }
        if lake.closed && map.receivers[i] != RECEIVER_SINK {
            problems.push(format!("closed lake {id} cell {i} drains somewhere"));
        }
    }
    for (k, lake) in map.lakes.iter().enumerate() {
        if lake.id as usize != k + 1 {
            problems.push(format!("lake at position {k} has id {}", lake.id));
        }
        if lake.level_m > lake.spill_m + 1e-3 {
            problems.push(format!("lake {} stands above its spill level", lake.id));
        }
        if counts.get(k + 1).copied().unwrap_or(0) != lake.cell_count {
            problems.push(format!(
                "lake {} cell count disagrees with the raster",
                lake.id
            ));
        }
    }

    // Reaches.
    for (k, reach) in map.reaches.iter().enumerate() {
        if reach.id as usize != k {
            problems.push(format!("reach at position {k} has id {}", reach.id));
        }
        if reach.cells.is_empty() {
            problems.push(format!("reach {k} has no cells"));
            continue;
        }
        for pair in reach.cells.windows(2) {
            let (a, b) = (pair[0] as usize, pair[1] as usize);
            if a >= n || b >= n || receiver_index[a] as usize != b {
                problems.push(format!(
                    "reach {k} does not follow the drainage at cell {a}"
                ));
                break;
            }
        }
        match (reach.terminus, reach.downstream) {
            (Terminus::Junction, Some(d)) if (d as usize) < map.reaches.len() => {
                let joins_at = map.reaches[d as usize].cells.first();
                if joins_at != reach.cells.last() {
                    problems.push(format!("reach {k} does not end where reach {d} begins"));
                }
            }
            (Terminus::Junction, _) => {
                problems.push(format!(
                    "reach {k} ends at a junction with no downstream reach"
                ));
            }
            (Terminus::Lake(id), _) if id == 0 || id as usize > map.lakes.len() => {
                problems.push(format!("reach {k} flows into missing lake {id}"));
            }
            _ => {}
        }
        if reach.order == 0 || !reach.discharge_m3s.is_finite() || reach.discharge_m3s < 0.0 {
            problems.push(format!("reach {k} has an invalid order or discharge"));
        }
    }
    problems
}
