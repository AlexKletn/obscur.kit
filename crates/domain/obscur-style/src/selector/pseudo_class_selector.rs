#![doc = include_str!("pseudo_class_selector.doc/_module.md")]

use super::complex_selector::ComplexSelector;
use super::pseudo_state::PseudoState;
use super::selector_list::SelectorList;
use super::selector_specificity::SelectorSpecificity;

#[doc = include_str!("pseudo_class_selector.doc/PseudoClassSelector.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PseudoClassSelector {
    #[doc = include_str!("pseudo_class_selector.doc/Hover.md")]
    Hover,
    #[doc = include_str!("pseudo_class_selector.doc/Focus.md")]
    Focus,
    #[doc = include_str!("pseudo_class_selector.doc/FocusVisible.md")]
    FocusVisible,
    #[doc = include_str!("pseudo_class_selector.doc/FocusWithin.md")]
    FocusWithin,
    #[doc = include_str!("pseudo_class_selector.doc/Active.md")]
    Active,
    #[doc = include_str!("pseudo_class_selector.doc/Disabled.md")]
    Disabled,
    #[doc = include_str!("pseudo_class_selector.doc/Enabled.md")]
    Enabled,
    #[doc = include_str!("pseudo_class_selector.doc/Checked.md")]
    Checked,
    #[doc = include_str!("pseudo_class_selector.doc/FirstChild.md")]
    FirstChild,
    #[doc = include_str!("pseudo_class_selector.doc/LastChild.md")]
    LastChild,
    #[doc = include_str!("pseudo_class_selector.doc/OnlyChild.md")]
    OnlyChild,
    #[doc = include_str!("pseudo_class_selector.doc/Root.md")]
    Root,
    #[doc = include_str!("pseudo_class_selector.doc/Empty.md")]
    Empty,
    #[doc = include_str!("pseudo_class_selector.doc/NthChild.md")]
    NthChild(AnB),
    #[doc = include_str!("pseudo_class_selector.doc/NthLastChild.md")]
    NthLastChild(AnB),
    #[doc = include_str!("pseudo_class_selector.doc/NthOfType.md")]
    NthOfType(AnB),
    #[doc = include_str!("pseudo_class_selector.doc/NthLastOfType.md")]
    NthLastOfType(AnB),
    #[doc = include_str!("pseudo_class_selector.doc/NthChildOf.md")]
    NthChildOf(AnB, SelectorList),
    #[doc = include_str!("pseudo_class_selector.doc/Is.md")]
    Is(SelectorList),
    #[doc = include_str!("pseudo_class_selector.doc/Where.md")]
    Where(SelectorList),
    #[doc = include_str!("pseudo_class_selector.doc/Not.md")]
    Not(SelectorList),
    #[doc = include_str!("pseudo_class_selector.doc/Has.md")]
    Has(SelectorList),
    #[doc = include_str!("pseudo_class_selector.doc/Other.md")]
    Other(Box<str>),
}

impl PseudoClassSelector {
    #[doc = include_str!("pseudo_class_selector.doc/required_state.md")]
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

    #[doc = include_str!("pseudo_class_selector.doc/specificity_contribution.md")]
    #[must_use]
    pub fn specificity_contribution(&self) -> SelectorSpecificity {
        #[doc = include_str!("pseudo_class_selector.doc/BASE.md")]
        const BASE: SelectorSpecificity = SelectorSpecificity::new(0, 1, 0);
        match self {
            Self::Is(list) | Self::Not(list) | Self::Has(list) => most_specific(list),
            Self::Where(_) => SelectorSpecificity::ZERO,
            Self::NthChildOf(_, list) => BASE.add(most_specific(list)),
            _ => BASE,
        }
    }
}

#[doc = include_str!("pseudo_class_selector.doc/most_specific.md")]
fn most_specific(list: &SelectorList) -> SelectorSpecificity {
    list.selectors()
        .iter()
        .map(ComplexSelector::specificity)
        .max()
        .unwrap_or(SelectorSpecificity::ZERO)
}

#[doc = include_str!("pseudo_class_selector.doc/AnB.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnB {
    #[doc = include_str!("pseudo_class_selector.doc/a.md")]
    pub a: i32,
    #[doc = include_str!("pseudo_class_selector.doc/b.md")]
    pub b: i32,
}

impl AnB {
    #[doc = include_str!("pseudo_class_selector.doc/new.md")]
    #[must_use]
    pub const fn new(a: i32, b: i32) -> Self {
        Self { a, b }
    }

    #[doc = include_str!("pseudo_class_selector.doc/matches_index.md")]
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
