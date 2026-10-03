//! Single-receiver (D8) flow routing: receivers, topological order and drainage accumulation.

use rayon::prelude::*;

use civ_core::rng::mix64;

use crate::grid::{D8, D8_DIST, neighbor};

/// Seeded perturbation of slope comparisons.
///
/// Pure D8 steepest descent on smooth slopes picks the same direction cell after cell and draws
/// rivers as ruler-straight 45° lines. Scaling each candidate slope by a per-cell, per-direction
/// factor in `[1 − amount, 1 + amount]` lets near-equal directions alternate, so channels wander
/// the way real ones do. Only descending neighbours are ever candidates, so every cell still drains.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Jitter {
    pub seed: u64,
    pub amount: f64,
}

impl Jitter {
    /// No perturbation.
    #[cfg(test)]
    pub const NONE: Jitter = Jitter {
        seed: 0,
        amount: 0.0,
    };

    #[inline]
    fn factor(&self, cell: usize, direction: usize) -> f64 {
        if self.amount == 0.0 {
            return 1.0;
        }
        let h = mix64(self.seed ^ ((cell as u64) << 3 | direction as u64));
        let unit = (h >> 11) as f64 * (1.0 / (1u64 << 53) as f64);
        1.0 + self.amount * (2.0 * unit - 1.0)
    }
}

/// Steepest-descent receiver of every cell on a routing surface. Seeds receive themselves.
///
/// On a priority-flood epsilon surface every non-seed cell has a strictly lower neighbour. A cell
/// without one (which would be a bug upstream) is made its own receiver, and validation reports it.
pub(crate) fn steepest_receivers(
    surface: &[f64],
    w: usize,
    h: usize,
    seeds: &[bool],
    jitter: Jitter,
) -> Vec<u32> {
    (0..w * h)
        .into_par_iter()
        .map(|i| {
            if seeds[i] {
                return i as u32;
            }
            let mut best = i;
            let mut best_slope = 0.0;
            for (d, &off) in D8.iter().enumerate() {
                if let Some(nb) = neighbor(w, h, i, off) {
                    let drop = surface[i] - surface[nb];
                    if drop > 0.0 {
                        let slope = drop / D8_DIST[d] * jitter.factor(i, d);
                        if slope > best_slope {
                            best_slope = slope;
                            best = nb;
                        }
                    }
                }
            }
            best as u32
        })
        .collect()
}

/// Cells ordered so that every cell comes after its receiver (outlets first). Built by
/// breadth-first search up the donor tree from cells that are their own receiver. If the receivers
/// contain a cycle, the cycle's cells are missing from the result, which is how validation
/// detects it.
pub(crate) fn stack_order(receivers: &[u32]) -> Vec<u32> {
    let n = receivers.len();
    let mut start = vec![0u32; n + 1];
    for (i, &r) in receivers.iter().enumerate() {
        if r as usize != i {
            start[r as usize + 1] += 1;
        }
    }
    for i in 0..n {
        start[i + 1] += start[i];
    }
    let mut fill = start.clone();
    let mut donors = vec![0u32; start[n] as usize];
    for (i, &r) in receivers.iter().enumerate() {
        if r as usize != i {
            let slot = &mut fill[r as usize];
            donors[*slot as usize] = i as u32;
            *slot += 1;
        }
    }
    let mut order: Vec<u32> = Vec::with_capacity(n);
    order.extend((0..n as u32).filter(|&i| receivers[i as usize] == i));
    let mut head = 0;
    while head < order.len() {
        let c = order[head] as usize;
        head += 1;
        order.extend_from_slice(&donors[start[c] as usize..start[c + 1] as usize]);
    }
    order
}

/// Drainage area of every cell: its own area plus any injected upstream area, plus everything
/// upstream of it. `extra` may be empty.
pub(crate) fn accumulate(
    order: &[u32],
    receivers: &[u32],
    cell_area: f64,
    extra: &[f64],
) -> Vec<f64> {
    let n = receivers.len();
    let mut area = vec![cell_area; n];
    if !extra.is_empty() {
        for (a, e) in area.iter_mut().zip(extra) {
            *a += e;
        }
    }
    for &i in order.iter().rev() {
        let i = i as usize;
        let r = receivers[i] as usize;
        if r != i {
            area[r] += area[i];
        }
    }
    area
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tilted_plane_drains_downhill_and_conserves_area() {
        let (w, h) = (6, 5);
        let surface: Vec<f64> = (0..w * h).map(|i| (i % w) as f64).collect(); // rises eastward
        let seeds: Vec<bool> = (0..w * h).map(|i| i % w == 0).collect(); // west edge drains
        let rcv = steepest_receivers(&surface, w, h, &seeds, Jitter::NONE);
        let order = stack_order(&rcv);
        assert_eq!(order.len(), w * h, "every cell reaches an outlet");
        let area = accumulate(&order, &rcv, 1.0, &[]);
        let outlet_total: f64 = (0..w * h).filter(|&i| seeds[i]).map(|i| area[i]).sum();
        assert!((outlet_total - (w * h) as f64).abs() < 1e-9);
    }

    #[test]
    fn cycles_are_visible_as_missing_cells() {
        // 0 -> 1 -> 0 is a cycle; 2 is its own outlet.
        let order = stack_order(&[1, 0, 2]);
        assert_eq!(order, vec![2]);
    }

    #[test]
    fn extra_area_flows_downstream() {
        let rcv = vec![0, 0, 1];
        let order = stack_order(&rcv);
        let area = accumulate(&order, &rcv, 10.0, &[0.0, 0.0, 5.0]);
        assert_eq!(area, vec![35.0, 25.0, 15.0]);
    }
}
