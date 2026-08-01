#![doc = include_str!("hyphenator.doc/_module.md")]

#[doc = include_str!("hyphenator.doc/Hyphenator.md")]
pub trait Hyphenator {
    #[doc = include_str!("hyphenator.doc/hyphenate.md")]
    fn hyphenate(&self, word: &str) -> Vec<usize>;
}

#[doc = include_str!("hyphenator.doc/NullHyphenator.md")]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NullHyphenator;

impl Hyphenator for NullHyphenator {
    #[doc = include_str!("hyphenator.doc/hyphenate_2.md")]
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
