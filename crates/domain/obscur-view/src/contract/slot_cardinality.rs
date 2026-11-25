//! `SlotCardinality`: how many children a slot may hold (spec §2.11, §2.12).

#[doc = include_str!("../../docs/api/contract/slot_cardinality.doc/SlotCardinality.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotCardinality {
    /// The slot must contain exactly one child.
    ExactlyOne,
    /// The slot may contain zero or one child.
    Optional,
    /// The slot may contain any number of children, including none.
    Many,
}

impl SlotCardinality {
    /// Returns whether the given number of children satisfies this cardinality.
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
