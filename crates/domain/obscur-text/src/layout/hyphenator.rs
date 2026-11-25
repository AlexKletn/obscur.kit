//! The replaceable hyphenation port and its no-op default (spec §5.10).

#[doc = include_str!("../../docs/api/layout/hyphenator.doc/Hyphenator.md")]
pub trait Hyphenator {
    /// Returns the interior byte offsets within `word` where a hyphenation break
    /// is permitted, in ascending order (spec §5.10).
    fn hyphenate(&self, word: &str) -> Vec<usize>;
}
