#![doc = include_str!("cache_entry.doc/_module.md")]

use crate::cache::CacheKey;
use crate::state::ResourceVersion;

#[doc = include_str!("cache_entry.doc/VisibilityPriority.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum VisibilityPriority {
    #[doc = include_str!("cache_entry.doc/Hidden.md")]
    Hidden,
    #[doc = include_str!("cache_entry.doc/Offscreen.md")]
    Offscreen,
    #[doc = include_str!("cache_entry.doc/NearViewport.md")]
    NearViewport,
    #[doc = include_str!("cache_entry.doc/Visible.md")]
    Visible,
}

#[doc = include_str!("cache_entry.doc/CacheEntry.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CacheEntry {
    #[doc = include_str!("cache_entry.doc/key.md")]
    key: CacheKey,
    #[doc = include_str!("cache_entry.doc/byte_size.md")]
    byte_size: u64,
    #[doc = include_str!("cache_entry.doc/version.md")]
    version: ResourceVersion,
    #[doc = include_str!("cache_entry.doc/visibility.md")]
    visibility: VisibilityPriority,
    #[doc = include_str!("cache_entry.doc/last_access.md")]
    last_access: u64,
    #[doc = include_str!("cache_entry.doc/hits.md")]
    hits: u64,
}

impl CacheEntry {
    #[doc = include_str!("cache_entry.doc/new.md")]
    #[must_use]
    pub const fn new(key: CacheKey, byte_size: u64, version: ResourceVersion) -> Self {
        Self {
            key,
            byte_size,
            version,
            visibility: VisibilityPriority::Visible,
            last_access: 0,
            hits: 0,
        }
    }

    #[doc = include_str!("cache_entry.doc/with_visibility.md")]
    #[must_use]
    pub const fn with_visibility(mut self, visibility: VisibilityPriority) -> Self {
        self.visibility = visibility;
        self
    }

    #[doc = include_str!("cache_entry.doc/touch.md")]
    pub fn touch(&mut self, tick: u64) {
        self.last_access = tick;
        self.hits = self.hits.saturating_add(1);
    }

    #[doc = include_str!("cache_entry.doc/record_hit.md")]
    pub fn record_hit(&mut self, tick: u64) {
        self.touch(tick);
    }

    #[doc = include_str!("cache_entry.doc/key_2.md")]
    #[must_use]
    pub const fn key(&self) -> CacheKey {
        self.key
    }

    #[doc = include_str!("cache_entry.doc/byte_size_2.md")]
    #[must_use]
    pub const fn byte_size(&self) -> u64 {
        self.byte_size
    }

    #[doc = include_str!("cache_entry.doc/version_2.md")]
    #[must_use]
    pub const fn version(&self) -> ResourceVersion {
        self.version
    }

    #[doc = include_str!("cache_entry.doc/visibility_2.md")]
    #[must_use]
    pub const fn visibility(&self) -> VisibilityPriority {
        self.visibility
    }

    #[doc = include_str!("cache_entry.doc/last_access_2.md")]
    #[must_use]
    pub const fn last_access(&self) -> u64 {
        self.last_access
    }

    #[doc = include_str!("cache_entry.doc/hits_2.md")]
    #[must_use]
    pub const fn hits(&self) -> u64 {
        self.hits
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_identity::ResourceId;

    fn key() -> CacheKey {
        CacheKey::for_resource(ResourceId::from_raw(1))
    }

    #[test]
    fn new_sets_defaults() {
        let e = CacheEntry::new(key(), 512, ResourceVersion::initial());
        assert_eq!(e.byte_size(), 512);
        assert_eq!(e.version(), ResourceVersion::initial());
        assert_eq!(e.visibility(), VisibilityPriority::Visible);
        assert_eq!(e.last_access(), 0);
        assert_eq!(e.hits(), 0);
    }

    #[test]
    fn with_visibility_overrides_default() {
        let e = CacheEntry::new(key(), 1, ResourceVersion::initial())
            .with_visibility(VisibilityPriority::Hidden);
        assert_eq!(e.visibility(), VisibilityPriority::Hidden);
    }

    #[test]
    fn touch_sets_tick_and_increments_hits() {
        let mut e = CacheEntry::new(key(), 1, ResourceVersion::initial());
        e.touch(10);
        assert_eq!(e.last_access(), 10);
        assert_eq!(e.hits(), 1);
        e.record_hit(25);
        assert_eq!(e.last_access(), 25);
        assert_eq!(e.hits(), 2);
    }

    #[test]
    fn hits_saturate() {
        let mut e = CacheEntry::new(key(), 1, ResourceVersion::initial());
        e.hits = u64::MAX;
        e.touch(1);
        assert_eq!(e.hits(), u64::MAX);
    }

    #[test]
    fn visibility_orders_hidden_smallest() {
        assert!(VisibilityPriority::Hidden < VisibilityPriority::Offscreen);
        assert!(VisibilityPriority::Offscreen < VisibilityPriority::NearViewport);
        assert!(VisibilityPriority::NearViewport < VisibilityPriority::Visible);
    }
}
