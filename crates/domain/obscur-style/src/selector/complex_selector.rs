//! A complex selector: compounds joined by combinators ([`ComplexSelector`]).

use super::compound_selector::CompoundSelector;
use super::selector_specificity::SelectorSpecificity;

use super::Combinator;

#[doc = include_str!("../../docs/api/selector/complex_selector.doc/ComplexSelector.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComplexSelector {
    head: CompoundSelector,
    tail: Vec<(Combinator, CompoundSelector)>,
}

impl ComplexSelector {
    /// Creates a complex selector from its lead compound and combinator-linked
    /// tail (both stored left-to-right).
    #[must_use]
    pub const fn new(head: CompoundSelector, tail: Vec<(Combinator, CompoundSelector)>) -> Self {
        Self { head, tail }
    }

    /// Returns the lead (leftmost) compound selector.
    #[must_use]
    pub const fn head(&self) -> &CompoundSelector {
        &self.head
    }

    /// Returns the combinator-linked tail compounds, left-to-right.
    #[must_use]
    pub fn tail(&self) -> &[(Combinator, CompoundSelector)] {
        &self.tail
    }

    /// Returns the subject compound: the rightmost compound, which the tested
    /// element itself must match.
    #[must_use]
    pub fn subject(&self) -> &CompoundSelector {
        self.tail.last().map_or(&self.head, |(_, c)| c)
    }

    /// Returns the specificity of this complex selector: the sum over all its
    /// compounds (combinators contribute nothing).
    #[must_use]
    pub fn specificity(&self) -> SelectorSpecificity {
        self.tail
            .iter()
            .fold(self.head.specificity(), |acc, (_, c)| {
                acc.add(c.specificity())
            })
    }
}

#[cfg(test)]
mod tests {
    use super::super::simple_selector::SimpleSelector;
    use super::*;

    fn compound(name: &str) -> CompoundSelector {
        CompoundSelector::new(vec![SimpleSelector::Type(name.into())])
    }

    #[test]
    fn subject_is_head_when_tail_empty() {
        let sel = ComplexSelector::new(compound("div"), Vec::new());
        assert_eq!(sel.subject(), &compound("div"));
    }

    #[test]
    fn subject_is_rightmost_compound() {
        let sel =
            ComplexSelector::new(compound("div"), vec![(Combinator::Child, compound("span"))]);
        assert_eq!(sel.subject(), &compound("span"));
        assert_eq!(sel.head(), &compound("div"));
        assert_eq!(sel.tail().len(), 1);
    }

    #[test]
    fn specificity_sums_all_compounds() {
        // div.a > #b -> (0,0,1) + (0,1,1)... build explicitly.
        let head = CompoundSelector::new(vec![
            SimpleSelector::Type("div".into()),
            SimpleSelector::Class("a".into()),
        ]);
        let subject = CompoundSelector::new(vec![SimpleSelector::Id("b".into())]);
        let sel = ComplexSelector::new(head, vec![(Combinator::Child, subject)]);
        // (0,1,1) + (1,0,0) = (1,1,1)
        assert_eq!(sel.specificity(), SelectorSpecificity::new(1, 1, 1));
    }
}
