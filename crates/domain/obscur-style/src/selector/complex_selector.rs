//! A complex selector: compounds joined by combinators ([`ComplexSelector`]).

use super::compound_selector::CompoundSelector;
use super::selector_specificity::SelectorSpecificity;

/// A combinator linking two compound selectors in a complex selector.
///
/// Each combinator constrains the relationship (read left-to-right) between the
/// compound *before* it and the compound *after* it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Combinator {
    /// Descendant combinator (whitespace): the right compound is a descendant
    /// of the left.
    Descendant,
    /// Child combinator (`>`): the right compound is a direct child of the left.
    Child,
    /// Next-sibling combinator (`+`): the right compound immediately follows the
    /// left among element siblings.
    NextSibling,
    /// Subsequent-sibling combinator (`~`): the right compound follows the left
    /// among element siblings (not necessarily immediately).
    SubsequentSibling,
}

/// A complex selector — a lead compound followed by combinator-linked compounds
/// (`div > .a + span`).
///
/// The compounds are stored left-to-right: `head` is the leftmost compound and
/// each `tail` entry pairs the combinator that *precedes* a compound with that
/// compound. The **rightmost** compound (the last `tail` entry, or `head` when
/// `tail` is empty) is the **subject**: the element being tested must match the
/// subject, while the earlier compounds constrain its ancestors/siblings. This
/// is why the matcher evaluates right-to-left, starting from the subject.
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
