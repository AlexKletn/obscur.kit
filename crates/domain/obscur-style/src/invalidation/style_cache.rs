//! The per-node computed-style cache: [`StyleCache`] (§4.3, §5.19).
//!
//! [`StyleCache`] memoises each node's [`ComputedStyle`] keyed by [`NodeId`], so
//! startup and subsequent frames can reuse an unchanged node's style instead of
//! recomputing it. Entries are version-stamped: a lookup that supplies a newer
//! version than the stored entry misses, keeping stale styles out of the results
//! (§4.3). Per §5.19 the whole cache has a disable switch — a
//! [`disabled`](StyleCache::disabled) cache never stores and always misses — so
//! correctness tests can run the resolver with caching transparently turned off.

use crate::computed::ComputedStyle;
use crate::invalidation::CacheEntry;
use obscur_identity::{NodeId, VersionStamp};
use std::collections::HashMap;

/// A cache of computed styles keyed by [`NodeId`] (§4.3, §5.19).
///
/// When enabled it stores one [`CacheEntry`] per node and serves hits from it.
/// When disabled every store is a no-op and every lookup misses, which is the
/// per-cache disable switch §5.19 requires for correctness tests. Because
/// [`ComputedStyle`] carries `f32` data, this derives [`PartialEq`] but not
/// [`Eq`].
#[derive(Debug, Clone, PartialEq)]
pub struct StyleCache {
    entries: HashMap<NodeId, CacheEntry>,
    enabled: bool,
}

impl StyleCache {
    /// Creates an empty, enabled cache.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            enabled: true,
        }
    }

    /// Creates a disabled cache that never stores and always misses (§5.19).
    ///
    /// Use this in correctness tests to run resolution as if the cache were not
    /// present, without changing any call site.
    #[must_use]
    pub fn disabled() -> Self {
        Self {
            entries: HashMap::new(),
            enabled: false,
        }
    }

    /// Returns whether the cache is enabled.
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Stores `style` for `node`, tagged with `version`, replacing any existing
    /// entry.
    ///
    /// A no-op when the cache is disabled (§5.19).
    pub fn insert(&mut self, node: NodeId, version: VersionStamp, style: ComputedStyle) {
        if !self.enabled {
            return;
        }
        self.entries.insert(node, CacheEntry::new(version, style));
    }

    /// Returns the cached style for `node`, ignoring version.
    ///
    /// Always `None` when the cache is disabled.
    #[must_use]
    pub fn get(&self, node: NodeId) -> Option<&ComputedStyle> {
        if !self.enabled {
            return None;
        }
        self.entries.get(&node).map(CacheEntry::style)
    }

    /// Returns the cached style for `node` only if it was computed at exactly
    /// `version`; a stale (older or newer) stored version misses (§4.3).
    ///
    /// Always `None` when the cache is disabled.
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

    /// Evicts the entry for `node`, if any.
    pub fn invalidate(&mut self, node: NodeId) {
        self.entries.remove(&node);
    }

    /// Evicts every entry.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Returns whether an entry for `node` is stored.
    ///
    /// Always `false` when the cache is disabled.
    #[must_use]
    pub fn contains(&self, node: NodeId) -> bool {
        self.enabled && self.entries.contains_key(&node)
    }

    /// Returns the number of stored entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns whether the cache holds no entries.
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
        assert!(cache.get_versioned(node(1), VersionStamp::from_raw(6)).is_none());
        // An older one misses too.
        assert!(cache.get_versioned(node(1), VersionStamp::from_raw(4)).is_none());
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
        assert!(cache.get_versioned(node(1), VersionStamp::from_raw(1)).is_none());
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
