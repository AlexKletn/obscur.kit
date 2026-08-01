#![doc = include_str!("cache_statistics.doc/_module.md")]

#[doc = include_str!("cache_statistics.doc/CacheStatistics.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct CacheStatistics {
    #[doc = include_str!("cache_statistics.doc/hits.md")]
    hits: u64,
    #[doc = include_str!("cache_statistics.doc/misses.md")]
    misses: u64,
    #[doc = include_str!("cache_statistics.doc/evictions.md")]
    evictions: u64,
    #[doc = include_str!("cache_statistics.doc/insertions.md")]
    insertions: u64,
    #[doc = include_str!("cache_statistics.doc/entries.md")]
    entries: usize,
    #[doc = include_str!("cache_statistics.doc/bytes.md")]
    bytes: u64,
}

impl CacheStatistics {
    #[doc = include_str!("cache_statistics.doc/new.md")]
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

    #[doc = include_str!("cache_statistics.doc/record_hit.md")]
    pub fn record_hit(&mut self) {
        self.hits = self.hits.saturating_add(1);
    }

    #[doc = include_str!("cache_statistics.doc/record_miss.md")]
    pub fn record_miss(&mut self) {
        self.misses = self.misses.saturating_add(1);
    }

    #[doc = include_str!("cache_statistics.doc/record_eviction.md")]
    pub fn record_eviction(&mut self, freed_bytes: u64) {
        self.evictions = self.evictions.saturating_add(1);
        self.entries = self.entries.saturating_sub(1);
        self.bytes = self.bytes.saturating_sub(freed_bytes);
    }

    #[doc = include_str!("cache_statistics.doc/record_insertion.md")]
    pub fn record_insertion(&mut self, byte_size: u64) {
        self.insertions = self.insertions.saturating_add(1);
        self.entries = self.entries.saturating_add(1);
        self.bytes = self.bytes.saturating_add(byte_size);
    }

    #[doc = include_str!("cache_statistics.doc/hit_rate.md")]
    #[must_use]
    pub fn hit_rate(&self) -> f32 {
        let accesses = self.hits.saturating_add(self.misses);
        if accesses == 0 {
            0.0
        } else {
            self.hits as f32 / accesses as f32
        }
    }

    #[doc = include_str!("cache_statistics.doc/hits_2.md")]
    #[must_use]
    pub const fn hits(&self) -> u64 {
        self.hits
    }

    #[doc = include_str!("cache_statistics.doc/misses_2.md")]
    #[must_use]
    pub const fn misses(&self) -> u64 {
        self.misses
    }

    #[doc = include_str!("cache_statistics.doc/evictions_2.md")]
    #[must_use]
    pub const fn evictions(&self) -> u64 {
        self.evictions
    }

    #[doc = include_str!("cache_statistics.doc/insertions_2.md")]
    #[must_use]
    pub const fn insertions(&self) -> u64 {
        self.insertions
    }

    #[doc = include_str!("cache_statistics.doc/entries_2.md")]
    #[must_use]
    pub const fn entries(&self) -> usize {
        self.entries
    }

    #[doc = include_str!("cache_statistics.doc/bytes_2.md")]
    #[must_use]
    pub const fn bytes(&self) -> u64 {
        self.bytes
    }

    #[doc = include_str!("cache_statistics.doc/reset.md")]
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
