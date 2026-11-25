use crate::span::TextRange;

use super::{ScriptTag, TextDirection, TextItem, TextItemizer};

#[doc = include_str!("../../docs/api/shaping/text_itemizer.doc/SingleRunItemizer.md")]
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
