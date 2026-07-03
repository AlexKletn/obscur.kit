//! A runtime cache of rasterized glyph run data (spec §5.19).

use std::collections::HashMap;
use std::sync::Arc;

/// A single cached glyph's rasterized coverage (spec §5.19).
///
/// Holds the glyph's pixel dimensions and its coverage bytes behind a shared
/// [`Arc`] so a cache hit is a cheap pointer clone rather than a copy. The blob
/// is opaque here; the eventual atlas packer decides its exact layout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlyphEntry {
    /// The rasterized glyph width, in pixels.
    pub width: u32,
    /// The rasterized glyph height, in pixels.
    pub height: u32,
    /// The glyph's rasterized coverage data, shared for cheap cloning.
    pub data: Arc<[u8]>,
}

impl GlyphEntry {
    /// Creates a glyph entry from its `width`, `height`, and coverage `data`.
    #[must_use]
    pub fn new(width: u32, height: u32, data: Arc<[u8]>) -> Self {
        Self {
            width,
            height,
            data,
        }
    }
}

/// An in-memory cache of rasterized glyphs keyed by a font+size+glyph hash
/// (spec §5.19).
///
/// Rasterizing a glyph run is costly and highly repetitive — the same glyph at
/// the same size recurs across frames and across text. This cache stores each
/// rasterized [`GlyphEntry`] under a caller-computed `u64` that folds together
/// the font identity, size, and glyph id. It holds no GPU resources and is fully
/// testable without a device.
///
/// This is deliberately a simple map: real glyph caching packs coverage into a
/// GPU atlas texture and tracks free rectangles. That atlas packing is a future
/// refinement; the key/entry contract here is the stable part callers depend on.
#[derive(Debug, Default, Clone)]
pub struct GlyphCache {
    entries: HashMap<u64, GlyphEntry>,
}

impl GlyphCache {
    /// Creates an empty glyph cache.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Inserts a rasterized `entry` under `key`, returning any previous entry.
    pub fn insert(&mut self, key: u64, entry: GlyphEntry) -> Option<GlyphEntry> {
        self.entries.insert(key, entry)
    }

    /// The cached glyph for `key`, if present.
    #[must_use]
    pub fn get(&self, key: u64) -> Option<&GlyphEntry> {
        self.entries.get(&key)
    }

    /// Whether a glyph is cached under `key`.
    #[must_use]
    pub fn contains(&self, key: u64) -> bool {
        self.entries.contains_key(&key)
    }

    /// Removes the entry for `key`, returning `true` if one was present.
    pub fn remove(&mut self, key: u64) -> bool {
        self.entries.remove(&key).is_some()
    }

    /// Clears every cached glyph.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// The number of cached glyphs.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the cache is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn an_entry(byte: u8) -> GlyphEntry {
        GlyphEntry::new(4, 4, Arc::from(vec![byte; 16].into_boxed_slice()))
    }

    #[test]
    fn new_is_empty() {
        let cache = GlyphCache::new();
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn insert_and_get_round_trip() {
        let mut cache = GlyphCache::new();
        assert!(cache.insert(1, an_entry(0xAB)).is_none());
        assert!(cache.contains(1));
        let got = cache.get(1).expect("entry present");
        assert_eq!(got.width, 4);
        assert_eq!(got.height, 4);
        assert_eq!(got.data.len(), 16);
        assert!(cache.get(2).is_none());
    }

    #[test]
    fn insert_replaces_existing() {
        let mut cache = GlyphCache::new();
        cache.insert(1, an_entry(0x00));
        let previous = cache.insert(1, an_entry(0xFF));
        assert!(previous.is_some());
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn remove_and_clear() {
        let mut cache = GlyphCache::new();
        cache.insert(1, an_entry(1));
        cache.insert(2, an_entry(2));
        assert!(cache.remove(1));
        assert!(!cache.remove(1));
        assert_eq!(cache.len(), 1);
        cache.clear();
        assert!(cache.is_empty());
    }
}
