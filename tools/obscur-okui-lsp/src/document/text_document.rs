#![doc = include_str!("text_document.doc/_module.md")]

use crate::diagnostics::Position;

#[doc = include_str!("text_document.doc/to_u32.md")]
fn to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

#[doc = include_str!("text_document.doc/TextDocument.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextDocument {
    #[doc = include_str!("text_document.doc/uri.md")]
    pub uri: Box<str>,
    #[doc = include_str!("text_document.doc/version.md")]
    pub version: i32,
    #[doc = include_str!("text_document.doc/text.md")]
    pub text: String,
}

impl TextDocument {
    #[doc = include_str!("text_document.doc/new.md")]
    #[must_use]
    pub fn new(uri: impl Into<Box<str>>, version: i32, text: impl Into<String>) -> Self {
        Self {
            uri: uri.into(),
            version,
            text: text.into(),
        }
    }

    #[doc = include_str!("text_document.doc/line_index.md")]
    #[must_use]
    pub fn line_index(&self) -> LineIndex {
        LineIndex::new(&self.text)
    }
}

#[doc = include_str!("text_document.doc/LineIndex.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineIndex {
    #[doc = include_str!("text_document.doc/line_starts.md")]
    line_starts: Vec<u32>,
    #[doc = include_str!("text_document.doc/text_len.md")]
    text_len: u32,
}

impl LineIndex {
    #[doc = include_str!("text_document.doc/new_2.md")]
    #[must_use]
    pub fn new(text: &str) -> Self {
        let mut line_starts = vec![0u32];
        for (index, byte) in text.bytes().enumerate() {
            if byte == b'\n' {
                line_starts.push(to_u32(index + 1));
            }
        }
        Self {
            line_starts,
            text_len: to_u32(text.len()),
        }
    }

    #[doc = include_str!("text_document.doc/line_count.md")]
    #[must_use]
    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }

    #[doc = include_str!("text_document.doc/offset_to_position.md")]
    #[must_use]
    pub fn offset_to_position(&self, offset: u32) -> Position {
        let offset = offset.min(self.text_len);
        // Index of the first line whose start is > offset; the owning line is
        // the one before it. `line_starts[0] == 0 <= offset`, so this is >= 1.
        let line = self.line_starts.partition_point(|&start| start <= offset) - 1;
        let line_start = self.line_starts[line];
        Position {
            line: to_u32(line),
            character: offset - line_start,
        }
    }

    #[doc = include_str!("text_document.doc/position_to_offset.md")]
    #[must_use]
    pub fn position_to_offset(&self, position: Position) -> usize {
        let last_line = self.line_starts.len() - 1;
        let line = (position.line as usize).min(last_line);
        let line_start = self.line_starts[line];
        let line_end = self
            .line_starts
            .get(line + 1)
            .copied()
            .unwrap_or(self.text_len);
        let offset = line_start.saturating_add(position.character).min(line_end);
        offset as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_document_has_one_line() {
        let index = LineIndex::new("");
        assert_eq!(index.line_count(), 1);
        assert_eq!(index.offset_to_position(0), Position::new(0, 0));
        // Offsets past the end clamp to the end.
        assert_eq!(index.offset_to_position(9), Position::new(0, 0));
        assert_eq!(index.position_to_offset(Position::new(0, 0)), 0);
    }

    #[test]
    fn single_line_maps_columns() {
        let index = LineIndex::new("<App/>");
        assert_eq!(index.offset_to_position(0), Position::new(0, 0));
        assert_eq!(index.offset_to_position(4), Position::new(0, 4));
        assert_eq!(index.offset_to_position(6), Position::new(0, 6));
    }

    #[test]
    fn multi_line_maps_lines_and_columns() {
        //             0123 4 567 8
        let text = "<A>\n  <B/>\nx";
        let index = LineIndex::new(text);
        assert_eq!(index.line_count(), 3);
        // '\n' at byte 3 belongs to line 0 (character 3).
        assert_eq!(index.offset_to_position(3), Position::new(0, 3));
        // Line 1 starts at byte 4.
        assert_eq!(index.offset_to_position(4), Position::new(1, 0));
        assert_eq!(index.offset_to_position(6), Position::new(1, 2));
        // Line 2 starts after the second '\n' (byte 11).
        assert_eq!(index.offset_to_position(11), Position::new(2, 0));
    }

    #[test]
    fn offset_position_round_trips_every_byte() {
        let text = "<Row a=\"1\">\n  {x}\n</Row>\n";
        let index = LineIndex::new(text);
        for offset in 0..=to_u32(text.len()) {
            let position = index.offset_to_position(offset);
            let back = index.position_to_offset(position);
            assert_eq!(back, offset as usize, "offset {offset} did not round-trip");
        }
    }

    #[test]
    fn trailing_newline_yields_a_final_empty_line() {
        let index = LineIndex::new("a\n");
        assert_eq!(index.line_count(), 2);
        // The newline byte (1) is on line 0.
        assert_eq!(index.offset_to_position(1), Position::new(0, 1));
        // The end-of-text offset (2) begins the empty final line.
        assert_eq!(index.offset_to_position(2), Position::new(1, 0));
    }

    #[test]
    fn position_to_offset_clamps_out_of_range_input() {
        let index = LineIndex::new("ab\ncd");
        // Line past the end clamps to the last line.
        assert_eq!(index.position_to_offset(Position::new(9, 0)), 3);
        // Character past the line end clamps to the line's end.
        assert_eq!(index.position_to_offset(Position::new(0, 99)), 3);
        assert_eq!(index.position_to_offset(Position::new(1, 99)), 5);
    }

    #[test]
    fn text_document_builds_its_own_index() {
        let doc = TextDocument::new("file:///a.okui", 1, "<A/>\n<B/>");
        let index = doc.line_index();
        assert_eq!(index.offset_to_position(5), Position::new(1, 0));
        assert_eq!(doc.version, 1);
    }
}
