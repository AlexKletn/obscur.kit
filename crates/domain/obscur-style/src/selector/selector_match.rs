//! The typed outcome of testing a selector against a node: [`SelectorMatch`].

use super::selector_specificity::SelectorSpecificity;

#[doc = include_str!("../../docs/api/selector/selector_match.doc/SelectorMatch.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectorMatch {
    /// The selector matched the node, contributing the given specificity.
    Matched {
        /// The specificity the matched selector contributes to the cascade.
        specificity: SelectorSpecificity,
    },
    /// The selector did not match the node.
    NoMatch,
}

impl SelectorMatch {
    /// Constructs a matched outcome carrying the given specificity.
    #[must_use]
    pub const fn matched(specificity: SelectorSpecificity) -> Self {
        Self::Matched { specificity }
    }

    /// Constructs a non-matching outcome.
    #[must_use]
    pub const fn no_match() -> Self {
        Self::NoMatch
    }

    /// Returns `true` if the selector matched the node.
    #[must_use]
    pub const fn is_match(self) -> bool {
        matches!(self, Self::Matched { .. })
    }

    /// Returns the contributed specificity when matched, otherwise `None`.
    #[must_use]
    pub const fn specificity(self) -> Option<SelectorSpecificity> {
        match self {
            Self::Matched { specificity } => Some(specificity),
            Self::NoMatch => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matched_reports_match_and_specificity() {
        let spec = SelectorSpecificity::new(1, 2, 3);
        let m = SelectorMatch::matched(spec);
        assert!(m.is_match());
        assert_eq!(m.specificity(), Some(spec));
    }

    #[test]
    fn no_match_reports_no_match() {
        let m = SelectorMatch::no_match();
        assert!(!m.is_match());
        assert_eq!(m.specificity(), None);
    }

    #[test]
    fn variants_are_distinct() {
        let a = SelectorMatch::matched(SelectorSpecificity::ZERO);
        let b = SelectorMatch::no_match();
        assert_ne!(a, b);
    }
}
