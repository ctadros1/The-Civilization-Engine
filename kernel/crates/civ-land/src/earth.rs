//! Earthworks (ADR-0010 §2-3; M3b slice Q): records of the ground people change, and the pure,
//! versioned expansion of a record into the surface it leaves and the earth it moves.
//!
//! A platform levels a plot by cut and fill: inside its rectangle the ground is brought to one
//! level, and around it the ground slopes back to the generated surface at the record's side slope.
//! The expansion samples the ground at a fine step, so it runs at the kernel's 8 m cells and at any
//! finer resolution alike, and it never rewrites the generated bed: what it gives is a difference.

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

/// The level a platform over `rect` would be cut and filled to on ground `base` (metres from the
/// map's corner to metres of height): the level at which what is cut inside its rectangle equals
/// what is filled there, found by halving. `None` for an empty rectangle.
pub fn platform_level(rect: &RectCm, base: &dyn Fn(f64, f64) -> f64) -> Option<f64> {
    let points = samples(rect, 0.0);
    if points.is_empty() {
        return None;
    }
    let heights: Vec<f64> = points.iter().map(|&(x, y)| base(x, y)).collect();
    let (mut lo, mut hi) = heights
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(l, h), &z| {
            (l.min(z), h.max(z))
        });
    // Cut minus fill falls as the level rises; find where it crosses zero.
    for _ in 0..60 {
        let mid = (lo + hi) / 2.0;
        let net: f64 = heights.iter().map(|&z| z - mid).sum();
        if net > 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Some((lo + hi) / 2.0)
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
        let level = platform_level(&plot(), &flat).expect("a level");
        assert!((level - 12.0).abs() < 1e-9);
        let s = shape_platform(&plot(), level, 1.5, 1.0, &flat, &mut |_, _, _, _| {});
        assert_eq!(s.cut_m3, 0.0);
        assert_eq!(s.fill_m3, 0.0);
    }

    #[test]
    fn a_platform_on_a_slope_is_cut_where_high_and_filled_where_low_in_balance() {
        // Ground rising 10 cm a metre eastward: 80 cm across the 8 m plot.
        let slope = |x: f64, _: f64| 50.0 + 0.1 * x;
        let level = platform_level(&plot(), &slope).expect("a level");
        // Balanced at the plot's middle, 14 m east of the corner.
        assert!((level - (50.0 + 0.1 * 14.0)).abs() < 1e-6, "{level}");
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

    /// A hash of everything a platform's expansion gives on a fixed slope, to the micrometre.
    fn golden() -> u64 {
        // Plain arithmetic only, which is exact on every platform.
        let ground = |x: f64, y: f64| 40.0 + 0.12 * x - 0.05 * y + 0.002 * x * y;
        let rect = plot();
        let level = platform_level(&rect, &ground).expect("a level");
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

    const GOLDEN_V1: u64 = 0xab80_fd09_54cf_79f4;

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
