//! Active interaction/state pseudo-classes: the [`PseudoState`] bitset.

#[doc = include_str!("../../docs/api/selector/pseudo_state.doc/PseudoState.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PseudoState(u32);

impl PseudoState {
    /// The pointer is hovering the node (`:hover`).
    pub const HOVER: Self = Self(1 << 0);
    /// The node holds keyboard focus (`:focus`).
    pub const FOCUS: Self = Self(1 << 1);
    /// The node holds focus and the UA/subsystem deems it should be visibly
    /// indicated (`:focus-visible`).
    pub const FOCUS_VISIBLE: Self = Self(1 << 2);
    /// The node, or a descendant, holds focus (`:focus-within`).
    pub const FOCUS_WITHIN: Self = Self(1 << 3);
    /// The node is being activated, e.g. pressed (`:active`).
    pub const ACTIVE: Self = Self(1 << 4);
    /// The node is disabled and does not accept interaction (`:disabled`).
    pub const DISABLED: Self = Self(1 << 5);
    /// The node is a checked control, e.g. a ticked checkbox (`:checked`).
    pub const CHECKED: Self = Self(1 << 6);

    /// Returns the empty set (no pseudo-states active).
    #[must_use]
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Returns `true` if no pseudo-states are active.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Returns `true` if every flag in `other` is set in `self`.
    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// Returns a copy of `self` with the flags in `other` added.
    #[must_use]
    pub const fn insert(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Returns a copy of `self` with the flags in `other` cleared.
    #[must_use]
    pub const fn remove(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }

    /// Returns the union of `self` and `other` (all flags from either).
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    #[doc = include_str!("../../docs/api/selector/pseudo_state.doc/bits.md")]
    #[must_use]
    pub const fn bits(self) -> u32 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_is_empty() {
        assert!(PseudoState::empty().is_empty());
        assert!(!PseudoState::HOVER.is_empty());
    }

    #[test]
    fn insert_and_contains() {
        let s = PseudoState::empty()
            .insert(PseudoState::HOVER)
            .insert(PseudoState::FOCUS);
        assert!(s.contains(PseudoState::HOVER));
        assert!(s.contains(PseudoState::FOCUS));
        assert!(!s.contains(PseudoState::ACTIVE));
    }

    #[test]
    fn contains_requires_all_flags() {
        let s = PseudoState::HOVER;
        let both = PseudoState::HOVER.insert(PseudoState::FOCUS);
        assert!(!s.contains(both));
        assert!(both.contains(s));
    }

    #[test]
    fn remove_clears_flag() {
        let s = PseudoState::HOVER
            .insert(PseudoState::FOCUS)
            .remove(PseudoState::HOVER);
        assert!(!s.contains(PseudoState::HOVER));
        assert!(s.contains(PseudoState::FOCUS));
    }

    #[test]
    fn union_merges() {
        let a = PseudoState::HOVER;
        let b = PseudoState::CHECKED;
        let u = a.union(b);
        assert!(u.contains(PseudoState::HOVER));
        assert!(u.contains(PseudoState::CHECKED));
    }

    #[test]
    fn flags_are_distinct_bits() {
        let all = [
            PseudoState::HOVER,
            PseudoState::FOCUS,
            PseudoState::FOCUS_VISIBLE,
            PseudoState::FOCUS_WITHIN,
            PseudoState::ACTIVE,
            PseudoState::DISABLED,
            PseudoState::CHECKED,
        ];
        let mut acc = PseudoState::empty();
        for flag in all {
            assert!(!acc.contains(flag), "flag bit overlapped a previous one");
            acc = acc.insert(flag);
        }
        assert_eq!(acc.bits().count_ones() as usize, all.len());
    }
}
