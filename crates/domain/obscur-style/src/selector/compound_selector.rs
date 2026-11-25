//! A compound selector: contiguous simple selectors ([`CompoundSelector`]).

use super::selector_specificity::SelectorSpecificity;
use super::simple_selector::SimpleSelector;

#[doc = include_str!("../../docs/api/selector/compound_selector.doc/CompoundSelector.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompoundSelector {
    simples: Vec<SimpleSelector>,
}

impl CompoundSelector {
    /// Creates a compound selector from its simple selectors.
    #[must_use]
    pub const fn new(simples: Vec<SimpleSelector>) -> Self {
        Self { simples }
    }

    /// Returns the simple selectors making up this compound.
    #[must_use]
    pub fn simples(&self) -> &[SimpleSelector] {
        &self.simples
    }

    /// Returns the specificity of this compound: the sum of its simples'
    /// contributions.
    #[must_use]
    pub fn specificity(&self) -> SelectorSpecificity {
        self.simples
            .iter()
            .fold(SelectorSpecificity::ZERO, |acc, s| {
                acc.add(s.specificity_contribution())
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accessor_returns_simples() {
        let compound = CompoundSelector::new(vec![
            SimpleSelector::Type("div".into()),
            SimpleSelector::Class("a".into()),
        ]);
        assert_eq!(compound.simples().len(), 2);
    }

    #[test]
    fn specificity_sums_contributions() {
        // div.a#b -> (1, 1, 1)
        let compound = CompoundSelector::new(vec![
            SimpleSelector::Type("div".into()),
            SimpleSelector::Class("a".into()),
            SimpleSelector::Id("b".into()),
        ]);
        assert_eq!(compound.specificity(), SelectorSpecificity::new(1, 1, 1));
    }

    #[test]
    fn empty_compound_has_zero_specificity() {
        assert_eq!(
            CompoundSelector::new(Vec::new()).specificity(),
            SelectorSpecificity::ZERO
        );
    }
}
