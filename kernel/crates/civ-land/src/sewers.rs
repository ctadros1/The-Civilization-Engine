//! A sewer's route (ADR-0041 §1; research 12-02 §1.5): a conveyance laid along ways, as a gas
//! main is (ADR-0034 §8), from a head to an outfall on a river, falling all the way by the
//! terrain's heights. No pumping: a route that would rise, or fall less than the least gradient,
//! is refused.

use std::collections::VecDeque;

use crate::ways::{Keeper, Ways};

/// The route of a sewer from cell `from` (its head) on a map `width` by `height` cells, each
/// `cell_m` metres, with heights `elevation` (metres, by cell): through cells of ways whose
/// stretch's keeper `may` give leave, each step to one of the eight neighbours falling at least
/// `fall` of its length, to the first cell `outfall` accepts (a river), which ends it. Cells in
/// order from the head to the outfall, both kept; the fewest cells, ties broken by the order the
/// neighbours are tried. `None` where no such route falls all the way.
#[allow(clippy::too_many_arguments)]
pub fn plan_sewer(
    ways: &Ways,
    width: u32,
    height: u32,
    elevation: &[f32],
    cell_m: f64,
    from: u32,
    fall: f64,
    outfall: impl Fn(u32) -> bool,
    may: impl Fn(Keeper) -> bool,
) -> Option<Vec<u32>> {
    let mut open: Vec<u32> = ways
        .list
        .iter()
        .flat_map(|w| w.segments.iter())
        .filter(|s| may(s.keeper))
        .flat_map(|s| s.cells.iter().copied())
        .collect();
    open.sort_unstable();
    open.dedup();
    let (w, h) = (i64::from(width.max(1)), i64::from(height));
    let z = |c: u32| elevation.get(c as usize).map(|&e| f64::from(e));
    let mut came: Vec<(u32, u32)> = vec![(from, from)];
    let mut queue = VecDeque::from([from]);
    while let Some(c) = queue.pop_front() {
        if c != from && outfall(c) {
            let mut path = vec![c];
            let mut at = c;
            while at != from {
                at = came[came.binary_search_by_key(&at, |&(k, _)| k).ok()?].1;
                path.push(at);
            }
            path.reverse();
            return Some(path);
        }
        let Some(zc) = z(c) else {
            continue;
        };
        let (x, y) = (i64::from(c) % w, i64::from(c) / w);
        for (dx, dy) in [
            (1, 0),
            (0, 1),
            (-1, 0),
            (0, -1),
            (1, 1),
            (-1, 1),
            (-1, -1),
            (1, -1),
        ] {
            let (nx, ny) = (x + dx, y + dy);
            if nx < 0 || ny < 0 || nx >= w || ny >= h {
                continue;
            }
            let Ok(n) = u32::try_from(ny * w + nx) else {
                continue;
            };
            if open.binary_search(&n).is_err() && !outfall(n) {
                continue;
            }
            let step_m = cell_m * ((dx * dx + dy * dy) as f64).sqrt();
            // It must fall at least the least gradient over the step.
            if z(n).is_none_or(|zn| zn > zc - fall * step_m) {
                continue;
            }
            if let Err(i) = came.binary_search_by_key(&n, |&(k, _)| k) {
                came.insert(i, (n, c));
                queue.push_back(n);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ways::{Rank, Segment, Surface, WALK, Way};
    use civ_core::PermanentId;

    fn ways(cells: Vec<u32>) -> Ways {
        let mut ways = Ways::default();
        ways.add(Way {
            id: PermanentId::from_raw(1).expect("id"),
            begun: 0,
            segments: vec![Segment {
                cells,
                corridor_m: 4.0,
                built_m: 4.0,
                usable_m: 4.0,
                surface: Surface::Cleared,
                keeper: Keeper::Nobody,
                kept: 0,
            }],
            modes: WALK,
            trips: 0.0,
            trips_day: 0,
            rank: Rank::Lane,
            track: None,
        });
        ways
    }

    #[test]
    fn a_sewer_falls_all_the_way_to_the_river_or_is_refused() {
        // A row ten cells wide of 10 m cells: the head at 0, a way over 1-8, the river at 9.
        let ways = ways((1..9).collect());
        let mut z: Vec<f32> = (0..10).map(|i| 10.0 - i as f32 * 0.5).collect();
        let river = |c: u32| c == 9;
        let any = |_: Keeper| true;
        assert_eq!(
            plan_sewer(&ways, 10, 1, &z, 10.0, 0, 0.002, river, any),
            Some((0..10).collect::<Vec<u32>>())
        );
        // A step that rises refuses the route.
        z[5] = z[4] + 0.1;
        assert_eq!(
            plan_sewer(&ways, 10, 1, &z, 10.0, 0, 0.002, river, any),
            None
        );
        // A step that falls less than the least gradient (1 cm in 10 m against 2 cm) too.
        z[5] = z[4] - 0.01;
        assert_eq!(
            plan_sewer(&ways, 10, 1, &z, 10.0, 0, 0.002, river, any),
            None
        );
        z[5] = z[4] - 0.03;
        assert!(plan_sewer(&ways, 10, 1, &z, 10.0, 0, 0.002, river, any).is_some());
        // Without leave, no route.
        assert_eq!(
            plan_sewer(&ways, 10, 1, &z, 10.0, 0, 0.002, river, |_| false),
            None
        );
    }
}
