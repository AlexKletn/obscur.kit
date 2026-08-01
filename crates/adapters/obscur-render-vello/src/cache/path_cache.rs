#![doc = include_str!("path_cache.doc/_module.md")]

use std::collections::HashMap;

use vello::kurbo::BezPath;

#[doc = include_str!("path_cache.doc/PathCache.md")]
#[derive(Debug, Default, Clone)]
pub struct PathCache {
    entries: HashMap<u64, BezPath>,
}

impl PathCache {
    #[doc = include_str!("path_cache.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    #[doc = include_str!("path_cache.doc/insert.md")]
    pub fn insert(&mut self, key: u64, path: BezPath) -> Option<BezPath> {
        self.entries.insert(key, path)
    }

    #[doc = include_str!("path_cache.doc/get.md")]
    #[must_use]
    pub fn get(&self, key: u64) -> Option<&BezPath> {
        self.entries.get(&key)
    }

    #[doc = include_str!("path_cache.doc/contains.md")]
    #[must_use]
    pub fn contains(&self, key: u64) -> bool {
        self.entries.contains_key(&key)
    }

    #[doc = include_str!("path_cache.doc/remove.md")]
    pub fn remove(&mut self, key: u64) -> bool {
        self.entries.remove(&key).is_some()
    }

    #[doc = include_str!("path_cache.doc/clear.md")]
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    #[doc = include_str!("path_cache.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[doc = include_str!("path_cache.doc/is_empty.md")]
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
