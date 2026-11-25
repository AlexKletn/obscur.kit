//! Memory budget and disable switch for one cache (spec §5.19 caches).

#[doc = include_str!("../../docs/api/cache/cache_budget.doc/CacheBudget.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CacheBudget {
    /// Maximum total bytes, or `None` for unlimited (spec §5.19 memory budgets).
    max_bytes: Option<u64>,
    /// Maximum total entry count, or `None` for unlimited.
    max_entries: Option<usize>,
    /// Whether the cache may store anything at all (spec §5.19 disable switch).
    enabled: bool,
}

impl CacheBudget {
    /// An enabled, unbounded budget: no byte or entry limit.
    #[must_use]
    pub const fn unlimited() -> Self {
        Self {
            max_bytes: None,
            max_entries: None,
            enabled: true,
        }
    }

    /// An enabled budget capped at `max_bytes` total bytes (spec §5.19).
    #[must_use]
    pub const fn bytes(max_bytes: u64) -> Self {
        Self {
            max_bytes: Some(max_bytes),
            max_entries: None,
            enabled: true,
        }
    }

    /// An enabled budget capped at `max_entries` total entries.
    #[must_use]
    pub const fn entries(max_entries: usize) -> Self {
        Self {
            max_bytes: None,
            max_entries: Some(max_entries),
            enabled: true,
        }
    }

    /// A disabled budget: nothing may be stored (spec §5.19 per-cache disable
    /// switch for tests).
    #[must_use]
    pub const fn disabled() -> Self {
        Self {
            max_bytes: None,
            max_entries: None,
            enabled: false,
        }
    }

    /// Returns this budget with its byte cap set (`None` = unlimited).
    #[must_use]
    pub const fn with_max_bytes(mut self, max_bytes: Option<u64>) -> Self {
        self.max_bytes = max_bytes;
        self
    }

    /// Returns this budget with its entry cap set (`None` = unlimited).
    #[must_use]
    pub const fn with_max_entries(mut self, max_entries: Option<usize>) -> Self {
        self.max_entries = max_entries;
        self
    }

    /// Returns this budget with its enabled flag set (spec §5.19 disable switch).
    #[must_use]
    pub const fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Returns whether the cache is enabled (spec §5.19 disable switch).
    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Returns the byte cap, or `None` if unlimited.
    #[must_use]
    pub const fn max_bytes(&self) -> Option<u64> {
        self.max_bytes
    }

    /// Returns the entry cap, or `None` if unlimited.
    #[must_use]
    pub const fn max_entries(&self) -> Option<usize> {
        self.max_entries
    }

    #[doc = include_str!("../../docs/api/cache/cache_budget.doc/admits.md")]
    #[must_use]
    pub const fn admits(
        &self,
        current_bytes: u64,
        current_entries: usize,
        incoming_bytes: u64,
    ) -> bool {
        if !self.enabled {
            return false;
        }
        if let Some(max) = self.max_bytes
            && current_bytes.saturating_add(incoming_bytes) > max
        {
            return false;
        }
        if let Some(max) = self.max_entries
            && current_entries.saturating_add(1) > max
        {
            return false;
        }
        true
    }

    #[doc = include_str!("../../docs/api/cache/cache_budget.doc/exceeds.md")]
    #[must_use]
    pub const fn exceeds(&self, current_bytes: u64, current_entries: usize) -> bool {
        if let Some(max) = self.max_bytes
            && current_bytes > max
        {
            return true;
        }
        if let Some(max) = self.max_entries
            && current_entries > max
        {
            return true;
        }
        false
    }
}

impl Default for CacheBudget {
    /// The default budget is [`CacheBudget::unlimited`].
    fn default() -> Self {
        Self::unlimited()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unlimited_admits_anything() {
        let b = CacheBudget::unlimited();
        assert!(b.is_enabled());
        assert!(b.admits(u64::MAX - 1, usize::MAX - 1, 1));
        assert!(!b.exceeds(u64::MAX, usize::MAX));
    }

    #[test]
    fn disabled_never_admits() {
        let b = CacheBudget::disabled();
        assert!(!b.is_enabled());
        assert!(!b.admits(0, 0, 0));
        assert!(!b.admits(0, 0, 1));
    }

    #[test]
    fn max_bytes_boundary() {
        let b = CacheBudget::bytes(100);
        // Exactly filling the cap is admitted.
        assert!(b.admits(60, 0, 40));
        // One byte over is rejected.
        assert!(!b.admits(60, 0, 41));
        // Saturating add cannot overflow past the cap.
        assert!(!b.admits(u64::MAX, 0, 1));
    }

    #[test]
    fn max_entries_boundary() {
        let b = CacheBudget::entries(2);
        // Filling to the cap is admitted.
        assert!(b.admits(0, 1, 0));
        // Adding a third entry is rejected.
        assert!(!b.admits(0, 2, 0));
    }

    #[test]
    fn exceeds_detects_over_budget() {
        let b = CacheBudget::bytes(100).with_max_entries(Some(3));
        assert!(!b.exceeds(100, 3));
        assert!(b.exceeds(101, 0));
        assert!(b.exceeds(0, 4));
    }

    #[test]
    fn fluent_setters_compose() {
        let b = CacheBudget::unlimited()
            .with_max_bytes(Some(10))
            .with_max_entries(Some(1))
            .with_enabled(false);
        assert_eq!(b.max_bytes(), Some(10));
        assert_eq!(b.max_entries(), Some(1));
        assert!(!b.is_enabled());
        assert!(!b.admits(0, 0, 1));
    }

    #[test]
    fn default_is_unlimited() {
        assert_eq!(CacheBudget::default(), CacheBudget::unlimited());
    }
}
