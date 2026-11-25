//! The runtime's budgeted, evicting cache of decoded images (spec §5.19).

use std::collections::HashMap;
use std::sync::OnceLock;

use obscur_resources::cache::{
    CacheBudget, CacheEntry, CacheKey, CacheStatistics, EvictionPolicy, VisibilityPriority,
};
use obscur_resources::state::ResourceVersion;
use obscur_scene::paint::SceneImage;

#[doc = include_str!("../../docs/api/runtime/image_cache.doc/DEFAULT_MAX_BYTES.md")]
const DEFAULT_MAX_BYTES: u64 = 128 * 1024 * 1024;

#[doc = include_str!("../../docs/api/runtime/image_cache.doc/DEFAULT_MAX_ENTRIES.md")]
const DEFAULT_MAX_ENTRIES: usize = 512;

const VISIBILITY_LEVELS: usize = VisibilityPriority::Visible as usize + 1;

#[doc = include_str!("../../docs/api/runtime/image_cache.doc/ImageCache.md")]
pub struct ImageCache {
    /// The decoded pixels, keyed by the raw authored URL/path string — the map
    /// compose looks images up in.
    images: HashMap<Box<str>, SceneImage>,
    /// Per-URL §5.19 bookkeeping (identity, byte size, version, visibility,
    /// recency), parallel to [`images`](Self::images).
    entries: HashMap<Box<str>, CacheEntry>,
    /// The memory budget admission and eviction are ruled against.
    budget: CacheBudget,
    /// How a victim is chosen when the budget is exceeded.
    policy: EvictionPolicy,
    /// Live totals plus the `DevTools` hit/miss/eviction counters (spec §5.19).
    stats: CacheStatistics,
    /// The logical clock recency is stamped from: a monotonic counter, never a
    /// wall-clock reading, so eviction order is reproducible.
    tick: u64,
    protected_usage: OnceLock<[(u64, usize); VISIBILITY_LEVELS]>,
}

impl ImageCache {
    #[doc = include_str!("../../docs/api/runtime/image_cache.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self {
            images: HashMap::new(),
            entries: HashMap::new(),
            budget: CacheBudget::bytes(DEFAULT_MAX_BYTES)
                .with_max_entries(Some(DEFAULT_MAX_ENTRIES)),
            policy: EvictionPolicy::default(),
            stats: CacheStatistics::new(),
            tick: 0,
            protected_usage: OnceLock::new(),
        }
    }

    /// The decoded images, for compose to paint from.
    #[must_use]
    pub fn images(&self) -> &HashMap<Box<str>, SceneImage> {
        &self.images
    }

    /// The number of resident images.
    #[must_use]
    pub fn len(&self) -> usize {
        self.images.len()
    }

    /// Whether no image is resident.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.images.is_empty()
    }

    /// The live metrics (spec §5.19 `DevTools` metrics).
    #[must_use]
    pub fn statistics(&self) -> CacheStatistics {
        self.stats
    }

    /// The memory budget in force (spec §5.19).
    #[must_use]
    pub fn budget(&self) -> CacheBudget {
        self.budget
    }

    /// Installs `policy` as the eviction strategy (spec §5.19 eviction).
    pub fn set_policy(&mut self, policy: EvictionPolicy) {
        self.policy = policy;
    }

    #[doc = include_str!("../../docs/api/runtime/image_cache.doc/set_budget.md")]
    pub fn set_budget(&mut self, budget: CacheBudget) -> Vec<Box<str>> {
        self.budget = budget;
        let mut evicted = Vec::new();
        while self
            .budget
            .exceeds(self.stats.bytes(), self.stats.entries())
        {
            let Some(url) = self.evict_one(None) else {
                break;
            };
            evicted.push(url);
        }
        evicted
    }

    #[doc = include_str!("../../docs/api/runtime/image_cache.doc/record_lookup.md")]
    pub fn record_lookup(&mut self, url: &str) -> bool {
        self.tick = self.tick.saturating_add(1);
        if let Some(entry) = self.entries.get_mut(url) {
            entry.touch(self.tick);
            self.stats.record_hit();
            return true;
        }
        self.stats.record_miss();
        false
    }

    #[doc = include_str!("../../docs/api/runtime/image_cache.doc/mark_visibility.md")]
    pub fn mark_visibility(&mut self, referenced: &HashMap<Box<str>, VisibilityPriority>) {
        self.protected_usage.take();
        for (url, entry) in &mut self.entries {
            let priority = referenced
                .get(url)
                .copied()
                .unwrap_or(VisibilityPriority::Hidden);
            *entry = entry.with_visibility(priority);
        }
    }

    /// Allows a retry to use free capacity or displace only lower-priority images.
    pub(super) fn can_retry(&self, bytes: u64, priority: VisibilityPriority) -> bool {
        if self
            .budget
            .admits(self.stats.bytes(), self.stats.entries(), bytes)
        {
            return true;
        }
        if !self.budget.admits(0, 0, bytes) {
            return false;
        }
        let usage = self.protected_usage.get_or_init(|| {
            let mut usage = [(0u64, 0usize); VISIBILITY_LEVELS];
            for entry in self.entries.values() {
                let bucket = &mut usage[entry.visibility() as usize];
                bucket.0 += entry.byte_size();
                bucket.1 += 1;
            }
            for i in (0..VISIBILITY_LEVELS - 1).rev() {
                usage[i].0 += usage[i + 1].0;
                usage[i].1 += usage[i + 1].1;
            }
            usage
        });
        let (protected_bytes, protected_entries) = usage[priority as usize];
        self.budget
            .admits(protected_bytes, protected_entries, bytes)
    }

    /// Inserts runtime pixels with visibility known before their next frame.
    pub(super) fn insert_for_frame(
        &mut self,
        url: Box<str>,
        key: CacheKey,
        version: ResourceVersion,
        image: SceneImage,
        priority: VisibilityPriority,
        retry: bool,
    ) -> Vec<Box<str>> {
        if retry && !self.can_retry(image.pixels().len() as u64, priority) {
            return Vec::new();
        }
        let evicted =
            self.insert_inner(url.clone(), key, version, image, retry.then_some(priority));
        if let Some(entry) = self.entries.get_mut(&url) {
            *entry = entry.with_visibility(priority);
        }
        evicted
    }

    #[doc = include_str!("../../docs/api/runtime/image_cache.doc/insert.md")]
    pub fn insert(
        &mut self,
        url: Box<str>,
        key: CacheKey,
        version: ResourceVersion,
        image: SceneImage,
    ) -> Vec<Box<str>> {
        self.insert_inner(url, key, version, image, None)
    }

    /// Inserts a bitmap and optionally restricts eviction to lower-priority entries.
    fn insert_inner(
        &mut self,
        url: Box<str>,
        key: CacheKey,
        version: ResourceVersion,
        image: SceneImage,
        below: Option<VisibilityPriority>,
    ) -> Vec<Box<str>> {
        let bytes = image.pixels().len() as u64;
        if let Some(freed) = self.remove(&url) {
            self.stats.record_eviction(freed);
        }
        let mut evicted = Vec::new();
        if !self.budget.admits(0, 0, bytes) {
            return evicted;
        }
        while !self
            .budget
            .admits(self.stats.bytes(), self.stats.entries(), bytes)
        {
            let Some(victim) = self.evict_one(below) else {
                break;
            };
            evicted.push(victim);
        }
        self.tick = self.tick.saturating_add(1);
        let mut entry = CacheEntry::new(key, bytes, version);
        entry.touch(self.tick);
        self.protected_usage.take();
        self.entries.insert(url.clone(), entry);
        self.images.insert(url, image);
        self.stats.record_insertion(bytes);
        evicted
    }

    /// Drops every resident image and resets the metrics.
    pub fn clear(&mut self) {
        self.protected_usage.take();
        self.images.clear();
        self.entries.clear();
        self.stats.reset();
    }

    /// Evicts one entry by the installed [`EvictionPolicy`], returning its URL.
    fn evict_one(&mut self, below: Option<VisibilityPriority>) -> Option<Box<str>> {
        let candidates: Vec<CacheEntry> = self
            .entries
            .values()
            .copied()
            .filter(|entry| below.is_none_or(|priority| entry.visibility() < priority))
            .collect();
        let victim = self.policy.choose_victim(&candidates)?.key();
        // `choose_victim` borrows the candidate copies, so the URL comes back from
        // the bookkeeping by the identity it chose.
        let url = self
            .entries
            .iter()
            .find(|(_, entry)| entry.key() == victim)
            .map(|(url, _)| url.clone())?;
        let freed = self.remove(&url)?;
        self.stats.record_eviction(freed);
        Some(url)
    }

    /// Removes `url`'s image and bookkeeping, returning the bytes it freed —
    /// without touching the metrics, which the caller records.
    fn remove(&mut self, url: &str) -> Option<u64> {
        let entry = self.entries.remove(url)?;
        self.protected_usage.take();
        self.images.remove(url);
        Some(entry.byte_size())
    }
}

impl Default for ImageCache {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for ImageCache {
    /// Elides the pixel buffers and the per-URL bookkeeping: a decoded image is
    /// megabytes of RGBA8 that no debug line wants, and `stats` already carries the
    /// entry count and byte total the bookkeeping sums to.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImageCache")
            .field("len", &self.images.len())
            .field("budget", &self.budget)
            .field("policy", &self.policy)
            .field("stats", &self.stats)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_identity::ResourceId;
    use obscur_scene::color::AlphaMode;
    use std::sync::Arc;

    /// An `n × 1` opaque image: `n * 4` bytes.
    fn image(n: u32) -> SceneImage {
        let pixels: Arc<[u8]> = Arc::from(vec![255u8; (n as usize) * 4].into_boxed_slice());
        SceneImage::new(n, 1, pixels, AlphaMode::Premultiplied).expect("valid buffer")
    }

    fn key(raw: u64) -> CacheKey {
        CacheKey::for_resource(ResourceId::from_raw(raw))
    }

    fn insert(cache: &mut ImageCache, url: &str, raw: u64, px: u32) -> Vec<Box<str>> {
        cache.insert(url.into(), key(raw), ResourceVersion::initial(), image(px))
    }

    #[test]
    fn the_default_budget_is_bounded_and_still_seats_a_screenful() {
        // Unlimited is not a defensible default: nothing ever leaves, so a
        // long-running program holds every image it has ever shown. Nor is a cap
        // so tight that an ordinary screen thrashes — so the floor is stated in
        // the units that matter, full-screen 4K images and thumbnails.
        let budget = ImageCache::new().budget();
        let bytes = budget.max_bytes().expect("the default is bounded");
        let entries = budget.max_entries().expect("the default bounds the count");
        let screen_4k = 3840u64 * 2160 * 4;
        assert!(
            bytes >= 4 * screen_4k,
            "{bytes} bytes is under four full-screen 4K images"
        );
        // The two caps meet at a 256x256 RGBA thumbnail, so neither is decorative.
        let entries = u64::try_from(entries).expect("a plausible entry cap");
        assert_eq!(bytes / entries, 256 * 256 * 4);
    }

    #[test]
    fn unlimited_never_evicts() {
        let mut cache = ImageCache::new();
        cache.set_budget(CacheBudget::unlimited());
        for i in 0..64u64 {
            let evicted = insert(&mut cache, &format!("u{i}"), i, 16);
            assert!(evicted.is_empty());
        }
        assert_eq!(cache.len(), 64);
        assert_eq!(cache.statistics().evictions(), 0);
    }

    #[test]
    fn byte_budget_evicts_and_totals_stay_within_it() {
        // Room for exactly two 40-byte images.
        let mut cache = ImageCache::new();
        cache.set_budget(CacheBudget::bytes(80));
        cache.set_policy(EvictionPolicy::Lru);

        insert(&mut cache, "a", 1, 10);
        insert(&mut cache, "b", 2, 10);
        assert_eq!(cache.statistics().bytes(), 80);

        // `a` is the older touch, so it is the victim.
        let evicted = insert(&mut cache, "c", 3, 10);
        assert_eq!(evicted, vec!["a".into()]);
        assert!(cache.images().contains_key("b"));
        assert!(cache.images().contains_key("c"));
        assert_eq!(cache.statistics().bytes(), 80);
        assert_eq!(cache.statistics().evictions(), 1);
    }

    #[test]
    fn lookup_recency_changes_the_victim() {
        let mut cache = ImageCache::new();
        cache.set_budget(CacheBudget::bytes(80));
        cache.set_policy(EvictionPolicy::Lru);
        insert(&mut cache, "a", 1, 10);
        insert(&mut cache, "b", 2, 10);
        // Touching `a` makes `b` the least recent.
        assert!(cache.record_lookup("a"));
        let evicted = insert(&mut cache, "c", 3, 10);
        assert_eq!(evicted, vec!["b".into()]);
    }

    #[test]
    fn hidden_entries_are_evicted_before_visible_ones() {
        let mut cache = ImageCache::new();
        cache.set_budget(CacheBudget::bytes(80));
        // `a` is inserted first (least recent) but is still on screen; `b` is
        // referenced by nothing. VisibilityAware must take `b`.
        insert(&mut cache, "a", 1, 10);
        insert(&mut cache, "b", 2, 10);
        let referenced = HashMap::from([("a".into(), VisibilityPriority::Visible)]);
        cache.mark_visibility(&referenced);
        let evicted = insert(&mut cache, "c", 3, 10);
        assert_eq!(evicted, vec!["b".into()]);
        assert!(cache.images().contains_key("a"));
    }

    #[test]
    fn an_image_larger_than_the_whole_budget_is_not_admitted() {
        let mut cache = ImageCache::new();
        cache.set_budget(CacheBudget::bytes(40));
        insert(&mut cache, "a", 1, 10);
        // 80 bytes cannot fit a 40-byte budget: `a` must survive untouched.
        let evicted = insert(&mut cache, "big", 2, 20);
        assert!(evicted.is_empty());
        assert!(!cache.images().contains_key("big"));
        assert!(cache.images().contains_key("a"));
        assert_eq!(cache.statistics().bytes(), 40);
    }

    #[test]
    fn disabled_budget_admits_nothing() {
        let mut cache = ImageCache::new();
        cache.set_budget(CacheBudget::disabled());
        let evicted = insert(&mut cache, "a", 1, 10);
        assert!(evicted.is_empty());
        assert!(cache.images().is_empty());
        assert_eq!(cache.statistics().insertions(), 0);
    }

    #[test]
    fn tightening_the_budget_evicts_immediately() {
        let mut cache = ImageCache::new();
        insert(&mut cache, "a", 1, 10);
        insert(&mut cache, "b", 2, 10);
        insert(&mut cache, "c", 3, 10);
        let evicted = cache.set_budget(CacheBudget::bytes(40));
        assert_eq!(evicted.len(), 2);
        assert_eq!(cache.len(), 1);
        assert!(cache.statistics().bytes() <= 40);
    }

    #[test]
    fn entry_budget_bounds_the_count() {
        let mut cache = ImageCache::new();
        cache.set_budget(CacheBudget::entries(3));
        for i in 0..20u64 {
            insert(&mut cache, &format!("u{i}"), i, 8);
            assert!(cache.len() <= 3, "entry cap exceeded: {}", cache.len());
        }
        assert_eq!(cache.statistics().entries(), 3);
    }

    #[test]
    fn metrics_track_hits_and_misses() {
        let mut cache = ImageCache::new();
        assert!(!cache.record_lookup("a"));
        insert(&mut cache, "a", 1, 10);
        assert!(cache.record_lookup("a"));
        assert!(cache.record_lookup("a"));
        let stats = cache.statistics();
        assert_eq!(stats.hits(), 2);
        assert_eq!(stats.misses(), 1);
        assert!((stats.hit_rate() - 2.0 / 3.0).abs() < 1e-6);
    }

    #[test]
    fn clear_drops_everything_including_metrics() {
        let mut cache = ImageCache::new();
        insert(&mut cache, "a", 1, 10);
        cache.clear();
        assert_eq!(cache.len(), 0);
        assert_eq!(cache.statistics(), CacheStatistics::new());
    }
}
