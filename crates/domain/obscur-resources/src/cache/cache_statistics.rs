//! `DevTools` metrics for one cache (spec §5.19 metrics, correctness transparency).

/// Running metrics for one runtime cache (spec §5.19 *`DevTools` metrics; correctness
/// transparency*).
///
/// Counts hits, misses, evictions, and insertions, and tracks the live entry
/// count and total bytes. These figures surface a cache's behaviour to `DevTools`
/// and make its correctness transparent — for example, an unexpectedly low
/// [`hit_rate`](CacheStatistics::hit_rate) or a runaway eviction count is visible
/// rather than hidden.
///
/// This is pure bookkeeping: every counter uses saturating arithmetic, so it can
/// never overflow or underflow, and it holds no artifact data. All fields are
/// integers, so the type derives `Eq`, `Hash`, and `Default`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct CacheStatistics {
    /// Number of lookups that found a live entry.
    hits: u64,
    /// Number of lookups that found nothing.
    misses: u64,
    /// Number of entries evicted to stay within budget (spec §5.19 eviction).
    evictions: u64,
    /// Number of entries inserted over the cache's lifetime.
    insertions: u64,
    /// Current number of live entries.
    entries: usize,
    /// Current total bytes held.
    bytes: u64,
}

impl CacheStatistics {
    /// Creates a zeroed set of metrics.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            hits: 0,
            misses: 0,
            evictions: 0,
            insertions: 0,
            entries: 0,
            bytes: 0,
        }
    }

    /// Records a cache hit (spec §5.19 metrics).
    pub fn record_hit(&mut self) {
        self.hits = self.hits.saturating_add(1);
    }

    /// Records a cache miss (spec §5.19 metrics).
    pub fn record_miss(&mut self) {
        self.misses = self.misses.saturating_add(1);
    }

    /// Records the eviction of an entry that freed `freed_bytes`, decrementing
    /// the live entry count and byte total (spec §5.19 eviction).
    ///
    /// Uses saturating subtraction so the counts can never underflow.
    pub fn record_eviction(&mut self, freed_bytes: u64) {
        self.evictions = self.evictions.saturating_add(1);
        self.entries = self.entries.saturating_sub(1);
        self.bytes = self.bytes.saturating_sub(freed_bytes);
    }

    /// Records the insertion of an entry of `byte_size`, incrementing the live
    /// entry count and byte total (spec §5.19 metrics).
    pub fn record_insertion(&mut self, byte_size: u64) {
        self.insertions = self.insertions.saturating_add(1);
        self.entries = self.entries.saturating_add(1);
        self.bytes = self.bytes.saturating_add(byte_size);
    }

    /// Returns the hit rate `hits / (hits + misses)`, or `0.0` when there have
    /// been no accesses (spec §5.19 correctness transparency).
    #[must_use]
    pub fn hit_rate(&self) -> f32 {
        let accesses = self.hits.saturating_add(self.misses);
        if accesses == 0 {
            0.0
        } else {
            self.hits as f32 / accesses as f32
        }
    }

    /// Returns the number of recorded hits.
    #[must_use]
    pub const fn hits(&self) -> u64 {
        self.hits
    }

    /// Returns the number of recorded misses.
    #[must_use]
    pub const fn misses(&self) -> u64 {
        self.misses
    }

    /// Returns the number of recorded evictions.
    #[must_use]
    pub const fn evictions(&self) -> u64 {
        self.evictions
    }

    /// Returns the number of recorded insertions.
    #[must_use]
    pub const fn insertions(&self) -> u64 {
        self.insertions
    }

    /// Returns the current number of live entries.
    #[must_use]
    pub const fn entries(&self) -> usize {
        self.entries
    }

    /// Returns the current total bytes held.
    #[must_use]
    pub const fn bytes(&self) -> u64 {
        self.bytes
    }

    /// Resets every metric back to zero.
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_and_default_are_zeroed() {
        assert_eq!(CacheStatistics::new(), CacheStatistics::default());
        assert_eq!(CacheStatistics::new().hit_rate(), 0.0);
    }

    #[test]
    fn record_hits_and_misses() {
        let mut s = CacheStatistics::new();
        s.record_hit();
        s.record_hit();
        s.record_miss();
        assert_eq!(s.hits(), 2);
        assert_eq!(s.misses(), 1);
    }

    #[test]
    fn hit_rate_computes_fraction() {
        let mut s = CacheStatistics::new();
        s.record_hit();
        s.record_hit();
        s.record_hit();
        s.record_miss();
        assert_eq!(s.hit_rate(), 0.75);
    }

    #[test]
    fn insertion_and_eviction_track_size() {
        let mut s = CacheStatistics::new();
        s.record_insertion(100);
        s.record_insertion(50);
        assert_eq!(s.insertions(), 2);
        assert_eq!(s.entries(), 2);
        assert_eq!(s.bytes(), 150);

        s.record_eviction(100);
        assert_eq!(s.evictions(), 1);
        assert_eq!(s.entries(), 1);
        assert_eq!(s.bytes(), 50);
    }

    #[test]
    fn eviction_saturates_and_never_underflows() {
        let mut s = CacheStatistics::new();
        // Evict with nothing stored: counts saturate at zero.
        s.record_eviction(1_000);
        assert_eq!(s.evictions(), 1);
        assert_eq!(s.entries(), 0);
        assert_eq!(s.bytes(), 0);
    }

    #[test]
    fn reset_zeroes_everything() {
        let mut s = CacheStatistics::new();
        s.record_hit();
        s.record_insertion(10);
        s.reset();
        assert_eq!(s, CacheStatistics::new());
    }
}
