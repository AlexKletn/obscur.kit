//! The port that splits text into uniform script + direction runs (spec §5.10).

use crate::shaping::{TextDirection, TextItem};

#[doc = include_str!("../../docs/api/shaping/text_itemizer.doc/TextItemizer.md")]
pub trait TextItemizer {
    #[doc = include_str!("../../docs/api/shaping/text_itemizer.doc/itemize.md")]
    #[must_use]
    fn itemize(&self, text: &str, base_direction: TextDirection) -> Vec<TextItem>;
}
