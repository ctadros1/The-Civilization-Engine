//! Priority-flood depression handling (Barnes, Lehman & Mulla 2014).
//!
//! Starting from seed cells (outlets), cells are visited in order of increasing spill elevation.
//! A cell lower than the one that reached it lies in a depression and is raised to that level.
//! Cells inside a depression go through a FIFO "pit" queue instead of the heap, which is the
//! paper's main speed-up.
//!
//! With `epsilon`, raised cells get the next representable value above their discoverer instead,
//! so the result has no flats: every non-seed cell has a strictly lower neighbour, and following
//! steepest descent from anywhere reaches a seed. The physical terrain is untouched; this produces
//! a *routing* surface (research 03-01 §3.1).

use std::cmp::Reverse;
use std::collections::{BinaryHeap, VecDeque};

use crate::grid::{D8, is_edge, neighbor};

/// Maps an `f64` to a `u64` whose unsigned order matches the float's total order.
#[inline]
pub(crate) fn ord_key(x: f64) -> u64 {
    let b = x.to_bits();
    if b >> 63 == 1 { !b } else { b | (1 << 63) }
}

/// Fills depressions draining to `seeds`. Returns the filled surface.
pub(crate) fn priority_flood(
    z: &[f64],
    w: usize,
    h: usize,
    seeds: &[bool],
    epsilon: bool,
) -> Vec<f64> {
    let n = w * h;
    debug_assert_eq!(z.len(), n);
    debug_assert_eq!(seeds.len(), n);
    let mut out = z.to_vec();
    let mut closed = vec![false; n];
    let mut open: BinaryHeap<Reverse<(u64, u32)>> = BinaryHeap::new();
    let mut pit: VecDeque<u32> = VecDeque::new();
    for i in 0..n {
        if seeds[i] {
            closed[i] = true;
            open.push(Reverse((ord_key(out[i]), i as u32)));
        }
    }
    loop {
        let c = match (pit.front(), open.peek()) {
            // An open cell exactly at the pit's level goes first (RichDEM's tie rule).
            (Some(&p), Some(&Reverse((k, o)))) if k == ord_key(out[p as usize]) => {
                open.pop();
                o
            }
            (Some(_), _) => pit.pop_front().unwrap_or_default(),
            (None, Some(_)) => match open.pop() {
                Some(Reverse((_, o))) => o,
                None => break,
            },
            (None, None) => break,
        } as usize;
        let level = if epsilon { out[c].next_up() } else { out[c] };
        for off in D8 {
            let Some(nb) = neighbor(w, h, c, off) else {
                continue;
            };
            if closed[nb] {
                continue;
            }
            closed[nb] = true;
            if out[nb] <= level {
                out[nb] = level;
                pit.push_back(nb as u32);
            } else {
                open.push(Reverse((ord_key(out[nb]), nb as u32)));
            }
        }
    }
    out
}

/// Cells below `sea_level` connected to the map edge through cells below `sea_level`. A low
/// inland basin is not ocean (research 03-01 §3.6).
pub(crate) fn ocean_mask(z: &[f64], w: usize, h: usize, sea_level: f64) -> Vec<bool> {
    let n = w * h;
    let mut ocean = vec![false; n];
    let mut queue = VecDeque::new();
    for i in 0..n {
        if is_edge(w, h, i) && z[i] < sea_level {
            ocean[i] = true;
            queue.push_back(i);
        }
    }
    while let Some(c) = queue.pop_front() {
        for off in D8 {
            if let Some(nb) = neighbor(w, h, c, off)
                && !ocean[nb]
                && z[nb] < sea_level
            {
                ocean[nb] = true;
                queue.push_back(nb);
            }
        }
    }
    ocean
}

#[cfg(test)]
mod tests {
    use super::*;

    fn edges(w: usize, h: usize) -> Vec<bool> {
        (0..w * h).map(|i| is_edge(w, h, i)).collect()
    }

    #[test]
    fn a_pit_is_filled_to_its_spill_level() {
        // 5x5 bowl: rim at 10, a saddle of 4 on the east edge, centre at 1.
        let w = 5;
        let mut z = vec![10.0; 25];
        for y in 1..4 {
            for x in 1..4 {
                z[y * w + x] = 5.0;
            }
        }
        z[2 * w + 2] = 1.0;
        z[2 * w + 4] = 4.0; // the edge cell that sets the spill level
        let filled = priority_flood(&z, w, 5, &edges(w, 5), false);
        assert_eq!(filled[2 * w + 2], 5.0, "the centre fills to the inner ring");
        assert_eq!(filled[2 * w + 1], 5.0);
        assert_eq!(filled[0], 10.0, "seeds keep their height");
    }

    #[test]
    fn epsilon_surfaces_always_have_a_lower_neighbour() {
        let w = 9;
        let h = 7;
        let z: Vec<f64> = (0..w * h)
            .map(|i| {
                let (x, y) = ((i % w) as f64, (i / w) as f64);
                10.0 - ((x - 4.0).powi(2) + (y - 3.0).powi(2)).sqrt() // a cone-shaped pit
            })
            .collect();
        let seeds = edges(w, h);
        let routed = priority_flood(&z, w, h, &seeds, true);
        for i in 0..w * h {
            if seeds[i] {
                continue;
            }
            let lower = D8
                .iter()
                .filter_map(|&o| neighbor(w, h, i, o))
                .any(|nb| routed[nb] < routed[i]);
            assert!(lower, "cell {i} has no lower neighbour");
            assert!(routed[i] >= z[i]);
        }
    }

    #[test]
    fn inland_basins_below_sea_level_are_not_ocean() {
        let w = 6;
        let mut z = vec![5.0; 36];
        z[0] = -1.0; // edge cell below sea level: ocean
        z[1] = -1.0;
        z[3 * w + 3] = -3.0; // enclosed: not ocean
        let ocean = ocean_mask(&z, w, 6, 0.0);
        assert!(ocean[0] && ocean[1]);
        assert!(!ocean[3 * w + 3]);
    }

    #[test]
    fn ord_key_preserves_float_order() {
        let values = [-1e9, -2.5, -0.0, 0.0, 1e-300, 3.0, 1e12];
        for pair in values.windows(2) {
            assert!(ord_key(pair[0]) <= ord_key(pair[1]), "{pair:?}");
        }
    }
}
