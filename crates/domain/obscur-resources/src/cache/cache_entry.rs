//! Bookkeeping for one cached item (spec §5.19 caches, visibility priority).

use crate::cache::{CacheKey, VisibilityPriority};
use crate::state::ResourceVersion;

#[doc = include_str!("../../docs/api/cache/cache_entry.doc/CacheEntry.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CacheEntry {
    /// Identity of the cached artifact.
    key: CacheKey,
    /// Size of the artifact in bytes, for memory budgeting (spec §5.19).
    byte_size: u64,
    /// The resource version this artifact was derived at (correctness).
    version: ResourceVersion,
    /// On-screen priority driving eviction order (spec §5.19).
    visibility: VisibilityPriority,
    /// Logical tick of the most recent access (deterministic, not wall-clock).
    last_access: u64,
    /// Number of times this entry has been accessed, for frequency policies.
    hits: u64,
}

impl CacheEntry {
    #[doc = include_str!("../../docs/api/cache/cache_entry.doc/new.md")]
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

    /// Returns this entry with its visibility priority set (spec §5.19).
    #[must_use]
    pub const fn with_visibility(mut self, visibility: VisibilityPriority) -> Self {
        self.visibility = visibility;
        self
    }

    #[doc = include_str!("../../docs/api/cache/cache_entry.doc/touch.md")]
    pub fn touch(&mut self, tick: u64) {
        self.last_access = tick;
        self.hits = self.hits.saturating_add(1);
    }

    /// Records a cache hit at logical `tick`; an alias for
    /// [`touch`](CacheEntry::touch).
    pub fn record_hit(&mut self, tick: u64) {
        self.touch(tick);
    }

    /// Returns the artifact's identity.
    #[must_use]
    pub const fn key(&self) -> CacheKey {
        self.key
    }

    /// Returns the artifact's size in bytes (spec §5.19 memory budgets).
    #[must_use]
    pub const fn byte_size(&self) -> u64 {
        self.byte_size
    }

    /// Returns the resource version this artifact was derived at.
    #[must_use]
    pub const fn version(&self) -> ResourceVersion {
        self.version
    }

    /// Returns the entry's visibility priority (spec §5.19).
    #[must_use]
    pub const fn visibility(&self) -> VisibilityPriority {
        self.visibility
    }

    /// Returns the logical tick of the most recent access.
    #[must_use]
    pub const fn last_access(&self) -> u64 {
        self.last_access
    }

    /// Returns how many times this entry has been accessed.
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
}
