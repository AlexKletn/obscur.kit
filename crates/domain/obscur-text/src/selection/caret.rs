//! A text insertion point: a byte offset plus wrap-boundary affinity (spec §5.11).

use super::CaretAffinity;

#[doc = include_str!("../../docs/api/selection/caret.doc/Caret.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Caret {
    /// The byte offset of the insertion point into the buffer.
    offset: usize,
    /// The wrap-boundary affinity of the offset.
    affinity: CaretAffinity,
}

impl Caret {
    /// Creates a caret at `offset` with the given `affinity`.
    #[must_use]
    pub const fn new(offset: usize, affinity: CaretAffinity) -> Self {
        Self { offset, affinity }
    }

    /// Creates a caret at `offset` with the default
    /// [`Downstream`](CaretAffinity::Downstream) affinity.
    #[must_use]
    pub const fn at(offset: usize) -> Self {
        Self::new(offset, CaretAffinity::Downstream)
    }

    /// Returns the byte offset of the insertion point.
    #[must_use]
    pub const fn offset(&self) -> usize {
        self.offset
    }

    /// Returns the wrap-boundary affinity of the offset.
    #[must_use]
    pub const fn affinity(&self) -> CaretAffinity {
        self.affinity
    }

    /// Returns a copy of this caret with `affinity` replaced, keeping the offset.
    #[must_use]
    pub const fn with_affinity(&self, affinity: CaretAffinity) -> Self {
        Self::new(self.offset, affinity)
    }

    /// Returns a copy of this caret with `offset` replaced, keeping the affinity.
    #[must_use]
    pub const fn with_offset(&self, offset: usize) -> Self {
        Self::new(offset, self.affinity)
    }

    /// Returns this caret moved by `delta` bytes, saturating at the bounds of
    /// `usize` so the offset never overflows or underflows. The affinity is
    /// preserved.
    #[must_use]
    pub const fn shifted(&self, delta: isize) -> Self {
        Self::new(self.offset.saturating_add_signed(delta), self.affinity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn at_uses_downstream_default() {
        let caret = Caret::at(5);
        assert_eq!(caret.offset(), 5);
        assert_eq!(caret.affinity(), CaretAffinity::Downstream);
    }

    #[test]
    fn with_setters_replace_one_field() {
        let caret = Caret::at(5);
        assert_eq!(
            caret.with_affinity(CaretAffinity::Upstream),
            Caret::new(5, CaretAffinity::Upstream)
        );
        assert_eq!(
            caret.with_offset(9),
            Caret::new(9, CaretAffinity::Downstream)
        );
    }

    #[test]
    fn shifted_saturates_and_keeps_affinity() {
        let caret = Caret::new(3, CaretAffinity::Upstream);
        assert_eq!(caret.shifted(2), Caret::new(5, CaretAffinity::Upstream));
        assert_eq!(caret.shifted(-2), Caret::new(1, CaretAffinity::Upstream));
        // Underflow saturates to zero, affinity unchanged.
        assert_eq!(caret.shifted(-10), Caret::new(0, CaretAffinity::Upstream));
        // Overflow saturates to usize::MAX.
        assert_eq!(Caret::at(usize::MAX - 1).shifted(5), Caret::at(usize::MAX));
    }

    #[test]
    fn ordering_is_offset_then_affinity() {
        // Lower offset sorts first regardless of affinity.
        assert!(Caret::new(2, CaretAffinity::Downstream) < Caret::new(3, CaretAffinity::Upstream));
        // Equal offset: Upstream sorts before Downstream (declaration order).
        assert!(Caret::new(2, CaretAffinity::Upstream) < Caret::new(2, CaretAffinity::Downstream));
    }
}
