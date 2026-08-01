#![doc = include_str!("style_cache.doc/_module.md")]

use crate::computed::ComputedStyle;
use crate::invalidation::CacheEntry;
use obscur_identity::{NodeId, VersionStamp};
use std::collections::HashMap;

#[doc = include_str!("style_cache.doc/StyleCache.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct StyleCache {
    entries: HashMap<NodeId, CacheEntry>,
    enabled: bool,
}

impl StyleCache {
    #[doc = include_str!("style_cache.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            enabled: true,
        }
    }

    #[doc = include_str!("style_cache.doc/disabled.md")]
    #[must_use]
    pub fn disabled() -> Self {
        Self {
            entries: HashMap::new(),
            enabled: false,
        }
    }

    #[doc = include_str!("style_cache.doc/is_enabled.md")]
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    #[doc = include_str!("style_cache.doc/insert.md")]
    pub fn insert(&mut self, node: NodeId, version: VersionStamp, style: ComputedStyle) {
        if !self.enabled {
            return;
        }
        self.entries.insert(node, CacheEntry::new(version, style));
    }

    #[doc = include_str!("style_cache.doc/get.md")]
    #[must_use]
    pub fn get(&self, node: NodeId) -> Option<&ComputedStyle> {
        if !self.enabled {
            return None;
        }
        self.entries.get(&node).map(CacheEntry::style)
    }

    #[doc = include_str!("style_cache.doc/get_versioned.md")]
    #[must_use]
    pub fn get_versioned(&self, node: NodeId, version: VersionStamp) -> Option<&ComputedStyle> {
        if !self.enabled {
            return None;
        }
        self.entries
            .get(&node)
            .filter(|entry| entry.is_current(version))
            .map(CacheEntry::style)
    }

    #[doc = include_str!("style_cache.doc/invalidate.md")]
    pub fn invalidate(&mut self, node: NodeId) {
        self.entries.remove(&node);
    }

    #[doc = include_str!("style_cache.doc/clear.md")]
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    #[doc = include_str!("style_cache.doc/contains.md")]
    #[must_use]
    pub fn contains(&self, node: NodeId) -> bool {
        self.enabled && self.entries.contains_key(&node)
    }

    #[doc = include_str!("style_cache.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[doc = include_str!("style_cache.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for StyleCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::computed::ComputedValue;
    use crate::property::PropertyId;

    fn node(raw: u64) -> NodeId {
        NodeId::from_raw(raw)
    }

    fn style_with_opacity(value: f32) -> ComputedStyle {
        let mut style = ComputedStyle::new();
        style.set(PropertyId::Opacity, ComputedValue::number(value));
        style
    }

    #[test]
    fn insert_then_get() {
        let mut cache = StyleCache::new();
        cache.insert(node(1), VersionStamp::from_raw(1), style_with_opacity(0.5));
        assert!(cache.contains(node(1)));
        assert_eq!(cache.len(), 1);
        assert_eq!(cache.get(node(1)), Some(&style_with_opacity(0.5)));
    }

    #[test]
    fn get_versioned_hits_on_matching_version() {
        let mut cache = StyleCache::new();
        cache.insert(node(1), VersionStamp::from_raw(5), style_with_opacity(0.5));
        assert_eq!(
            cache.get_versioned(node(1), VersionStamp::from_raw(5)),
            Some(&style_with_opacity(0.5))
        );
    }

    #[test]
    fn get_versioned_misses_on_stale_version() {
        let mut cache = StyleCache::new();
        cache.insert(node(1), VersionStamp::from_raw(5), style_with_opacity(0.5));
        // A newer requested version does not match the stored one.
        assert!(
            cache
                .get_versioned(node(1), VersionStamp::from_raw(6))
                .is_none()
        );
        // An older one misses too.
        assert!(
            cache
                .get_versioned(node(1), VersionStamp::from_raw(4))
                .is_none()
        );
        // The unversioned accessor still hits.
        assert!(cache.get(node(1)).is_some());
    }

    #[test]
    fn disabled_cache_never_stores() {
        let mut cache = StyleCache::disabled();
        assert!(!cache.is_enabled());
        cache.insert(node(1), VersionStamp::from_raw(1), style_with_opacity(0.5));
        assert!(cache.is_empty());
        assert!(!cache.contains(node(1)));
        assert!(cache.get(node(1)).is_none());
        assert!(
            cache
                .get_versioned(node(1), VersionStamp::from_raw(1))
                .is_none()
        );
    }

    #[test]
    fn invalidate_and_clear_evict() {
        let mut cache = StyleCache::new();
        cache.insert(node(1), VersionStamp::from_raw(1), style_with_opacity(0.5));
        cache.insert(node(2), VersionStamp::from_raw(1), style_with_opacity(0.5));
        cache.invalidate(node(1));
        assert!(!cache.contains(node(1)));
        assert!(cache.contains(node(2)));
        cache.clear();
        assert!(cache.is_empty());
    }
}
