#![doc = include_str!("../../docs/api/virtualization/overscan_policy.doc/_module.md")]

#[doc = include_str!("../../docs/api/virtualization/overscan_policy.doc/OverscanPolicy.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OverscanPolicy {
    /// Extra items to materialize before the first visible item.
    leading: usize,
    /// Extra items to materialize after the last visible item.
    trailing: usize,
}

impl OverscanPolicy {
    /// Creates a policy with `leading` extra items before and `trailing` extra
    /// items after the visible range (spec §4.9).
    #[must_use]
    pub const fn new(leading: usize, trailing: usize) -> Self {
        Self { leading, trailing }
    }

    /// Creates a policy with the same `count` of extra items on both sides.
    #[must_use]
    pub const fn symmetric(count: usize) -> Self {
        Self::new(count, count)
    }

    /// A policy with no overscan on either side (`0`, `0`).
    #[must_use]
    pub const fn none() -> Self {
        Self::new(0, 0)
    }

    /// Extra items materialized before the first visible item.
    #[must_use]
    pub const fn leading(self) -> usize {
        self.leading
    }

    /// Extra items materialized after the last visible item.
    #[must_use]
    pub const fn trailing(self) -> usize {
        self.trailing
    }

    #[doc = include_str!("../../docs/api/virtualization/overscan_policy.doc/expand.md")]
    #[must_use]
    pub const fn expand(&self, start: usize, end: usize, item_count: usize) -> (usize, usize) {
        let expanded_start = start.saturating_sub(self.leading);
        let expanded_end = end.saturating_add(self.trailing);
        let clamped_end = if expanded_end > item_count {
            item_count
        } else {
            expanded_end
        };
        (expanded_start, clamped_end)
    }
}

impl Default for OverscanPolicy {
    /// The default policy has no overscan (spec §4.9): [`OverscanPolicy::none`].
    fn default() -> Self {
        Self::none()
    }
}

#[cfg(test)]
mod tests {
    use super::OverscanPolicy;

    #[test]
    fn symmetric_sets_both_sides() {
        let policy = OverscanPolicy::symmetric(3);
        assert_eq!(policy.leading(), 3);
        assert_eq!(policy.trailing(), 3);
    }

    #[test]
    fn none_and_default_are_zero() {
        assert_eq!(OverscanPolicy::none(), OverscanPolicy::new(0, 0));
        assert_eq!(OverscanPolicy::default(), OverscanPolicy::none());
    }

    #[test]
    fn expand_grows_range_within_bounds() {
        let policy = OverscanPolicy::new(2, 3);
        // Visible [5, 8) in a 100-item list expands by 2 leading, 3 trailing.
        assert_eq!(policy.expand(5, 8, 100), (3, 11));
    }

    #[test]
    fn expand_saturates_at_zero() {
        let policy = OverscanPolicy::new(10, 0);
        // Leading overscan cannot go below 0.
        assert_eq!(policy.expand(3, 6, 100), (0, 6));
    }

    #[test]
    fn expand_clamps_at_item_count() {
        let policy = OverscanPolicy::new(0, 10);
        // Trailing overscan cannot exceed item_count.
        assert_eq!(policy.expand(90, 98, 100), (90, 100));
    }

    #[test]
    fn expand_clamps_both_edges() {
        let policy = OverscanPolicy::symmetric(50);
        assert_eq!(policy.expand(1, 4, 5), (0, 5));
    }
}
