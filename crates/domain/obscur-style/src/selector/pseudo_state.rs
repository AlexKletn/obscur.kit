#![doc = include_str!("pseudo_state.doc/_module.md")]

#[doc = include_str!("pseudo_state.doc/PseudoState.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PseudoState(u32);

impl PseudoState {
    #[doc = include_str!("pseudo_state.doc/HOVER.md")]
    pub const HOVER: Self = Self(1 << 0);
    #[doc = include_str!("pseudo_state.doc/FOCUS.md")]
    pub const FOCUS: Self = Self(1 << 1);
    #[doc = include_str!("pseudo_state.doc/FOCUS_VISIBLE.md")]
    pub const FOCUS_VISIBLE: Self = Self(1 << 2);
    #[doc = include_str!("pseudo_state.doc/FOCUS_WITHIN.md")]
    pub const FOCUS_WITHIN: Self = Self(1 << 3);
    #[doc = include_str!("pseudo_state.doc/ACTIVE.md")]
    pub const ACTIVE: Self = Self(1 << 4);
    #[doc = include_str!("pseudo_state.doc/DISABLED.md")]
    pub const DISABLED: Self = Self(1 << 5);
    #[doc = include_str!("pseudo_state.doc/CHECKED.md")]
    pub const CHECKED: Self = Self(1 << 6);

    #[doc = include_str!("pseudo_state.doc/empty.md")]
    #[must_use]
    pub const fn empty() -> Self {
        Self(0)
    }

    #[doc = include_str!("pseudo_state.doc/is_empty.md")]
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    #[doc = include_str!("pseudo_state.doc/contains.md")]
    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    #[doc = include_str!("pseudo_state.doc/insert.md")]
    #[must_use]
    pub const fn insert(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    #[doc = include_str!("pseudo_state.doc/remove.md")]
    #[must_use]
    pub const fn remove(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }

    #[doc = include_str!("pseudo_state.doc/union.md")]
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    #[doc = include_str!("pseudo_state.doc/bits.md")]
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
