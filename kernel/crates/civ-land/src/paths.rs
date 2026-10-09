//! Ground worn by walking, and the trails it becomes (ADR-0004 §4; research 10-03 §1.1, §2.2).
//!
//! Every 8 m cell has a wear value `w` from 0 (untrodden) to 1. Walking across a cell wears it,
//! `w ← w + α(1 − w)`, so each walk adds less than the one before; unused wear fades with an
//! authored half-life as plants grow back. This is the exact solution of research 10-03's
//! `dw/dt = αq(1 − w) − βw` with each walk an instant, so wear never leaves 0–1 however busy a
//! cell is. Wear lives in sparse 64 × 64-cell tiles where people have walked; a tile is brought up
//! to date, in whole days, when it is next walked or surveyed.
//!
//! A cell becomes **trail** when its wear reaches `trail_at` and stays trail until it fades below
//! `trail_until`: a trail outlives a lapse in use (hysteresis).
//!
//! Once a month the paths are **surveyed**: every tile is brought up to date, trail states change,
//! faded tiles are forgotten, the **routing view** is rebuilt and the trails are traced into
//! polylines. Routes are planned on that view, so a cached route, a household's travel-time field
//! and the trails drawn on the map agree until the next survey. Walking is faster the more worn the
//! ground (`civ_world::nav::NavParams::offtrail_factor`), so people are drawn to the ground others
//! wore and use makes more use.

use std::collections::{HashMap, HashSet, VecDeque};

use civ_world::nav::simplify_indices;

/// Cells per tile side.
pub const TILE: u32 = 64;
/// Cells per tile.
pub const TILE_CELLS: usize = (TILE * TILE) as usize;
/// Wear below which a cell counts as untrodden; a tile with nothing above it and no trail is
/// forgotten at a survey.
const FORGOTTEN: f32 = 0.005;
/// Trails run through their cells' centres, simplified to within a cell: where in its cell a
/// trail runs is not known, so the steps of a line across the grid are not kept.
const TRAIL_TOLERANCE_CELLS: f32 = 1.0;
/// Branches of a trail shorter than this many cells that end in nothing are survey noise around a
/// junction, not trails of their own.
const SPUR_CELLS: usize = 3;

/// How walking wears the ground. Authored in content.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PathParams {
    /// Share of a cell's remaining unworn state one walk across it wears away (α).
    pub wear_per_walk: f64,
    /// Days unused wear takes to halve.
    pub half_life_days: f64,
    /// Wear at which a cell becomes trail.
    pub trail_at: f64,
    /// Wear below which a trail cell stops being one.
    pub trail_until: f64,
}

impl PathParams {
    /// Share of wear left after `days` unused.
    pub fn fade(&self, days: i64) -> f64 {
        if days <= 0 {
            1.0
        } else {
            0.5f64.powf(days as f64 / self.half_life_days)
        }
    }

    /// Wear a cell settles at when walked `per_day` times a day, every day.
    pub fn steady_wear(&self, per_day: f64) -> f64 {
        let beta = std::f64::consts::LN_2 / self.half_life_days;
        let gain = -(1.0 - self.wear_per_walk).ln() * per_day;
        if gain + beta <= 0.0 {
            0.0
        } else {
            gain / (gain + beta)
        }
    }
}

/// One 64 × 64-cell tile of wear.
#[derive(Clone, Debug, PartialEq)]
pub struct WearTile {
    /// Tile index: tile row × tiles per row + tile column.
    pub index: u32,
    /// The day its values are current to.
    pub day: i64,
    /// Wear of each cell, row by row.
    pub wear: Vec<f32>,
    /// Trail state of each cell, one bit each, row by row.
    pub trail: Vec<u64>,
}

impl WearTile {
    /// An untrodden tile current to `day`.
    pub fn new(index: u32, day: i64) -> Self {
        WearTile {
            index,
            day,
            wear: vec![0.0; TILE_CELLS],
            trail: vec![0; TILE_CELLS / 64],
        }
    }

    /// Whether cell `local` of the tile is trail.
    pub fn is_trail(&self, local: usize) -> bool {
        self.trail[local / 64] >> (local % 64) & 1 == 1
    }

    fn set_trail(&mut self, local: usize, on: bool) {
        if on {
            self.trail[local / 64] |= 1 << (local % 64);
        } else {
            self.trail[local / 64] &= !(1 << (local % 64));
        }
    }

    /// Fades the tile's wear to `day` and lets trails that faded below `trail_until` lapse.
    pub fn bring_to(&mut self, day: i64, params: &PathParams) {
        if day <= self.day {
            return;
        }
        let f = params.fade(day - self.day) as f32;
        for w in &mut self.wear {
            *w *= f;
        }
        self.day = day;
        let until = params.trail_until as f32;
        for k in 0..self.trail.len() {
            let mut bits = self.trail[k];
            while bits != 0 {
                let b = bits.trailing_zeros() as usize;
                bits &= bits - 1;
                if self.wear[k * 64 + b] < until {
                    self.trail[k] &= !(1u64 << b);
                }
            }
        }
    }

    fn forgotten(&self) -> bool {
        self.trail.iter().all(|&w| w == 0) && self.wear.iter().all(|&w| w < FORGOTTEN)
    }
}

/// One tile of the routing view as surveyed: what a save keeps so that routes are planned on the
/// same paths after loading (ADR-0011 §5).
#[derive(Clone, Debug, PartialEq)]
pub struct ViewTile {
    /// Tile index, as [`WearTile::index`].
    pub index: u32,
    /// Wear of each cell as surveyed, row by row, 0–255 for 0–1.
    pub cells: Vec<u8>,
    /// Trail state of each cell as surveyed, as [`WearTile::trail`].
    pub trail: Vec<u64>,
}

/// A trail traced through trail cells: a polyline in metres from the map's north-west corner.
#[derive(Clone, Debug, PartialEq)]
pub struct Trail {
    /// Vertices, metres.
    pub points: Vec<(f32, f32)>,
    /// Mean wear of the cells it runs through.
    pub wear: f32,
}

impl Trail {
    /// Length, metres.
    pub fn length_m(&self) -> f32 {
        self.points
            .windows(2)
            .map(|w| ((w[1].0 - w[0].0).powi(2) + (w[1].1 - w[0].1).powi(2)).sqrt())
            .sum()
    }
}

/// The worn ground of a world. The tiles and the routing view are saved; the trails are traced
/// from the view at each survey and on load.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Wear {
    width: u32,
    height: u32,
    tiles_x: u32,
    cell_m: f32,
    tiles: Vec<WearTile>,
    slot: HashMap<u32, usize>,
    /// Changes at every survey: the routing view and trails it describes.
    rev: u32,
    /// The day of the last survey.
    surveyed: i64,
    view_slot: Vec<u32>,
    view: Vec<u8>,
    /// The view again, a byte per cell of the map row by row, for [`Wear::factor`]: a route
    /// search asks it for every cell it looks at.
    view_cells: Vec<u8>,
    view_max: u8,
    /// Per tile of the view, in order: its index and trail bits as surveyed.
    view_index: Vec<u32>,
    view_trail: Vec<u64>,
    trails: Vec<Trail>,
}

impl Wear {
    /// Untrodden ground for a map of `width` × `height` cells of `cell_m` metres.
    pub fn new(width: u32, height: u32, cell_m: f32) -> Self {
        let tiles_x = width.div_ceil(TILE);
        let tiles_y = height.div_ceil(TILE);
        Wear {
            width,
            height,
            tiles_x,
            cell_m,
            view_slot: vec![u32::MAX; (tiles_x * tiles_y) as usize],
            ..Wear::default()
        }
    }

    /// Restores saved tiles. Their trail states are kept as saved; the routing view and trails
    /// wait for [`Wear::with_view`] or [`Wear::survey`].
    pub fn with_tiles(mut self, mut tiles: Vec<WearTile>) -> Self {
        tiles.sort_by_key(|t| t.index);
        self.slot = tiles
            .iter()
            .enumerate()
            .map(|(k, t)| (t.index, k))
            .collect();
        self.tiles = tiles;
        self
    }

    /// Tiles per row.
    pub fn tiles_x(&self) -> u32 {
        self.tiles_x
    }

    /// Number of tiles in the grid (most hold nothing).
    pub fn tile_count(&self) -> u32 {
        self.tiles_x * self.height.div_ceil(TILE)
    }

    /// The routing view's revision.
    pub fn rev(&self) -> u32 {
        self.rev
    }

    /// Whether the routing view has been drawn, by a survey or from a save.
    pub fn has_view(&self) -> bool {
        self.rev != 0
    }

    /// Restores the routing view surveyed on `surveyed`, as [`Wear::surveyed_tiles`] gave it, and
    /// traces its trails: what [`Wear::survey`] drew then, without surveying again.
    pub fn with_view(mut self, surveyed: i64, mut tiles: Vec<ViewTile>) -> Self {
        tiles.sort_by_key(|t| t.index);
        let words = TILE_CELLS / 64;
        self.view_slot = vec![u32::MAX; self.tile_count() as usize];
        self.view = Vec::with_capacity(tiles.len() * TILE_CELLS);
        self.view_trail = Vec::with_capacity(tiles.len() * words);
        self.view_index = Vec::with_capacity(tiles.len());
        self.view_max = 0;
        for (k, t) in tiles.into_iter().enumerate() {
            if let Some(s) = self.view_slot.get_mut(t.index as usize) {
                *s = k as u32;
            }
            self.view_index.push(t.index);
            let mut cells = t.cells;
            cells.resize(TILE_CELLS, 0);
            self.view_max = self.view_max.max(cells.iter().copied().max().unwrap_or(0));
            self.view.extend_from_slice(&cells);
            let mut trail = t.trail;
            trail.resize(words, 0);
            self.view_trail.extend_from_slice(&trail);
        }
        self.spread_view();
        self.trails = self.trace();
        self.surveyed = surveyed;
        self.rev = self.rev.wrapping_add(1);
        self
    }

    /// The day of the last survey.
    pub fn surveyed(&self) -> i64 {
        self.surveyed
    }

    /// The tiles people have walked, in no particular order, with values as of each tile's day.
    pub fn tiles(&self) -> &[WearTile] {
        &self.tiles
    }

    /// The tiles as they stand on `day`: faded, with lapsed trails cleared and forgotten tiles
    /// left out, in tile order. What a save keeps.
    pub fn current(&self, day: i64, params: &PathParams) -> Vec<WearTile> {
        let mut out: Vec<WearTile> = self
            .tiles
            .iter()
            .map(|t| {
                let mut t = t.clone();
                t.bring_to(day, params);
                t
            })
            .filter(|t| !t.forgotten())
            .collect();
        out.sort_by_key(|t| t.index);
        out
    }

    /// The trails traced at the last survey.
    pub fn trails(&self) -> &[Trail] {
        &self.trails
    }

    /// Tile index and position within it of a cell.
    pub fn locate(&self, cell: usize) -> (u32, usize) {
        let w = self.width as usize;
        let (x, y) = (cell % w, cell / w);
        let t = (y / TILE as usize) * self.tiles_x as usize + x / TILE as usize;
        let local = (y % TILE as usize) * TILE as usize + x % TILE as usize;
        (t as u32, local)
    }

    /// Wear of `cell` on `day`.
    pub fn wear_at(&self, cell: usize, day: i64, params: &PathParams) -> f64 {
        let (t, local) = self.locate(cell);
        self.slot.get(&t).map_or(0.0, |&k| {
            let tile = &self.tiles[k];
            f64::from(tile.wear[local]) * params.fade(day - tile.day)
        })
    }

    /// Whether `cell` is trail, as last brought up to date.
    pub fn is_trail(&self, cell: usize) -> bool {
        let (t, local) = self.locate(cell);
        self.slot
            .get(&t)
            .is_some_and(|&k| self.tiles[k].is_trail(local))
    }

    /// Someone walked across `cells` on `day` (each cell once).
    pub fn walk(&mut self, cells: &[u32], day: i64, params: &PathParams) {
        let alpha = params.wear_per_walk as f32;
        let at = params.trail_at as f32;
        let mut last: Option<(u32, usize)> = None;
        for &cell in cells {
            if cell >= self.width * self.height {
                continue;
            }
            let (t, local) = self.locate(cell as usize);
            let k = match last {
                Some((lt, k)) if lt == t => k,
                _ => {
                    let k = match self.slot.get(&t) {
                        Some(&k) => k,
                        None => {
                            self.tiles.push(WearTile::new(t, day));
                            self.slot.insert(t, self.tiles.len() - 1);
                            self.tiles.len() - 1
                        }
                    };
                    self.tiles[k].bring_to(day, params);
                    last = Some((t, k));
                    k
                }
            };
            let tile = &mut self.tiles[k];
            let w = &mut tile.wear[local];
            *w += alpha * (1.0 - *w);
            if *w >= at {
                tile.set_trail(local, true);
            }
        }
    }

    /// Brings every tile up to `day`, forgets faded tiles, rebuilds the routing view and traces
    /// the trails.
    pub fn survey(&mut self, day: i64, params: &PathParams) {
        for t in &mut self.tiles {
            t.bring_to(day, params);
        }
        self.tiles.retain(|t| !t.forgotten());
        self.tiles.sort_by_key(|t| t.index);
        self.slot = self
            .tiles
            .iter()
            .enumerate()
            .map(|(k, t)| (t.index, k))
            .collect();
        self.view_slot = vec![u32::MAX; self.tile_count() as usize];
        self.view = Vec::with_capacity(self.tiles.len() * TILE_CELLS);
        self.view_max = 0;
        self.view_index = self.tiles.iter().map(|t| t.index).collect();
        let words = TILE_CELLS / 64;
        self.view_trail = self
            .tiles
            .iter()
            .flat_map(|t| (0..words).map(|j| t.trail.get(j).copied().unwrap_or(0)))
            .collect();
        for (k, t) in self.tiles.iter().enumerate() {
            if let Some(s) = self.view_slot.get_mut(t.index as usize) {
                *s = k as u32;
            }
            for &w in &t.wear {
                let v = (w.clamp(0.0, 1.0) * 255.0).round() as u8;
                self.view_max = self.view_max.max(v);
                self.view.push(v);
            }
        }
        self.spread_view();
        self.trails = self.trace();
        self.surveyed = day;
        self.rev = self.rev.wrapping_add(1);
    }

    /// How worn `cell` was at the last survey, 0–1: the trail factor routes are planned with.
    pub fn factor(&self, cell: usize) -> f32 {
        self.view_cells
            .get(cell)
            .map_or(0.0, |&v| f32::from(v) / 255.0)
    }

    /// Lays the view's tiles out a byte per cell of the map ([`Wear::factor`]).
    fn spread_view(&mut self) {
        let (w, h) = (self.width as usize, self.height as usize);
        self.view_cells.clear();
        self.view_cells.resize(w * h, 0);
        let tile = TILE as usize;
        for (k, &index) in self.view_index.iter().enumerate() {
            let (tx, ty) = (
                index as usize % self.tiles_x as usize,
                index as usize / self.tiles_x as usize,
            );
            let cells = &self.view[k * TILE_CELLS..(k + 1) * TILE_CELLS];
            for ly in 0..tile {
                let y = ty * tile + ly;
                if y >= h {
                    break;
                }
                let x0 = tx * tile;
                if x0 >= w {
                    break;
                }
                let n = tile.min(w - x0);
                self.view_cells[y * w + x0..y * w + x0 + n]
                    .copy_from_slice(&cells[ly * tile..ly * tile + n]);
            }
        }
    }

    /// The highest [`Wear::factor`] of any cell.
    pub fn max_factor(&self) -> f32 {
        f32::from(self.view_max) / 255.0
    }

    /// The ground as last surveyed, tile by tile in tile order: the tile's index, the wear of
    /// each cell row by row (0–255 for 0–1), and its trail bits.
    pub fn surveyed_tiles(&self) -> impl Iterator<Item = (u32, &[u8], &[u64])> + '_ {
        let words = TILE_CELLS / 64;
        self.view_index.iter().enumerate().map(move |(k, &index)| {
            (
                index,
                &self.view[k * TILE_CELLS..(k + 1) * TILE_CELLS],
                &self.view_trail[k * words..(k + 1) * words],
            )
        })
    }

    /// The cells people have walked as last surveyed, as the north-west and south-east corner
    /// cells of the tiles around them, if they have walked anywhere.
    pub fn walked_area(&self) -> Option<((usize, usize), (usize, usize))> {
        let (tx, tile) = (self.tiles_x as usize, TILE as usize);
        let (mut x0, mut y0, mut x1, mut y1) = (usize::MAX, usize::MAX, 0, 0);
        for &index in &self.view_index {
            let (x, y) = (index as usize % tx, index as usize / tx);
            (x0, x1) = (x0.min(x), x1.max(x));
            (y0, y1) = (y0.min(y), y1.max(y));
        }
        let (w, h) = (self.width as usize, self.height as usize);
        (x0 <= x1).then(|| {
            (
                (x0 * tile, y0 * tile),
                (((x1 + 1) * tile).min(w) - 1, ((y1 + 1) * tile).min(h) - 1),
            )
        })
    }

    /// What is wrong with saved wear for its map, if anything.
    pub fn problems(&self) -> Vec<String> {
        let mut out = Vec::new();
        let count = self.tile_count();
        let mut seen = HashSet::new();
        for t in &self.tiles {
            if t.index >= count {
                out.push(format!("wear tile {} is outside the map", t.index));
            }
            if !seen.insert(t.index) {
                out.push(format!("wear tile {} is listed twice", t.index));
            }
            if t.wear.len() != TILE_CELLS || t.trail.len() != TILE_CELLS / 64 {
                out.push(format!("wear tile {} has the wrong size", t.index));
            }
            if t.wear.iter().any(|w| !(0.0..=1.0).contains(w)) {
                out.push(format!("wear tile {} has wear outside 0-1", t.index));
            }
        }
        out
    }

    /// Traces trail cells into polylines: thinned to single cells, then followed from end to end
    /// and junction to junction.
    /// The trails of the routing view: its trail cells, traced.
    fn trace(&self) -> Vec<Trail> {
        let mut cells: HashSet<(i32, i32)> = HashSet::new();
        let tx = self.tiles_x as i32;
        let words = TILE_CELLS / 64;
        for (t, &index) in self.view_index.iter().enumerate() {
            let (ox, oy) = (
                (index as i32 % tx) * TILE as i32,
                (index as i32 / tx) * TILE as i32,
            );
            for k in 0..words {
                let mut bits = self.view_trail[t * words + k];
                while bits != 0 {
                    let b = bits.trailing_zeros() as usize;
                    bits &= bits - 1;
                    let local = (k * 64 + b) as i32;
                    cells.insert((ox + local % TILE as i32, oy + local / TILE as i32));
                }
            }
        }
        let mut out = Vec::new();
        for component in components(&close(&cells)) {
            let skeleton = thin(&component);
            for chain in chains(&skeleton) {
                let pts: Vec<(f32, f32)> = chain
                    .iter()
                    .map(|&(x, y)| {
                        (
                            (x as f32 + 0.5) * self.cell_m,
                            (y as f32 + 0.5) * self.cell_m,
                        )
                    })
                    .collect();
                let wear = chain
                    .iter()
                    .map(|&(x, y)| self.factor((y as u32 * self.width + x as u32) as usize))
                    .sum::<f32>()
                    / chain.len().max(1) as f32;
                let keep = simplify_indices(&pts, TRAIL_TOLERANCE_CELLS * self.cell_m);
                out.push(Trail {
                    points: keep.iter().map(|&k| pts[k]).collect(),
                    wear,
                });
            }
        }
        out
    }
}

/// The cells under a walk along `points` (metres), in order, each listed once in a row: the
/// polyline sampled every half cell. Cells off a `width` × `height` map are left out.
pub fn cells_along(points: &[(f32, f32)], cell_m: f32, width: u32, height: u32) -> Vec<u32> {
    let mut out: Vec<u32> = Vec::new();
    let mut push = |x: f32, y: f32| {
        let (cx, cy) = ((x / cell_m).floor(), (y / cell_m).floor());
        if cx < 0.0 || cy < 0.0 || cx >= width as f32 || cy >= height as f32 {
            return;
        }
        let c = cy as u32 * width + cx as u32;
        if out.last() != Some(&c) {
            out.push(c);
        }
    };
    if let Some(&(x, y)) = points.first() {
        push(x, y);
    }
    for w in points.windows(2) {
        let (a, b) = (w[0], w[1]);
        let len = ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt();
        let steps = (len / (cell_m * 0.5)).ceil().max(1.0) as u32;
        for s in 1..=steps {
            let t = s as f32 / steps as f32;
            push(a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
        }
    }
    out
}

const N8: [(i32, i32); 8] = [
    (0, -1),
    (1, -1),
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
    (-1, -1),
];

/// Morphological closing: grows the cells by one in every direction, then shrinks them back, so
/// a trail broken by a cell or two that walks wore a little less is traced as one.
fn close(cells: &HashSet<(i32, i32)>) -> HashSet<(i32, i32)> {
    let mut grown: HashSet<(i32, i32)> = cells.clone();
    for &(x, y) in cells {
        for (dx, dy) in N8 {
            grown.insert((x + dx, y + dy));
        }
    }
    grown
        .iter()
        .copied()
        .filter(|&(x, y)| N8.iter().all(|&(dx, dy)| grown.contains(&(x + dx, y + dy))))
        .collect()
}

/// Groups cells that touch (eight ways), each group sorted, groups in order of their first cell.
fn components(cells: &HashSet<(i32, i32)>) -> Vec<Vec<(i32, i32)>> {
    let mut sorted: Vec<(i32, i32)> = cells.iter().copied().collect();
    sorted.sort_by_key(|&(x, y)| (y, x));
    let mut seen: HashSet<(i32, i32)> = HashSet::new();
    let mut out = Vec::new();
    for &start in &sorted {
        if !seen.insert(start) {
            continue;
        }
        let mut group = vec![start];
        let mut queue = VecDeque::from([start]);
        while let Some((x, y)) = queue.pop_front() {
            for (dx, dy) in N8 {
                let n = (x + dx, y + dy);
                if cells.contains(&n) && seen.insert(n) {
                    group.push(n);
                    queue.push_back(n);
                }
            }
        }
        group.sort_by_key(|&(x, y)| (y, x));
        out.push(group);
    }
    out
}

/// Zhang–Suen thinning with Lü and Wang's correction (a cell with only two neighbours is never
/// peeled): thins a group of cells down to lines about one cell wide, keeping its shape and
/// connections. Without the correction a line of cells stepping diagonally, as a straight walk
/// at most angles leaves, is eaten away from its ends.
fn thin(cells: &[(i32, i32)]) -> HashSet<(i32, i32)> {
    let mut set: HashSet<(i32, i32)> = cells.iter().copied().collect();
    let mut order: Vec<(i32, i32)> = cells.to_vec();
    loop {
        let mut changed = false;
        for pass in 0..2 {
            let mut gone = Vec::new();
            for &(x, y) in &order {
                if !set.contains(&(x, y)) {
                    continue;
                }
                let p: [bool; 8] = N8.map(|(dx, dy)| set.contains(&(x + dx, y + dy)));
                let b = p.iter().filter(|&&v| v).count();
                let a = (0..8).filter(|&i| !p[i] && p[(i + 1) % 8]).count();
                // p[0] north, p[2] east, p[4] south, p[6] west.
                let (n, e, s, w) = (p[0], p[2], p[4], p[6]);
                let rule = if pass == 0 {
                    !(n && e && s) && !(e && s && w)
                } else {
                    !(n && e && w) && !(n && s && w)
                };
                if (3..=6).contains(&b) && a == 1 && rule {
                    gone.push((x, y));
                }
            }
            for c in &gone {
                set.remove(c);
            }
            changed |= !gone.is_empty();
        }
        if !changed {
            break;
        }
        order.retain(|c| set.contains(c));
    }
    set
}

type Cell = (i32, i32);

/// Follows a skeleton one cell wide into chains of cells between ends and junctions. Cells link
/// orthogonally, and diagonally only where no orthogonal neighbour joins them already, so a corner
/// or a staircase is one line rather than a knot of junctions.
fn chains(skeleton: &HashSet<Cell>) -> Vec<Vec<Cell>> {
    let order = |c: &Cell| (c.1, c.0);
    let mut sorted: Vec<Cell> = skeleton.iter().copied().collect();
    sorted.sort_by_key(order);
    let adj: HashMap<Cell, Vec<Cell>> = sorted
        .iter()
        .map(|&(x, y)| {
            let mut next: Vec<Cell> = N8
                .iter()
                .filter(|&&(dx, dy)| {
                    let n = (x + dx, y + dy);
                    skeleton.contains(&n)
                        && (dx == 0
                            || dy == 0
                            || (!skeleton.contains(&(x + dx, y))
                                && !skeleton.contains(&(x, y + dy))))
                })
                .map(|&(dx, dy)| (x + dx, y + dy))
                .collect();
            next.sort_by_key(order);
            ((x, y), next)
        })
        .collect();
    let degree = |c: &Cell| adj.get(c).map_or(0, Vec::len);
    let key = |a: Cell, b: Cell| {
        if order(&a) <= order(&b) {
            (a, b)
        } else {
            (b, a)
        }
    };
    let walk = |start: Cell, next: Cell, used: &mut HashSet<(Cell, Cell)>| {
        let mut chain = vec![start];
        let (mut prev, mut cur) = (start, next);
        used.insert(key(prev, cur));
        loop {
            chain.push(cur);
            if degree(&cur) != 2 || cur == start {
                break;
            }
            let Some(&n) = adj[&cur].iter().find(|&&n| n != prev) else {
                break;
            };
            if !used.insert(key(cur, n)) {
                break;
            }
            prev = cur;
            cur = n;
        }
        chain
    };
    let mut used: HashSet<(Cell, Cell)> = HashSet::new();
    let mut out: Vec<Vec<Cell>> = Vec::new();
    for &c in &sorted {
        if degree(&c) == 2 {
            continue;
        }
        for &n in &adj[&c] {
            if !used.contains(&key(c, n)) {
                out.push(walk(c, n, &mut used));
            }
        }
    }
    // What is left are loops with no ends or junctions.
    for &c in &sorted {
        for &n in &adj[&c] {
            if !used.contains(&key(c, n)) {
                out.push(walk(c, n, &mut used));
            }
        }
    }
    // Short branches that end in nothing are noise from the thinning around junctions.
    out.retain(|chain| {
        let (da, db) = (degree(&chain[0]), degree(&chain[chain.len() - 1]));
        let spur = (da == 1 && db >= 3) || (db == 1 && da >= 3);
        let alone = da <= 1 && db <= 1;
        chain.len() >= 2 && !(spur && chain.len() <= SPUR_CELLS) && !(alone && chain.len() < 3)
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params() -> PathParams {
        PathParams {
            wear_per_walk: 0.01,
            half_life_days: 120.0,
            trail_at: 0.3,
            trail_until: 0.15,
        }
    }

    #[test]
    fn walking_wears_toward_one_and_unused_wear_halves() {
        let p = params();
        let mut wear = Wear::new(256, 256, 8.0);
        let cell = 70 * 256 + 70;
        for _ in 0..10 {
            wear.walk(&[cell], 0, &p);
        }
        let ten = 1.0 - 0.99f64.powi(10);
        assert!((wear.wear_at(cell as usize, 0, &p) - ten).abs() < 1e-5);
        assert!((wear.wear_at(cell as usize, 120, &p) - ten / 2.0).abs() < 1e-5);
        for _ in 0..2000 {
            wear.walk(&[cell], 0, &p);
        }
        let w = wear.wear_at(cell as usize, 0, &p);
        assert!(w > 0.999 && w <= 1.0, "saturates at one, got {w}");
    }

    #[test]
    fn steady_use_settles_where_wear_and_fading_balance() {
        let p = params();
        let mut wear = Wear::new(64, 64, 8.0);
        // Twice a day for three years.
        for day in 0..3 * 365 {
            wear.walk(&[100], day, &p);
            wear.walk(&[100], day, &p);
        }
        let w = wear.wear_at(100, 3 * 365, &p);
        let expected = p.steady_wear(2.0);
        assert!((w - expected).abs() < 0.03, "{w} against {expected}");
    }

    #[test]
    fn trails_form_at_one_threshold_and_lapse_at_a_lower_one() {
        let p = params();
        let mut wear = Wear::new(128, 128, 8.0);
        let cell = 5u32;
        let mut walks = 0;
        while !wear.is_trail(cell as usize) {
            wear.walk(&[cell], 0, &p);
            walks += 1;
        }
        // 1 - 0.99^n reaches 0.3 first at n = 36.
        assert_eq!(walks, 36);
        // After one half-life the wear is about 0.152, still above trail_until.
        wear.survey(120, &p);
        assert!(
            wear.is_trail(cell as usize),
            "a trail outlives a lapse in use"
        );
        wear.survey(130, &p);
        assert!(!wear.is_trail(cell as usize), "and fades in the end");
    }

    #[test]
    fn surveys_forget_faded_ground_and_feed_the_routing_view() {
        let p = params();
        let mut wear = Wear::new(200, 200, 8.0);
        let path: Vec<u32> = (10..60).map(|x| 20 * 200 + x).collect();
        for _ in 0..80 {
            wear.walk(&path, 0, &p);
        }
        assert_eq!(
            wear.factor(path[0] as usize),
            0.0,
            "routes wait for a survey"
        );
        wear.survey(1, &p);
        let rev = wear.rev();
        let f = wear.factor(path[3] as usize);
        let expected = (1.0 - 0.99f64.powi(80)) * p.fade(1);
        assert!((f64::from(f) - expected).abs() < 0.01, "{f}");
        assert!(wear.max_factor() >= f);
        assert_eq!(wear.factor(0), 0.0);
        let surveyed: Vec<_> = wear.surveyed_tiles().collect();
        assert_eq!(surveyed.len(), 1);
        let (index, view, trail) = surveyed[0];
        assert_eq!(index, 0);
        assert_eq!(
            f32::from(view[(path[3] % 200) as usize + 20 * 64]) / 255.0,
            f
        );
        assert!(trail.iter().any(|&w| w != 0), "walked eighty times: trail");
        assert_eq!(wear.tiles().len(), 1);
        // Ten years unused: forgotten.
        wear.survey(1 + 3650, &p);
        assert!(wear.tiles().is_empty());
        assert_eq!(wear.factor(path[3] as usize), 0.0);
        assert_ne!(wear.rev(), rev);
    }

    #[test]
    fn what_a_save_keeps_is_faded_to_the_day() {
        let p = params();
        let mut wear = Wear::new(128, 128, 8.0);
        for _ in 0..40 {
            wear.walk(&[1, 2, 3], 0, &p);
        }
        wear.walk(&[70 * 128 + 70], 200, &p);
        let kept = wear.current(240, &p);
        assert_eq!(kept.len(), 2);
        assert!(kept.iter().all(|t| t.day == 240));
        let restored = Wear::new(128, 128, 8.0).with_tiles(kept.clone());
        assert_eq!(restored.current(240, &p), kept);
        assert!(restored.problems().is_empty());
        // Trails that faded below the lower threshold are not kept as trails.
        assert!(!kept[0].is_trail(1));
    }

    #[test]
    fn walks_are_sampled_into_the_cells_they_cross() {
        let cells = cells_along(&[(4.0, 4.0), (60.0, 4.0)], 8.0, 32, 32);
        assert_eq!(cells, (0..8).collect::<Vec<u32>>());
        let diag = cells_along(&[(4.0, 4.0), (36.0, 36.0)], 8.0, 32, 32);
        assert_eq!(diag.first(), Some(&0));
        assert_eq!(diag.last(), Some(&(4 * 32 + 4)));
        assert!(diag.windows(2).all(|w| w[0] != w[1]));
        // Off the map is left out.
        assert!(cells_along(&[(-20.0, 4.0), (-4.0, 4.0)], 8.0, 32, 32).is_empty());
    }

    fn traced(cells: &[Cell]) -> Vec<Vec<Cell>> {
        let set: HashSet<Cell> = cells.iter().copied().collect();
        let mut out = Vec::new();
        for c in components(&set) {
            out.extend(chains(&thin(&c)));
        }
        out
    }

    #[test]
    fn a_band_of_worn_cells_traces_as_one_line() {
        // Three cells wide, thirty long.
        let band: Vec<Cell> = (0..30)
            .flat_map(|x| (10..13).map(move |y| (x, y)))
            .collect();
        let lines = traced(&band);
        assert_eq!(lines.len(), 1, "{lines:?}");
        assert!(lines[0].len() >= 26);
    }

    #[test]
    fn a_crossing_traces_as_four_arms() {
        let mut cells: Vec<Cell> = (0..21).map(|x| (x, 10)).collect();
        cells.extend((0..21).filter(|&y| y != 10).map(|y| (10, y)));
        let lines = traced(&cells);
        assert_eq!(lines.len(), 4, "{lines:?}");
        for l in &lines {
            assert!(l.contains(&(10, 10)), "every arm meets at the junction");
        }
    }

    #[test]
    fn a_ring_traces_as_a_closed_line() {
        let mut ring = Vec::new();
        for i in 0..20 {
            ring.push((i, 0));
            ring.push((i, 19));
            ring.push((0, i));
            ring.push((19, i));
        }
        ring.sort_unstable();
        ring.dedup();
        let lines = traced(&ring);
        let cells: usize = lines.iter().map(|l| l.len() - 1).sum();
        assert!(cells >= 70, "the ring is traced all round: {lines:?}");
        let ends: Vec<Cell> = lines.iter().flat_map(|l| [l[0], l[l.len() - 1]]).collect();
        for e in &ends {
            assert!(
                ends.iter().filter(|&&x| x == *e).count() >= 2,
                "no loose end at {e:?}"
            );
        }
    }

    #[test]
    fn a_diagonal_staircase_traces_as_one_line() {
        let mut cells = Vec::new();
        for i in 0..20 {
            cells.push((i, i));
            cells.push((i + 1, i));
        }
        let lines = traced(&cells);
        assert_eq!(lines.len(), 1, "{lines:?}");
        assert_eq!(lines[0].len(), 40, "every cell of it: {lines:?}");
    }

    #[test]
    fn surveyed_trails_are_polylines_in_metres() {
        let p = params();
        let mut wear = Wear::new(256, 256, 8.0);
        // An L-shaped path walked often enough to become trail.
        let mut path: Vec<u32> = (20..80).map(|x| 30 * 256 + x).collect();
        path.extend((31..90).map(|y| y * 256 + 79));
        for _ in 0..60 {
            wear.walk(&path, 0, &p);
        }
        wear.survey(0, &p);
        assert_eq!(wear.trails().len(), 1, "{:?}", wear.trails());
        let t = &wear.trails()[0];
        assert!(t.points.len() >= 3 && t.points.len() <= 5, "{:?}", t.points);
        let expected = (59.0 + 59.0) * 8.0;
        assert!((t.length_m() - expected).abs() < 24.0, "{}", t.length_m());
        assert!(t.wear > 0.4);
    }

    #[test]
    fn a_restored_view_routes_and_traces_as_the_survey_drew_it() {
        // Surveyed, then walked on: the tiles move on and the view stays as surveyed. Tiles and
        // view restored as they were route and trace as before, with no survey (ADR-0011 §5).
        let p = params();
        let mut wear = Wear::new(256, 256, 8.0);
        let path: Vec<u32> = (20..120).map(|x| 40 * 256 + x).collect();
        for day in 0..40 {
            wear.walk(&path, day, &p);
        }
        wear.survey(40, &p);
        let other: Vec<u32> = (50..100).map(|y| y * 256 + 30).collect();
        for day in 41..50 {
            wear.walk(&other, day, &p);
        }
        let view = wear
            .surveyed_tiles()
            .map(|(index, cells, trail)| ViewTile {
                index,
                cells: cells.to_vec(),
                trail: trail.to_vec(),
            })
            .collect();
        let restored = Wear::new(256, 256, 8.0)
            .with_tiles(wear.tiles().to_vec())
            .with_view(wear.surveyed(), view);
        assert!(restored.has_view());
        assert_eq!(restored.surveyed(), 40);
        assert_eq!(restored.trails(), wear.trails());
        assert_eq!(restored.max_factor(), wear.max_factor());
        for c in path.iter().chain(&other) {
            assert_eq!(restored.factor(*c as usize), wear.factor(*c as usize));
        }
        // The cells walked after the survey are not yet on it.
        assert_eq!(restored.factor(other[10] as usize), 0.0);
        assert_eq!(restored.tiles(), wear.tiles());
    }
}
