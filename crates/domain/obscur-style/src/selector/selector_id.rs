//! Opaque handle to a compiled selector: [`SelectorId`].

/// A stable, opaque identifier for a compiled selector.
///
/// Selectors are parsed and compiled by `obscur-css`; the concrete selector
/// syntax and the matching engine (combinator walking, `:nth-*`, attribute
/// tests, pseudo-state requirements) live there, not in this crate. The
/// cascade references selectors by this id — to order declarations or to look
/// up match results — without re-parsing or inspecting selector internals.
///
/// The backing integer is a handle minted by the compiler; it carries no
/// meaning outside that subsystem and must never be treated as a public
/// schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SelectorId(u64);

impl SelectorId {
    /// Wraps a raw handle value into a `SelectorId`.
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
    use super::*;

    #[test]
    fn round_trips_raw_value() {
        let id = SelectorId::from_raw(42);
        assert_eq!(id.raw(), 42);
    }

    #[test]
    fn equal_when_same_handle() {
        assert_eq!(SelectorId::from_raw(7), SelectorId::from_raw(7));
        assert_ne!(SelectorId::from_raw(7), SelectorId::from_raw(8));
    }

    #[test]
    fn orders_by_handle() {
        assert!(SelectorId::from_raw(1) < SelectorId::from_raw(2));
    }
}
