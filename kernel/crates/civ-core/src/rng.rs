//! Random numbers.
//!
//! The kernel carries its own two small generators instead of depending on a crate:
//!
//! - [`SplitMix64`] for seeding and for mixing keys into seeds;
//! - [`Rng64`], xoshiro256** (Blackman & Vigna), for streams of draws.
//!
//! Owning them means a seed keeps describing the same generated world after dependency upgrades;
//! random-number crates have changed their value streams across major versions before.
//!
//! Determinism is not a goal for the simulation (plan §1, rule 6). These generators exist for two
//! narrower jobs: world generation that is repeatable from a seed on a given build, and **keyed
//! streams**. A keyed stream, derived from stable identities with [`Rng64::from_key`], keeps the
//! number of worker threads from deciding which person receives which draw (research 01-01 §4.2).

/// The SplitMix64 generator (Steele, Lea & Flood; Vigna's reference constants).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    /// A generator starting from `seed`.
    pub const fn new(seed: u64) -> Self {
        SplitMix64 { state: seed }
    }

    /// The next value.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        mix64(self.state)
    }
}

/// The SplitMix64 output function: a strong, stateless 64-bit mixer.
pub const fn mix64(x: u64) -> u64 {
    let mut z = x;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Folds a sequence of key parts into one 64-bit key. Order matters: `[a, b]` and `[b, a]` give
/// different keys.
pub fn key(parts: &[u64]) -> u64 {
    let mut h = 0x243F_6A88_85A3_08D3u64; // pi, an arbitrary non-zero start
    for &part in parts {
        h = mix64(h ^ mix64(part.wrapping_add(0x9E37_79B9_7F4A_7C15)));
    }
    h
}

/// xoshiro256**: fast, small-state, and good enough for simulation draws. Not cryptographic.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rng64 {
    s: [u64; 4],
}

impl Rng64 {
    /// A stream seeded by expanding `seed` through SplitMix64, as the authors recommend.
    pub fn seed_from_u64(seed: u64) -> Self {
        let mut sm = SplitMix64::new(seed);
        let s = [sm.next_u64(), sm.next_u64(), sm.next_u64(), sm.next_u64()];
        // SplitMix64 never yields four zeros in a row, so the state is valid.
        Rng64 { s }
    }

    /// A stream derived from key parts, such as `[world_seed, PURPOSE_TERRAIN, tile]`.
    pub fn from_key(parts: &[u64]) -> Self {
        Rng64::seed_from_u64(key(parts))
    }

    /// A stream restored from saved state. `None` for the all-zero state, which xoshiro cannot
    /// leave.
    pub fn from_state(s: [u64; 4]) -> Option<Self> {
        if s == [0; 4] { None } else { Some(Rng64 { s }) }
    }

    /// The state, for saving.
    pub fn state(&self) -> [u64; 4] {
        self.s
    }

    /// The next 64 random bits.
    pub fn next_u64(&mut self) -> u64 {
        let result = self.s[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let t = self.s[1] << 17;
        self.s[2] ^= self.s[0];
        self.s[3] ^= self.s[1];
        self.s[1] ^= self.s[2];
        self.s[0] ^= self.s[3];
        self.s[2] ^= t;
        self.s[3] = self.s[3].rotate_left(45);
        result
    }

    /// The next 32 random bits (the high half, which is the stronger half).
    pub fn next_u32(&mut self) -> u32 {
        (self.next_u64() >> 32) as u32
    }

    /// Uniform in `[0, 1)`, with 53 bits of precision.
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }

    /// Uniform integer in `[0, n)`, without modulo bias (Lemire's method). Returns 0 when `n` is 0.
    pub fn below(&mut self, n: u64) -> u64 {
        if n == 0 {
            return 0;
        }
        let mut m = u128::from(self.next_u64()) * u128::from(n);
        let mut low = m as u64;
        if low < n {
            let threshold = n.wrapping_neg() % n;
            while low < threshold {
                m = u128::from(self.next_u64()) * u128::from(n);
                low = m as u64;
            }
        }
        (m >> 64) as u64
    }

    /// Uniform integer in `[lo, hi]`. Returns `lo` when `hi < lo`.
    pub fn range_i64(&mut self, lo: i64, hi: i64) -> i64 {
        if hi <= lo {
            return lo;
        }
        let span = (hi as i128 - lo as i128 + 1) as u128;
        if span > u128::from(u64::MAX) {
            return self.next_u64() as i64;
        }
        (lo as i128 + self.below(span as u64) as i128) as i64
    }

    /// Uniform in `[lo, hi)`.
    pub fn range_f64(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (hi - lo) * self.next_f64()
    }

    /// True with probability `p` (clamped to `[0, 1]`).
    pub fn chance(&mut self, p: f64) -> bool {
        self.next_f64() < p
    }

    /// An independent child stream; advances this stream by one draw.
    pub fn fork(&mut self) -> Rng64 {
        Rng64::seed_from_u64(self.next_u64())
    }

    /// Shuffles a slice uniformly (Fisher–Yates).
    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            let j = self.below(i as u64 + 1) as usize;
            items.swap(i, j);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Reference values computed by an independent Python transcription of the published C
    // reference implementations (Vigna's splitmix64.c and xoshiro256starstar.c).

    #[test]
    fn splitmix64_matches_the_reference() {
        let mut sm = SplitMix64::new(1_234_567);
        let got: Vec<u64> = (0..5).map(|_| sm.next_u64()).collect();
        assert_eq!(
            got,
            vec![
                6_457_827_717_110_365_317,
                3_203_168_211_198_807_973,
                9_817_491_932_198_370_423,
                4_593_380_528_125_082_431,
                16_408_922_859_458_223_821,
            ]
        );
    }

    #[test]
    fn xoshiro256starstar_matches_the_reference() {
        let mut r = Rng64::from_state([1, 2, 3, 4]).expect("non-zero");
        let got: Vec<u64> = (0..6).map(|_| r.next_u64()).collect();
        assert_eq!(
            got,
            vec![
                11_520,
                0,
                1_509_978_240,
                1_215_971_899_390_074_240,
                1_216_172_134_540_287_360,
                607_988_272_756_665_600,
            ]
        );
    }

    #[test]
    fn seeding_through_splitmix_matches_the_reference() {
        let mut r = Rng64::seed_from_u64(42);
        assert_eq!(
            r.state(),
            [
                13_679_457_532_755_275_413,
                2_949_826_092_126_892_291,
                5_139_283_748_462_763_858,
                6_349_198_060_258_255_764,
            ]
        );
        let got: Vec<u64> = (0..3).map(|_| r.next_u64()).collect();
        assert_eq!(
            got,
            vec![
                1_546_998_764_402_558_742,
                6_990_951_692_964_543_102,
                12_544_586_762_248_559_009,
            ]
        );
    }

    #[test]
    fn floats_stay_in_the_unit_interval() {
        let mut r = Rng64::seed_from_u64(9);
        for _ in 0..10_000 {
            let x = r.next_f64();
            assert!((0.0..1.0).contains(&x));
        }
    }

    #[test]
    fn below_respects_its_bound_and_covers_it() {
        let mut r = Rng64::seed_from_u64(5);
        let mut hits = [0u32; 7];
        for _ in 0..7_000 {
            let v = r.below(7);
            assert!(v < 7);
            hits[v as usize] += 1;
        }
        // Each value expects 1000; a 6-sigma band (~±190) keeps false alarms negligible.
        for h in hits {
            assert!((800..=1200).contains(&h), "bucket {h}");
        }
        assert_eq!(r.below(0), 0);
        assert_eq!(r.below(1), 0);
    }

    #[test]
    fn ranges_include_both_ends() {
        let mut r = Rng64::seed_from_u64(1);
        let mut saw = [false; 3];
        for _ in 0..200 {
            let v = r.range_i64(-1, 1);
            saw[(v + 1) as usize] = true;
        }
        assert_eq!(saw, [true; 3]);
        assert_eq!(r.range_i64(5, 5), 5);
        assert_eq!(r.range_i64(5, 4), 5);
    }

    #[test]
    fn keys_are_order_sensitive_and_spread() {
        assert_ne!(key(&[1, 2]), key(&[2, 1]));
        assert_ne!(key(&[0]), key(&[0, 0]));
        assert_ne!(
            Rng64::from_key(&[7, 1]).next_u64(),
            Rng64::from_key(&[7, 2]).next_u64()
        );
    }

    #[test]
    fn shuffle_is_a_permutation() {
        let mut r = Rng64::seed_from_u64(3);
        let mut v: Vec<u32> = (0..50).collect();
        r.shuffle(&mut v);
        let mut sorted = v.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, (0..50).collect::<Vec<_>>());
        assert_ne!(v, (0..50).collect::<Vec<_>>());
    }

    #[test]
    fn zero_state_is_rejected() {
        assert!(Rng64::from_state([0; 4]).is_none());
    }
}
