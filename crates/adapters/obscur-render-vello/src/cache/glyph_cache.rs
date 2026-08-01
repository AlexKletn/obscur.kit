#![doc = include_str!("glyph_cache.doc/_module.md")]

use std::collections::HashMap;
use std::sync::Arc;

#[doc = include_str!("glyph_cache.doc/GlyphEntry.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlyphEntry {
    #[doc = include_str!("glyph_cache.doc/width.md")]
    pub width: u32,
    #[doc = include_str!("glyph_cache.doc/height.md")]
    pub height: u32,
    #[doc = include_str!("glyph_cache.doc/data.md")]
    pub data: Arc<[u8]>,
}

impl GlyphEntry {
    #[doc = include_str!("glyph_cache.doc/new.md")]
    #[must_use]
    pub fn new(width: u32, height: u32, data: Arc<[u8]>) -> Self {
        Self {
            width,
            height,
            data,
        }
    }
}

#[doc = include_str!("glyph_cache.doc/GlyphCache.md")]
#[derive(Debug, Default, Clone)]
pub struct GlyphCache {
    entries: HashMap<u64, GlyphEntry>,
}

impl GlyphCache {
    #[doc = include_str!("glyph_cache.doc/new_2.md")]
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    #[doc = include_str!("glyph_cache.doc/insert.md")]
    pub fn insert(&mut self, key: u64, entry: GlyphEntry) -> Option<GlyphEntry> {
        self.entries.insert(key, entry)
    }

    #[doc = include_str!("glyph_cache.doc/get.md")]
    #[must_use]
    pub fn get(&self, key: u64) -> Option<&GlyphEntry> {
        self.entries.get(&key)
    }

    #[doc = include_str!("glyph_cache.doc/contains.md")]
    #[must_use]
    pub fn contains(&self, key: u64) -> bool {
        self.entries.contains_key(&key)
    }

    #[doc = include_str!("glyph_cache.doc/remove.md")]
    pub fn remove(&mut self, key: u64) -> bool {
        self.entries.remove(&key).is_some()
    }

    #[doc = include_str!("glyph_cache.doc/clear.md")]
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    #[doc = include_str!("glyph_cache.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[doc = include_str!("glyph_cache.doc/is_empty.md")]
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
