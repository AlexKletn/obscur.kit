//! The replaceable hyphenation port and its no-op default (spec §5.10).

/// A hyphenation port: given a single word, returns the byte offsets **within**
/// that word at which a hyphenation break is permitted (spec §5.10).
///
/// This is a pure, replaceable seam. The domain never embeds Knuth-Liang
/// patterns or a language dictionary; a real adapter (behind the xtask boundary)
/// implements this trait with actual hyphenation tables, while the domain wraps
/// text through the trait object. It is deliberately **object-safe** so the
/// runtime can hold a `Box<dyn Hyphenator>`.
///
/// # Contract
///
/// - The returned offsets are **byte** indices into `word` (UTF-8), each marking
///   a position at which `word` may be split (the tail begins at that byte).
/// - By convention an implementation returns only *interior* break points:
///   never `0` (the very start) and never `word.len()` (the very end), and it
///   should respect a sensible minimum prefix/suffix so a break does not strand
///   a single letter. The domain additionally clamps any offset that is not
///   strictly interior, so an over-eager adapter cannot corrupt wrapping.
/// - The result must be **deterministic**: the same `word` always yields the
///   same offsets, so measure and compose agree on the same lines.
///
/// Author soft hyphens (`U+00AD`) are handled separately by the wrapper and do
/// **not** need to be reported here; this port only proposes *algorithmic*
/// break points inside an otherwise unbroken word.
pub trait Hyphenator {
    /// Returns the interior byte offsets within `word` where a hyphenation break
    /// is permitted, in ascending order (spec §5.10).
    fn hyphenate(&self, word: &str) -> Vec<usize>;
}

/// A [`Hyphenator`] that never proposes a break point (spec §5.10).
///
/// This is the domain default: wrapping through it reproduces the historical
/// whitespace-only line breaking exactly, so a paragraph laid out with a
/// `NullHyphenator` is byte-identical to one laid out before hyphenation existed.
/// A host installs a real adapter to enable in-word breaking.
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
