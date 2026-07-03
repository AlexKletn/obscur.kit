//! Opaque identifier for a layout context.

/// Opaque identifier for a [`LayoutContext`](super::LayoutContext) — an
/// invalidation boundary (spec §4.2, §4.3).
///
/// A layout context is anchored at a node that establishes a boundary; this id
/// names that context. It is a newtype over an integer handle whose numeric
/// value carries no meaning outside the subsystem that mints it and must never
/// be treated as a public schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LayoutContextId(u64);

impl LayoutContextId {
    /// Wraps a raw handle value into a `LayoutContextId`.
    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    /// Returns the raw handle value backing this identifier.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::LayoutContextId;

    #[test]
    fn round_trips_raw_value() {
        let id = LayoutContextId::from_raw(42);
        assert_eq!(id.raw(), 42);
    }

    #[test]
    fn equal_ids_compare_equal() {
        assert_eq!(LayoutContextId::from_raw(7), LayoutContextId::from_raw(7));
        assert_ne!(LayoutContextId::from_raw(7), LayoutContextId::from_raw(8));
    }

    #[test]
    fn orders_by_raw_value() {
        assert!(LayoutContextId::from_raw(1) < LayoutContextId::from_raw(2));
    }
}
