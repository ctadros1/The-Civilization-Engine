//! Track (M10b slice DV; ADR-0034 §2; the rail brief §1.1–§1.2; research 07-08 §5.5): a line of
//! grid cells a wagon runs along. Rail cannot turn sharply, so a line turns at most 45° from one
//! cell to the next (ADR-0034 §2's turn rule, the brief's open question 2 answered "cells with a
//! turn rule"), and it climbs no steeper than its ruling grade, since on rails grade, not
//! distance, is what a load costs: the pull `M·g·(C + s)` ([`crate::carting::pull_n`]) is some
//! six times the level pull on a 1 % rise at an iron rail's `C` of 0.002 (the brief §1.2). Track
//! lies on land or on a crossing's deck, never through water.
//!
//! A wagon on its track is [`crate::carting::Wheels`] whose ground resistance is the rail's.

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

use civ_world::WATER_LAND;

use crate::carting::Terrain;

/// The eight directions from a cell, clockwise from east, as steps in columns and rows.
const DIRS: [(i64, i64); 8] = [
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
    (-1, -1),
    (0, -1),
    (1, -1),
];

/// No heading yet: the first step of a line may go any way.
const ANY: u8 = 8;

/// Whether a line heading `from` may turn to heading `to` in one cell: by at most 45°.
fn turns_ok(from: u8, to: u8) -> bool {
    if from == ANY {
        return true;
    }
    let a = (i16::from(to) - i16::from(from)).rem_euclid(8);
    a.min(8 - a) <= 1
}

#[derive(Clone, Copy, PartialEq)]
struct Open {
    f: f64,
    g: f64,
    cell: u32,
    dir: u8,
}

impl Eq for Open {}

impl Ord for Open {
    fn cmp(&self, other: &Self) -> Ordering {
        // A min-heap on the estimate; ties broken by cell and heading so the search is
        // deterministic.
        other
            .f
            .total_cmp(&self.f)
            .then_with(|| other.cell.cmp(&self.cell))
            .then_with(|| other.dir.cmp(&self.dir))
    }
}

impl PartialOrd for Open {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Terrain<'_> {
    /// The ground's height at `cell` for track: a deck's where one lies over it, else the
    /// ground's, and `None` where track cannot lie (water with no deck, or off the map).
    fn track_height(&self, cell: u32) -> Option<f32> {
        if let Ok(i) = self.decks.binary_search_by_key(&cell, |&(c, _)| c) {
            return Some(self.decks[i].1);
        }
        (self.water.get(cell as usize) == Some(&WATER_LAND)).then(|| self.elevation[cell as usize])
    }

    /// The shortest line of track from cell `from` to cell `to`, cells in order, that turns at
    /// most 45° a cell and climbs or falls no steeper than `ruling_grade` (rise over run) between
    /// neighbouring cells; `None` if there is none within `max_cells` cells searched.
    pub fn lay_track(
        &self,
        from: u32,
        to: u32,
        ruling_grade: f64,
        max_cells: usize,
    ) -> Option<Vec<u32>> {
        let (w, h) = (i64::from(self.width), i64::from(self.height));
        let xy = |c: u32| (i64::from(c) % w, i64::from(c) / w);
        self.track_height(from)?;
        self.track_height(to)?;
        let (tx, ty) = xy(to);
        // Octile distance in metres: no line is shorter.
        let estimate = |c: u32| {
            let (x, y) = xy(c);
            let (dx, dy) = ((x - tx).abs() as f64, (y - ty).abs() as f64);
            (dx.max(dy) + (std::f64::consts::SQRT_2 - 1.0) * dx.min(dy)) * self.cell_m
        };
        let mut best: HashMap<(u32, u8), f64> = HashMap::new();
        let mut came: HashMap<(u32, u8), (u32, u8)> = HashMap::new();
        let mut open = BinaryHeap::new();
        best.insert((from, ANY), 0.0);
        open.push(Open {
            f: estimate(from),
            g: 0.0,
            cell: from,
            dir: ANY,
        });
        let mut searched = 0usize;
        while let Some(Open { g, cell, dir, .. }) = open.pop() {
            if g > best.get(&(cell, dir)).copied().unwrap_or(f64::INFINITY) {
                continue;
            }
            if cell == to {
                let mut cells = vec![cell];
                let mut at = (cell, dir);
                while let Some(&prev) = came.get(&at) {
                    cells.push(prev.0);
                    at = prev;
                }
                cells.reverse();
                return Some(cells);
            }
            searched += 1;
            if searched > max_cells {
                return None;
            }
            let Some(z) = self.track_height(cell) else {
                continue;
            };
            let (x, y) = xy(cell);
            for (d, &(dx, dy)) in DIRS.iter().enumerate() {
                let d = d as u8;
                if !turns_ok(dir, d) {
                    continue;
                }
                let (nx, ny) = (x + dx, y + dy);
                if nx < 0 || ny < 0 || nx >= w || ny >= h {
                    continue;
                }
                let next = (ny * w + nx) as u32;
                let Some(nz) = self.track_height(next) else {
                    continue;
                };
                let run = if dx != 0 && dy != 0 {
                    self.cell_m * std::f64::consts::SQRT_2
                } else {
                    self.cell_m
                };
                if f64::from((nz - z).abs()) / run > ruling_grade {
                    continue;
                }
                let ng = g + run;
                if ng < best.get(&(next, d)).copied().unwrap_or(f64::INFINITY) {
                    best.insert((next, d), ng);
                    came.insert((next, d), (cell, dir));
                    open.push(Open {
                        f: ng + estimate(next),
                        g: ng,
                        cell: next,
                        dir: d,
                    });
                }
            }
        }
        None
    }
}

/// The sharpest turn, in eighths of a full turn, between consecutive steps of `cells` on a map
/// `width` cells wide: at most 1 where the line keeps the turn rule.
pub fn sharpest_turn(cells: &[u32], width: u32) -> u8 {
    let w = i64::from(width);
    let dir = |a: u32, b: u32| {
        let (dx, dy) = (
            i64::from(b) % w - i64::from(a) % w,
            i64::from(b) / w - i64::from(a) / w,
        );
        DIRS.iter().position(|&s| s == (dx, dy)).map(|d| d as u8)
    };
    let dirs: Vec<u8> = cells.windows(2).filter_map(|p| dir(p[0], p[1])).collect();
    dirs.windows(2)
        .map(|p| {
            let a = (i16::from(p[1]) - i16::from(p[0])).rem_euclid(8);
            a.min(8 - a) as u8
        })
        .max()
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::carting::Wheels;
    use civ_world::WATER_RIVER;

    /// The brief §1.2's wagon on timber rails: 2 t aboard a 0.5 t wagon at `C` = 0.005, pushed
    /// by one person of 200 N and 75 W walking at 1.4 m/s.
    fn wagon() -> Wheels {
        Wheels {
            mass_kg: 2_500.0,
            width_m: 1.0,
            way_c: 0.005,
            trail_c: 0.005,
            pull_n: 200.0,
            power_w: 75.0,
            walk_m_s: 1.4,
            brake_grade: 0.02,
        }
    }

    #[test]
    fn a_pushed_wagon_runs_the_level_and_stops_at_a_one_percent_rise() {
        let w = wagon();
        // 2,500 × 9.81 × 0.005 ≈ 123 N on the level, inside a person's 200 N, at 75 W / 123 N.
        let level = crate::carting::pull_n(w.mass_kg, w.way_c, 0.0);
        assert!((level - 122.6).abs() < 0.1, "{level}");
        let v = w.speed(w.way_c, 0.0).expect("it runs on the level");
        assert!((v - 0.61).abs() < 0.01, "{v} m/s");
        // Some six times a handcart's 150 kg at 1.4 m/s in load moved a second.
        assert!(2_000.0 * v / (150.0 * 1.4) > 5.0);
        // A 1 % rise needs about 368 N, more than a person gives: pushed wagonways run level.
        let rise = crate::carting::pull_n(w.mass_kg, w.way_c, 0.01);
        assert!((rise - 367.9).abs() < 0.1, "{rise}");
        assert_eq!(w.speed(w.way_c, 0.01), None);
    }

    fn flat(w: u32, h: u32) -> (Vec<f32>, Vec<u8>) {
        (
            vec![0.0; (w * h) as usize],
            vec![WATER_LAND; (w * h) as usize],
        )
    }

    #[test]
    fn track_turns_at_most_45_degrees_a_cell() {
        let (w, h) = (24u32, 24u32);
        let (z, water) = flat(w, h);
        let t = Terrain {
            elevation: &z,
            water: &water,
            decks: &[],
            width: w,
            height: h,
            cell_m: 8.0,
        };
        let line = t
            .lay_track(10 * w + 4, 10 * w + 12, 0.01, 100_000)
            .expect("a straight line");
        assert_eq!(line.len(), 9);
        assert_eq!(sharpest_turn(&line, w), 0);
        // A line round a right-angle bend: no step turns more than 45°.
        let bend = t
            .lay_track(4 * w + 4, 16 * w + 16, 0.01, 100_000)
            .expect("a line");
        assert!(sharpest_turn(&bend, w) <= 1);
        assert_eq!(bend.first(), Some(&(4 * w + 4)));
        assert_eq!(bend.last(), Some(&(16 * w + 16)));
        // The turn rule itself: 45° either way, never 90°.
        assert!(turns_ok(0, 1) && turns_ok(0, 7) && turns_ok(ANY, 4));
        assert!(!turns_ok(0, 2) && !turns_ok(0, 4) && !turns_ok(6, 0));
    }

    #[test]
    fn track_keeps_its_ruling_grade_and_keeps_out_of_water() {
        let (w, h) = (30u32, 16u32);
        let (mut z, mut water) = flat(w, h);
        // A ridge across columns 12–14, 0.5 m a cell (6 % on 8 m cells), but for a level gap
        // in rows 0–2.
        for y in 3..h {
            for x in 12..15u32 {
                z[(y * w + x) as usize] = if x == 13 { 1.0 } else { 0.5 };
            }
        }
        let t = Terrain {
            elevation: &z,
            water: &water,
            decks: &[],
            width: w,
            height: h,
            cell_m: 8.0,
        };
        let (from, to) = (10 * w + 4, 10 * w + 24);
        // Under a 1 % ruling grade it goes round by the gap; at 10 % it goes straight over.
        let round = t
            .lay_track(from, to, 0.01, 1_000_000)
            .expect("round by the gap");
        assert!(round.iter().all(|&c| {
            let (x, y) = (c % w, c / w);
            !(12..15).contains(&x) || y < 3
        }));
        assert!(sharpest_turn(&round, w) <= 1);
        let over = t
            .lay_track(from, to, 0.10, 1_000_000)
            .expect("over the ridge");
        // Round goes up to the gap; over keeps to its row.
        assert!(round.iter().any(|&c| c / w < 3));
        assert!(over.iter().all(|&c| c / w == 10), "{over:?}");
        // Water across every row with no deck: no line at all.
        for y in 0..h {
            water[(y * w + 18) as usize] = WATER_RIVER;
        }
        let t = Terrain {
            elevation: &z,
            water: &water,
            decks: &[],
            width: w,
            height: h,
            cell_m: 8.0,
        };
        assert_eq!(t.lay_track(from, to, 0.10, 1_000_000), None);
        // A deck over the river at row 10 carries it.
        let decks = [(10 * w + 18, 0.0f32)];
        let t = Terrain {
            elevation: &z,
            water: &water,
            decks: &decks,
            width: w,
            height: h,
            cell_m: 8.0,
        };
        let bridged = t
            .lay_track(from, to, 0.10, 1_000_000)
            .expect("over the deck");
        assert!(bridged.contains(&(10 * w + 18)));
    }
}
