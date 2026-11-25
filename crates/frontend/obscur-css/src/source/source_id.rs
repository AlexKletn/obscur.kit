//! A stable identifier for a CSS source (stylesheet) (spec §2.5 source maps).

#[doc = include_str!("../../docs/api/source/source_id.doc/SourceId.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SourceId(u32);

impl SourceId {
    /// The id reserved for anonymous sources with no stable identity.
    pub const ANONYMOUS: SourceId = SourceId::from_raw(0);

    /// Creates a `SourceId` from its raw underlying value.
    #[must_use]
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    /// Returns the raw underlying value of this id.
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_raw_and_raw_roundtrip() {
        let id = SourceId::from_raw(42);
        assert_eq!(id.raw(), 42);
        assert_eq!(id, SourceId::from_raw(42));
        assert_ne!(id, SourceId::from_raw(7));
    }

    #[test]
    fn anonymous_is_zero() {
        assert_eq!(SourceId::ANONYMOUS.raw(), 0);
        assert_eq!(SourceId::ANONYMOUS, SourceId::from_raw(0));
    }

    #[test]
    fn is_ordered() {
        assert!(SourceId::from_raw(1) < SourceId::from_raw(2));
    }
}
