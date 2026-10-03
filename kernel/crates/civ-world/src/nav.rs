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

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

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
    width: usize,
    height: usize,
    cell_m: f64,
    params: NavParams,
    ground: Vec<f32>,
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

#[derive(Clone, Copy, PartialEq)]
struct Open {
    f: f32,
    g: f32,
    cell: u32,
}

impl Eq for Open {}

impl Ord for Open {
    fn cmp(&self, other: &Self) -> Ordering {
        // Min-heap on f, then on g (prefer deeper nodes), then cell index for a total order.
        other
            .f
            .total_cmp(&self.f)
            .then_with(|| self.g.total_cmp(&other.g))
            .then_with(|| other.cell.cmp(&self.cell))
    }
}

impl PartialOrd for Open {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
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
        let ground = f64::from(self.ground[to]);
        if ground <= 0.0 {
            return None;
        }
        let run = dist_cells * self.cell_m;
        let rise = f64::from(elevation[to] - elevation[from]);
        let slope = rise / run;
        if slope.abs() > self.params.max_slope {
            return None;
        }
        let surface = self.params.offtrail_factor
            + (1.0 - self.params.offtrail_factor) * f64::from(trail.clamp(0.0, 1.0));
        let speed = self.params.tobler_ms(slope) * surface * ground;
        Some((run / speed) as f32)
    }

    /// Seconds to walk straight from `from` to `to` at `speed`, an underestimate of any route
    /// when `speed` is the fastest any step can be walked.
    fn heuristic(&self, from: usize, to: usize, speed: f64) -> f32 {
        let (fx, fy) = ((from % self.width) as f64, (from / self.width) as f64);
        let (tx, ty) = ((to % self.width) as f64, (to / self.width) as f64);
        let dist = ((fx - tx).powi(2) + (fy - ty).powi(2)).sqrt() * self.cell_m;
        (dist / speed) as f32
    }

    fn neighbours(&self, i: usize) -> impl Iterator<Item = (usize, f64)> + '_ {
        let (w, h) = (self.width as i64, self.height as i64);
        let (x, y) = ((i % self.width) as i64, (i / self.width) as i64);
        D8.iter().zip(D8_DIST).filter_map(move |(&(dx, dy), dist)| {
            let (nx, ny) = (x + i64::from(dx), y + i64::from(dy));
            if nx < 0 || ny < 0 || nx >= w || ny >= h {
                None
            } else {
                Some(((ny * w + nx) as usize, dist))
            }
        })
    }

    /// The fastest route from `from` to `to` (A*). `trail` gives the trail factor (0–1) of a
    /// cell. The search expands at most `budget` cells.
    pub fn route(
        &self,
        elevation: &[f32],
        from: usize,
        to: usize,
        trail: &dyn Fn(usize) -> f32,
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
        trail: &dyn Fn(usize) -> f32,
        trail_max: f32,
        budget: usize,
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
        let mut best: HashMap<u32, (f32, u32)> = HashMap::new();
        let mut open = BinaryHeap::new();
        best.insert(from as u32, (0.0, u32::MAX));
        open.push(Open {
            f: self.heuristic(from, to, fastest),
            g: 0.0,
            cell: from as u32,
        });
        let mut expanded = 0usize;
        while let Some(Open { g, cell, .. }) = open.pop() {
            let i = cell as usize;
            if best.get(&cell).is_some_and(|&(bg, _)| g > bg) {
                continue;
            }
            if i == to {
                return RouteResult::Found(reconstruct(&best, from, to));
            }
            expanded += 1;
            if expanded > budget {
                return RouteResult::BudgetExhausted;
            }
            for (j, dist) in self.neighbours(i) {
                let Some(step) = self.step_seconds(elevation, i, j, dist, trail(j)) else {
                    continue;
                };
                let ng = g + step;
                let better = best.get(&(j as u32)).is_none_or(|&(bg, _)| ng < bg);
                if better {
                    best.insert(j as u32, (ng, cell));
                    open.push(Open {
                        f: ng + self.heuristic(j, to, fastest),
                        g: ng,
                        cell: j as u32,
                    });
                }
            }
        }
        RouteResult::Unreachable
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
        let bw = field.w;
        field.seconds = vec![f32::INFINITY; bw * field.h];
        // Each cell's trail factor, asked once.
        let mut trails = vec![f32::NAN; bw * field.h];
        let mut open = BinaryHeap::new();
        field.seconds[(fy - y0) * bw + (fx - x0)] = 0.0;
        open.push(Open {
            f: 0.0,
            g: 0.0,
            cell: from as u32,
        });
        while let Some(Open { g, cell, .. }) = open.pop() {
            let i = cell as usize;
            let (x, y) = (i % self.width, i / self.width);
            if g > field.seconds[(y - y0) * bw + (x - x0)] {
                continue;
            }
            field.reached.push(cell);
            for (&(dx, dy), dist) in D8.iter().zip(D8_DIST) {
                let (nx, ny) = (x as i64 + i64::from(dx), y as i64 + i64::from(dy));
                if nx < x0 as i64 || ny < y0 as i64 || nx > x1 as i64 || ny > y1 as i64 {
                    continue;
                }
                let (nx, ny) = (nx as usize, ny as usize);
                let j = ny * self.width + nx;
                let sj = (ny - y0) * bw + (nx - x0);
                if trails[sj].is_nan() {
                    trails[sj] = trail(j);
                }
                let Some(step) = self.step_seconds(elevation, i, j, dist, trails[sj]) else {
                    continue;
                };
                let ng = g + step;
                if ng > max_seconds || ng >= field.seconds[sj] {
                    continue;
                }
                field.seconds[sj] = ng;
                open.push(Open {
                    f: ng,
                    g: ng,
                    cell: j as u32,
                });
            }
        }
        field
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

fn reconstruct(best: &HashMap<u32, (f32, u32)>, from: usize, to: usize) -> Route {
    let mut cells = Vec::new();
    let mut seconds = Vec::new();
    let mut cur = to as u32;
    loop {
        let (g, parent) = best[&cur];
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
