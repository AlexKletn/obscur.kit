//! A runtime cache of converted Bézier paths (spec §5.19).

use std::collections::HashMap;

use vello::kurbo::BezPath;

/// An in-memory cache of converted [`BezPath`]s keyed by a content hash
/// (spec §5.19).
///
/// Path conversion (obscur-scene → kurbo) is pure but not free; a scene that
/// re-draws the same geometry every frame would otherwise re-run the conversion
/// each time. This cache stores the converted [`BezPath`] under a caller-computed
/// `u64` content hash so a repeat draw is a hash-map lookup. It holds no GPU
/// resources and is fully testable without a device.
///
/// The key is opaque: callers are responsible for hashing the source path's
/// geometry into a stable `u64`. Two different paths that collide on a hash would
/// alias — callers must use a hash wide enough to make that negligible.
#[derive(Debug, Default, Clone)]
pub struct PathCache {
    entries: HashMap<u64, BezPath>,
}

impl PathCache {
    /// Creates an empty path cache.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Inserts a converted `path` under `key`, returning any previous entry.
    pub fn insert(&mut self, key: u64, path: BezPath) -> Option<BezPath> {
        self.entries.insert(key, path)
    }

    /// The cached path for `key`, if present.
    #[must_use]
    pub fn get(&self, key: u64) -> Option<&BezPath> {
        self.entries.get(&key)
    }

    /// Whether a path is cached under `key`.
    #[must_use]
    pub fn contains(&self, key: u64) -> bool {
        self.entries.contains_key(&key)
    }

    /// Removes the entry for `key`, returning `true` if one was present.
    pub fn remove(&mut self, key: u64) -> bool {
        self.entries.remove(&key).is_some()
    }

    /// Clears every cached path.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// The number of cached paths.
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
    use vello::kurbo::{Rect, Shape};

    fn a_path(x: f64) -> BezPath {
        Rect::new(x, x, x + 10.0, x + 10.0).to_path(0.1)
    }

    #[test]
    fn new_is_empty() {
        let cache = PathCache::new();
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn insert_and_get_round_trip() {
        let mut cache = PathCache::new();
        assert!(cache.insert(1, a_path(0.0)).is_none());
        assert!(cache.contains(1));
        assert_eq!(cache.len(), 1);
        assert!(cache.get(1).is_some());
        assert!(cache.get(2).is_none());
    }

    #[test]
    fn insert_replaces_existing() {
        let mut cache = PathCache::new();
        cache.insert(1, a_path(0.0));
        let previous = cache.insert(1, a_path(5.0));
        assert!(previous.is_some());
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn remove_and_clear() {
        let mut cache = PathCache::new();
        cache.insert(1, a_path(0.0));
        cache.insert(2, a_path(1.0));
        assert!(cache.remove(1));
        assert!(!cache.remove(1));
        assert_eq!(cache.len(), 1);
        cache.clear();
        assert!(cache.is_empty());
    }
}
