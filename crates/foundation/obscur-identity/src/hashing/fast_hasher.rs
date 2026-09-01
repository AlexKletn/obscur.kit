#![doc = include_str!("fast_hasher.doc/_module.md")]

use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasherDefault, Hasher};

#[doc = include_str!("fast_hasher.doc/SEED.md")]
const SEED: u64 = 0x51_7c_c1_b7_27_22_0a_95;

#[doc = include_str!("fast_hasher.doc/FINALIZER.md")]
const FINALIZER: u64 = 0xd6e8_feb8_6659_fd93;

#[doc = include_str!("fast_hasher.doc/FastHasher.md")]
#[derive(Debug, Clone, Copy, Default)]
pub struct FastHasher {
    hash: u64,
}

impl FastHasher {
    #[doc = include_str!("fast_hasher.doc/absorb.md")]
    fn absorb(&mut self, word: u64) {
        self.hash = (self.hash.rotate_left(5) ^ word).wrapping_mul(SEED);
    }
}

impl Hasher for FastHasher {
    // The absorbing step alone leaves the *top* bits of a dense key sequence on
    // a lattice — `0, 1, 2, …` times an odd constant walks the high end in
    // near-uniform strides — and those are the bits `hashbrown` stamps each slot
    // with and compares before it compares a key. Two shifts, two xors and a
    // multiply is the whole price of spreading them, against the roughly thirty
    // times that which SipHash charged.
    fn finish(&self) -> u64 {
        let mut hash = self.hash;
        hash ^= hash >> 32;
        hash = hash.wrapping_mul(FINALIZER);
        hash ^= hash >> 29;
        hash
    }

    fn write(&mut self, bytes: &[u8]) {
        let mut chunks = bytes.chunks_exact(8);
        for chunk in &mut chunks {
            let mut word = [0u8; 8];
            word.copy_from_slice(chunk);
            self.absorb(u64::from_ne_bytes(word));
        }
        let tail = chunks.remainder();
        if !tail.is_empty() {
            let mut word = [0u8; 8];
            word[..tail.len()].copy_from_slice(tail);
            self.absorb(u64::from_ne_bytes(word));
        }
    }

    // The derived `Hash` of a fieldless enum writes one integer and nothing
    // else, and that single call is the whole reason this type exists: it has
    // to compile to a rotate, an xor and a multiply, not to a loop over a byte
    // slice.
    fn write_u8(&mut self, value: u8) {
        self.absorb(u64::from(value));
    }

    fn write_u16(&mut self, value: u16) {
        self.absorb(u64::from(value));
    }

    fn write_u32(&mut self, value: u32) {
        self.absorb(u64::from(value));
    }

    fn write_u64(&mut self, value: u64) {
        self.absorb(value);
    }

    fn write_u128(&mut self, value: u128) {
        self.write(&value.to_ne_bytes());
    }

    fn write_usize(&mut self, value: usize) {
        self.absorb(value as u64);
    }
}

#[doc = include_str!("fast_hasher.doc/FastBuildHasher.md")]
pub type FastBuildHasher = BuildHasherDefault<FastHasher>;

#[doc = include_str!("fast_hasher.doc/FastMap.md")]
pub type FastMap<K, V> = HashMap<K, V, FastBuildHasher>;

#[doc = include_str!("fast_hasher.doc/FastSet.md")]
pub type FastSet<T> = HashSet<T, FastBuildHasher>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::hash::Hash;

    fn hash_of<T: Hash>(value: &T) -> u64 {
        let mut hasher = FastHasher::default();
        value.hash(&mut hasher);
        hasher.finish()
    }

    #[test]
    fn a_hasher_is_a_function_of_its_input_and_nothing_else() {
        // Unlike `RandomState`, which reseeds per process: two runs of the same
        // program must agree, or a golden test over an iteration order would
        // pass on one run and fail on the next.
        assert_eq!(hash_of(&7u64), hash_of(&7u64));
        assert_ne!(hash_of(&7u64), hash_of(&8u64));
    }

    #[test]
    fn adjacent_small_integers_land_in_different_buckets() {
        // The property the whole substitution rests on. Enum discriminants and
        // node ids are dense and start at zero, and a hash that leaves the low
        // bits nearly untouched would pile them into one bucket and turn every
        // lookup into a scan — slower than the SipHash it replaced.
        const BUCKETS: u64 = 64;
        let occupied: FastSet<u64> = (0..BUCKETS).map(|id| hash_of(&id) % BUCKETS).collect();
        assert!(
            occupied.len() >= 40,
            "{} of {BUCKETS} buckets used — the low bits are not spread",
            occupied.len()
        );
    }

    #[test]
    fn the_top_bits_move_too_because_that_is_what_the_table_tags_with() {
        // `hashbrown` tags each slot with the top seven bits of the hash and
        // compares those before it ever compares a key. If they were constant
        // across dense keys, every probe would hit a tag match and fall through
        // to a full key comparison.
        const KEYS: u64 = 64;
        let tags: FastSet<u64> = (0..KEYS).map(|id| hash_of(&id) >> 57).collect();
        assert!(tags.len() >= 40, "{} of {KEYS} tags distinct", tags.len());
    }

    #[test]
    fn a_byte_string_is_hashed_whole_including_a_tail_shorter_than_a_word() {
        // `write` walks eight bytes at a time; the remainder is the branch that
        // is easy to drop, and dropping it makes every string that shares a
        // multiple-of-eight prefix collide.
        assert_ne!(hash_of(&"abcdefghi"), hash_of(&"abcdefghj"));
        assert_ne!(hash_of(&"abcdefgh"), hash_of(&"abcdefghi"));
    }

    #[test]
    fn a_map_under_this_hasher_still_behaves_like_a_map() {
        let mut map: FastMap<u32, &str> = FastMap::default();
        for key in 0..1000u32 {
            map.insert(key, "v");
        }
        assert_eq!(map.len(), 1000);
        assert!((0..1000u32).all(|key| map.get(&key) == Some(&"v")));
        assert_eq!(map.get(&1000), None);
    }
}
