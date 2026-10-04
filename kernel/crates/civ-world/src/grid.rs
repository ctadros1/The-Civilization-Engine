//! Raster helpers: row-major indexing and the D8 neighbourhood.
//!
//! Grid coordinates have x growing east and y growing south. Cell `i` is at `(i % w, i / w)` and
//! its centre is at `((x + 0.5) * cell, (y + 0.5) * cell)` metres from the map's north-west corner.

use std::f64::consts::SQRT_2;

/// D8 offsets, clockwise from east. A receiver is stored as its index in this table, and that code
/// is part of the save format: never reorder.
pub const D8: [(i32, i32); 8] = [
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
    (-1, -1),
    (0, -1),
    (1, -1),
];

/// Length of each D8 step, in cells.
pub const D8_DIST: [f64; 8] = [1.0, SQRT_2, 1.0, SQRT_2, 1.0, SQRT_2, 1.0, SQRT_2];

/// The four edge-sharing neighbours.
pub const D4: [(i32, i32); 4] = [(1, 0), (0, 1), (-1, 0), (0, -1)];

/// The neighbour of cell `i` in direction `(dx, dy)`, if it is inside the grid.
#[inline]
pub fn neighbor(w: usize, h: usize, i: usize, (dx, dy): (i32, i32)) -> Option<usize> {
    let x = (i % w) as i64 + i64::from(dx);
    let y = (i / w) as i64 + i64::from(dy);
    if x < 0 || y < 0 || x >= w as i64 || y >= h as i64 {
        None
    } else {
        Some(y as usize * w + x as usize)
    }
}

/// Whether cell `i` lies on the grid's outer ring.
#[inline]
pub fn is_edge(w: usize, h: usize, i: usize) -> bool {
    let (x, y) = (i % w, i / w);
    x == 0 || y == 0 || x + 1 == w || y + 1 == h
}

/// The D8 code of the step from `from` to the adjacent cell `to`.
pub fn direction_between(w: usize, from: usize, to: usize) -> Option<u8> {
    let dx = (to % w) as i64 - (from % w) as i64;
    let dy = (to / w) as i64 - (from / w) as i64;
    D8.iter()
        .position(|&(ox, oy)| i64::from(ox) == dx && i64::from(oy) == dy)
        .map(|d| d as u8)
}

/// Distance in metres between the centres of two adjacent cells.
#[inline]
pub fn step_length(w: usize, from: usize, to: usize, cell: f64) -> f64 {
    if from % w == to % w || from / w == to / w {
        cell
    } else {
        cell * SQRT_2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neighbours_respect_bounds() {
        assert_eq!(neighbor(4, 3, 0, (-1, 0)), None);
        assert_eq!(neighbor(4, 3, 0, (1, 1)), Some(5));
        assert_eq!(neighbor(4, 3, 11, (1, 0)), None);
        assert!(is_edge(4, 3, 3));
        assert!(!is_edge(4, 3, 5));
    }

    #[test]
    fn directions_round_trip() {
        let w = 5;
        let centre = 12;
        for (code, &off) in D8.iter().enumerate() {
            let to = neighbor(w, 5, centre, off).expect("inside");
            assert_eq!(direction_between(w, centre, to), Some(code as u8));
        }
        assert_eq!(direction_between(w, 0, 2), None);
    }
}
