//! Gradient noise built from integer hashing and polynomial interpolation only.
//!
//! No trigonometry or exponentials are involved, so a value depends only on the seed, the
//! coordinates and IEEE-754 addition and multiplication. That keeps generation from a seed
//! repeatable on any build of this code.

use std::f64::consts::{FRAC_1_SQRT_2, SQRT_2};

use civ_core::rng::mix64;

/// Sixteen unit gradients at 22.5° steps (precomputed so no trigonometry runs at sample time).
const GRADIENTS: [(f64, f64); 16] = [
    (1.0, 0.0),
    (0.923_879_532_511_286_7, 0.382_683_432_365_089_8),
    (FRAC_1_SQRT_2, FRAC_1_SQRT_2),
    (0.382_683_432_365_089_8, 0.923_879_532_511_286_7),
    (0.0, 1.0),
    (-0.382_683_432_365_089_8, 0.923_879_532_511_286_7),
    (-FRAC_1_SQRT_2, FRAC_1_SQRT_2),
    (-0.923_879_532_511_286_7, 0.382_683_432_365_089_8),
    (-1.0, 0.0),
    (-0.923_879_532_511_286_7, -0.382_683_432_365_089_8),
    (-FRAC_1_SQRT_2, -FRAC_1_SQRT_2),
    (-0.382_683_432_365_089_8, -0.923_879_532_511_286_7),
    (0.0, -1.0),
    (0.382_683_432_365_089_8, -0.923_879_532_511_286_7),
    (FRAC_1_SQRT_2, -FRAC_1_SQRT_2),
    (0.923_879_532_511_286_7, -0.382_683_432_365_089_8),
];

/// One seeded field of Perlin-style gradient noise.
#[derive(Clone, Copy, Debug)]
pub struct Noise {
    seed: u64,
}

#[inline]
fn fade(t: f64) -> f64 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

impl Noise {
    /// A noise field for `seed`.
    pub fn new(seed: u64) -> Self {
        Noise { seed: mix64(seed) }
    }

    #[inline]
    fn gradient(&self, ix: i64, iy: i64) -> (f64, f64) {
        let h = mix64(
            self.seed
                ^ mix64(
                    (ix as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
                        ^ (iy as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F),
                ),
        );
        GRADIENTS[(h >> 60) as usize]
    }

    /// Gradient noise with unit lattice spacing, roughly in `[-1, 1]`.
    pub fn sample(&self, x: f64, y: f64) -> f64 {
        let x0 = x.floor();
        let y0 = y.floor();
        let (ix, iy) = (x0 as i64, y0 as i64);
        let (fx, fy) = (x - x0, y - y0);
        let dot = |gx: i64, gy: i64, dx: f64, dy: f64| {
            let (a, b) = self.gradient(gx, gy);
            a * dx + b * dy
        };
        let n00 = dot(ix, iy, fx, fy);
        let n10 = dot(ix + 1, iy, fx - 1.0, fy);
        let n01 = dot(ix, iy + 1, fx, fy - 1.0);
        let n11 = dot(ix + 1, iy + 1, fx - 1.0, fy - 1.0);
        let u = fade(fx);
        let v = fade(fy);
        let a = n00 + u * (n10 - n00);
        let b = n01 + u * (n11 - n01);
        // The 2D maximum of unit-gradient Perlin noise is sqrt(2)/2.
        (a + v * (b - a)) * SQRT_2
    }

    /// Fractal sum of `octaves` layers, normalised to roughly `[-1, 1]`.
    pub fn fbm(&self, x: f64, y: f64, octaves: u32) -> f64 {
        let mut sum = 0.0;
        let mut norm = 0.0;
        let mut amp = 1.0;
        let mut freq = 1.0;
        for o in 0..octaves {
            // Offsetting each octave avoids every layer sharing the lattice origin.
            let shift = f64::from(o) * 17.31;
            sum += amp * self.sample(x * freq + shift, y * freq - shift);
            norm += amp;
            amp *= 0.5;
            freq *= 2.03;
        }
        if norm > 0.0 { sum / norm } else { 0.0 }
    }

    /// Ridged multifractal in `[0, 1]`: sharp crests where the noise crosses zero, each octave
    /// weighted by the one before so detail gathers on the ridges.
    pub fn ridged(&self, x: f64, y: f64, octaves: u32) -> f64 {
        let mut sum = 0.0;
        let mut norm = 0.0;
        let mut amp = 1.0;
        let mut freq = 1.0;
        let mut weight = 1.0;
        for o in 0..octaves {
            let shift = f64::from(o) * 23.71;
            let n = 1.0 - self.sample(x * freq + shift, y * freq + shift).abs();
            let n = n * n * weight;
            weight = (n * 1.5).clamp(0.0, 1.0);
            sum += amp * n;
            norm += amp;
            amp *= 0.5;
            freq *= 2.07;
        }
        if norm > 0.0 {
            (sum / norm).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noise_is_repeatable_and_seed_dependent() {
        let a = Noise::new(1);
        let b = Noise::new(1);
        let c = Noise::new(2);
        let p = (12.37, -4.81);
        assert_eq!(a.sample(p.0, p.1).to_bits(), b.sample(p.0, p.1).to_bits());
        assert_ne!(a.sample(p.0, p.1), c.sample(p.0, p.1));
    }

    #[test]
    fn noise_is_zero_on_the_lattice_and_bounded() {
        let n = Noise::new(9);
        assert_eq!(n.sample(3.0, -7.0), 0.0);
        let mut lo = f64::MAX;
        let mut hi = f64::MIN;
        for i in 0..20_000 {
            let x = f64::from(i) * 0.137;
            let v = n.sample(x, x * 0.71 + 0.3);
            lo = lo.min(v);
            hi = hi.max(v);
        }
        assert!(lo >= -1.0001 && hi <= 1.0001, "range {lo}..{hi}");
        assert!(lo < -0.4 && hi > 0.4, "range too narrow {lo}..{hi}");
    }

    #[test]
    fn ridged_stays_in_unit_interval() {
        let n = Noise::new(4);
        for i in 0..5_000 {
            let x = f64::from(i) * 0.091;
            let v = n.ridged(x, -x * 0.5, 5);
            assert!((0.0..=1.0).contains(&v));
        }
    }
}
