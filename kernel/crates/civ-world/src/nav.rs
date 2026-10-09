//! Walking across the terrain: travel times per step, least-time routes and travel-time fields
//! (research 01-08 §2A, §4, §8).
//!
//! Speed follows Tobler's hiking function on the slope of each step, `v = v₀·e^(−a·|s + b|)`,
//! fastest slightly downhill. Ground that has not been walked is slower than a trail
//! ([`NavParams::offtrail_factor`]); a fully worn trail restores the full speed and never beats
//! it, so the A* heuristic (straight-line distance at the top speed) stays admissible.
//!
//! Lakes and the sea cannot be walked. A river cell can be waded where its reach carries less than
//! [`NavParams::ford_max_discharge_m3s`], slowly; bigger rivers cannot be crossed on foot.
//!
//! Travel time (what advances the clock) and route choice are the same thing here: the cost of a
//! step is the seconds it takes.

use std::cell::RefCell;
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::sync::atomic::{AtomicU64, Ordering};

use civ_core::FastMap;

use crate::grid::{D8, D8_DIST};
use crate::{WATER_LAKE, WATER_OCEAN, WATER_RIVER, WorldMap};

/// How people walk. Authored in content.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NavParams {
    /// Tobler's top speed, km/h, reached on the best slope.
    pub top_speed_kmh: f64,
    /// Tobler's slope sensitivity `a`.
    pub slope_sensitivity: f64,
    /// Tobler's best slope offset `b` (0.05 = fastest at a 5 % descent).
    pub best_slope_offset: f64,
    /// Speed on unworn ground relative to a trail, 0–1.
    pub offtrail_factor: f64,
    /// Speed when wading a fordable river cell, relative to dry ground, 0–1.
    pub wading_factor: f64,
    /// Largest mean discharge a river can be waded at, m³/s.
    pub ford_max_discharge_m3s: f64,
    /// Steepest step that can be walked, rise over run.
    pub max_slope: f64,
}

impl NavParams {
    /// Top speed in metres per second.
    pub fn top_speed_ms(&self) -> f64 {
        self.top_speed_kmh / 3.6
    }

    /// Walking speed on ground of `slope` (rise over run, positive uphill), m/s, on a trail.
    pub fn tobler_ms(&self, slope: f64) -> f64 {
        self.top_speed_ms()
            * (-self.slope_sensitivity * (slope + self.best_slope_offset).abs()).exp()
    }
}

/// Per-cell ground factors derived from a map: 1 on dry land, the wading factor in fordable river
/// cells, 0 where people cannot walk. Rebuilt on load.
#[derive(Clone, Debug)]
pub struct NavGrid {
    /// Unique to the grid (a copy shares it, as it shares the ground): what [`StepSpeeds`] are
    /// kept for.
    id: u64,
    width: usize,
    height: usize,
    cell_m: f64,
    params: NavParams,
    ground: Vec<f32>,
}

/// Cells the route searches on this thread have expanded, summed: what a benchmark reads to
/// see how hard searches work.
pub fn cells_searched() -> u64 {
    CELLS_SEARCHED.with(std::cell::Cell::get)
}

/// Adds a search's expanded cells to [`cells_searched`].
fn count_searched(cells: usize) {
    CELLS_SEARCHED.with(|c| c.set(c.get() + cells as u64));
}

/// Landmarks round the area people walk ([`NavGrid::landmarks`]).
pub const LANDMARKS: usize = 8;

/// Most threads the landmarks' searches run on side by side.
const LANDMARK_THREADS: usize = 4;

/// Fastest walking times between a few landmark cells and every cell of an area, built on the
/// paths as surveyed: by the triangle inequality, the time from a cell `v` to a cell `t` is at
/// least the time from a landmark to `t` less that to `v`, and at least the time from `v` to a
/// landmark less that from `t` (research 01-08 §2A, "ALT"). Searches between cells of the area
/// are bounded by them; elsewhere by the straight line alone.
#[derive(Clone)]
pub struct Landmarks {
    /// Cells of the grid they were built for.
    cells: usize,
    map_width: usize,
    /// The area, in cells: its north-west corner and size.
    x0: usize,
    y0: usize,
    w: usize,
    h: usize,
    /// Landmarks.
    count: usize,
    /// Per cell of the area row by row, for each landmark the seconds from it to the cell and
    /// from the cell to it, side by side so a cell's are read together. A time is exact when
    /// not negative (infinite where there is no way); a negative one was not worked out, and
    /// the time is at least its magnitude.
    seconds: Vec<f32>,
}

impl std::fmt::Debug for Landmarks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Landmarks({} over {}x{} cells at ({}, {}))",
            self.count, self.w, self.h, self.x0, self.y0
        )
    }
}

impl Landmarks {
    /// Share of the triangle bounds used: the times are sums of many steps in single
    /// precision, each a little off, and a bound must never exceed the time a search adds up.
    const SURE: f32 = 1.0 - 4e-4;

    /// A cell's times, from and to each landmark in turn, if it lies in the area.
    fn at(&self, cell: usize) -> Option<&[f32]> {
        let (x, y) = (cell % self.map_width, cell / self.map_width);
        if x < self.x0 || y < self.y0 || x >= self.x0 + self.w || y >= self.y0 + self.h {
            return None;
        }
        let k = (y - self.y0) * self.w + (x - self.x0);
        Some(&self.seconds[k * 2 * self.count..(k + 1) * 2 * self.count])
    }

    /// The least time from the cell whose times are `v` to the one whose times are `t`
    /// (infinite when a landmark shows there is no way; 0 when none says more).
    fn bound(v: &[f32], t: &[f32]) -> f32 {
        // At least: a time's magnitude. At most: a time worked out, or no limit.
        let least = |x: f32| x.abs();
        let most = |x: f32| if x < 0.0 { f32::INFINITY } else { x };
        let mut out = 0.0f32;
        for (v, t) in v.chunks_exact(2).zip(t.chunks_exact(2)) {
            // From the landmark, d(L,t) - d(L,v); to it, d(v,L) - d(t,L). Infinite less
            // infinite says nothing (NaN, which `max` passes over).
            let from = least(t[0]) * Self::SURE - most(v[0]);
            let to = least(v[1]) * Self::SURE - most(t[1]);
            out = out.max(from).max(to);
        }
        out
    }
}

/// How a route search ended.
#[derive(Clone, Debug, PartialEq)]
pub enum RouteResult {
    /// The fastest route found.
    Found(Route),
    /// No walkable route exists.
    Unreachable,
    /// The search gave up after its node budget; a route may still exist.
    BudgetExhausted,
}

/// A walkable route between two cells.
#[derive(Clone, Debug, PartialEq)]
pub struct Route {
    /// Cells from start to goal, inclusive.
    pub cells: Vec<u32>,
    /// Seconds from the start to reach each cell (the first is 0).
    pub seconds: Vec<f32>,
}

impl Route {
    /// Total walking time, seconds.
    pub fn total_seconds(&self) -> f32 {
        self.seconds.last().copied().unwrap_or(0.0)
    }
}

/// Fastest travel times from one cell to every cell within a time limit. Kept densely over the
/// box of cells the limit could reach at the top speed.
#[derive(Clone, Debug, Default)]
pub struct TravelField {
    /// Map width, cells.
    map_width: usize,
    /// The box: first column and row, width and height, cells.
    x0: usize,
    y0: usize,
    w: usize,
    h: usize,
    /// Seconds per cell of the box; infinite where not reached.
    seconds: Vec<f32>,
    /// Reached cells (map indices), nearest first.
    reached: Vec<u32>,
}

impl TravelField {
    fn slot(&self, cell: usize) -> Option<usize> {
        if self.map_width == 0 {
            return None;
        }
        let (x, y) = (cell % self.map_width, cell / self.map_width);
        if x < self.x0 || y < self.y0 || x >= self.x0 + self.w || y >= self.y0 + self.h {
            return None;
        }
        Some((y - self.y0) * self.w + (x - self.x0))
    }

    /// Seconds to reach `cell`, if it is within the field's limit.
    pub fn seconds_to(&self, cell: usize) -> Option<f32> {
        let s = self.seconds[self.slot(cell)?];
        s.is_finite().then_some(s)
    }

    /// Number of cells reached.
    pub fn len(&self) -> usize {
        self.reached.len()
    }

    /// Whether no cell was reached.
    pub fn is_empty(&self) -> bool {
        self.reached.is_empty()
    }

    /// Every reached cell with its travel time, nearest first.
    pub fn iter(&self) -> impl Iterator<Item = (usize, f32)> + '_ {
        self.reached.iter().map(|&c| {
            let c = c as usize;
            (c, self.seconds_to(c).unwrap_or(f32::INFINITY))
        })
    }
}

/// The best time found to each cell and the cell it came from, during one route search.
trait Scores {
    fn get(&self, cell: u32) -> Option<(f32, u32)>;
    fn set(&mut self, cell: u32, g: f32, parent: u32);
}

impl Scores for FastMap<u32, (f32, u32)> {
    fn get(&self, cell: u32) -> Option<(f32, u32)> {
        FastMap::get(self, &cell).copied()
    }

    fn set(&mut self, cell: u32, g: f32, parent: u32) {
        self.insert(cell, (g, parent));
    }
}

/// One cell's entry in [`DenseScores`]: the search that wrote it, its best time and the cell it
/// came from, side by side so a look-up touches one place in memory.
#[derive(Clone, Copy, Default)]
struct Score {
    stamp: u32,
    g: f32,
    parent: u32,
}

/// [`Scores`] over the whole grid in one flat array, kept on a thread between searches and
/// cleared lazily by a generation stamp, so a search allocates nothing (research 01-08 §7:
/// reusable search scratch).
#[derive(Default)]
struct DenseScores {
    generation: u32,
    cells: Vec<Score>,
}

impl DenseScores {
    /// Readies the array for a new search over `cells` cells.
    fn begin(&mut self, cells: usize) {
        if self.cells.len() != cells {
            self.cells = vec![Score::default(); cells];
            self.generation = 0;
        }
        self.generation = self.generation.wrapping_add(1);
        if self.generation == 0 {
            self.cells.fill(Score::default());
            self.generation = 1;
        }
    }
}

impl Scores for DenseScores {
    fn get(&self, cell: u32) -> Option<(f32, u32)> {
        let s = self.cells[cell as usize];
        (s.stamp == self.generation).then_some((s.g, s.parent))
    }

    fn set(&mut self, cell: u32, g: f32, parent: u32) {
        self.cells[cell as usize] = Score {
            stamp: self.generation,
            g,
            parent,
        };
    }
}

/// Tobler's speed on each step from a cell to its eight neighbours, m/s on a trail, worked out
/// the first time a search or travel field takes the step and kept: the ground under a grid does
/// not change, and the value is the same arithmetic [`NavGrid::step_seconds`] does, so routes and
/// travel times come out bit for bit as without it. 0 marks a step not yet worked out (no
/// walkable step is that slow) and a negative value one that cannot be walked. Kept per thread
/// for the grid and the elevation it was filled from; its pages are only committed where
/// searches go.
#[derive(Default)]
struct StepSpeeds {
    grid: u64,
    elevation: usize,
    cells: usize,
    speed: Vec<f64>,
}

impl StepSpeeds {
    /// Readies the table for `grid` over `elevation`, emptying it if it was another's.
    fn ready(&mut self, grid: &NavGrid, elevation: &[f32]) {
        let cells = grid.width * grid.height;
        let key = (grid.id, elevation.as_ptr() as usize, cells);
        if (self.grid, self.elevation, self.cells) != key || self.speed.len() != cells * 8 {
            (self.grid, self.elevation, self.cells) = key;
            self.speed = vec![0.0; cells * 8];
        }
    }
}

/// Each grid's id, for [`StepSpeeds`].
static NEXT_GRID_ID: AtomicU64 = AtomicU64::new(1);

/// Grids up to this many cells (a 2048² map) search with [`DenseScores`], about 12 bytes a cell;
/// larger ones with a map of the cells reached.
const DENSE_SCORES_MAX_CELLS: usize = 1 << 22;

thread_local! {
    /// Cells route searches on this thread have looked at ([`cells_searched`]).
    static CELLS_SEARCHED: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    static DENSE_SCORES: RefCell<DenseScores> = RefCell::new(DenseScores::default());
    static STEP_SPEEDS: RefCell<StepSpeeds> = RefCell::new(StepSpeeds::default());
}

/// A cell on the open list with its scores, packed into one integer that orders as the list
/// pops: least `f` first, then the greatest `g` (deeper nodes), then the least cell index, a total
/// order. Scores are never negative, so their bits order as they do.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Open(Reverse<u128>);

impl Open {
    fn new(f: f32, g: f32, cell: u32) -> Open {
        let (f, g) = (u128::from(f.to_bits()), u128::from(u32::MAX - g.to_bits()));
        Open(Reverse((f << 64) | (g << 32) | u128::from(cell)))
    }

    fn g(self) -> f32 {
        f32::from_bits(u32::MAX - (self.0.0 >> 32) as u32)
    }

    fn cell(self) -> u32 {
        self.0.0 as u32
    }
}

impl NavGrid {
    /// Derives walkability from a map.
    pub fn new(map: &WorldMap, params: NavParams) -> Self {
        let n = map.cell_count();
        let mut ground = vec![1.0f32; n];
        for (i, g) in ground.iter_mut().enumerate() {
            match map.water[i] {
                WATER_LAKE | WATER_OCEAN => *g = 0.0,
                // River cells start impassable; fordable reaches open them below.
                WATER_RIVER => *g = 0.0,
                _ => {}
            }
        }
        for reach in &map.reaches {
            if f64::from(reach.discharge_m3s) <= params.ford_max_discharge_m3s {
                for &c in &reach.cells {
                    let c = c as usize;
                    if map.water[c] == WATER_RIVER {
                        ground[c] = params.wading_factor as f32;
                    }
                }
            }
        }
        NavGrid {
            id: NEXT_GRID_ID.fetch_add(1, Ordering::Relaxed),
            width: map.width as usize,
            height: map.height as usize,
            cell_m: f64::from(map.cell_size_m),
            params,
            ground,
        }
    }

    /// The parameters this grid was built with.
    pub fn params(&self) -> &NavParams {
        &self.params
    }

    /// Whether a person can stand in `cell`.
    pub fn walkable(&self, cell: usize) -> bool {
        self.ground.get(cell).is_some_and(|&g| g > 0.0)
    }

    /// Seconds to step from `from` to the neighbouring `to`, given the trail factor (0–1) of the
    /// cell being entered; `None` if the step cannot be walked.
    pub fn step_seconds(
        &self,
        elevation: &[f32],
        from: usize,
        to: usize,
        dist_cells: f64,
        trail: f32,
    ) -> Option<f32> {
        let tobler = self.step_speed(elevation, from, to, dist_cells)?;
        Some(self.seconds_at(tobler, to, dist_cells, trail))
    }

    /// Tobler's speed on the step from `from` to its neighbour `to`, m/s on a trail; `None` if
    /// the step cannot be walked.
    fn step_speed(
        &self,
        elevation: &[f32],
        from: usize,
        to: usize,
        dist_cells: f64,
    ) -> Option<f64> {
        if f64::from(self.ground[to]) <= 0.0 {
            return None;
        }
        let run = dist_cells * self.cell_m;
        let rise = f64::from(elevation[to] - elevation[from]);
        let slope = rise / run;
        if slope.abs() > self.params.max_slope {
            return None;
        }
        Some(self.params.tobler_ms(slope))
    }

    /// Seconds for a walkable step into `to` at Tobler's speed `tobler` (from
    /// [`NavGrid::step_speed`]), given the trail factor of `to`.
    #[inline]
    fn seconds_at(&self, tobler: f64, to: usize, dist_cells: f64, trail: f32) -> f32 {
        let run = dist_cells * self.cell_m;
        let surface = self.params.offtrail_factor
            + (1.0 - self.params.offtrail_factor) * f64::from(trail.clamp(0.0, 1.0));
        let speed = tobler * surface * f64::from(self.ground[to]);
        (run / speed) as f32
    }

    /// [`NavGrid::step_speed`] for the step from `from` in direction `k` of [`D8`] to `to`, kept
    /// in `speeds` (a [`StepSpeeds`] table, or empty to work it out each time).
    #[inline]
    fn kept_speed(
        &self,
        speeds: &mut [f64],
        elevation: &[f32],
        (from, k, to): (usize, usize, usize),
        dist_cells: f64,
    ) -> Option<f64> {
        match speeds.get_mut(from * 8 + k) {
            Some(kept) => {
                if *kept == 0.0 {
                    *kept = self
                        .step_speed(elevation, from, to, dist_cells)
                        .unwrap_or(-1.0);
                }
                (*kept > 0.0).then_some(*kept)
            }
            None => self.step_speed(elevation, from, to, dist_cells),
        }
    }

    /// [`NavGrid::kept_speed`] from a table another thread keeps: a step not yet in it is worked
    /// out (the same arithmetic) and not kept.
    #[inline]
    fn shared_speed(
        &self,
        speeds: &[f64],
        elevation: &[f32],
        (from, k, to): (usize, usize, usize),
        dist_cells: f64,
    ) -> Option<f64> {
        match speeds.get(from * 8 + k) {
            Some(&kept) if kept != 0.0 => (kept > 0.0).then_some(kept),
            _ => self.step_speed(elevation, from, to, dist_cells),
        }
    }

    /// Works out and keeps in `speeds` the steps out of `cell` to each neighbour.
    fn step_speeds_out(&self, speeds: &mut [f64], elevation: &[f32], cell: usize) {
        let (x, y) = ((cell % self.width) as i64, (cell / self.width) as i64);
        for (k, (&(dx, dy), dist)) in D8.iter().zip(D8_DIST).enumerate() {
            let (nx, ny) = (x + i64::from(dx), y + i64::from(dy));
            if nx < 0 || ny < 0 || nx >= self.width as i64 || ny >= self.height as i64 {
                continue;
            }
            let j = ny as usize * self.width + nx as usize;
            let _ = self.kept_speed(speeds, elevation, (cell, k, j), dist);
        }
    }

    /// Runs `f` with the thread's [`StepSpeeds`] for this grid over `elevation`, or with no
    /// table on a grid too large for one or from inside another search.
    fn with_speeds<R>(&self, elevation: &[f32], f: impl FnOnce(&mut [f64]) -> R) -> R {
        let n = self.width * self.height;
        if n > DENSE_SCORES_MAX_CELLS || elevation.len() < n {
            return f(&mut []);
        }
        let mut f = Some(f);
        let kept = STEP_SPEEDS.with(|t| {
            t.try_borrow_mut().ok().map(|mut t| {
                t.ready(self, elevation);
                (f.take().expect("not yet run"))(&mut t.speed)
            })
        });
        match kept {
            Some(r) => r,
            None => (f.take().expect("not yet run"))(&mut []),
        }
    }

    /// Seconds to walk straight from `from` to `to` at `speed`, an underestimate of any route
    /// when `speed` is the fastest any step can be walked.
    #[inline]
    fn heuristic(&self, from: usize, to: usize, speed: f64) -> f32 {
        let (fx, fy) = ((from % self.width) as f64, (from / self.width) as f64);
        let (tx, ty) = ((to % self.width) as f64, (to / self.width) as f64);
        let dist = ((fx - tx).powi(2) + (fy - ty).powi(2)).sqrt() * self.cell_m;
        (dist / speed) as f32
    }

    /// The fastest route from `from` to `to` (A*). `trail` gives the trail factor (0–1) of a
    /// cell. The search expands at most `budget` cells.
    pub fn route(
        &self,
        elevation: &[f32],
        from: usize,
        to: usize,
        trail: &(impl Fn(usize) -> f32 + ?Sized),
        budget: usize,
    ) -> RouteResult {
        self.route_bounded(elevation, from, to, trail, 1.0, budget)
    }

    /// [`NavGrid::route`] when no cell's trail value exceeds `trail_max`: the search can then
    /// assume a lower top speed, which is still exact and expands far fewer cells (with no
    /// trails at all, walking is off-trail everywhere).
    pub fn route_bounded(
        &self,
        elevation: &[f32],
        from: usize,
        to: usize,
        trail: &(impl Fn(usize) -> f32 + ?Sized),
        trail_max: f32,
        budget: usize,
    ) -> RouteResult {
        self.route_with(elevation, (from, to), trail, trail_max, budget, None)
    }

    /// [`NavGrid::route_bounded`] from `from` to `to`, its search also bounded below by
    /// `landmarks` ([`NavGrid::landmarks`], built on the same `trail`) when given: the same
    /// fastest time, found expanding far fewer cells. Where two routes are equally fast, which
    /// one is found can differ with the bounds.
    pub fn route_with(
        &self,
        elevation: &[f32],
        (from, to): (usize, usize),
        trail: &(impl Fn(usize) -> f32 + ?Sized),
        trail_max: f32,
        budget: usize,
        landmarks: Option<&Landmarks>,
    ) -> RouteResult {
        let p = &self.params;
        let fastest = p.top_speed_ms()
            * (p.offtrail_factor
                + (1.0 - p.offtrail_factor) * f64::from(trail_max.clamp(0.0, 1.0)));
        let n = self.width * self.height;
        if from >= n || to >= n || !self.walkable(to) {
            return RouteResult::Unreachable;
        }
        if from == to {
            return RouteResult::Found(Route {
                cells: vec![from as u32],
                seconds: vec![0.0],
            });
        }
        // The search is compiled for each kind of score table and trail, so that their look-ups
        // are inlined into its inner loop.
        self.with_speeds(elevation, |speeds| {
            let landmarks = landmarks.filter(|l| l.cells == n);
            let search = (elevation, from, to, fastest, budget, landmarks);
            if n <= DENSE_SCORES_MAX_CELLS {
                // A search started from inside another on this thread (none does today) gets
                // its own arrays.
                let shared = DENSE_SCORES.with(|d| {
                    d.try_borrow_mut().ok().map(|mut d| {
                        d.begin(n);
                        self.search(search, trail, &mut *d, speeds)
                    })
                });
                if let Some(found) = shared {
                    return found;
                }
                let mut own = DenseScores::default();
                own.begin(n);
                return self.search(search, trail, &mut own, speeds);
            }
            self.search(search, trail, &mut FastMap::default(), speeds)
        })
    }

    /// The A* search of [`NavGrid::route_bounded`], keeping its scores in `best` and Tobler's
    /// speeds in `speeds` ([`NavGrid::kept_speed`]).
    fn search(
        &self,
        (elevation, from, to, fastest, budget, landmarks): (
            &[f32],
            usize,
            usize,
            f64,
            usize,
            Option<&Landmarks>,
        ),
        trail: &(impl Fn(usize) -> f32 + ?Sized),
        best: &mut impl Scores,
        speeds: &mut [f64],
    ) -> RouteResult {
        let (w, h) = (self.width as i64, self.height as i64);
        // The least time from a cell to `to`: the straight line at the top speed, or what the
        // landmarks show, whichever is more.
        let target = landmarks.and_then(|l| Some((l, l.at(to)?)));
        let bound = |j: usize| {
            let line = self.heuristic(j, to, fastest);
            match target.and_then(|(l, t)| Some((l.at(j)?, t))) {
                Some((v, t)) => line.max(Landmarks::bound(v, t)),
                None => line,
            }
        };
        let mut open = BinaryHeap::new();
        best.set(from as u32, 0.0, u32::MAX);
        open.push(Open::new(bound(from), 0.0, from as u32));
        let mut expanded = 0usize;
        while let Some(next) = open.pop() {
            let (g, cell) = (next.g(), next.cell());
            let i = cell as usize;
            if best.get(cell).is_some_and(|(bg, _)| g > bg) {
                continue;
            }
            if i == to {
                count_searched(expanded);
                return RouteResult::Found(reconstruct(best, from, to));
            }
            expanded += 1;
            if expanded > budget {
                count_searched(expanded);
                return RouteResult::BudgetExhausted;
            }
            let (x, y) = ((i % self.width) as i64, (i / self.width) as i64);
            for (k, (&(dx, dy), dist)) in D8.iter().zip(D8_DIST).enumerate() {
                let (nx, ny) = (x + i64::from(dx), y + i64::from(dy));
                if nx < 0 || ny < 0 || nx >= w || ny >= h {
                    continue;
                }
                let j = (ny * w + nx) as usize;
                let Some(tobler) = self.kept_speed(speeds, elevation, (i, k, j), dist) else {
                    continue;
                };
                let ng = g + self.seconds_at(tobler, j, dist, trail(j));
                let better = best.get(j as u32).is_none_or(|(bg, _)| ng < bg);
                if better {
                    best.set(j as u32, ng, cell);
                    let f = ng + bound(j);
                    // A cell the landmarks show cannot reach `to` is not worth a visit.
                    if f.is_finite() {
                        open.push(Open::new(f, ng, j as u32));
                    }
                }
            }
        }
        count_searched(expanded);
        RouteResult::Unreachable
    }

    /// Lower bounds on walking times for route searches on `trail` between cells of `area`
    /// (research 01-08 §2A, "ALT"): the fastest times from and to [`LANDMARKS`] cells round its
    /// edge, to and from each of its cells. `area` is the north-west and south-east corner cells,
    /// clipped to the grid. Each landmark's two searches go out until every walkable cell of
    /// the area is reached, or as far as walking three times across it takes; they hold until
    /// walking costs change (the paths are surveyed again).
    pub fn landmarks(
        &self,
        elevation: &[f32],
        trail: &(impl Fn(usize) -> f32 + Sync + ?Sized),
        area: ((usize, usize), (usize, usize)),
    ) -> Landmarks {
        let n = self.width * self.height;
        let ((ax, ay), (bx, by)) = area;
        let (x0, y0) = (ax.min(self.width - 1), ay.min(self.height - 1));
        let (x1, y1) = (bx.clamp(x0, self.width - 1), by.clamp(y0, self.height - 1));
        let (w, h) = (x1 - x0 + 1, y1 - y0 + 1);
        let sites = self.landmark_cells((x0, y0), (x1, y1));
        let k = sites.len();
        let mut seconds = vec![f32::INFINITY; w * h * 2 * k];
        // As far as walking three times across the area off the trails takes.
        let across = (w as f64).hypot(h as f64) * self.cell_m;
        let slow = self.params.tobler_ms(0.0) * self.params.offtrail_factor;
        let cap = (3.0 * across / slow.max(1e-6)) as f32;
        let area = (x0, y0, w, h);
        let jobs: Vec<(usize, bool)> = sites
            .iter()
            .flat_map(|&site| [(site, false), (site, true)])
            .collect();
        self.with_speeds(elevation, |speeds| {
            // The steps out of the area's cells are worked out here and shared; the searches
            // run side by side, each the same whichever thread runs it.
            if !speeds.is_empty() {
                for c in (y0..=y1).flat_map(|y| (x0..=x1).map(move |x| y * self.width + x)) {
                    self.step_speeds_out(speeds, elevation, c);
                }
            }
            let speeds: &[f64] = speeds;
            let next = std::sync::atomic::AtomicUsize::new(0);
            let threads = std::thread::available_parallelism()
                .map_or(1, |t| t.get())
                .clamp(1, LANDMARK_THREADS)
                .min(jobs.len());
            let done: Vec<(usize, Vec<f32>)> = std::thread::scope(|scope| {
                let workers: Vec<_> = (0..threads)
                    .map(|_| {
                        scope.spawn(|| {
                            let mut out = Vec::new();
                            let mut scores = DenseScores::default();
                            loop {
                                let j = next.fetch_add(1, Ordering::Relaxed);
                                let Some(&job) = jobs.get(j) else {
                                    break out;
                                };
                                let mut times = vec![f32::INFINITY; w * h];
                                let into = (&mut times[..], &mut scores);
                                self.fill_area(elevation, job, trail, area, cap, into, speeds);
                                out.push((j, times));
                            }
                        })
                    })
                    .collect();
                workers
                    .into_iter()
                    .flat_map(|t| t.join().expect("a landmark search ends"))
                    .collect()
            });
            for (j, times) in done {
                // Job j is landmark j / 2, from it (even) or to it (odd).
                for (c, &t) in times.iter().enumerate() {
                    seconds[c * 2 * k + j] = t;
                }
            }
        });
        Landmarks {
            cells: n,
            map_width: self.width,
            x0,
            y0,
            w,
            h,
            count: k,
            seconds,
        }
    }

    /// Where the landmarks stand: for each corner of the area and the middle of each side, the
    /// walkable cell of the area nearest it (the least index among equals), each once.
    fn landmark_cells(&self, (x0, y0): (usize, usize), (x1, y1): (usize, usize)) -> Vec<usize> {
        let (mx, my) = ((x0 + x1) / 2, (y0 + y1) / 2);
        let anchors = [
            (x0, y0),
            (mx, y0),
            (x1, y0),
            (x1, my),
            (x1, y1),
            (mx, y1),
            (x0, y1),
            (x0, my),
        ];
        let mut out: Vec<usize> = Vec::new();
        for (ax, ay) in anchors {
            let nearest = (y0..=y1)
                .flat_map(|y| (x0..=x1).map(move |x| (x, y)))
                .map(|(x, y)| y * self.width + x)
                .filter(|&c| self.walkable(c))
                .min_by_key(|&c| {
                    let (x, y) = ((c % self.width) as i64, (c / self.width) as i64);
                    ((x - ax as i64).pow(2) + (y - ay as i64).pow(2), c)
                });
            if let Some(c) = nearest
                && !out.contains(&c)
            {
                out.push(c);
            }
        }
        out
    }

    /// Fastest times from `source` to each cell of `area` (its corner and size), or with
    /// `reverse` from each to `source`, into `out` row by row; Dijkstra over the grid, until
    /// every walkable cell of the area has its time or the times reach `cap`. A time not
    /// worked out is written as the negative of what it is at least; one with no way, infinite.
    /// `scores` is scratch for the search.
    #[allow(clippy::too_many_arguments)]
    fn fill_area(
        &self,
        elevation: &[f32],
        (source, reverse): (usize, bool),
        trail: &(impl Fn(usize) -> f32 + ?Sized),
        (x0, y0, aw, ah): (usize, usize, usize, usize),
        cap: f32,
        (out, scores): (&mut [f32], &mut DenseScores),
        speeds: &[f64],
    ) {
        let (w, h) = (self.width as i64, self.height as i64);
        let inside = |c: usize| {
            let (x, y) = (c % self.width, c / self.width);
            (x >= x0 && y >= y0 && x < x0 + aw && y < y0 + ah).then(|| (y - y0) * aw + (x - x0))
        };
        let mut left = (y0..y0 + ah)
            .flat_map(|y| (x0..x0 + aw).map(move |x| y * self.width + x))
            .filter(|&c| self.walkable(c))
            .count();
        out.fill(f32::INFINITY);
        // Best times so far and whether final, cleared lazily as for route searches.
        const FINAL: u32 = 1;
        scores.begin(self.width * self.height);
        let key = |g: f32, cell: u32| Reverse((u64::from(g.to_bits()) << 32) | u64::from(cell));
        let mut open = BinaryHeap::new();
        scores.set(source as u32, 0.0, 0);
        open.push(key(0.0, source as u32));
        let mut stopped_at = None;
        while let Some(Reverse(packed)) = open.pop() {
            let (g, cell) = (f32::from_bits((packed >> 32) as u32), packed as u32);
            let i = cell as usize;
            match scores.get(cell) {
                Some((bg, done)) if done == FINAL || g > bg => continue,
                _ => {}
            }
            if g > cap {
                stopped_at = Some(g);
                break;
            }
            scores.set(cell, g, FINAL);
            if let Some(k) = inside(i) {
                out[k] = g;
                // In reverse a cell people cannot walk is reached too, as where a walk might
                // start; only walkable cells are waited for.
                if self.walkable(i) {
                    left -= 1;
                    if left == 0 {
                        break;
                    }
                }
            }
            let (x, y) = ((i % self.width) as i64, (i / self.width) as i64);
            for (k, (&(dx, dy), dist)) in D8.iter().zip(D8_DIST).enumerate() {
                let (nx, ny) = (x + i64::from(dx), y + i64::from(dy));
                if nx < 0 || ny < 0 || nx >= w || ny >= h {
                    continue;
                }
                let j = (ny * w + nx) as usize;
                if scores.get(j as u32).is_some_and(|(_, done)| done == FINAL) {
                    continue;
                }
                // Forward, the step from `i` to `j`; in reverse, the step from `j` to `i`.
                let (a, d, b) = if reverse {
                    (j, (k + 4) % 8, i)
                } else {
                    (i, k, j)
                };
                let Some(tobler) = self.shared_speed(speeds, elevation, (a, d, b), dist) else {
                    continue;
                };
                let ng = g + self.seconds_at(tobler, b, dist, trail(b));
                if scores.get(j as u32).is_none_or(|(bg, _)| ng < bg) {
                    scores.set(j as u32, ng, 0);
                    open.push(key(ng, j as u32));
                }
            }
        }
        // Stopped at the cap: every cell not yet final takes at least that long.
        if let Some(g) = stopped_at {
            for (k, t) in out.iter_mut().enumerate() {
                let c = (y0 + k / aw) * self.width + x0 + k % aw;
                if t.is_infinite() && self.walkable(c) {
                    *t = -g;
                }
            }
        }
    }

    /// Fastest travel times from `from` to every cell reachable within `max_seconds` (Dijkstra).
    pub fn travel_field(
        &self,
        elevation: &[f32],
        from: usize,
        max_seconds: f32,
        trail: &dyn Fn(usize) -> f32,
    ) -> TravelField {
        if from >= self.width * self.height {
            return TravelField::default();
        }
        // No cell beyond what the top speed reaches in the limit can be reached.
        let radius = (f64::from(max_seconds) * self.params.top_speed_ms() / self.cell_m).ceil();
        let r = if radius.is_finite() && radius < (self.width.max(self.height) as f64) {
            radius as usize + 1
        } else {
            self.width.max(self.height)
        };
        let (fx, fy) = (from % self.width, from / self.width);
        let (x0, y0) = (fx.saturating_sub(r), fy.saturating_sub(r));
        let (x1, y1) = ((fx + r).min(self.width - 1), (fy + r).min(self.height - 1));
        let mut field = TravelField {
            map_width: self.width,
            x0,
            y0,
            w: x1 - x0 + 1,
            h: y1 - y0 + 1,
            seconds: Vec::new(),
            reached: Vec::new(),
        };
        field.seconds = vec![f32::INFINITY; field.w * field.h];
        self.with_speeds(elevation, |speeds| {
            self.fill_field(elevation, from, max_seconds, trail, &mut field, speeds);
        });
        field
    }

    /// The Dijkstra search of [`NavGrid::travel_field`] over `field`'s box, keeping Tobler's
    /// speeds in `speeds` ([`NavGrid::kept_speed`]).
    fn fill_field(
        &self,
        elevation: &[f32],
        from: usize,
        max_seconds: f32,
        trail: &dyn Fn(usize) -> f32,
        field: &mut TravelField,
        speeds: &mut [f64],
    ) {
        let (x0, y0, bw) = (field.x0, field.y0, field.w);
        let (x1, y1) = (x0 + field.w - 1, y0 + field.h - 1);
        let (fx, fy) = (from % self.width, from / self.width);
        // Each cell's trail factor, asked once.
        let mut trails = vec![f32::NAN; bw * field.h];
        // Cells whose time is final: none is reached sooner from a cell reached later.
        let mut done = vec![false; bw * field.h];
        // Cells to visit, nearest first and then by index, packed into one integer: times are
        // never negative, so their bits order as the times do.
        let key = |g: f32, cell: u32| Reverse((u64::from(g.to_bits()) << 32) | u64::from(cell));
        let mut open = BinaryHeap::new();
        field.seconds[(fy - y0) * bw + (fx - x0)] = 0.0;
        open.push(key(0.0, from as u32));
        while let Some(Reverse(k)) = open.pop() {
            let (g, cell) = (f32::from_bits((k >> 32) as u32), k as u32);
            let i = cell as usize;
            let (x, y) = (i % self.width, i / self.width);
            let si = (y - y0) * bw + (x - x0);
            if g > field.seconds[si] {
                continue;
            }
            done[si] = true;
            field.reached.push(cell);
            for (d, (&(dx, dy), dist)) in D8.iter().zip(D8_DIST).enumerate() {
                let (nx, ny) = (x as i64 + i64::from(dx), y as i64 + i64::from(dy));
                if nx < x0 as i64 || ny < y0 as i64 || nx > x1 as i64 || ny > y1 as i64 {
                    continue;
                }
                let (nx, ny) = (nx as usize, ny as usize);
                let j = ny * self.width + nx;
                let sj = (ny - y0) * bw + (nx - x0);
                // Every step takes some time, so a cell already final is no sooner from here.
                if done[sj] {
                    continue;
                }
                let Some(tobler) = self.kept_speed(speeds, elevation, (i, d, j), dist) else {
                    continue;
                };
                if trails[sj].is_nan() {
                    trails[sj] = trail(j);
                }
                let ng = g + self.seconds_at(tobler, j, dist, trails[sj]);
                if ng > max_seconds || ng >= field.seconds[sj] {
                    continue;
                }
                field.seconds[sj] = ng;
                open.push(key(ng, j as u32));
            }
        }
    }

    /// Ground height at a point, metres: bilinear between cell centres.
    fn height_at(&self, elevation: &[f32], (x, y): (f32, f32)) -> f64 {
        let c = self.cell_m as f32;
        let (u, v) = (x / c - 0.5, y / c - 0.5);
        let (max_x, max_y) = ((self.width - 1) as f32, (self.height - 1) as f32);
        let (u, v) = (u.clamp(0.0, max_x), v.clamp(0.0, max_y));
        let (x0, y0) = (u.floor() as usize, v.floor() as usize);
        let (x1, y1) = ((x0 + 1).min(self.width - 1), (y0 + 1).min(self.height - 1));
        let (fx, fy) = (f64::from(u - x0 as f32), f64::from(v - y0 as f32));
        let z = |x: usize, y: usize| f64::from(elevation[y * self.width + x]);
        let top = z(x0, y0) * (1.0 - fx) + z(x1, y0) * fx;
        let bottom = z(x0, y1) * (1.0 - fx) + z(x1, y1) * fx;
        top * (1.0 - fy) + bottom * fy
    }

    /// Seconds to walk in a straight line from `a` to `b` (metres) at the standard speed, the
    /// ground sampled every half cell; `None` if the line crosses ground that cannot be walked or
    /// a step too steep.
    pub fn segment_seconds(
        &self,
        elevation: &[f32],
        a: (f32, f32),
        b: (f32, f32),
        trail: &dyn Fn(usize) -> f32,
    ) -> Option<f32> {
        let p = &self.params;
        let len = f64::from(((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt());
        if len == 0.0 {
            return Some(0.0);
        }
        let steps = (len / (self.cell_m * 0.5)).ceil().max(1.0) as u32;
        let run = len / f64::from(steps);
        let mut prev = a;
        let mut z_prev = self.height_at(elevation, a);
        let mut total = 0.0f64;
        for s in 1..=steps {
            let t = s as f32 / steps as f32;
            let q = (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
            let mid = ((prev.0 + q.0) * 0.5, (prev.1 + q.1) * 0.5);
            let (cx, cy) = (
                (mid.0 / self.cell_m as f32).floor(),
                (mid.1 / self.cell_m as f32).floor(),
            );
            if cx < 0.0 || cy < 0.0 || cx >= self.width as f32 || cy >= self.height as f32 {
                return None;
            }
            let cell = cy as usize * self.width + cx as usize;
            let ground = f64::from(self.ground[cell]);
            if ground <= 0.0 {
                return None;
            }
            let z = self.height_at(elevation, q);
            let slope = (z - z_prev) / run;
            if slope.abs() > p.max_slope {
                return None;
            }
            let surface = p.offtrail_factor
                + (1.0 - p.offtrail_factor) * f64::from(trail(cell).clamp(0.0, 1.0));
            total += run / (p.tobler_ms(slope) * surface * ground);
            prev = q;
            z_prev = z;
        }
        Some(total as f32)
    }

    /// The route as a polyline in metres with the seconds from the start at each vertex, pulled
    /// straight wherever a straight line is walkable and no slower than the route it replaces.
    /// A grid route steps only along eight directions; people walk straight across open ground and
    /// keep to the route only where it pays (a trail, a ford, a way round a slope).
    pub fn straighten(
        &self,
        elevation: &[f32],
        route: &Route,
        trail: &dyn Fn(usize) -> f32,
    ) -> (Vec<(f32, f32)>, Vec<f32>) {
        let c = self.cell_m as f32;
        let w = self.width;
        let centre = |i: u32| {
            let i = i as usize;
            (((i % w) as f32 + 0.5) * c, ((i / w) as f32 + 0.5) * c)
        };
        let n = route.cells.len();
        if n <= 2 {
            let pts = route.cells.iter().map(|&i| centre(i)).collect();
            return (pts, route.seconds.clone());
        }
        let mut pts = vec![centre(route.cells[0])];
        let mut secs = vec![0.0f32];
        let mut i = 0usize;
        // A straight line must not be slower than the route by more than rounding.
        let ok = |i: usize, j: usize| -> Option<f32> {
            let s = self.segment_seconds(
                elevation,
                centre(route.cells[i]),
                centre(route.cells[j]),
                trail,
            )?;
            let along = route.seconds[j] - route.seconds[i];
            (s <= along * 1.0001 + 1e-3).then_some(s)
        };
        while i < n - 1 {
            // The farthest vertex a straight line reaches: grow the step while lines hold, then
            // narrow down between the last that held and the first that did not.
            let mut good = i + 1;
            let mut good_s = route.seconds[i + 1] - route.seconds[i];
            let mut step = 2;
            let mut bad = None;
            while good < n - 1 {
                let j = (i + step).min(n - 1);
                match ok(i, j) {
                    Some(s) => {
                        good = j;
                        good_s = s;
                        step *= 2;
                    }
                    None => {
                        bad = Some(j);
                        break;
                    }
                }
            }
            if let Some(mut hi) = bad {
                let mut lo = good;
                while hi - lo > 1 {
                    let mid = (lo + hi) / 2;
                    match ok(i, mid) {
                        Some(s) => {
                            lo = mid;
                            good = mid;
                            good_s = s;
                        }
                        None => hi = mid,
                    }
                }
            }
            pts.push(centre(route.cells[good]));
            secs.push(secs.last().copied().unwrap_or(0.0) + good_s);
            i = good;
        }
        (pts, secs)
    }

    /// The route as a polyline of cell centres in metres, simplified to within `tolerance_m`,
    /// with the seconds from the start at each kept vertex.
    pub fn polyline(&self, route: &Route, tolerance_m: f32) -> (Vec<(f32, f32)>, Vec<f32>) {
        let c = self.cell_m as f32;
        let w = self.width;
        let points: Vec<(f32, f32)> = route
            .cells
            .iter()
            .map(|&i| {
                let i = i as usize;
                (((i % w) as f32 + 0.5) * c, ((i / w) as f32 + 0.5) * c)
            })
            .collect();
        let keep = simplify_indices(&points, tolerance_m);
        let pts = keep.iter().map(|&k| points[k]).collect();
        let secs = keep.iter().map(|&k| route.seconds[k]).collect();
        (pts, secs)
    }
}

fn reconstruct(best: &dyn Scores, from: usize, to: usize) -> Route {
    let mut cells = Vec::new();
    let mut seconds = Vec::new();
    let mut cur = to as u32;
    while let Some((g, parent)) = best.get(cur) {
        cells.push(cur);
        seconds.push(g);
        if cur as usize == from || parent == u32::MAX {
            break;
        }
        cur = parent;
    }
    cells.reverse();
    seconds.reverse();
    Route { cells, seconds }
}

/// Indices of the points kept by Douglas–Peucker simplification (always the first and last).
pub fn simplify_indices(points: &[(f32, f32)], tolerance: f32) -> Vec<usize> {
    if points.len() <= 2 {
        return (0..points.len()).collect();
    }
    let mut keep = vec![false; points.len()];
    keep[0] = true;
    keep[points.len() - 1] = true;
    let mut stack = vec![(0usize, points.len() - 1)];
    while let Some((a, b)) = stack.pop() {
        if b <= a + 1 {
            continue;
        }
        let (mut worst, mut worst_d) = (a, 0.0f32);
        for (k, &p) in points.iter().enumerate().take(b).skip(a + 1) {
            let d = segment_distance(p, points[a], points[b]);
            if d > worst_d {
                worst = k;
                worst_d = d;
            }
        }
        if worst_d > tolerance {
            keep[worst] = true;
            stack.push((a, worst));
            stack.push((worst, b));
        }
    }
    keep.iter()
        .enumerate()
        .filter_map(|(i, &k)| k.then_some(i))
        .collect()
}

fn segment_distance(p: (f32, f32), a: (f32, f32), b: (f32, f32)) -> f32 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let len2 = dx * dx + dy * dy;
    if len2 == 0.0 {
        return ((p.0 - a.0).powi(2) + (p.1 - a.1).powi(2)).sqrt();
    }
    let t = (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / len2).clamp(0.0, 1.0);
    let (cx, cy) = (a.0 + t * dx, a.1 + t * dy);
    ((p.0 - cx).powi(2) + (p.1 - cy).powi(2)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Climate, RiverReach, Terminus, WATER_LAND};
    use civ_core::Rng64;

    pub(crate) fn params() -> NavParams {
        NavParams {
            top_speed_kmh: 6.0,
            slope_sensitivity: 3.5,
            best_slope_offset: 0.05,
            offtrail_factor: 0.6,
            wading_factor: 0.25,
            ford_max_discharge_m3s: 2.0,
            max_slope: 1.0,
        }
    }

    fn flat(w: u32, h: u32) -> WorldMap {
        let n = (w * h) as usize;
        WorldMap {
            width: w,
            height: h,
            cell_size_m: 8.0,
            sea_level_m: 0.0,
            elevation: vec![100.0; n],
            receivers: vec![8; n],
            water: vec![WATER_LAND; n],
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

    fn found(r: RouteResult) -> Route {
        match r {
            RouteResult::Found(route) => route,
            other => panic!("expected a route, got {other:?}"),
        }
    }

    #[test]
    fn tobler_is_fastest_slightly_downhill() {
        let p = params();
        assert!((p.tobler_ms(-0.05) - 6.0 / 3.6).abs() < 1e-9);
        assert!(
            (p.tobler_ms(0.0) * 3.6 - 5.04).abs() < 0.01,
            "about 5 km/h on the flat"
        );
        assert!(p.tobler_ms(0.3) < p.tobler_ms(0.0));
        assert!(p.tobler_ms(-0.3) < p.tobler_ms(-0.05));
    }

    #[test]
    fn a_flat_straight_walk_takes_distance_over_speed() {
        let map = flat(64, 64);
        let nav = NavGrid::new(&map, params());
        let off = |_| 0.0;
        let route = found(nav.route(&map.elevation, 0, 40, &off, 100_000));
        let metres = 40.0 * 8.0;
        let expected = metres / (params().tobler_ms(0.0) * 0.6);
        assert!((f64::from(route.total_seconds()) - expected).abs() < 0.01 * expected);
        assert_eq!(route.cells.first(), Some(&0));
        assert_eq!(route.cells.last(), Some(&40));
        assert!(route.seconds.windows(2).all(|w| w[1] > w[0]));
    }

    #[test]
    fn trails_are_faster_but_never_beat_the_top_speed() {
        let map = flat(64, 64);
        let nav = NavGrid::new(&map, params());
        let off = nav.route(&map.elevation, 0, 40, &|_| 0.0, 100_000);
        let on = nav.route(&map.elevation, 0, 40, &|_| 1.0, 100_000);
        let (off, on) = (found(off), found(on));
        assert!(on.total_seconds() < off.total_seconds());
        let metres = 40.0 * 8.0;
        assert!(f64::from(on.total_seconds()) >= metres / params().top_speed_ms() - 1e-3);
    }

    #[test]
    fn lakes_force_a_detour_and_islands_are_unreachable() {
        let mut map = flat(32, 32);
        // A wall of lake across the middle with a gap at the bottom.
        for y in 0..28 {
            map.water[y * 32 + 16] = WATER_LAKE;
        }
        let nav = NavGrid::new(&map, params());
        let route = found(nav.route(&map.elevation, 10 * 32 + 8, 10 * 32 + 24, &|_| 0.0, 100_000));
        assert!(
            route
                .cells
                .iter()
                .all(|&c| map.water[c as usize] == WATER_LAND)
        );
        assert!(route.cells.len() > 17, "the route goes around the lake");

        // Close the gap: the far side becomes unreachable.
        for y in 28..32 {
            map.water[y * 32 + 16] = WATER_LAKE;
        }
        let nav = NavGrid::new(&map, params());
        assert_eq!(
            nav.route(&map.elevation, 10 * 32 + 8, 10 * 32 + 24, &|_| 0.0, 100_000),
            RouteResult::Unreachable
        );
        assert_eq!(
            nav.route(&map.elevation, 0, 31 * 32 + 31, &|_| 0.0, 10),
            RouteResult::BudgetExhausted
        );
    }

    #[test]
    fn small_rivers_are_waded_and_big_ones_are_not() {
        let mut map = flat(32, 32);
        let cells: Vec<u32> = (0..32).map(|y| y * 32 + 16).collect();
        for &c in &cells {
            map.water[c as usize] = WATER_RIVER;
        }
        map.reaches.push(RiverReach {
            id: 0,
            cells,
            downstream: None,
            terminus: Terminus::Outlet,
            order: 1,
            discharge_m3s: 1.0,
            width_m: 4.0,
            drainage_area_km2: 50.0,
        });
        let nav = NavGrid::new(&map, params());
        assert!(matches!(
            nav.route(&map.elevation, 5 * 32 + 8, 5 * 32 + 24, &|_| 0.0, 100_000),
            RouteResult::Found(_)
        ));
        map.reaches[0].discharge_m3s = 40.0;
        let nav = NavGrid::new(&map, params());
        assert_eq!(
            nav.route(&map.elevation, 5 * 32 + 8, 5 * 32 + 24, &|_| 0.0, 100_000),
            RouteResult::Unreachable
        );
    }

    #[test]
    fn a_star_matches_dijkstra_on_rough_ground() {
        let mut map = flat(48, 48);
        let mut rng = Rng64::seed_from_u64(7);
        for z in &mut map.elevation {
            *z = 100.0 + (rng.next_f64() * 6.0) as f32;
        }
        for _ in 0..200 {
            let c = (rng.next_u64() % 2304) as usize;
            map.water[c] = WATER_LAKE;
        }
        let nav = NavGrid::new(&map, params());
        let trail = |c: usize| if c.is_multiple_of(7) { 1.0 } else { 0.0 };
        let mut checked = 0;
        for _ in 0..60 {
            let a = (rng.next_u64() % 2304) as usize;
            let b = (rng.next_u64() % 2304) as usize;
            if !nav.walkable(a) || !nav.walkable(b) {
                continue;
            }
            let field = nav.travel_field(&map.elevation, a, f32::MAX, &trail);
            match nav.route(&map.elevation, a, b, &trail, 1_000_000) {
                RouteResult::Found(r) => {
                    let exact = field.seconds_to(b).expect("dijkstra reached it");
                    assert!((r.total_seconds() - exact).abs() <= 1e-3 * exact.max(1.0));
                    checked += 1;
                }
                RouteResult::Unreachable => assert!(field.seconds_to(b).is_none()),
                RouteResult::BudgetExhausted => panic!("budget too small"),
            }
        }
        assert!(checked > 20);
    }

    #[test]
    fn landmarks_bound_searches_without_changing_route_times() {
        // Hills (uphill and downhill differ), lakes, an island and trails; landmarks over part
        // of the map only, with a time cap short enough that some cells are not worked out.
        let mut map = flat(64, 64);
        let mut rng = Rng64::seed_from_u64(11);
        for (c, z) in map.elevation.iter_mut().enumerate() {
            let (x, y) = ((c % 64) as f32, (c / 64) as f32);
            *z = 100.0 + 8.0 * (x / 9.0).sin() * (y / 7.0).cos() + (rng.next_f64() * 2.0) as f32;
        }
        for _ in 0..300 {
            let c = (rng.next_u64() % 4096) as usize;
            map.water[c] = WATER_LAKE;
        }
        // A ring of water round an island at (50, 50).
        for y in 44..57usize {
            for x in 44..57usize {
                let ring = x == 44 || x == 56 || y == 44 || y == 56;
                if ring {
                    map.water[y * 64 + x] = WATER_LAKE;
                } else {
                    map.water[y * 64 + x] = WATER_LAND;
                }
            }
        }
        let nav = NavGrid::new(&map, params());
        let trail = |c: usize| if (c / 64) % 9 == 3 { 1.0 } else { 0.0 };
        let whole = nav.landmarks(&map.elevation, &trail, ((0, 0), (63, 63)));
        let part = nav.landmarks(&map.elevation, &trail, ((10, 5), (60, 58)));
        let mut capped = part.clone();
        // Pretend the far cells were never worked out: at least their time's half.
        for t in &mut capped.seconds {
            if t.is_finite() && *t > 200.0 {
                *t = -*t * 0.5;
            }
        }
        let (mut checked, mut unreachable) = (0, 0);
        for _ in 0..400 {
            let a = (rng.next_u64() % 4096) as usize;
            let b = (rng.next_u64() % 4096) as usize;
            if !nav.walkable(b) {
                continue;
            }
            let plain = nav.route(&map.elevation, a, b, &trail, 1_000_000);
            for lm in [&whole, &part, &capped] {
                let bounded =
                    nav.route_with(&map.elevation, (a, b), &trail, 1.0, 1_000_000, Some(lm));
                match (&plain, &bounded) {
                    (RouteResult::Found(p), RouteResult::Found(q)) => {
                        assert_eq!(
                            p.total_seconds(),
                            q.total_seconds(),
                            "{a} to {b} with {lm:?}"
                        );
                        assert_eq!(q.cells.first(), Some(&(a as u32)));
                        assert_eq!(q.cells.last(), Some(&(b as u32)));
                    }
                    (RouteResult::Unreachable, RouteResult::Unreachable) => {}
                    (p, q) => panic!("{a} to {b} with {lm:?}: {p:?} against {q:?}"),
                }
            }
            match plain {
                RouteResult::Found(_) => checked += 1,
                _ => unreachable += 1,
            }
        }
        assert!(
            checked > 100 && unreachable > 5,
            "{checked} found, {unreachable} not"
        );
        // The bounds pay: fewer cells looked at for the same route.
        let before = cells_searched();
        let _ = nav.route(&map.elevation, 64 + 1, 62 * 64 + 2, &trail, 1_000_000);
        let plain = cells_searched() - before;
        let before = cells_searched();
        let _ = nav.route_with(
            &map.elevation,
            (64 + 1, 62 * 64 + 2),
            &trail,
            1.0,
            1_000_000,
            Some(&whole),
        );
        let bounded = cells_searched() - before;
        assert!(
            bounded < plain,
            "{bounded} cells with landmarks, {plain} without"
        );
    }

    #[test]
    fn kept_step_speeds_give_the_times_each_step_takes() {
        let mut map = flat(48, 48);
        let mut rng = civ_core::Rng64::seed_from_u64(5);
        for (i, z) in map.elevation.iter_mut().enumerate() {
            *z = (i % 48) as f32 * 0.9 + (rng.next_f64() as f32) * 4.0;
        }
        let trail = |c: usize| ((c * 7) % 11) as f32 / 10.0;
        let (a, b) = (3 * 48 + 2, 44 * 48 + 45);
        // A fresh grid fills its table; the second search reads it; a second grid starts anew.
        let nav = NavGrid::new(&map, params());
        let first = nav.route(&map.elevation, a, b, &trail, 1_000_000);
        let again = nav.route(&map.elevation, a, b, &trail, 1_000_000);
        let other = NavGrid::new(&map, params()).route(&map.elevation, a, b, &trail, 1_000_000);
        assert_eq!(first, again);
        assert_eq!(first, other);
        let RouteResult::Found(r) = first else {
            panic!("a route across open ground");
        };
        // Each time is the one before plus the step as `step_seconds` works it out, exactly.
        for k in 1..r.cells.len() {
            let (i, j) = (r.cells[k - 1] as usize, r.cells[k] as usize);
            let diagonal = i % 48 != j % 48 && i / 48 != j / 48;
            let dist = if diagonal {
                std::f64::consts::SQRT_2
            } else {
                1.0
            };
            let step = nav
                .step_seconds(&map.elevation, i, j, dist, trail(j))
                .expect("a walked step can be walked");
            assert_eq!(r.seconds[k].to_bits(), (r.seconds[k - 1] + step).to_bits());
        }
        // A travel field gives the same time to the goal.
        let field = nav.travel_field(&map.elevation, a, f32::MAX, &trail);
        let exact = field.seconds_to(b).expect("reached");
        assert!((r.total_seconds() - exact).abs() <= 1e-3 * exact);
    }

    #[test]
    fn a_bounded_search_without_trails_is_still_exact() {
        let mut map = flat(48, 48);
        let mut rng = civ_core::Rng64::seed_from_u64(11);
        for (i, z) in map.elevation.iter_mut().enumerate() {
            *z = (i % 48) as f32 * 0.7 + (rng.next_f64() as f32) * 3.0;
        }
        let nav = NavGrid::new(&map, params());
        let none = |_: usize| 0.0;
        for _ in 0..30 {
            let a = (rng.next_u64() % 2304) as usize;
            let b = (rng.next_u64() % 2304) as usize;
            let field = nav.travel_field(&map.elevation, a, f32::MAX, &none);
            if let RouteResult::Found(r) =
                nav.route_bounded(&map.elevation, a, b, &none, 0.0, 1_000_000)
            {
                let exact = field.seconds_to(b).expect("dijkstra reached it");
                assert!((r.total_seconds() - exact).abs() <= 1e-3 * exact.max(1.0));
            }
        }
    }

    #[test]
    fn open_ground_is_walked_straight_and_faster_than_the_grid_allows() {
        let map = flat(64, 64);
        let nav = NavGrid::new(&map, params());
        let off = |_: usize| 0.0;
        let route = found(nav.route(&map.elevation, 0, 20 * 64 + 40, &off, 1_000_000));
        let (pts, secs) = nav.straighten(&map.elevation, &route, &off);
        assert_eq!(pts.len(), 2, "one straight line: {pts:?}");
        let metres = ((40.0f64).powi(2) + 20.0f64.powi(2)).sqrt() * 8.0;
        let expected = metres / (params().tobler_ms(0.0) * 0.6);
        let total = f64::from(*secs.last().expect("non-empty"));
        assert!(
            (total - expected).abs() < 0.01 * expected,
            "{total} against {expected}"
        );
        assert!(
            secs.last() < route.seconds.last(),
            "shorter than the grid's route"
        );
    }

    #[test]
    fn straightened_routes_keep_out_of_water_and_to_trails_that_pay() {
        let mut map = flat(48, 48);
        for y in 0..40 {
            map.water[y * 48 + 24] = WATER_LAKE;
        }
        let nav = NavGrid::new(&map, params());
        let off = |_: usize| 0.0;
        let route = found(nav.route(&map.elevation, 10 * 48 + 8, 10 * 48 + 40, &off, 1_000_000));
        let (pts, secs) = nav.straighten(&map.elevation, &route, &off);
        assert!(pts.len() >= 3, "round the end of the lake: {pts:?}");
        for w in pts.windows(2) {
            assert!(
                nav.segment_seconds(&map.elevation, w[0], w[1], &off)
                    .is_some()
            );
        }
        assert!(secs.last() <= route.seconds.last());

        // An L-shaped trail: the corner is cut only if leaving the trail is not slower.
        let map = flat(64, 64);
        let nav = NavGrid::new(&map, params());
        let on_l = |c: usize| {
            let (x, y) = (c % 64, c / 64);
            if (y == 5 && x <= 45) || (x == 45 && y >= 5) {
                1.0
            } else {
                0.0
            }
        };
        let route = found(nav.route(&map.elevation, 5 * 64 + 5, 45 * 64 + 45, &on_l, 1_000_000));
        let (pts, secs) = nav.straighten(&map.elevation, &route, &on_l);
        let total = *secs.last().expect("non-empty");
        assert!(total <= *route.seconds.last().expect("non-empty") * 1.001);
        assert!(
            pts.iter()
                .any(|&(x, y)| (x - 45.5 * 8.0).abs() < 9.0 && (y - 5.5 * 8.0).abs() < 9.0),
            "keeps to the trail round its corner: {pts:?}"
        );
    }

    #[test]
    fn polylines_keep_timings_at_their_vertices() {
        let map = flat(64, 64);
        let nav = NavGrid::new(&map, params());
        let route = found(nav.route(&map.elevation, 0, 63 * 64 + 40, &|_| 0.0, 1_000_000));
        let (pts, secs) = nav.polyline(&route, 4.0);
        assert!(pts.len() >= 2 && pts.len() < route.cells.len());
        assert_eq!(pts.len(), secs.len());
        assert_eq!(secs[0], 0.0);
        assert_eq!(*secs.last().expect("non-empty"), route.total_seconds());
        assert!(secs.windows(2).all(|w| w[1] >= w[0]));
    }
}
