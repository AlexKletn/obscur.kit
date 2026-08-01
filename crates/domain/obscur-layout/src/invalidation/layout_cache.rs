#![doc = include_str!("layout_cache.doc/_module.md")]

use crate::invalidation::CacheEntry;
use crate::model::{LayoutBox, LayoutResult};
use obscur_geometry::BoxConstraints;
use obscur_identity::{NodeId, VersionStamp};
use std::collections::HashMap;

#[doc = include_str!("layout_cache.doc/LayoutCache.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct LayoutCache {
    entries: HashMap<NodeId, CacheEntry>,
    enabled: bool,
}

impl LayoutCache {
    #[doc = include_str!("layout_cache.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            enabled: true,
        }
    }

    #[doc = include_str!("layout_cache.doc/disabled.md")]
    #[must_use]
    pub fn disabled() -> Self {
        Self {
            entries: HashMap::new(),
            enabled: false,
        }
    }

    #[doc = include_str!("layout_cache.doc/is_enabled.md")]
    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

    #[doc = include_str!("layout_cache.doc/insert_measure.md")]
    pub fn insert_measure(
        &mut self,
        node: NodeId,
        constraints: BoxConstraints,
        result: LayoutResult,
        version: VersionStamp,
    ) {
        if !self.enabled {
            return;
        }
        self.entries
            .insert(node, CacheEntry::new(constraints, result, version));
    }

    #[doc = include_str!("layout_cache.doc/get_measure.md")]
    #[must_use]
    pub fn get_measure(&self, node: NodeId, constraints: BoxConstraints) -> Option<LayoutResult> {
        if !self.enabled {
            return None;
        }
        self.entries
            .get(&node)
            .filter(|entry| entry.matches_constraints(constraints))
            .map(CacheEntry::result)
    }

    #[doc = include_str!("layout_cache.doc/insert_arrange.md")]
    pub fn insert_arrange(&mut self, node: NodeId, layout_box: LayoutBox) {
        if !self.enabled {
            return;
        }
        if let Some(entry) = self.entries.get_mut(&node) {
            entry.set_arranged(layout_box);
        }
    }

    #[doc = include_str!("layout_cache.doc/get_arrange.md")]
    #[must_use]
    pub fn get_arrange(&self, node: NodeId) -> Option<LayoutBox> {
        if !self.enabled {
            return None;
        }
        self.entries.get(&node).and_then(CacheEntry::arranged)
    }

    #[doc = include_str!("layout_cache.doc/invalidate.md")]
    pub fn invalidate(&mut self, node: NodeId) {
        self.entries.remove(&node);
    }

    #[doc = include_str!("layout_cache.doc/clear.md")]
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    #[doc = include_str!("layout_cache.doc/contains.md")]
    #[must_use]
    pub fn contains(&self, node: NodeId) -> bool {
        self.entries.contains_key(&node)
    }

    #[doc = include_str!("layout_cache.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[doc = include_str!("layout_cache.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for LayoutCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::LayoutCache;
    use crate::model::{LayoutBox, LayoutResult};
    use obscur_geometry::{BoxConstraints, Size};
    use obscur_identity::{NodeId, VersionStamp};

    fn node(n: u64) -> NodeId {
        NodeId::from_raw(n)
    }

    fn constraints(w: f32, h: f32) -> BoxConstraints {
        BoxConstraints::tight(Size::new(w, h))
    }

    fn result(w: f32, h: f32) -> LayoutResult {
        LayoutResult::new(Size::new(w, h))
    }

    #[test]
    fn default_is_enabled_and_empty() {
        let cache = LayoutCache::default();
        assert!(cache.is_enabled());
        assert!(cache.is_empty());
    }

    #[test]
    fn hit_on_identical_constraints() {
        let mut cache = LayoutCache::new();
        let c = constraints(100.0, 50.0);
        cache.insert_measure(node(1), c, result(100.0, 50.0), VersionStamp::from_raw(1));
        assert_eq!(cache.get_measure(node(1), c), Some(result(100.0, 50.0)));
    }

    #[test]
    fn miss_on_different_constraints() {
        let mut cache = LayoutCache::new();
        cache.insert_measure(
            node(1),
            constraints(100.0, 50.0),
            result(100.0, 50.0),
            VersionStamp::from_raw(1),
        );
        assert_eq!(cache.get_measure(node(1), constraints(100.0, 60.0)), None);
    }

    #[test]
    fn miss_on_unknown_node() {
        let cache = LayoutCache::new();
        assert_eq!(cache.get_measure(node(99), constraints(1.0, 1.0)), None);
    }

    #[test]
    fn disabled_never_stores_and_always_misses() {
        let mut cache = LayoutCache::disabled();
        assert!(!cache.is_enabled());
        let c = constraints(10.0, 10.0);
        cache.insert_measure(node(1), c, result(10.0, 10.0), VersionStamp::from_raw(1));
        assert!(cache.is_empty());
        assert_eq!(cache.get_measure(node(1), c), None);
    }

    #[test]
    fn invalidate_evicts_one() {
        let mut cache = LayoutCache::new();
        let c = constraints(10.0, 10.0);
        cache.insert_measure(node(1), c, result(10.0, 10.0), VersionStamp::from_raw(1));
        assert!(cache.contains(node(1)));
        cache.invalidate(node(1));
        assert!(!cache.contains(node(1)));
        assert_eq!(cache.get_measure(node(1), c), None);
    }

    #[test]
    fn clear_evicts_all() {
        let mut cache = LayoutCache::new();
        cache.insert_measure(
            node(1),
            constraints(10.0, 10.0),
            result(10.0, 10.0),
            VersionStamp::from_raw(1),
        );
        cache.insert_measure(
            node(2),
            constraints(20.0, 20.0),
            result(20.0, 20.0),
            VersionStamp::from_raw(1),
        );
        assert_eq!(cache.len(), 2);
        cache.clear();
        assert!(cache.is_empty());
    }

    #[test]
    fn arrange_round_trips_after_measure() {
        let mut cache = LayoutCache::new();
        let c = constraints(10.0, 10.0);
        cache.insert_measure(node(1), c, result(10.0, 10.0), VersionStamp::from_raw(1));
        let b = LayoutBox::from_size(Size::new(10.0, 10.0));
        cache.insert_arrange(node(1), b);
        assert_eq!(cache.get_arrange(node(1)), Some(b));
    }

    #[test]
    fn arrange_without_measure_is_noop() {
        let mut cache = LayoutCache::new();
        cache.insert_arrange(node(1), LayoutBox::from_size(Size::new(10.0, 10.0)));
        assert_eq!(cache.get_arrange(node(1)), None);
    }

    #[test]
    fn disabled_arrange_misses() {
        let mut cache = LayoutCache::disabled();
        cache.insert_arrange(node(1), LayoutBox::from_size(Size::new(10.0, 10.0)));
        assert_eq!(cache.get_arrange(node(1)), None);
    }
}
