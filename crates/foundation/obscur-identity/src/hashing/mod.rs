#![doc = include_str!("mod.doc/_module.md")]
mod fast_hasher;

pub use fast_hasher::{FastBuildHasher, FastHasher, FastMap, FastSet};
