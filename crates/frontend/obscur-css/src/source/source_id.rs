#![doc = include_str!("source_id.doc/_module.md")]

#[doc = include_str!("source_id.doc/SourceId.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SourceId(u32);

impl SourceId {
    #[doc = include_str!("source_id.doc/ANONYMOUS.md")]
    pub const ANONYMOUS: SourceId = SourceId::from_raw(0);

    #[doc = include_str!("source_id.doc/from_raw.md")]
    #[must_use]
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    #[doc = include_str!("source_id.doc/raw.md")]
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
