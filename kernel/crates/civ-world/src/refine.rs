//! Refinement from the coarse context to simulation resolution (research 03-01 §1.6, §5.2).
//!
//! Each level doubles the resolution with Catmull–Rom interpolation, then adds detail scaled to
//! local relief (rough mountains, smooth valley floors) and runs a few erosion passes, so fine
//! tributaries form in agreement with the coarse drainage instead of being painted on.

use rayon::prelude::*;

use civ_core::rng::key;

use crate::grid::{D4, is_edge, neighbor};
use crate::lem::{Erosion, Fields, Landscape, purpose};
use crate::noise::Noise;
use crate::params::TerrainParams;

/// Copies the rectangle `(x0, y0, w, h)` out of a context landscape. The crop's outer ring
/// becomes its base level, and uplift stops: refinement only erodes.
pub(crate) fn crop(ls: &Landscape, rect: (usize, usize, usize, usize)) -> Landscape {
    let (x0, y0, w, h) = rect;
    let mut z = Vec::with_capacity(w * h);
    let mut erodibility = Vec::with_capacity(w * h);
    for y in y0..y0 + h {
        let row = y * ls.w;
        z.extend_from_slice(&ls.z[row + x0..row + x0 + w]);
        erodibility.extend_from_slice(&ls.erodibility[row + x0..row + x0 + w]);
    }
    Landscape {
        w,
        h,
        cell: ls.cell,
        origin: (
            ls.origin.0 + x0 as f64 * ls.cell,
            ls.origin.1 + y0 as f64 * ls.cell,
        ),
        z,
        uplift: vec![0.0; w * h],
        erodibility,
        base: (0..w * h).map(|i| is_edge(w, h, i)).collect(),
        inflow: vec![0.0; w * h],
        routing_seed: ls.routing_seed,
        steps: ls.steps,
        jitter: ls.jitter,
    }
}

#[inline]
fn catmull_rom(p0: f64, p1: f64, p2: f64, p3: f64, t: f64) -> f64 {
    let t2 = t * t;
    let t3 = t2 * t;
    0.5 * (2.0 * p1
        + (p2 - p0) * t
        + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t2
        + (3.0 * p1 - p0 - 3.0 * p2 + p3) * t3)
}

/// Doubles a raster's resolution with bicubic (Catmull–Rom) interpolation between cell centres.
pub(crate) fn upsample2(src: &[f64], w: usize, h: usize) -> Vec<f64> {
    let (fw, fh) = (w * 2, h * 2);
    let at = |x: i64, y: i64| {
        let x = x.clamp(0, w as i64 - 1) as usize;
        let y = y.clamp(0, h as i64 - 1) as usize;
        src[y * w + x]
    };
    let mut out = vec![0.0; fw * fh];
    out.par_chunks_mut(fw).enumerate().for_each(|(fy, row)| {
        let sy = (fy as f64 + 0.5) / 2.0 - 0.5;
        let y0 = sy.floor();
        let ty = sy - y0;
        let y0 = y0 as i64;
        for (fx, v) in row.iter_mut().enumerate() {
            let sx = (fx as f64 + 0.5) / 2.0 - 0.5;
            let x0 = sx.floor();
            let tx = sx - x0;
            let x0 = x0 as i64;
            let mut cols = [0.0; 4];
            for (k, col) in cols.iter_mut().enumerate() {
                let yy = y0 - 1 + k as i64;
                *col = catmull_rom(
                    at(x0 - 1, yy),
                    at(x0, yy),
                    at(x0 + 1, yy),
                    at(x0 + 2, yy),
                    tx,
                );
            }
            *v = catmull_rom(cols[0], cols[1], cols[2], cols[3], ty);
        }
    });
    out
}

/// Places boundary inflows (points in grid-relative metres with an area) on the lowest cell of the
/// grid's second ring near each point, so the water enters the map rather than an outlet.
pub(crate) fn place_inflows(ls: &Landscape, points: &[(f64, f64, f64)], radius_m: f64) -> Vec<f64> {
    let mut inflow = vec![0.0; ls.w * ls.h];
    if ls.w < 4 || ls.h < 4 {
        return inflow;
    }
    for &(px, py, area) in points {
        let r = (radius_m / ls.cell).ceil() as i64;
        let cx = (px / ls.cell - 0.5).round() as i64;
        let cy = (py / ls.cell - 0.5).round() as i64;
        let mut best: Option<usize> = None;
        for y in (cy - r).max(1)..=(cy + r).min(ls.h as i64 - 2) {
            for x in (cx - r).max(1)..=(cx + r).min(ls.w as i64 - 2) {
                let on_second_ring =
                    x == 1 || y == 1 || x == ls.w as i64 - 2 || y == ls.h as i64 - 2;
                if !on_second_ring {
                    continue;
                }
                let i = y as usize * ls.w + x as usize;
                if best.is_none_or(|b| ls.z[i] < ls.z[b]) {
                    best = Some(i);
                }
            }
        }
        if let Some(b) = best {
            inflow[b] += area;
        }
    }
    inflow
}

/// Doubles the landscape's resolution, adds relief-scaled detail and erodes it.
pub(crate) fn refine_level(
    ls: &mut Landscape,
    level: usize,
    seed: u64,
    params: &TerrainParams,
    fields: &Fields,
    inflow_points: &[(f64, f64, f64)],
) {
    let z = upsample2(&ls.z, ls.w, ls.h);
    let (w, h) = (ls.w * 2, ls.h * 2);
    let cell = ls.cell / 2.0;
    let origin = ls.origin;
    let centres = |i: usize| {
        (
            origin.0 + ((i % w) as f64 + 0.5) * cell,
            origin.1 + ((i / w) as f64 + 0.5) * cell,
        )
    };
    let erodibility: Vec<f64> = (0..w * h)
        .into_par_iter()
        .map(|i| {
            let (x, y) = centres(i);
            fields.erodibility(params, x, y)
        })
        .collect();
    *ls = Landscape {
        w,
        h,
        cell,
        origin,
        z,
        uplift: vec![0.0; w * h],
        erodibility,
        base: (0..w * h).map(|i| is_edge(w, h, i)).collect(),
        inflow: vec![0.0; w * h],
        routing_seed: ls.routing_seed,
        steps: ls.steps,
        jitter: ls.jitter,
    };
    ls.inflow = place_inflows(ls, inflow_points, cell * 4.0);

    // Detail: proportional to local relief, damped where drainage area is large (valley floors).
    let routing = ls.route();
    let detail = Noise::new(key(&[seed, purpose::DETAIL + level as u64]));
    let reference_area = cell * cell * 64.0;
    let z_old = ls.z.clone();
    let base = &ls.base;
    ls.z.par_iter_mut().enumerate().for_each(|(i, zi)| {
        if base[i] {
            return;
        }
        let mut relief: f64 = 0.0;
        for off in D4 {
            if let Some(nb) = neighbor(w, h, i, off) {
                relief = relief.max((z_old[i] - z_old[nb]).abs());
            }
        }
        let valley = 1.0 / (1.0 + (routing.area[i] / reference_area).sqrt());
        let (x, y) = centres(i);
        let n = detail.fbm(x / (cell * 5.0), y / (cell * 5.0), 2);
        *zi += params.detail_amplitude * relief * valley * n;
    });

    let e = Erosion::new(params, params.fine_dt_years);
    let mut scratch = Vec::new();
    for _ in 0..params.fine_iterations[level.min(2)] {
        ls.step(&e, &mut scratch);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upsampling_preserves_planes() {
        // Catmull–Rom reproduces linear functions away from clamped borders.
        let (w, h) = (8, 6);
        let src: Vec<f64> = (0..w * h)
            .map(|i| 2.0 * (i % w) as f64 + 3.0 * (i / w) as f64)
            .collect();
        let up = upsample2(&src, w, h);
        let fw = w * 2;
        for fy in 4..h * 2 - 4 {
            for fx in 4..fw - 4 {
                let sx = (fx as f64 + 0.5) / 2.0 - 0.5;
                let sy = (fy as f64 + 0.5) / 2.0 - 0.5;
                let expected = 2.0 * sx + 3.0 * sy;
                assert!((up[fy * fw + fx] - expected).abs() < 1e-9);
            }
        }
    }

    #[test]
    fn inflows_land_on_the_second_ring() {
        let (w, h) = (10, 10);
        let ls = Landscape {
            w,
            h,
            cell: 10.0,
            origin: (0.0, 0.0),
            z: (0..w * h).map(|i| (i % w) as f64).collect(),
            uplift: vec![0.0; w * h],
            erodibility: vec![0.0; w * h],
            base: (0..w * h).map(|i| is_edge(w, h, i)).collect(),
            inflow: vec![0.0; w * h],
            routing_seed: 0,
            steps: 0,
            jitter: 0.0,
        };
        let inflow = place_inflows(&ls, &[(45.0, 5.0, 1000.0)], 30.0);
        let placed: Vec<usize> = (0..w * h).filter(|&i| inflow[i] > 0.0).collect();
        assert_eq!(placed.len(), 1);
        let i = placed[0];
        assert_eq!(i / w, 1, "second row");
        assert!(!ls.base[i]);
    }
}
