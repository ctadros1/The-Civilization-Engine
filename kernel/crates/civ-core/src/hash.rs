//! A fast hasher for the engine's own integer keys (ids, cells, patches), for maps that are only
//! looked up, never iterated where order could reach a result or a save. The default hasher
//! (SipHash with random keys) resists collision attacks the engine does not face, at several
//! times the cost (research 01-02 §2.3: compact hot state and cheap common paths). This is the
//! multiply-rotate scheme of rustc's own `FxHasher`: deterministic and not attack-resistant.

use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasherDefault, Hasher};

const SEED: u64 = 0x51_7c_c1_b7_27_22_0a_95;

/// The hasher. Use it through [`FastMap`] and [`FastSet`].
#[derive(Clone, Copy, Debug, Default)]
pub struct FastHasher(u64);

impl FastHasher {
    #[inline]
    fn add(&mut self, word: u64) {
        self.0 = (self.0.rotate_left(5) ^ word).wrapping_mul(SEED);
    }
}

impl Hasher for FastHasher {
    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }

    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        let mut chunks = bytes.chunks_exact(8);
        for chunk in &mut chunks {
            let mut word = [0u8; 8];
            word.copy_from_slice(chunk);
            self.add(u64::from_le_bytes(word));
        }
        for &b in chunks.remainder() {
            self.add(u64::from(b));
        }
    }

    #[inline]
    fn write_u8(&mut self, i: u8) {
        self.add(u64::from(i));
    }

    #[inline]
    fn write_u16(&mut self, i: u16) {
        self.add(u64::from(i));
    }

    #[inline]
    fn write_u32(&mut self, i: u32) {
        self.add(u64::from(i));
    }

    #[inline]
    fn write_u64(&mut self, i: u64) {
        self.add(i);
    }

    #[inline]
    fn write_usize(&mut self, i: usize) {
        self.add(i as u64);
    }
}

/// A `HashMap` keyed by the engine's own integers, hashed with [`FastHasher`].
pub type FastMap<K, V> = HashMap<K, V, BuildHasherDefault<FastHasher>>;

/// A `HashSet` of the engine's own integers, hashed with [`FastHasher`].
pub type FastSet<K> = HashSet<K, BuildHasherDefault<FastHasher>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_find_what_they_hold_and_hash_the_same_every_time() {
        let mut m: FastMap<u32, u32> = FastMap::default();
        for i in 0..10_000u32 {
            m.insert(i.wrapping_mul(2_654_435_761), i);
        }
        assert_eq!(m.len(), 10_000);
        assert!((0..10_000u32).all(|i| m.get(&i.wrapping_mul(2_654_435_761)) == Some(&i)));
        let hash = |x: u64| {
            let mut h = FastHasher::default();
            h.write_u64(x);
            h.finish()
        };
        assert_eq!(hash(42), hash(42), "no random keys");
        assert_ne!(hash(42), hash(43));
        let mut s: FastSet<(u16, u32)> = FastSet::default();
        assert!(s.insert((1, 7)) && !s.insert((1, 7)) && s.insert((2, 7)));
    }
}
