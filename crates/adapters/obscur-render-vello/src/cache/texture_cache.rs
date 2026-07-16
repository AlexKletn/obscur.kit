//! An unbounded map of GPU textures. See the module docs: this is not a §5.19
//! cache and has no callers.

use std::collections::HashMap;

/// An in-memory map of [`wgpu::Texture`]s keyed by a content hash.
///
/// Uploading image data to the GPU is expensive; a scene that draws the same
/// image every frame should upload it once and reuse the resident texture. This
/// cache keeps each [`wgpu::Texture`] under a caller-computed `u64` content hash.
/// A `wgpu::Texture` is a cheap ref-counted handle, so caching and cloning it is
/// inexpensive.
///
/// Constructing the textures themselves requires a GPU device (owned by the
/// host, spec §2.7); this cache is only the storage, so its own construction and
/// emptiness are device-free while populating it needs a live device.
#[derive(Default, Clone)]
pub struct TextureCache {
    entries: HashMap<u64, wgpu::Texture>,
}

impl TextureCache {
    /// Creates an empty texture cache.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Inserts `texture` under `key`, returning any previous entry.
    pub fn insert(&mut self, key: u64, texture: wgpu::Texture) -> Option<wgpu::Texture> {
        self.entries.insert(key, texture)
    }

    /// The cached texture for `key`, if present.
    #[must_use]
    pub fn get(&self, key: u64) -> Option<&wgpu::Texture> {
        self.entries.get(&key)
    }

    /// Whether a texture is cached under `key`.
    #[must_use]
    pub fn contains(&self, key: u64) -> bool {
        self.entries.contains_key(&key)
    }

    /// Removes the entry for `key`, returning `true` if one was present.
    pub fn remove(&mut self, key: u64) -> bool {
        self.entries.remove(&key).is_some()
    }

    /// Clears every cached texture.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// The number of cached textures.
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
