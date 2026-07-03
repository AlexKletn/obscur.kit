//! Bookkeeping for one cached item (spec §5.19 caches, visibility priority).

use crate::cache::CacheKey;
use crate::state::ResourceVersion;

/// How visible a cached artifact is on screen, driving eviction priority (spec
/// §5.19 *visibility priority*).
///
/// The variants are declared in **ascending importance** order so the derived
/// `Ord` makes the most-evictable the *smallest*: `Hidden < Offscreen <
/// NearViewport < Visible`. An eviction policy that prefers to drop the least
/// important artifact can therefore simply pick the minimum visibility (see
/// [`crate::cache::EvictionPolicy`]). `Hidden` is thus the first eviction
/// candidate and `Visible` is retained longest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum VisibilityPriority {
    /// Not shown at all; the first artifact to evict.
    Hidden,
    /// Fully off-screen but still live.
    Offscreen,
    /// Just outside the viewport; likely to be needed soon.
    NearViewport,
    /// Currently on screen; retained longest.
    Visible,
}

/// Bookkeeping metadata for one item held in a runtime cache (spec §5.19).
///
/// A cache entry records what a store needs to make eviction and metrics
/// decisions without holding the artifact bytes themselves: the [`CacheKey`]
/// identity, the artifact's `byte_size` (for memory budgets), the
/// [`ResourceVersion`] it was derived at (for correctness transparency — a stale
/// version can be invalidated), its [`VisibilityPriority`], and access
/// bookkeeping.
///
/// `last_access` is a **logical tick**: a monotonically increasing `u64` the
/// caller supplies via [`touch`](CacheEntry::touch), never a wall-clock reading.
/// This keeps recency deterministic (spec §5.19 caches are pure, reproducible
/// bookkeeping).
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
    /// Creates a fresh entry for `key` of `byte_size` bytes at `version`.
    ///
    /// Visibility defaults to [`VisibilityPriority::Visible`], `last_access` to
    /// tick `0`, and `hits` to `0`.
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

    /// Records an access at logical `tick`: updates `last_access` and increments
    /// the hit count (spec §5.19).
    ///
    /// `tick` is a logical, caller-supplied counter — never a wall-clock reading
    /// — so recency stays deterministic. The hit count saturates.
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

    #[test]
    fn visibility_orders_hidden_smallest() {
        assert!(VisibilityPriority::Hidden < VisibilityPriority::Offscreen);
        assert!(VisibilityPriority::Offscreen < VisibilityPriority::NearViewport);
        assert!(VisibilityPriority::NearViewport < VisibilityPriority::Visible);
    }
}
