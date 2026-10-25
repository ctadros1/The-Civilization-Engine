//! Track (M10b slice DV; ADR-0034 §2; the rail brief §1.1–§1.2; research 07-08 §5.5): a line of
//! grid cells a wagon runs along. Rail cannot turn sharply, so a line turns at most 45° from one
//! cell to the next (ADR-0034 §2's turn rule, the brief's open question 2 answered "cells with a
//! turn rule"), and it climbs no steeper than its ruling grade, since on rails grade, not
//! distance, is what a load costs: the pull `M·g·(C + s)` ([`crate::carting::pull_n`]) is some
//! six times the level pull on a 1 % rise at an iron rail's `C` of 0.002 (the brief §1.2). Track
//! lies on land or on a crossing's deck, never through water.
//!
//! A wagon on its track is [`crate::carting::Wheels`] whose ground resistance is the rail's.
//!
//! The second part: a line is laid over ground no steeper than a searching grade, and its bed is
//! graded to the ruling grade by cut and fill ([`bed_profile`]; ADR-0029 §6's grading, the brief
//! §1.1: "grading is a prerequisite"). Laying it takes hours for the earth moved and for each
//! metre of sleepers and rails, and timber for them ([`TrackParams`]); the plan is a [`Track`]
//! kept with its way.
//!
//! The third part: a wagon is pushed along its line's bed, its speed on each step its pusher's
//! power over the pull there ([`push_seconds`]), and it carries no more than its pusher can take
//! up the steepest rise it meets ([`most_load`]): on a pushed wagonway the ruling grade sets the
//! load.

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

use civ_world::WATER_LAND;

use crate::carting::{Terrain, Wheels};

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
        self.cut_track(from, to, ruling_grade, ruling_grade, 0.0, max_cells)
    }

    /// The cheapest line of track from cell `from` to cell `to` that turns at most 45° a cell
    /// and crosses no ground steeper than `steepest` between neighbouring cells (M10b slice DV,
    /// third part): each step costs its run, metres, and `earth_m` metres more for each square
    /// metre of rise beyond `ruling_grade` times its run, the earth a bed graded to that grade
    /// would move there; `None` if there is none within `max_cells` cells searched.
    pub fn cut_track(
        &self,
        from: u32,
        to: u32,
        ruling_grade: f64,
        steepest: f64,
        earth_m: f64,
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
                let rise = f64::from((nz - z).abs());
                if rise / run > steepest.max(ruling_grade) {
                    continue;
                }
                let ng = g + run + earth_m * (rise - ruling_grade * run).max(0.0) * run;
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

/// What laying track takes (M10b slice DV, second part). Authored in content.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrackParams {
    /// The good its sleepers and rails are made of (an index into the catalog's goods).
    pub good: usize,
    /// Kilograms of that good a metre of track takes.
    pub good_kg_m: f64,
    /// Hours of a capable adult's work to hew and lay a metre of sleepers and rails.
    pub lay_h_m: f64,
    /// Hours of a capable adult's work to dig or place a cubic metre of earth.
    pub earth_h_m3: f64,
    /// The bed's width beyond the gauge on each side, metres.
    pub shoulder_m: f32,
    /// The steepest ground between neighbouring cells a line is laid over before its bed is
    /// graded, rise over run.
    pub search_grade: f64,
}

/// A line of track planned or laid: the gauge it is laid to, its bed's height at each of its
/// way's cells in order, the earth its grading cuts and fills, and what laying it takes in all.
#[derive(Clone, Debug, PartialEq)]
pub struct Track {
    pub gauge_m: f32,
    pub bed: Vec<f32>,
    pub cut_m3: f32,
    pub fill_m3: f32,
    /// Kilograms of [`TrackParams::good`] it takes.
    pub good_kg: f32,
    /// Hours of a capable adult's work it takes.
    pub labour_h: f32,
}

/// The distance from each cell of `cells` to the one before it on a map `width` cells wide,
/// `cell_m` metres a cell (0 for the first).
pub fn runs(cells: &[u32], width: u32, cell_m: f64) -> Vec<f64> {
    let w = i64::from(width.max(1));
    let mut out = Vec::with_capacity(cells.len());
    out.push(0.0);
    out.extend(cells.windows(2).map(|p| {
        let (dx, dy) = (
            (i64::from(p[1]) % w - i64::from(p[0]) % w).abs(),
            (i64::from(p[1]) / w - i64::from(p[0]) / w).abs(),
        );
        cell_m * (((dx * dx + dy * dy) as f64).sqrt())
    }));
    out
}

/// A bed along a line whose ground is `ground` at each cell (`runs` apart, as [`runs`] gives)
/// that rises or falls no steeper than `ruling` anywhere: midway between the lowest bed that
/// stays on or above the ground's every cone of that grade and the highest that stays on or
/// below it. Where the ground already keeps the grade, the bed is the ground; elsewhere it is
/// cut into rises and filled over hollows about equally.
pub fn bed_profile(ground: &[f32], runs: &[f64], ruling: f64) -> Vec<f32> {
    let n = ground.len();
    let g: Vec<f64> = ground.iter().map(|&z| f64::from(z)).collect();
    let (mut upper, mut lower) = (g.clone(), g.clone());
    for i in 1..n {
        upper[i] = upper[i].min(upper[i - 1] + ruling * runs[i]);
        lower[i] = lower[i].max(lower[i - 1] - ruling * runs[i]);
    }
    for i in (0..n.saturating_sub(1)).rev() {
        upper[i] = upper[i].min(upper[i + 1] + ruling * runs[i + 1]);
        lower[i] = lower[i].max(lower[i + 1] - ruling * runs[i + 1]);
    }
    upper
        .iter()
        .zip(&lower)
        .map(|(u, l)| ((u + l) / 2.0) as f32)
        .collect()
}

/// The earth cut and filled, cubic metres, to bring a bed `width` metres wide from `ground` to
/// `bed` along a line whose cells are `runs` apart: each cell's share of the line is half the
/// run on either side of it.
pub fn earth_m3(ground: &[f32], bed: &[f32], runs: &[f64], width: f64) -> (f64, f64) {
    let n = ground.len();
    let (mut cut, mut fill) = (0.0, 0.0);
    for i in 0..n {
        let length = (runs[i] + runs.get(i + 1).copied().unwrap_or(0.0)) / 2.0;
        let dz = f64::from(bed[i] - ground[i]);
        if dz > 0.0 {
            fill += dz * length * width;
        } else {
            cut -= dz * length * width;
        }
    }
    (cut, fill)
}

impl Terrain<'_> {
    /// A line of track of gauge `gauge_m` from cell `from` to cell `to` (`max_cells` searched at
    /// most): its cells, the cheapest in hours over ground no steeper than `params.search_grade`
    /// (a step's earth beyond the ruling grade reckoned as half its excess rise times its run
    /// across the bed, at `earth_h_m3` against `lay_h_m` a metre), and its bed graded to `ruling`
    /// with what that and its sleepers and rails take.
    pub fn plan_track(
        &self,
        from: u32,
        to: u32,
        gauge_m: f32,
        ruling: f64,
        params: &TrackParams,
        max_cells: usize,
    ) -> Option<(Vec<u32>, Track)> {
        let width = f64::from(gauge_m + 2.0 * params.shoulder_m);
        let earth_m = params.earth_h_m3 * width / (2.0 * params.lay_h_m.max(1e-9));
        let cells = self.cut_track(from, to, ruling, params.search_grade, earth_m, max_cells)?;
        let ground: Vec<f32> = cells
            .iter()
            .map(|&c| self.track_height(c))
            .collect::<Option<_>>()?;
        let runs = runs(&cells, self.width, self.cell_m);
        let bed = bed_profile(&ground, &runs, ruling);
        let (cut, fill) = earth_m3(&ground, &bed, &runs, width);
        let length: f64 = runs.iter().sum();
        let track = Track {
            gauge_m,
            bed,
            cut_m3: cut as f32,
            fill_m3: fill as f32,
            good_kg: (length * params.good_kg_m) as f32,
            labour_h: (length * params.lay_h_m + (cut + fill) * params.earth_h_m3) as f32,
        };
        Some((cells, track))
    }
}

/// The grade of each step along a bed (`runs` apart, as [`runs`] gives), rise over run, going
/// from its first cell to its last, or from its last to its first when `back`.
pub fn step_grades(bed: &[f32], runs: &[f64], back: bool) -> Vec<f64> {
    let mut grades: Vec<f64> = bed
        .windows(2)
        .zip(&runs[1.min(runs.len())..])
        .map(|(p, &run)| f64::from(p[1] - p[0]) / run.max(1e-9))
        .collect();
    if back {
        grades.reverse();
        grades.iter_mut().for_each(|g| *g = -*g);
    }
    grades
}

/// The steepest rise met going along a bed that way, rise over run: 0 where it never climbs.
pub fn steepest_rise(bed: &[f32], runs: &[f64], back: bool) -> f64 {
    step_grades(bed, runs, back).into_iter().fold(0.0, f64::max)
}

/// The seconds to push `wheels` along a bed that way on rails of rolling resistance `c`: each
/// step at its pusher's power over the pull there, no faster than they walk; `None` where it
/// stalls on a rise or cannot be held back on a fall.
pub fn push_seconds(bed: &[f32], runs: &[f64], wheels: &Wheels, c: f64, back: bool) -> Option<f64> {
    let mut steps: Vec<f64> = runs.get(1..).unwrap_or(&[]).to_vec();
    if back {
        steps.reverse();
    }
    step_grades(bed, runs, back)
        .into_iter()
        .zip(steps)
        .map(|(grade, run)| wheels.speed(c, grade).map(|v| run / v))
        .sum()
}

/// The most a wagon of its own mass `tare_kg` carries along a bed that way on rails of rolling
/// resistance `c`, no more than `payload_kg`: what a pusher's `pull_n` takes up the steepest rise
/// it meets, `M·g·(C + s) ≤ pull`.
pub fn most_load(
    bed: &[f32],
    runs: &[f64],
    back: bool,
    tare_kg: f64,
    payload_kg: f64,
    c: f64,
    pull_n: f64,
) -> f64 {
    let per_kg = crate::carting::pull_n(1.0, c, steepest_rise(bed, runs, back));
    if per_kg <= 0.0 {
        return payload_kg;
    }
    (pull_n / per_kg - tare_kg).clamp(0.0, payload_kg)
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

    #[test]
    fn a_graded_bed_keeps_the_ruling_grade_and_cuts_about_what_it_fills() {
        // A hump 2 m high over 5 cells of 8 m: 25 % a side, graded to 1 %.
        let ground = [0.0, 0.0, 0.0, 1.0, 2.0, 1.0, 0.0, 0.0, 0.0f32];
        let runs = vec![0.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0];
        let bed = bed_profile(&ground, &runs, 0.01);
        for i in 1..bed.len() {
            assert!(
                f64::from((bed[i] - bed[i - 1]).abs()) <= 0.01 * runs[i] + 1e-6,
                "{bed:?}"
            );
        }
        // Symmetric ground grades to a bed that is its own mirror, through the hump's middle.
        for i in 0..bed.len() {
            assert!((bed[i] - bed[bed.len() - 1 - i]).abs() < 1e-6);
        }
        let (cut, fill) = earth_m3(&ground, &bed, &runs, 2.0);
        assert!(cut > 0.0 && fill > 0.0);
        // Ground that already keeps the grade is left as it lies: no earth moved.
        let gentle = [0.0, 0.04, 0.08, 0.06, 0.02f32];
        let runs = vec![0.0, 8.0, 8.0, 8.0, 8.0];
        assert_eq!(bed_profile(&gentle, &runs, 0.01), gentle.to_vec());
        assert_eq!(earth_m3(&gentle, &gentle, &runs, 2.0), (0.0, 0.0));
    }

    #[test]
    fn the_ruling_grade_sets_a_pushed_wagons_load_and_its_time() {
        let w = wagon();
        // A bed level for 80 m, then rising 0.2 m over 40 m (0.5 %), on 8 m steps.
        let bed: Vec<f32> = (0..16)
            .map(|i| if i <= 10 { 0.0 } else { 0.04 * (i - 10) as f32 })
            .collect();
        let runs: Vec<f64> = (0..16).map(|i| if i == 0 { 0.0 } else { 8.0 }).collect();
        assert!((steepest_rise(&bed, &runs, false) - 0.005).abs() < 1e-6);
        assert_eq!(steepest_rise(&bed, &runs, true), 0.0);
        // Up the rise one person's 200 N moves 2,039 kg in all: 1,539 kg aboard a 500 kg wagon.
        let up = most_load(&bed, &runs, false, 500.0, 2_000.0, 0.005, 200.0);
        assert!((up - (200.0 / (9.81 * 0.01) - 500.0)).abs() < 0.01, "{up}");
        // The other way it is level or downhill: a full load.
        assert_eq!(
            most_load(&bed, &runs, true, 500.0, 2_000.0, 0.005, 200.0),
            2_000.0
        );
        // Loaded downhill the full 2 t goes at 75 W over its pull on the level stretch.
        let full = push_seconds(&bed, &runs, &w, 0.005, true).expect("runs downhill");
        let level = 80.0 / w.speed(0.005, 0.0).expect("level");
        assert!(full > level, "{full} {level}");
        // Uphill with the full load it stalls on the rise; with the most it climbs it runs.
        assert_eq!(push_seconds(&bed, &runs, &w, 0.005, false), None);
        let light = Wheels {
            mass_kg: 500.0 + up,
            ..w
        };
        assert!(push_seconds(&bed, &runs, &light, 0.005, false).is_some());
    }

    #[test]
    fn a_line_cuts_through_a_ridge_when_going_round_costs_more_than_the_earth() {
        let (w, h) = (30u32, 16u32);
        let (mut z, water) = flat(w, h);
        // A ridge 0.6 m high across columns 12–14 but for a gap at rows 0–1.
        for y in 2..h {
            for x in 12..15u32 {
                z[(y * w + x) as usize] = if x == 13 { 0.6 } else { 0.3 };
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
        // Earth dear: round by the gap. Earth cheap: straight through. Ground too steep to cut:
        // round again, whatever the earth costs.
        let round = t
            .cut_track(from, to, 0.003, 0.4, 100.0, 1_000_000)
            .expect("round");
        assert!(round.iter().any(|&c| c / w < 2));
        let through = t
            .cut_track(from, to, 0.003, 0.4, 0.01, 1_000_000)
            .expect("through");
        assert!(through.iter().all(|&c| c / w == 10), "{through:?}");
        let cliff = t
            .cut_track(from, to, 0.003, 0.02, 0.01, 1_000_000)
            .expect("round");
        assert!(cliff.iter().any(|&c| c / w < 2));
        // With no earth to weigh and the steepest at the ruling grade it is `lay_track`.
        assert_eq!(
            t.cut_track(from, to, 0.01, 0.01, 0.0, 1_000_000),
            t.lay_track(from, to, 0.01, 1_000_000)
        );
    }

    #[test]
    fn a_planned_line_reckons_its_timber_and_hours_by_length_and_earth() {
        let (w, h) = (24u32, 8u32);
        let (mut z, water) = flat(w, h);
        // A rise of 0.2 m at column 10 (2.5 % on 8 m cells), within the steepest ground cut.
        z[(4 * w + 10) as usize] = 0.2;
        let t = Terrain {
            elevation: &z,
            water: &water,
            decks: &[],
            width: w,
            height: h,
            cell_m: 8.0,
        };
        let params = TrackParams {
            good: 0,
            good_kg_m: 50.0,
            lay_h_m: 1.5,
            earth_h_m3: 2.3,
            shoulder_m: 0.5,
            search_grade: 0.05,
        };
        let (cells, track) = t
            .plan_track(4 * w + 4, 4 * w + 16, 1.0, 0.003, &params, 100_000)
            .expect("a line");
        assert_eq!(cells.len(), 13);
        assert_eq!(track.bed.len(), cells.len());
        // 96 m of track: 4,800 kg of timber and 144 hours' laying, and the earth graded at
        // 2.3 hours a cubic metre over a 2 m bed.
        assert!((track.good_kg - 4_800.0).abs() < 1e-3);
        let earth = f64::from(track.cut_m3 + track.fill_m3);
        assert!(earth > 0.0);
        assert!((f64::from(track.labour_h) - (144.0 + earth * 2.3)).abs() < 1e-3);
        assert!(
            track
                .bed
                .windows(2)
                .all(|p| (p[1] - p[0]).abs() <= 0.003 * 8.0 + 1e-6)
        );
    }
}
