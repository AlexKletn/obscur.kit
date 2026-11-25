//! A comma-separated list of complex selectors: [`SelectorList`].

use super::complex_selector::ComplexSelector;

#[doc = include_str!("../../docs/api/selector/selector_list.doc/SelectorList.md")]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SelectorList(Vec<ComplexSelector>);

impl SelectorList {
    /// Creates a selector list from its complex selectors.
    #[must_use]
    pub const fn new(selectors: Vec<ComplexSelector>) -> Self {
        Self(selectors)
    }

    /// Returns the complex selectors in this list.
    #[must_use]
    pub fn selectors(&self) -> &[ComplexSelector] {
        &self.0
    }

    /// Returns `true` if the list contains no selectors.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::super::compound_selector::CompoundSelector;
    use super::super::simple_selector::SimpleSelector;
    use super::*;

    fn complex(name: &str) -> ComplexSelector {
        ComplexSelector::new(
            CompoundSelector::new(vec![SimpleSelector::Type(name.into())]),
            Vec::new(),
        )
    }

    #[test]
    fn empty_list_reports_empty() {
        assert!(SelectorList::default().is_empty());
        assert!(SelectorList::new(Vec::new()).is_empty());
    }

    #[test]
    fn accessor_returns_selectors() {
        let list = SelectorList::new(vec![complex("a"), complex("b")]);
        assert!(!list.is_empty());
        assert_eq!(list.selectors().len(), 2);
    }
}
