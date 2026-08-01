#![doc = include_str!("compound_selector.doc/_module.md")]

use super::selector_specificity::SelectorSpecificity;
use super::simple_selector::SimpleSelector;

#[doc = include_str!("compound_selector.doc/CompoundSelector.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompoundSelector {
    simples: Vec<SimpleSelector>,
}

impl CompoundSelector {
    #[doc = include_str!("compound_selector.doc/new.md")]
    #[must_use]
    pub const fn new(simples: Vec<SimpleSelector>) -> Self {
        Self { simples }
    }

    #[doc = include_str!("compound_selector.doc/simples.md")]
    #[must_use]
    pub fn simples(&self) -> &[SimpleSelector] {
        &self.simples
    }

    #[doc = include_str!("compound_selector.doc/specificity.md")]
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
