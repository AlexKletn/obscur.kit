use super::Hyphenator;

#[doc = include_str!("../../docs/api/layout/hyphenator.doc/NullHyphenator.md")]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NullHyphenator;

impl Hyphenator for NullHyphenator {
    /// Always returns an empty vector: no interior break points, so no word is
    /// ever split by the hyphenator (spec §5.10).
    fn hyphenate(&self, _word: &str) -> Vec<usize> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_hyphenator_proposes_no_breaks() {
        assert!(NullHyphenator.hyphenate("hyphenation").is_empty());
        assert!(NullHyphenator.hyphenate("").is_empty());
    }

    #[test]
    fn null_hyphenator_is_object_safe() {
        // Compile-time check the port is usable as a trait object.
        let hyphenator: &dyn Hyphenator = &NullHyphenator;
        assert!(hyphenator.hyphenate("word").is_empty());
    }
}
