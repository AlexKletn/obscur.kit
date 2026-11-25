//! Opaque handle to a compiled selector: [`SelectorId`].

#[doc = include_str!("../../docs/api/selector/selector_id.doc/SelectorId.md")]
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
