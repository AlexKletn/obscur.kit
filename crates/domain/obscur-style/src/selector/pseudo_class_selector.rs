//! Structural and state pseudo-classes: [`PseudoClassSelector`] and [`AnB`].

use super::complex_selector::ComplexSelector;
use super::pseudo_state::PseudoState;
use super::selector_list::SelectorList;
use super::selector_specificity::SelectorSpecificity;

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
    /// `:nth-child(An+B of S)` — position among the *inclusive siblings that
    /// match `S`*, counting from the front (selectors-4 §13.3.1).
    ///
    /// The element must itself match `S`; its index is then its position in the
    /// `S`-filtered sibling list, not in the full one. The plain
    /// [`NthChild`](Self::NthChild) form is kept separate rather than carrying an
    /// `Option`, so the (common) no-`of` case allocates nothing.
    NthChildOf(AnB, SelectorList),
    /// `:is(S)` — matches any element that matches any selector in `S`
    /// (selectors-4 §4.2).
    Is(SelectorList),
    /// `:where(S)` — matches exactly as [`Is`](Self::Is) does, but contributes
    /// **zero** specificity (selectors-4 §4.4).
    Where(SelectorList),
    /// `:not(S)` — matches any element that matches *no* selector in `S`
    /// (selectors-4 §4.3).
    Not(SelectorList),
    /// `:has(S)` — the relational pseudo-class (selectors-4 §4.5).
    ///
    /// Modelled for **specificity only**; the matcher answers `false` for it
    /// unconditionally. Deciding it requires navigating *downward/forward* from
    /// the subject, which [`MatchElement`](super::match_element::MatchElement)
    /// deliberately cannot do — see the matcher's note and
    /// `obscur-runtime`'s `RestyleScope`, whose invalidation bound is a proof
    /// resting on that navigation surface. `obscur-css` reports the selector as
    /// an unsupported feature so the rule is not a silent drop.
    Has(SelectorList),
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
            | Self::NthChildOf(..)
            | Self::Is(_)
            | Self::Where(_)
            | Self::Not(_)
            | Self::Has(_)
            | Self::Other(_) => None,
        }
    }

    /// Returns the specificity this pseudo-class contributes (selectors-4 §15).
    ///
    /// Most pseudo-classes count as one class-level selector, `(0, 1, 0)`. The
    /// four that "provide evaluation contexts for other selectors" are defined
    /// specially, and they genuinely differ:
    ///
    /// - `:is()`, `:not()` and `:has()` — "The specificity of an `:is()`,
    ///   `:not()`, or `:has()` pseudo-class is replaced by the specificity of the
    ///   most specific complex selector in its selector list argument." The
    ///   pseudo-class's own `(0, 1, 0)` is **replaced**, not added to, so
    ///   `:is(em, #foo)` is `(1,0,0)` and `:not(em, strong#foo)` is `(1,0,1)`.
    /// - `:where()` — "The specificity of a `:where()` pseudo-class is replaced
    ///   by zero"; neither it nor its arguments contribute.
    /// - `:nth-child(An+B of S)` — "Analogously, the specificity of an
    ///   `:nth-child()` or `:nth-last-child()` selector is the specificity of the
    ///   pseudo-class itself (counting as one pseudo-class selector) **plus** the
    ///   specificity of the most specific complex selector in its selector list
    ///   argument (if any)." Here the base *is* added, which is why
    ///   `S:nth-child(An+B)` and `:nth-child(An+B of S)` tie.
    ///
    /// An empty argument list has no most-specific member and so contributes
    /// nothing, leaving `:is()`/`:not()`/`:has()` at zero and
    /// `:nth-child(… of )` at the bare `(0, 1, 0)`.
    #[must_use]
    pub fn specificity_contribution(&self) -> SelectorSpecificity {
        /// One class-level selector: what an ordinary pseudo-class contributes.
        const BASE: SelectorSpecificity = SelectorSpecificity::new(0, 1, 0);
        match self {
            Self::Is(list) | Self::Not(list) | Self::Has(list) => most_specific(list),
            Self::Where(_) => SelectorSpecificity::ZERO,
            Self::NthChildOf(_, list) => BASE.add(most_specific(list)),
            _ => BASE,
        }
    }
}

/// Returns the specificity of the most specific complex selector in `list`, or
/// [`SelectorSpecificity::ZERO`] when the list is empty.
fn most_specific(list: &SelectorList) -> SelectorSpecificity {
    list.selectors()
        .iter()
        .map(ComplexSelector::specificity)
        .max()
        .unwrap_or(SelectorSpecificity::ZERO)
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
        assert_eq!(
            PseudoClassSelector::Hover.required_state(),
            Some(PseudoState::HOVER)
        );
        assert_eq!(
            PseudoClassSelector::Focus.required_state(),
            Some(PseudoState::FOCUS)
        );
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
        assert_eq!(
            PseudoClassSelector::NthChild(AnB::new(2, 1)).required_state(),
            None
        );
    }

    /// Builds a one-compound complex selector from `simples`.
    fn complex(simples: Vec<super::super::simple_selector::SimpleSelector>) -> ComplexSelector {
        ComplexSelector::new(
            super::super::compound_selector::CompoundSelector::new(simples),
            Vec::new(),
        )
    }

    /// `em, strong#foo` — the spec's own `:not()` example argument.
    fn em_and_strong_foo() -> SelectorList {
        use super::super::simple_selector::SimpleSelector as S;
        SelectorList::new(vec![
            complex(vec![S::Type("em".into())]),
            complex(vec![S::Type("strong".into()), S::Id("foo".into())]),
        ])
    }

    #[test]
    fn ordinary_pseudo_class_contributes_one_class() {
        assert_eq!(
            PseudoClassSelector::Hover.specificity_contribution(),
            SelectorSpecificity::new(0, 1, 0)
        );
        assert_eq!(
            PseudoClassSelector::NthChild(AnB::new(2, 1)).specificity_contribution(),
            SelectorSpecificity::new(0, 1, 0)
        );
    }

    /// selectors-4 §15: ":is(em, #foo) has a specificity of (1,0,0)".
    #[test]
    fn is_takes_the_most_specific_argument() {
        use super::super::simple_selector::SimpleSelector as S;
        let list = SelectorList::new(vec![
            complex(vec![S::Type("em".into())]),
            complex(vec![S::Id("foo".into())]),
        ]);
        assert_eq!(
            PseudoClassSelector::Is(list).specificity_contribution(),
            SelectorSpecificity::new(1, 0, 0)
        );
    }

    /// selectors-4 §15: ":not(em, strong#foo)" is (1,0,1) — the whole compound
    /// `strong#foo`, not just its id.
    #[test]
    fn not_takes_the_most_specific_argument() {
        assert_eq!(
            PseudoClassSelector::Not(em_and_strong_foo()).specificity_contribution(),
            SelectorSpecificity::new(1, 0, 1)
        );
    }

    /// selectors-4 §15: ":has()" scores like ":is()"/":not()" — even though the
    /// matcher cannot decide it.
    #[test]
    fn has_takes_the_most_specific_argument() {
        assert_eq!(
            PseudoClassSelector::Has(em_and_strong_foo()).specificity_contribution(),
            SelectorSpecificity::new(1, 0, 1)
        );
    }

    /// selectors-4 §15: "The specificity of a :where() pseudo-class is replaced
    /// by zero" — the arguments contribute nothing either.
    #[test]
    fn where_contributes_zero_however_specific_its_argument() {
        assert_eq!(
            PseudoClassSelector::Where(em_and_strong_foo()).specificity_contribution(),
            SelectorSpecificity::ZERO
        );
    }

    /// selectors-4 §13.3.1: "S:nth-child(An+B) and :nth-child(An+B of S) have
    /// the exact same specificity" — the pseudo-class's own (0,1,0) *plus* S.
    #[test]
    fn nth_child_of_s_adds_its_argument_to_the_base() {
        use super::super::simple_selector::SimpleSelector as S;
        let list = SelectorList::new(vec![complex(vec![
            S::Type("li".into()),
            S::Class("important".into()),
        ])]);
        // (0,1,0) + (0,1,1) == (0,2,1), which is exactly what
        // `li.important:nth-child(-n+3)` sums to.
        assert_eq!(
            PseudoClassSelector::NthChildOf(AnB::new(-1, 3), list).specificity_contribution(),
            SelectorSpecificity::new(0, 2, 1)
        );
    }

    #[test]
    fn empty_argument_lists_contribute_nothing_extra() {
        assert_eq!(
            PseudoClassSelector::Is(SelectorList::default()).specificity_contribution(),
            SelectorSpecificity::ZERO
        );
        assert_eq!(
            PseudoClassSelector::NthChildOf(AnB::new(0, 1), SelectorList::default())
                .specificity_contribution(),
            SelectorSpecificity::new(0, 1, 0)
        );
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
