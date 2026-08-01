#![doc = include_str!("english_hyphenator.doc/_module.md")]

use hyphenation::{Hyphenator as _, Language, Load, Standard};
use obscur_text::layout::Hyphenator;

#[doc = include_str!("english_hyphenator.doc/EnglishHyphenator.md")]
pub struct EnglishHyphenator {
    dictionary: Standard,
}

impl EnglishHyphenator {
    #[doc = include_str!("english_hyphenator.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        let dictionary = Standard::from_embedded(Language::EnglishUS)
            .expect("embedded US-English hyphenation patterns");
        Self { dictionary }
    }
}

impl Default for EnglishHyphenator {
    fn default() -> Self {
        Self::new()
    }
}

impl Hyphenator for EnglishHyphenator {
    #[doc = include_str!("english_hyphenator.doc/hyphenate.md")]
    fn hyphenate(&self, word: &str) -> Vec<usize> {
        self.dictionary.hyphenate(word).breaks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hyphenates_a_known_word_at_interior_points() {
        // "hyphenation" hyphenates as hy-phen-a-tion; the exact points are the
        // dictionary's, but they must be strictly interior and ascending.
        let hyphenator = EnglishHyphenator::new();
        let breaks = hyphenator.hyphenate("hyphenation");
        assert!(!breaks.is_empty(), "a real dictionary proposes ≥1 break");
        assert!(
            breaks.iter().all(|&b| b > 0 && b < "hyphenation".len()),
            "breaks {breaks:?} must be strictly interior"
        );
        assert!(
            breaks.windows(2).all(|w| w[0] < w[1]),
            "breaks {breaks:?} must be strictly ascending"
        );
    }

    #[test]
    fn short_word_yields_no_break() {
        // Below the minimum prefix + suffix, no break is possible.
        assert!(EnglishHyphenator::new().hyphenate("at").is_empty());
    }

    #[test]
    fn is_object_safe_as_the_domain_port() {
        let hyphenator: Box<dyn Hyphenator> = Box::new(EnglishHyphenator::new());
        assert!(!hyphenator.hyphenate("computer").is_empty());
    }
}
