#![doc = include_str!("selector_match.doc/_module.md")]

use super::selector_specificity::SelectorSpecificity;

#[doc = include_str!("selector_match.doc/SelectorMatch.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectorMatch {
    #[doc = include_str!("selector_match.doc/Matched.md")]
    Matched {
        #[doc = include_str!("selector_match.doc/specificity.md")]
        specificity: SelectorSpecificity,
    },
    #[doc = include_str!("selector_match.doc/NoMatch.md")]
    NoMatch,
}

impl SelectorMatch {
    #[doc = include_str!("selector_match.doc/matched.md")]
    #[must_use]
    pub const fn matched(specificity: SelectorSpecificity) -> Self {
        Self::Matched { specificity }
    }

    #[doc = include_str!("selector_match.doc/no_match.md")]
    #[must_use]
    pub const fn no_match() -> Self {
        Self::NoMatch
    }

    #[doc = include_str!("selector_match.doc/is_match.md")]
    #[must_use]
    pub const fn is_match(self) -> bool {
        matches!(self, Self::Matched { .. })
    }

    #[doc = include_str!("selector_match.doc/specificity_2.md")]
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
