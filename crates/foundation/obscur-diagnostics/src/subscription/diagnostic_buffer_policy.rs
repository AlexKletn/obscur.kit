//! Buffering and backpressure configuration for a subscription.

use super::OverflowStrategy;

#[doc = include_str!("../../docs/api/subscription/diagnostic_buffer_policy.doc/DiagnosticBufferPolicy.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiagnosticBufferPolicy {
    /// Maximum number of buffered diagnostics. `usize::MAX` is treated as
    /// effectively unbounded.
    pub capacity: usize,
    /// Strategy applied when the buffer is at capacity.
    pub overflow: OverflowStrategy,
}

impl DiagnosticBufferPolicy {
    /// Creates a policy with the given `capacity` and `overflow` strategy.
    #[must_use]
    pub const fn new(capacity: usize, overflow: OverflowStrategy) -> Self {
        Self { capacity, overflow }
    }

    /// Creates a bounded policy of `capacity` entries that drops the oldest
    /// diagnostic on overflow.
    #[must_use]
    pub const fn bounded(capacity: usize) -> Self {
        Self::new(capacity, OverflowStrategy::DropOldest)
    }

    /// Creates an effectively unbounded policy (no backpressure). The overflow
    /// strategy is irrelevant since the capacity is never reached in practice.
    #[must_use]
    pub const fn unbounded() -> Self {
        Self::new(usize::MAX, OverflowStrategy::DropOldest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_defaults_to_drop_oldest() {
        let p = DiagnosticBufferPolicy::bounded(8);
        assert_eq!(p.capacity, 8);
        assert_eq!(p.overflow, OverflowStrategy::DropOldest);
    }

    #[test]
    fn unbounded_has_max_capacity() {
        let p = DiagnosticBufferPolicy::unbounded();
        assert_eq!(p.capacity, usize::MAX);
    }

    #[test]
    fn new_preserves_strategy() {
        let p = DiagnosticBufferPolicy::new(4, OverflowStrategy::Coalesce);
        assert_eq!(p.capacity, 4);
        assert_eq!(p.overflow, OverflowStrategy::Coalesce);
    }
}
