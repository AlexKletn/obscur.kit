#![doc = include_str!("diagnostic_buffer_policy.doc/_module.md")]

#[doc = include_str!("diagnostic_buffer_policy.doc/OverflowStrategy.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverflowStrategy {
    #[doc = include_str!("diagnostic_buffer_policy.doc/DropOldest.md")]
    DropOldest,
    #[doc = include_str!("diagnostic_buffer_policy.doc/DropNewest.md")]
    DropNewest,
    #[doc = include_str!("diagnostic_buffer_policy.doc/Coalesce.md")]
    Coalesce,
}

#[doc = include_str!("diagnostic_buffer_policy.doc/DiagnosticBufferPolicy.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiagnosticBufferPolicy {
    #[doc = include_str!("diagnostic_buffer_policy.doc/capacity.md")]
    pub capacity: usize,
    #[doc = include_str!("diagnostic_buffer_policy.doc/overflow.md")]
    pub overflow: OverflowStrategy,
}

impl DiagnosticBufferPolicy {
    #[doc = include_str!("diagnostic_buffer_policy.doc/new.md")]
    #[must_use]
    pub const fn new(capacity: usize, overflow: OverflowStrategy) -> Self {
        Self { capacity, overflow }
    }

    #[doc = include_str!("diagnostic_buffer_policy.doc/bounded.md")]
    #[must_use]
    pub const fn bounded(capacity: usize) -> Self {
        Self::new(capacity, OverflowStrategy::DropOldest)
    }

    #[doc = include_str!("diagnostic_buffer_policy.doc/unbounded.md")]
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
