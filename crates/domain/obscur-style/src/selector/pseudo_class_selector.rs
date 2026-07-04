//! Structural and state pseudo-classes: [`PseudoClassSelector`] and [`AnB`].

use super::pseudo_state::PseudoState;

/// A single pseudo-class in a compound selector (`:hover`, `:nth-child(2n+1)`).
///
/// Pseudo-classes fall into two families:
///
/// - **State** pseudo-classes (`:hover`, `:focus`, …) match against the
///   interaction [`PseudoState`] bitset the element exposes. Each maps to the
///   bit it requires via [`required_state`](Self::required_state).
/// - **Structural** pseudo-classes (`:first-child`, `:nth-child(…)`, `:root`,
///   …) match against the element's position in the tree; they require no
///   interaction state and so return [`None`] from
///   [`required_state`](Self::required_state).
///
/// Any pseudo-class this crate does not model is preserved as
/// [`Other`](Self::Other) and matches nothing (a conservative, forward-safe
/// default), while still contributing class-level specificity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PseudoClassSelector {
    /// `:hover` — the pointer is over the element.
    Hover,
    /// `:focus` — the element holds keyboard focus.
    Focus,
    /// `:focus-visible` — focus that the UA deems should be indicated.
    FocusVisible,
    /// `:focus-within` — the element or a descendant holds focus.
    FocusWithin,
    /// `:active` — the element is being activated (pressed).
    Active,
    /// `:disabled` — the element is disabled.
    Disabled,
    /// `:enabled` — the element is not disabled.
    Enabled,
    /// `:checked` — the element is a checked control.
    Checked,
    /// `:first-child` — the element is the first element among its siblings.
    FirstChild,
    /// `:last-child` — the element is the last element among its siblings.
    LastChild,
    /// `:only-child` — the element is the sole element among its siblings.
    OnlyChild,
    /// `:root` — the element is the root of the document tree.
    Root,
    /// `:empty` — the element has no element children.
    Empty,
    /// `:nth-child(An+B)` — position among siblings, counting from the front.
    NthChild(AnB),
    /// `:nth-last-child(An+B)` — position among siblings, counting from the back.
    NthLastChild(AnB),
    /// `:nth-of-type(An+B)` — position among same-type siblings, from the front.
    NthOfType(AnB),
    /// `:nth-last-of-type(An+B)` — position among same-type siblings, from the back.
    NthLastOfType(AnB),
    /// An unmodeled pseudo-class, preserved by name. Matches nothing.
    Other(Box<str>),
}

impl PseudoClassSelector {
    /// Returns the interaction [`PseudoState`] bit this pseudo-class requires,
    /// or [`None`] for structural pseudo-classes (and for the negation-based
    /// [`Enabled`](Self::Enabled), which is evaluated as "not disabled" by the
    /// matcher rather than by a single required bit).
    #[must_use]
    pub const fn required_state(&self) -> Option<PseudoState> {
        match self {
            Self::Hover => Some(PseudoState::HOVER),
            Self::Focus => Some(PseudoState::FOCUS),
            Self::FocusVisible => Some(PseudoState::FOCUS_VISIBLE),
            Self::FocusWithin => Some(PseudoState::FOCUS_WITHIN),
            Self::Active => Some(PseudoState::ACTIVE),
            Self::Disabled => Some(PseudoState::DISABLED),
            Self::Checked => Some(PseudoState::CHECKED),
            Self::Enabled
            | Self::FirstChild
            | Self::LastChild
            | Self::OnlyChild
            | Self::Root
            | Self::Empty
            | Self::NthChild(_)
            | Self::NthLastChild(_)
            | Self::NthOfType(_)
            | Self::NthLastOfType(_)
            | Self::Other(_) => None,
        }
    }
}

/// The `An+B` argument of a `:nth-*` pseudo-class.
///
/// It represents the arithmetic sequence `a*n + b` for all integers `n >= 0`.
/// A one-based index matches when it equals one term of that sequence — see
/// [`matches_index`](Self::matches_index).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnB {
    /// The step (`a` in `An+B`).
    pub a: i32,
    /// The offset (`b` in `An+B`).
    pub b: i32,
}

impl AnB {
    /// Creates an `An+B` value from its step `a` and offset `b`.
    #[must_use]
    pub const fn new(a: i32, b: i32) -> Self {
        Self { a, b }
    }

    /// Returns `true` if `one_based_index` is a term of `a*n + b` for some
    /// integer `n >= 0`.
    ///
    /// Examples: `2n+1` matches `1, 3, 5, …`; `0n+3` (i.e. `a == 0`) matches
    /// only `3`; `-n+3` matches `1, 2, 3`.
    #[must_use]
    pub const fn matches_index(&self, one_based_index: i32) -> bool {
        let diff = one_based_index - self.b;
        if self.a == 0 {
            return diff == 0;
        }
        // `n = diff / a` must be a non-negative integer.
        diff % self.a == 0 && diff / self.a >= 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_pseudo_classes_map_to_bits() {
        assert_eq!(PseudoClassSelector::Hover.required_state(), Some(PseudoState::HOVER));
        assert_eq!(PseudoClassSelector::Focus.required_state(), Some(PseudoState::FOCUS));
        assert_eq!(
            PseudoClassSelector::Checked.required_state(),
            Some(PseudoState::CHECKED)
        );
    }

    #[test]
    fn structural_and_enabled_have_no_required_state() {
        assert_eq!(PseudoClassSelector::FirstChild.required_state(), None);
        assert_eq!(PseudoClassSelector::Root.required_state(), None);
        assert_eq!(PseudoClassSelector::Enabled.required_state(), None);
        assert_eq!(PseudoClassSelector::NthChild(AnB::new(2, 1)).required_state(), None);
    }

    #[test]
    fn anb_two_n_plus_one_matches_odd() {
        let anb = AnB::new(2, 1);
        assert!(anb.matches_index(1));
        assert!(anb.matches_index(3));
        assert!(anb.matches_index(5));
        assert!(!anb.matches_index(2));
        assert!(!anb.matches_index(4));
    }

    #[test]
    fn anb_constant_matches_only_that_index() {
        let anb = AnB::new(0, 3);
        assert!(anb.matches_index(3));
        assert!(!anb.matches_index(1));
        assert!(!anb.matches_index(6));
    }

    #[test]
    fn anb_zero_matches_nothing_for_one_based() {
        let anb = AnB::new(0, 0);
        assert!(!anb.matches_index(1));
        assert!(!anb.matches_index(2));
    }

    #[test]
    fn anb_negative_step_matches_leading_range() {
        let anb = AnB::new(-1, 3);
        assert!(anb.matches_index(1));
        assert!(anb.matches_index(2));
        assert!(anb.matches_index(3));
        assert!(!anb.matches_index(4));
    }

    #[test]
    fn anb_even_matches_two_n() {
        let anb = AnB::new(2, 0);
        assert!(anb.matches_index(2));
        assert!(anb.matches_index(4));
        assert!(!anb.matches_index(1));
        assert!(!anb.matches_index(3));
    }
}
