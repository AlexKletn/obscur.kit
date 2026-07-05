//! The port that splits text into uniform script + direction runs (spec §5.10).

use crate::shaping::{ScriptTag, TextDirection, TextItem};
use crate::span::TextRange;

/// A port that splits text into uniform runs for shaping (spec §5.10).
///
/// An itemizer walks the source text in *logical* order and cuts it into
/// [`TextItem`]s that are each uniform in script and bidirectional direction.
/// The returned items tile the whole input: their byte ranges are contiguous,
/// non-overlapping, and together cover `0..text.len()` with no gaps. Splitting
/// happens before shaping so a [`TextShaper`](crate::shaping::TextShaper) only
/// ever sees a single-script, single-direction run.
///
/// The Unicode-driven adapter that consults real script and bidi tables lives
/// outside the domain; [`SingleRunItemizer`] is the deterministic, table-free
/// reference implementation used to exercise the port.
pub trait TextItemizer {
    /// Splits `text` into uniform script + direction runs in logical order,
    /// seeded by the paragraph `base_direction`.
    ///
    /// The returned items cover `0..text.len()` contiguously with no gaps or
    /// overlaps. An empty `text` yields a single empty item `0..0`.
    #[must_use]
    fn itemize(&self, text: &str, base_direction: TextDirection) -> Vec<TextItem>;
}

/// The trivial reference [`TextItemizer`]: the whole text as one run (spec §5.10).
///
/// `SingleRunItemizer` performs no script or bidi analysis at all. It returns
/// exactly one [`TextItem`] spanning `0..text.len()` (or `0..0` for an empty
/// string), with script [`ScriptTag::COMMON`], the given base direction, and
/// embedding level `0` for left-to-right or `1` for right-to-left. It exists so
/// the port can be tested and wired without any Unicode tables; the real,
/// table-driven itemizer arrives in a later step.
#[derive(Debug, Clone, Copy, Default)]
pub struct SingleRunItemizer;

impl SingleRunItemizer {
    /// Creates a `SingleRunItemizer`.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl TextItemizer for SingleRunItemizer {
    fn itemize(&self, text: &str, base_direction: TextDirection) -> Vec<TextItem> {
        let level = u8::from(base_direction.is_rtl());
        vec![TextItem::new(
            TextRange::new(0, text.len()),
            ScriptTag::COMMON,
            base_direction,
            level,
        )]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_run_covers_whole_non_empty_text() {
        let items = SingleRunItemizer::new().itemize("hello", TextDirection::LeftToRight);
        assert_eq!(items.len(), 1);
        let item = items[0];
        assert_eq!(item.range(), TextRange::new(0, 5));
        assert_eq!(item.script(), ScriptTag::COMMON);
        assert_eq!(item.direction(), TextDirection::LeftToRight);
        assert_eq!(item.level(), 0);
    }

    #[test]
    fn empty_text_yields_one_empty_item() {
        let items = SingleRunItemizer.itemize("", TextDirection::LeftToRight);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].range(), TextRange::empty(0));
        assert!(items[0].range().is_empty());
    }

    #[test]
    fn base_rtl_sets_level_one() {
        let items = SingleRunItemizer.itemize("שלום", TextDirection::RightToLeft);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].direction(), TextDirection::RightToLeft);
        assert_eq!(items[0].level(), 1);
        assert_eq!(items[0].range(), TextRange::new(0, "שלום".len()));
    }

    #[test]
    fn is_object_safe() {
        let itemizer: Box<dyn TextItemizer> = Box::new(SingleRunItemizer::new());
        let items = itemizer.itemize("ab", TextDirection::LeftToRight);
        assert_eq!(items[0].range(), TextRange::new(0, 2));
    }
}
