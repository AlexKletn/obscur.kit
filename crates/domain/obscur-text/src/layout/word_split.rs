use super::{paragraph_word::ParagraphWord, tagged_glyph::TaggedGlyph};

/// The result of breaking a [`ParagraphWord`] at a hyphenation point: the head
/// glyphs (with an appended hyphen when one was supplied) and the remaining tail.
pub(super) struct WordSplit {
    pub(super) head: Vec<TaggedGlyph>,
    pub(super) tail: ParagraphWord,
}
