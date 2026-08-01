#![doc = include_str!("texture_cache.doc/_module.md")]

use std::collections::HashMap;

#[doc = include_str!("texture_cache.doc/TextureCache.md")]
#[derive(Default, Clone)]
pub struct TextureCache {
    entries: HashMap<u64, wgpu::Texture>,
}

impl TextureCache {
    #[doc = include_str!("texture_cache.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    #[doc = include_str!("texture_cache.doc/insert.md")]
    pub fn insert(&mut self, key: u64, texture: wgpu::Texture) -> Option<wgpu::Texture> {
        self.entries.insert(key, texture)
    }

    #[doc = include_str!("texture_cache.doc/get.md")]
    #[must_use]
    pub fn get(&self, key: u64) -> Option<&wgpu::Texture> {
        self.entries.get(&key)
    }

    #[doc = include_str!("texture_cache.doc/contains.md")]
    #[must_use]
    pub fn contains(&self, key: u64) -> bool {
        self.entries.contains_key(&key)
    }

    #[doc = include_str!("texture_cache.doc/remove.md")]
    pub fn remove(&mut self, key: u64) -> bool {
        self.entries.remove(&key).is_some()
    }

    #[doc = include_str!("texture_cache.doc/clear.md")]
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    #[doc = include_str!("texture_cache.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[doc = include_str!("texture_cache.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl std::fmt::Debug for TextureCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // `wgpu::Texture` values are opaque; report only the population count.
        f.debug_struct("TextureCache")
            .field("len", &self.entries.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // NOTE: A `wgpu::Texture` cannot be constructed without a GPU device, which
    // this environment lacks (spec §2.7). Tests therefore cover only the
    // device-free lifecycle: an empty cache. Populated behaviour is exercised by
    // the shared map API used identically by `PathCache`/`GlyphCache`.

    #[test]
    fn new_is_empty() {
        let cache = TextureCache::new();
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
        assert!(!cache.contains(0));
        assert!(cache.get(0).is_none());
    }

    #[test]
    fn default_is_empty() {
        assert!(TextureCache::default().is_empty());
    }
}
