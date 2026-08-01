#![doc = include_str!("slot_cardinality.doc/_module.md")]

#[doc = include_str!("slot_cardinality.doc/SlotCardinality.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotCardinality {
    #[doc = include_str!("slot_cardinality.doc/ExactlyOne.md")]
    ExactlyOne,
    #[doc = include_str!("slot_cardinality.doc/Optional.md")]
    Optional,
    #[doc = include_str!("slot_cardinality.doc/Many.md")]
    Many,
}

impl SlotCardinality {
    #[doc = include_str!("slot_cardinality.doc/accepts.md")]
    #[must_use]
    pub const fn accepts(self, count: usize) -> bool {
        match self {
            Self::ExactlyOne => count == 1,
            Self::Optional => count <= 1,
            Self::Many => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exactly_one_requires_single_child() {
        assert!(!SlotCardinality::ExactlyOne.accepts(0));
        assert!(SlotCardinality::ExactlyOne.accepts(1));
        assert!(!SlotCardinality::ExactlyOne.accepts(2));
    }

    #[test]
    fn optional_and_many_bounds() {
        assert!(SlotCardinality::Optional.accepts(0));
        assert!(SlotCardinality::Optional.accepts(1));
        assert!(!SlotCardinality::Optional.accepts(2));
        assert!(SlotCardinality::Many.accepts(0));
        assert!(SlotCardinality::Many.accepts(42));
    }
}
