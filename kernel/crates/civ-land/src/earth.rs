//! Earthworks (ADR-0010 §2-3; M3b slice Q): records of the ground people change, and the pure,
//! versioned expansion of a record into the surface it leaves and the earth it moves.
//!
//! A platform levels a plot by cut and fill: inside its rectangle the ground is brought to one
//! level, and around it the ground slopes back to the generated surface at the record's side slope.
//! A pit is dug for a deposit's goods: its rectangle deepens as it is worked, and what is dug and
//! not carried away (the cover over the body, and what of it is unfit) goes on a spoil heap beside
//! it.
//! The expansion samples the ground at a fine step, so it runs at the kernel's 8 m cells and at any
//! finer resolution alike, and it never rewrites the generated bed: what it gives is a difference.

use std::collections::BTreeMap;

use civ_core::{PermanentId, SimTime};
use civ_world::WorldMap;

use crate::fields::RectCm;

/// The expansion's version: records keep the version they were made under (ADR-0010 §2).
pub const EARTH_VERSION: u16 = 1;

/// The step at which the expansion samples the ground, metres.
pub const SAMPLE_M: f64 = 0.5;

/// What an earthwork is. Numeric in saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EarthKind {
    /// A plot levelled by cut and fill.
    Platform,
    /// A pit dug for a deposit's goods, deepening as it is worked.
    Pit,
    /// A heap of what a pit's digging leaves: the cover over the body, and what is unfit.
    Spoil,
}

/// The surface an earthwork leaves, over the area it touches, and the earth it moves.
#[derive(Clone, Debug, PartialEq)]
pub struct Shaped {
    /// Earth cut from above the finished surface, cubic metres in the bank.
    pub cut_m3: f64,
    /// Earth placed below it, cubic metres in place.
    pub fill_m3: f64,
    /// The area it touches, square metres (its rectangle and the slopes around it).
    pub area_m2: f64,
}

/// The level a platform over `rect`, its sides at `side_run`, would be cut and filled to on ground
/// `base` (metres from the map's corner to metres of height): the level at which all it cuts,
/// sides and all, equals all it fills, so its earth balances (ADR-0010 §2), found by halving
/// between the lowest and highest ground under it. `None` for an empty rectangle.
pub fn platform_level(rect: &RectCm, side_run: f64, base: &dyn Fn(f64, f64) -> f64) -> Option<f64> {
    let points = samples(rect, 0.0);
    if points.is_empty() {
        return None;
    }
    let (mut lo, mut hi) = points
        .iter()
        .map(|&(x, y)| base(x, y))
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(l, h), z| {
            (l.min(z), h.max(z))
        });
    // What it fills less what it cuts rises with the level; find where it crosses zero.
    for _ in 0..40 {
        let mid = (lo + hi) / 2.0;
        let s = shape_platform(rect, mid, side_run, 1.0, base, &mut |_, _, _, _| {});
        if s.fill_m3 < s.cut_m3 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Some((lo + hi) / 2.0)
}

/// The highest of the ground `base` over `rect`, metres.
pub fn highest(rect: &RectCm, base: &dyn Fn(f64, f64) -> f64) -> f64 {
    samples(rect, 0.0)
        .iter()
        .map(|&(x, y)| base(x, y))
        .fold(f64::NEG_INFINITY, f64::max)
}

/// How far the ground `base` drops across `rect`, metres: its highest less its lowest there.
pub fn drop_across(rect: &RectCm, base: &dyn Fn(f64, f64) -> f64) -> f64 {
    let (lo, hi) = samples(rect, 0.0)
        .iter()
        .map(|&(x, y)| base(x, y))
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(l, h), z| {
            (l.min(z), h.max(z))
        });
    if hi >= lo { hi - lo } else { 0.0 }
}

/// Whether a platform over `rect` whose ground drops `drop_m` across it, its sides at `side_run`,
/// stays clear of water: every cell its sides could reach is dry land on the map. Earthworks are
/// refused on or beside water while routing and drainage stay as generated (ADR-0010 §3).
pub fn clear_of_water(map: &WorldMap, rect: &RectCm, drop_m: f64, side_run: f64) -> bool {
    let cell = f64::from(map.cell_size_m);
    if cell <= 0.0 {
        return false;
    }
    let reach = drop_m.max(0.0) * side_run.max(0.0) + SAMPLE_M;
    let (x0, y0) = (
        f64::from(rect.x) / 100.0 - reach,
        f64::from(rect.y) / 100.0 - reach,
    );
    let (x1, y1) = (
        f64::from(rect.x + rect.w) / 100.0 + reach,
        f64::from(rect.y + rect.h) / 100.0 + reach,
    );
    let (w, h) = (f64::from(map.width) * cell, f64::from(map.height) * cell);
    if x0 < 0.0 || y0 < 0.0 || x1 > w || y1 > h {
        return false;
    }
    let (cx0, cy0) = ((x0 / cell) as usize, (y0 / cell) as usize);
    let (cx1, cy1) = (
        ((x1 / cell) as usize).min(map.width as usize - 1),
        ((y1 / cell) as usize).min(map.height as usize - 1),
    );
    let width = map.width as usize;
    (cy0..=cy1).all(|y| (cx0..=cx1).all(|x| map.water[y * width + x] == civ_world::WATER_LAND))
}

/// The finished surface at `(x, y)` of a platform over `rect` levelled to `level_m`, with sides
/// sloping back to `base` at `side_run` metres across per metre up or down.
pub fn platform_surface(
    rect: &RectCm,
    level_m: f64,
    side_run: f64,
    (x, y): (f64, f64),
    base: f64,
) -> f64 {
    let (x0, y0) = (f64::from(rect.x) / 100.0, f64::from(rect.y) / 100.0);
    let (x1, y1) = (
        x0 + f64::from(rect.w) / 100.0,
        y0 + f64::from(rect.h) / 100.0,
    );
    let dx = (x0 - x).max(0.0).max(x - x1);
    let dy = (y0 - y).max(0.0).max(y - y1);
    // A square root, exact on every platform, so a version's expansion is the same everywhere.
    let d = (dx * dx + dy * dy).sqrt();
    let reach = d / side_run.max(1e-6);
    base.clamp(level_m - reach, level_m + reach)
}

/// What a platform over `rect` levelled to `level_m`, its sides at `side_run`, does to ground
/// `base`, and its change at each point of the area it touches through `each` (x, y, metres of
/// height added, square metres the point stands for), `share` of the way done (0 to 1).
pub fn shape_platform(
    rect: &RectCm,
    level_m: f64,
    side_run: f64,
    share: f64,
    base: &dyn Fn(f64, f64) -> f64,
    each: &mut dyn FnMut(f64, f64, f64, f64),
) -> Shaped {
    // How far beyond the rectangle the sides can reach: its height range over the side slope.
    let inside: Vec<f64> = samples(rect, 0.0)
        .iter()
        .map(|&(x, y)| base(x, y))
        .collect();
    let (lo, hi) = inside
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(l, h), &z| {
            (l.min(z), h.max(z))
        });
    let margin = if inside.is_empty() {
        0.0
    } else {
        ((level_m - lo).abs().max((hi - level_m).abs()) * side_run).max(0.0) + SAMPLE_M
    };
    let share = share.clamp(0.0, 1.0);
    let cell = SAMPLE_M * SAMPLE_M;
    let mut out = Shaped {
        cut_m3: 0.0,
        fill_m3: 0.0,
        area_m2: 0.0,
    };
    for (x, y) in samples(rect, margin) {
        let z = base(x, y);
        let change = (platform_surface(rect, level_m, side_run, (x, y), z) - z) * share;
        if change.abs() < 1e-9 {
            continue;
        }
        out.area_m2 += cell;
        if change > 0.0 {
            out.fill_m3 += change * cell;
        } else {
            out.cut_m3 -= change * cell;
        }
        each(x, y, change, cell);
    }
    out
}

/// An earthwork's record (ADR-0010 §2): what it is, where, the level it is cut and filled to and
/// how far it has gone. Saved; what it has done to the ground is kept in [`GroundDelta`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Earthwork {
    /// Permanent id.
    pub id: PermanentId,
    /// What it is.
    pub kind: EarthKind,
    /// The area it levels.
    pub rect: RectCm,
    /// The level it is cut and filled to, centimetres of height.
    pub level_cm: i32,
    /// Its sides' run, centimetres across for each metre up or down.
    pub side_run_cm: u16,
    /// The plot it levels, if any.
    pub plot: Option<PermanentId>,
    /// For a pit or its heap: the deposit the pit is dug for.
    pub deposit: Option<PermanentId>,
    /// For a pit: its spoil heap.
    pub heap: Option<PermanentId>,
    /// The household that made it.
    pub household: PermanentId,
    /// Earth it moves, cubic metres as it lay in the ground: a platform's cut when done, what a pit
    /// has been dug of so far, what a heap has been given.
    pub cut_m3: f32,
    /// The share of it done, 0 to 1: a platform's; a pit and a heap are as far as their earth.
    pub done: f32,
    /// The expansion version it was made under.
    pub version: u16,
    /// When it was begun.
    pub begun: SimTime,
}

impl Earthwork {
    /// Its level, metres.
    pub fn level_m(&self) -> f64 {
        f64::from(self.level_cm) / 100.0
    }

    /// Its sides' run, metres across per metre up or down.
    pub fn side_run(&self) -> f64 {
        f64::from(self.side_run_cm) / 100.0
    }

    /// A pit's depth, or a heap's height, metres: its earth over its area.
    pub fn depth_m(&self) -> f64 {
        let area = f64::from(self.rect.w.max(0)) * f64::from(self.rect.h.max(0)) / 10_000.0;
        if area > 0.0 {
            f64::from(self.cut_m3) / area
        } else {
            0.0
        }
    }
}

/// Cells a side of a tile of [`GroundDelta`].
pub const DELTA_TILE: u32 = 64;

/// What earthworks have done to the ground (ADR-0010 §3): for each cell they touched, the change
/// in its mean height, metres, in sparse tiles of [`DELTA_TILE`] cells a side, each with a
/// revision. The generated bed is never rewritten; the ground in use is the bed plus this.
#[derive(Clone, Debug, PartialEq)]
pub struct GroundDelta {
    width: u32,
    height: u32,
    cell_m: f32,
    tiles: BTreeMap<u32, DeltaTile>,
}

/// One tile of a [`GroundDelta`].
#[derive(Clone, Debug, PartialEq)]
pub struct DeltaTile {
    /// Changes each time a cell of it changes.
    pub rev: u32,
    /// Each cell's change in mean height, metres, row by row.
    pub cells: Vec<f32>,
}

impl GroundDelta {
    /// No change yet, over a map of `width` by `height` cells of `cell_m` metres.
    pub fn new(width: u32, height: u32, cell_m: f32) -> Self {
        Self {
            width,
            height,
            cell_m,
            tiles: BTreeMap::new(),
        }
    }

    /// Tiles across the map.
    pub fn tiles_x(&self) -> u32 {
        self.width.div_ceil(DELTA_TILE)
    }

    /// The change in cell `cell`'s mean height, metres.
    pub fn at(&self, cell: usize) -> f32 {
        let w = self.width as usize;
        let (x, y) = ((cell % w.max(1)) as u32, (cell / w.max(1)) as u32);
        let index = (y / DELTA_TILE) * self.tiles_x() + x / DELTA_TILE;
        self.tiles.get(&index).map_or(0.0, |t| {
            t.cells[((y % DELTA_TILE) * DELTA_TILE + x % DELTA_TILE) as usize]
        })
    }

    /// Adds `volume_m3` of earth (removes, when negative) at `(x, y)` metres, spread over the
    /// cell there as a change in its mean height. Off the map, nothing.
    pub fn add(&mut self, (x, y): (f64, f64), volume_m3: f64) {
        let cell_m = f64::from(self.cell_m);
        if !(x >= 0.0 && y >= 0.0) || cell_m <= 0.0 {
            return;
        }
        let (cx, cy) = ((x / cell_m) as u32, (y / cell_m) as u32);
        if cx >= self.width || cy >= self.height {
            return;
        }
        let index = (cy / DELTA_TILE) * self.tiles_x() + cx / DELTA_TILE;
        let tile = self.tiles.entry(index).or_insert_with(|| DeltaTile {
            rev: 0,
            cells: vec![0.0; (DELTA_TILE * DELTA_TILE) as usize],
        });
        let at = ((cy % DELTA_TILE) * DELTA_TILE + cx % DELTA_TILE) as usize;
        tile.cells[at] += (volume_m3 / (cell_m * cell_m)) as f32;
        tile.rev = tile.rev.wrapping_add(1);
    }

    /// Every tile there is, by index (row by row across the map).
    pub fn tiles(&self) -> impl Iterator<Item = (u32, &DeltaTile)> {
        self.tiles.iter().map(|(&i, t)| (i, t))
    }

    /// Sets tile `index` (as loaded from a save). `false` if it is off the map or the wrong size.
    pub fn set_tile(&mut self, index: u32, tile: DeltaTile) -> bool {
        let count = self.tiles_x() * self.height.div_ceil(DELTA_TILE);
        if index >= count || tile.cells.len() != (DELTA_TILE * DELTA_TILE) as usize {
            return false;
        }
        self.tiles.insert(index, tile);
        true
    }

    /// The changes over the `w` by `h` cells from `(x0, y0)`, metres, row by row; `None` when no
    /// changed tile touches them (the ground there is as generated).
    pub fn region(&self, (x0, y0): (u32, u32), (w, h): (u32, u32)) -> Option<Vec<f32>> {
        let (x1, y1) = (
            x0.saturating_add(w).min(self.width),
            y0.saturating_add(h).min(self.height),
        );
        if x0 >= x1 || y0 >= y1 {
            return None;
        }
        let tiles_x = self.tiles_x();
        let mut out: Option<Vec<f32>> = None;
        for (&index, tile) in &self.tiles {
            let (tx, ty) = (
                (index % tiles_x) * DELTA_TILE,
                (index / tiles_x) * DELTA_TILE,
            );
            let (ax, ay) = (tx.max(x0), ty.max(y0));
            let (bx, by) = ((tx + DELTA_TILE).min(x1), (ty + DELTA_TILE).min(y1));
            if ax >= bx || ay >= by {
                continue;
            }
            let dense = out.get_or_insert_with(|| vec![0.0; (w as usize) * (h as usize)]);
            for y in ay..by {
                for x in ax..bx {
                    let from = ((y - ty) * DELTA_TILE + (x - tx)) as usize;
                    dense[((y - y0) * w + (x - x0)) as usize] = tile.cells[from];
                }
            }
        }
        out
    }

    /// The sum of every cell's change times its area, cubic metres: earth added less earth taken.
    pub fn net_m3(&self) -> f64 {
        let area = f64::from(self.cell_m) * f64::from(self.cell_m);
        self.tiles
            .values()
            .flat_map(|t| t.cells.iter())
            .map(|&dz| f64::from(dz) * area)
            .sum()
    }
}

/// The generated ground's height at `(x, y)` metres: the cells' heights at their centres, joined
/// bilinearly; at the map's edges, the edge cells'.
pub fn bed_height(map: &WorldMap, (x, y): (f64, f64)) -> f64 {
    let cell = f64::from(map.cell_size_m);
    let (w, h) = (map.width as usize, map.height as usize);
    if w == 0 || h == 0 || cell <= 0.0 {
        return 0.0;
    }
    let fx = (x / cell - 0.5).clamp(0.0, (w - 1) as f64);
    let fy = (y / cell - 0.5).clamp(0.0, (h - 1) as f64);
    let (x0, y0) = (fx.floor() as usize, fy.floor() as usize);
    let (x1, y1) = ((x0 + 1).min(w - 1), (y0 + 1).min(h - 1));
    let (tx, ty) = (fx - x0 as f64, fy - y0 as f64);
    let z = |cx: usize, cy: usize| f64::from(map.elevation[cy * w + cx]);
    let top = z(x0, y0) * (1.0 - tx) + z(x1, y0) * tx;
    let bottom = z(x0, y1) * (1.0 - tx) + z(x1, y1) * tx;
    top * (1.0 - ty) + bottom * ty
}

/// Advances earthwork `work` on `map` to `share` done, adding what the step does to `ground`.
/// Nothing for a share it has already reached.
pub fn advance(work: &mut Earthwork, map: &WorldMap, ground: &mut GroundDelta, share: f32) {
    let share = share.clamp(0.0, 1.0);
    if share <= work.done {
        return;
    }
    let step = f64::from(share - work.done);
    let base = |x: f64, y: f64| bed_height(map, (x, y));
    match work.kind {
        EarthKind::Platform => {
            shape_platform(
                &work.rect,
                work.level_m(),
                work.side_run(),
                step,
                &base,
                &mut |x, y, dz, area| ground.add((x, y), dz * area),
            );
        }
        // A pit or a heap is as far as its earth ([`dig`]).
        EarthKind::Pit | EarthKind::Spoil => return,
    }
    work.done = share;
}

/// Spreads `volume_m3` of earth evenly over `rect` (removes it, when negative).
fn spread(rect: &RectCm, volume_m3: f64, ground: &mut GroundDelta) {
    let points = samples(rect, 0.0);
    if points.is_empty() {
        return;
    }
    let each = volume_m3 / points.len() as f64;
    for p in points {
        ground.add(p, each);
    }
}

/// Digs `m3` more from pit `pit` (cubic metres as it lay in the ground), of which `kept_m3` is
/// carried away as goods and the rest goes on heap `heap` (ADR-0010 §2): the pit's ground falls by
/// all of it over its rectangle, and the heap's rises by the rest over its, so the ground loses
/// only what is carried away. The pit's level is its floor's, the heap's its top's, over the
/// generated ground at their middles.
pub fn dig(
    pit: &mut Earthwork,
    heap: &mut Earthwork,
    map: &WorldMap,
    ground: &mut GroundDelta,
    m3: f64,
    kept_m3: f64,
) {
    let m3 = m3.max(0.0);
    let kept = kept_m3.clamp(0.0, m3);
    dig_out(pit, map, ground, m3);
    spread(&heap.rect, m3 - kept, ground);
    heap.cut_m3 += (m3 - kept) as f32;
    heap.level_cm = ((middle_height(map, heap) + heap.depth_m()) * 100.0).round() as i32;
}

/// Digs `m3` more from pit `pit`, all of it taken away (as daub for walls): its ground falls by
/// it over its rectangle, and its level is its floor's over the generated ground at its middle.
pub fn dig_out(pit: &mut Earthwork, map: &WorldMap, ground: &mut GroundDelta, m3: f64) {
    let m3 = m3.max(0.0);
    spread(&pit.rect, -m3, ground);
    pit.cut_m3 += m3 as f32;
    pit.level_cm = ((middle_height(map, pit) - pit.depth_m()) * 100.0).round() as i32;
}

/// The generated ground at the middle of `w`'s rectangle, metres.
fn middle_height(map: &WorldMap, w: &Earthwork) -> f64 {
    let (x, y) = w.rect.centre_m();
    bed_height(map, (f64::from(x), f64::from(y)))
}

/// Whether a pit or spoil heap among `works` lies within `gap_cm` of `rect`: ground nobody builds
/// or farms on. A platform is its plot's ground and is not counted.
pub fn dug_near(works: &[Earthwork], rect: &RectCm, gap_cm: i32) -> bool {
    works
        .iter()
        .any(|w| w.kind != EarthKind::Platform && w.rect.near(rect, gap_cm))
}

/// What record `work` has done to the ground, applied afresh to `ground` (the records reproduce
/// the tiles, ADR-0010 §3): a platform as far as it is done, a pit lowered and a heap raised by
/// all their earth.
pub fn replay(work: &Earthwork, map: &WorldMap, ground: &mut GroundDelta) {
    match work.kind {
        EarthKind::Platform => {
            let mut fresh = Earthwork { done: 0.0, ..*work };
            advance(&mut fresh, map, ground, work.done);
        }
        EarthKind::Pit => spread(&work.rect, -f64::from(work.cut_m3), ground),
        EarthKind::Spoil => spread(&work.rect, f64::from(work.cut_m3), ground),
    }
}

/// The centres of the squares of [`SAMPLE_M`] covering `rect` widened by `margin` metres.
fn samples(rect: &RectCm, margin: f64) -> Vec<(f64, f64)> {
    let (x0, y0) = (
        f64::from(rect.x) / 100.0 - margin,
        f64::from(rect.y) / 100.0 - margin,
    );
    let (w, h) = (
        f64::from(rect.w.max(0)) / 100.0 + 2.0 * margin,
        f64::from(rect.h.max(0)) / 100.0 + 2.0 * margin,
    );
    let (nx, ny) = (
        (w / SAMPLE_M).ceil() as usize,
        (h / SAMPLE_M).ceil() as usize,
    );
    let mut out = Vec::with_capacity(nx * ny);
    for j in 0..ny {
        for i in 0..nx {
            out.push((
                x0 + (i as f64 + 0.5) * SAMPLE_M,
                y0 + (j as f64 + 0.5) * SAMPLE_M,
            ));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plot() -> RectCm {
        RectCm {
            x: 1_000,
            y: 2_000,
            w: 800,
            h: 600,
        }
    }

    #[test]
    fn flat_ground_needs_no_earth_moved() {
        let flat = |_: f64, _: f64| 12.0;
        let level = platform_level(&plot(), 1.5, &flat).expect("a level");
        assert!((level - 12.0).abs() < 1e-9);
        let s = shape_platform(&plot(), level, 1.5, 1.0, &flat, &mut |_, _, _, _| {});
        assert_eq!(s.cut_m3, 0.0);
        assert_eq!(s.fill_m3, 0.0);
    }

    #[test]
    fn a_platform_on_a_slope_is_cut_where_high_and_filled_where_low_in_balance() {
        // Ground rising 10 cm a metre eastward: 80 cm across the 8 m plot.
        let slope = |x: f64, _: f64| 50.0 + 0.1 * x;
        let level = platform_level(&plot(), 1.5, &slope).expect("a level");
        // Balanced, sides and all, at the plot's middle, 14 m east of the corner.
        assert!((level - (50.0 + 0.1 * 14.0)).abs() < 1e-3, "{level}");
        let mut changes = 0.0;
        let s = shape_platform(&plot(), level, 1.5, 1.0, &slope, &mut |_, _, dz, a| {
            changes += dz * a;
        });
        // Within the plot: a wedge 0.4 m deep over half its 48 m², twice, cut and fill alike.
        let wedge = 0.5 * 0.4 * 4.0 * 6.0;
        assert!(s.cut_m3 > wedge * 0.95 && s.fill_m3 > wedge * 0.95, "{s:?}");
        assert!((s.cut_m3 - s.fill_m3).abs() < 0.02 * s.cut_m3, "{s:?}");
        assert!((changes - (s.fill_m3 - s.cut_m3)).abs() < 1e-9);
        // Half done moves half the earth.
        let half = shape_platform(&plot(), level, 1.5, 0.5, &slope, &mut |_, _, _, _| {});
        assert!((half.cut_m3 - s.cut_m3 / 2.0).abs() < 1e-9);
    }

    #[test]
    fn a_middling_huts_plot_dropping_a_metre_cuts_about_six_and_a_half_cubic_metres() {
        // A hut of 2.5 m radius under a 0.5 m overhang claims a 6 m square; the ground drops 1 m
        // across it. Within the plot the cut is A·drop/8 (11-12 §2.5B); its sides add about 2 m³.
        let rect = RectCm {
            x: 10_000,
            y: 10_000,
            w: 600,
            h: 600,
        };
        let slope = |x: f64, _: f64| 30.0 - (x - 100.0) / 6.0;
        let level = platform_level(&rect, 1.5, &slope).expect("a level");
        let s = shape_platform(&rect, level, 1.5, 1.0, &slope, &mut |_, _, _, _| {});
        let inside = 36.0 * 1.0 / 8.0;
        assert!(s.cut_m3 > inside + 1.5 && s.cut_m3 < 7.0, "{s:?}");
        assert!((s.cut_m3 - s.fill_m3).abs() < 0.02 * s.cut_m3, "{s:?}");
    }

    #[test]
    fn a_platform_whose_sides_could_reach_water_is_refused() {
        // The test map's river runs along its west 16 m.
        let map = crate::tests::map();
        let rect = RectCm {
            x: 2_000,
            y: 2_000,
            w: 600,
            h: 600,
        };
        // Its sides reach 2 m beyond it for a metre's drop: still dry.
        assert!(clear_of_water(&map, &rect, 1.0, 1.5));
        // For 3 m, 5 m beyond it, over the river.
        assert!(!clear_of_water(&map, &rect, 3.0, 1.5));
        // Nor past the map's edge.
        let corner = RectCm { x: 100, ..rect };
        assert!(!clear_of_water(&map, &corner, 0.5, 1.5));
    }

    #[test]
    fn a_region_of_the_ground_reads_the_changes_under_it_and_nothing_elsewhere() {
        let mut ground = GroundDelta::new(200, 100, 8.0);
        // Half a cubic metre onto cell (70, 3), in the second tile, and a metre off cell (5, 5).
        ground.add((70.0 * 8.0 + 1.0, 3.0 * 8.0 + 1.0), 0.5);
        ground.add((5.0 * 8.0 + 1.0, 5.0 * 8.0 + 1.0), -1.0);
        let r = ground.region((68, 2), (4, 3)).expect("a change in it");
        assert_eq!(r.len(), 12);
        assert!((r[4 + 2] - 0.5 / 64.0).abs() < 1e-7, "{r:?}");
        assert_eq!(r.iter().filter(|&&dz| dz != 0.0).count(), 1);
        let whole = ground.region((0, 0), (200, 100)).expect("changes");
        assert!((whole[5 * 200 + 5] + 1.0 / 64.0).abs() < 1e-7);
        assert_eq!(whole[3 * 200 + 70], ground.at(3 * 200 + 70));
        // Nowhere near a change, and off the map.
        assert_eq!(ground.region((130, 70), (10, 10)), None);
        assert_eq!(ground.region((300, 0), (4, 4)), None);
    }

    #[test]
    fn a_pit_or_heap_keeps_plots_and_fields_away_but_a_platform_is_its_plot_s_ground() {
        let rect = |x, y| RectCm {
            x,
            y,
            w: 300,
            h: 300,
        };
        let work = |kind, at: RectCm| Earthwork {
            id: PermanentId::from_raw(1).expect("nonzero"),
            kind,
            rect: at,
            level_cm: 0,
            side_run_cm: 100,
            plot: None,
            deposit: None,
            heap: None,
            household: PermanentId::from_raw(2).expect("nonzero"),
            cut_m3: 1.0,
            done: 1.0,
            version: EARTH_VERSION,
            begun: SimTime::ZERO,
        };
        let pit = [work(EarthKind::Pit, rect(1_000, 1_000))];
        let heap = [work(EarthKind::Spoil, rect(1_000, 1_000))];
        let platform = [work(EarthKind::Platform, rect(1_000, 1_000))];
        // A plot over it, or within the gap of it, is refused; one beyond the gap is not.
        for works in [&pit, &heap] {
            assert!(dug_near(works, &rect(1_100, 1_100), 0));
            assert!(dug_near(works, &rect(1_350, 1_000), 100));
            assert!(!dug_near(works, &rect(1_450, 1_000), 100));
        }
        assert!(!dug_near(&platform, &rect(1_100, 1_100), 100));
    }

    #[test]
    fn digging_a_pit_heaps_what_is_left_and_only_the_goods_leave_the_ground() {
        let map = crate::tests::map();
        let square = |x: i32| RectCm {
            x,
            y: 2_000,
            w: 300,
            h: 300,
        };
        let work = |id: u64, kind, rect| Earthwork {
            id: PermanentId::from_raw(id).expect("nonzero"),
            kind,
            rect,
            level_cm: 1_000,
            side_run_cm: 100,
            plot: None,
            deposit: PermanentId::from_raw(9),
            heap: None,
            household: PermanentId::from_raw(6).expect("nonzero"),
            cut_m3: 0.0,
            done: 1.0,
            version: EARTH_VERSION,
            begun: SimTime::ZERO,
        };
        let mut pit = work(7, EarthKind::Pit, square(2_000));
        let mut heap = work(8, EarthKind::Spoil, square(2_350));
        let mut ground = GroundDelta::new(map.width, map.height, map.cell_size_m);
        // Two cubic metres dug, one of them carried away; then more, all of it spoil.
        dig(&mut pit, &mut heap, &map, &mut ground, 2.0, 1.0);
        dig(&mut pit, &mut heap, &map, &mut ground, 0.7, 0.0);
        assert!((f64::from(pit.cut_m3) - 2.7).abs() < 1e-6);
        assert!((f64::from(heap.cut_m3) - 1.7).abs() < 1e-6);
        assert!((pit.depth_m() - 0.3).abs() < 1e-6, "{}", pit.depth_m());
        assert!((ground.net_m3() + 1.0).abs() < 1e-3, "{}", ground.net_m3());
        // The pit's floor is below the ground there, the heap's top above it.
        assert!(pit.level_m() < bed_height(&map, (21.5, 21.5)));
        assert!(heap.level_m() > bed_height(&map, (25.0, 21.5)));
        // The records, applied afresh, give the same ground.
        let mut replayed = GroundDelta::new(map.width, map.height, map.cell_size_m);
        replay(&pit, &map, &mut replayed);
        replay(&heap, &map, &mut replayed);
        for cell in 0..(map.width * map.height) as usize {
            assert!(
                (replayed.at(cell) - ground.at(cell)).abs() < 1e-5,
                "cell {cell}"
            );
        }
        // Advancing a pit by share does nothing: a pit is as far as its earth.
        let before = ground.clone();
        advance(&mut pit, &map, &mut ground, 1.0);
        assert_eq!(ground, before);
    }

    /// A hash of everything a platform's expansion gives on a fixed slope, to the micrometre.
    fn golden() -> u64 {
        // Plain arithmetic only, which is exact on every platform.
        let ground = |x: f64, y: f64| 40.0 + 0.12 * x - 0.05 * y + 0.002 * x * y;
        let rect = plot();
        let level = platform_level(&rect, 1.5, &ground).expect("a level");
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut mix = |v: f64| {
            for b in ((v * 1e6).round() as i64).to_le_bytes() {
                h = (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3);
            }
        };
        mix(level);
        let mut changes = Vec::new();
        let s = shape_platform(&rect, level, 1.5, 0.75, &ground, &mut |x, y, dz, a| {
            changes.push((x, y, dz, a));
        });
        for (x, y, dz, a) in changes {
            mix(x);
            mix(y);
            mix(dz);
            mix(a);
        }
        mix(s.cut_m3);
        mix(s.fill_m3);
        mix(s.area_m2);
        h
    }

    #[test]
    fn version_1_is_pinned() {
        // Changing what version 1 gives changes saved ground: bump EARTH_VERSION instead.
        assert_eq!(EARTH_VERSION, 1);
        assert_eq!(golden(), GOLDEN_V1, "{:#x}", golden());
    }

    const GOLDEN_V1: u64 = 0x1e97_baba_e75b_ad2e;

    #[test]
    fn levelling_a_plot_moves_earth_within_the_ground_and_never_twice() {
        let map = crate::tests::map();
        // The plot over the steep south-east of the test map.
        let rect = RectCm {
            x: 3_400,
            y: 3_400,
            w: 1_200,
            h: 1_200,
        };
        let base = |x: f64, y: f64| bed_height(&map, (x, y));
        let level = platform_level(&rect, 1.5, &base).expect("a level");
        let whole = shape_platform(&rect, level, 1.5, 1.0, &base, &mut |_, _, _, _| {});
        assert!(whole.cut_m3 > 1.0, "{whole:?}");
        let mut work = Earthwork {
            id: PermanentId::from_raw(5).expect("nonzero"),
            kind: EarthKind::Platform,
            rect,
            level_cm: (level * 100.0).round() as i32,
            side_run_cm: 150,
            plot: None,
            deposit: None,
            heap: None,
            household: PermanentId::from_raw(6).expect("nonzero"),
            cut_m3: whole.cut_m3 as f32,
            done: 0.0,
            version: EARTH_VERSION,
            begun: SimTime::ZERO,
        };
        let mut ground = GroundDelta::new(map.width, map.height, map.cell_size_m);
        advance(&mut work, &map, &mut ground, 0.5);
        advance(&mut work, &map, &mut ground, 0.5);
        assert_eq!(work.done, 0.5);
        advance(&mut work, &map, &mut ground, 1.0);
        // What is cut is filled, give or take the rounding of its level to a centimetre and the
        // sides' earth beyond the map's edge.
        assert!(
            ground.net_m3().abs() < 0.05 * whole.cut_m3 + 1.0,
            "{} of {whole:?}",
            ground.net_m3()
        );
        // The plot's cells are brought toward its level: the high ones down, the low ones up.
        let cells: Vec<f32> = (0..(map.width * map.height) as usize)
            .map(|c| ground.at(c))
            .collect();
        assert!(cells.iter().any(|&dz| dz < -0.01) && cells.iter().any(|&dz| dz > 0.01));
        let tiles: Vec<u32> = ground.tiles().map(|(i, _)| i).collect();
        assert_eq!(tiles, vec![0]);
    }

    #[test]
    fn the_sides_slope_back_to_the_ground() {
        let rect = plot();
        // 2 m east of the plot's edge on ground 1 m above the level, sides at 1.5:1: the cut
        // there reaches 2 / 1.5 m below the level's top at most, so the ground stays.
        let at = (f64::from(rect.x + rect.w) / 100.0 + 2.0, 23.0);
        assert_eq!(platform_surface(&rect, 10.0, 1.5, at, 11.0), 11.0);
        // Ground 2 m above it is cut down to the side slope.
        let s = platform_surface(&rect, 10.0, 1.5, at, 12.0);
        assert!((s - (10.0 + 2.0 / 1.5)).abs() < 1e-9, "{s}");
        // Inside, the level.
        assert_eq!(platform_surface(&rect, 10.0, 1.5, (12.0, 23.0), 11.0), 10.0);
    }
}
