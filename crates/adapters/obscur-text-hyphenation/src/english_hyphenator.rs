//! The US-English Knuth-Liang [`Hyphenator`] adapter (spec §5.10).

use hyphenation::{Hyphenator as _, Language, Load, Standard};
use obscur_text::layout::Hyphenator;

/// A [`Hyphenator`] backed by the embedded US-English Knuth-Liang patterns
/// (spec §5.10).
///
/// [`EnglishHyphenator::new`] loads the `hyphenation` crate's embedded
/// `Language::EnglishUS` dictionary once; [`hyphenate`](Hyphenator::hyphenate)
/// then asks it for a word's break opportunities and returns their byte offsets.
/// The dictionary's built-in minimum prefix/suffix bounds keep a break from
/// stranding a single letter, and the standard patterns are deterministic, so
/// the same word always yields the same offsets.
pub struct EnglishHyphenator {
    dictionary: Standard,
}

impl EnglishHyphenator {
    /// Loads the embedded US-English dictionary.
    ///
    /// The dictionary is compiled into the binary by the `hyphenation` crate's
    /// `embed_en-us` feature, so loading it is infallible in practice.
    ///
    /// # Panics
    ///
    /// Panics if the embedded dictionary cannot be deserialized, which indicates
    /// a broken build rather than a runtime condition.
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
    /// Returns the interior byte offsets within `word` at which US-English
    /// hyphenation permits a break, ascending (spec §5.10).
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
