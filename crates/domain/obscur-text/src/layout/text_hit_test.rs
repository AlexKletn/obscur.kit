#![doc = include_str!("text_hit_test.doc/_module.md")]

use obscur_geometry::{Point, Rect, Size};

use super::{TextLayout, TextLine};

#[doc = include_str!("text_hit_test.doc/TextHitTest.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextHitTest {
    #[doc = include_str!("text_hit_test.doc/line_index.md")]
    line_index: usize,
    #[doc = include_str!("text_hit_test.doc/byte_offset.md")]
    byte_offset: usize,
    #[doc = include_str!("text_hit_test.doc/is_trailing.md")]
    is_trailing: bool,
}

impl TextHitTest {
    #[doc = include_str!("text_hit_test.doc/new.md")]
    #[must_use]
    pub const fn new(line_index: usize, byte_offset: usize, is_trailing: bool) -> Self {
        Self {
            line_index,
            byte_offset,
            is_trailing,
        }
    }

    #[doc = include_str!("text_hit_test.doc/line_index_2.md")]
    #[must_use]
    pub const fn line_index(&self) -> usize {
        self.line_index
    }

    #[doc = include_str!("text_hit_test.doc/byte_offset_2.md")]
    #[must_use]
    pub const fn byte_offset(&self) -> usize {
        self.byte_offset
    }

    #[doc = include_str!("text_hit_test.doc/is_trailing_2.md")]
    #[must_use]
    pub const fn is_trailing(&self) -> bool {
        self.is_trailing
    }

    #[doc = include_str!("text_hit_test.doc/hit.md")]
    #[must_use]
    pub fn hit(layout: &TextLayout, point: Point) -> Self {
        let lines = layout.lines();
        if lines.is_empty() {
            return Self::new(0, 0, false);
        }

        let line_index = line_index_at(lines, point.y);
        let line = &lines[line_index];

        let mut pen = line.x_offset();
        if point.x < pen {
            return Self::new(line_index, line.range().start, false);
        }

        // Walk glyphs across the line's runs in visual order.
        for run in line.runs() {
            for glyph in run.glyphs() {
                let advance = glyph.x_advance();
                if point.x < pen + advance {
                    let is_trailing = point.x >= pen + advance / 2.0;
                    return Self::new(line_index, glyph.cluster(), is_trailing);
                }
                pen += advance;
            }
        }

        Self::new(line_index, line.range().end, false)
    }

    #[doc = include_str!("text_hit_test.doc/caret_rect.md")]
    #[must_use]
    pub fn caret_rect(
        layout: &TextLayout,
        line_index: usize,
        byte_offset: usize,
        caret_width: f32,
    ) -> Option<Rect> {
        let line = layout.lines().get(line_index)?;
        // Default to the trailing edge (offset at or past the line end).
        let mut caret_x = line.x_offset() + line.width();
        let mut pen = line.x_offset();
        'scan: for run in line.runs() {
            for glyph in run.glyphs() {
                if byte_offset <= glyph.cluster() {
                    caret_x = pen;
                    break 'scan;
                }
                pen += glyph.x_advance();
            }
        }
        Some(Rect::new(
            Point::new(caret_x, line.top()),
            Size::new(caret_width, line.height()),
        ))
    }
}

fn line_index_at(lines: &[TextLine], y: f32) -> usize {
    for (index, line) in lines.iter().enumerate() {
        if y < line.top() + line.height() {
            return index;
        }
    }
    lines.len() - 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::{
        FontAttributes, FontFace, FontFamily, FontMetrics, FontSource, GenericFamily,
    };
    use crate::layout::{TextAlignment, TextOverflow};
    use crate::shaping::{GlyphRun, MockTextShaper, ShapingRequest, TextShaper};

    fn mono_face() -> FontFace {
        FontFace::new(
            FontFamily::generic(GenericFamily::Monospace),
            FontAttributes::default(),
            FontSource::system("mono"),
            FontMetrics::new(1000, 800.0, -200.0, 0.0),
        )
    }

    fn shape(text: &str) -> GlyphRun {
        MockTextShaper::new().shape(&ShapingRequest::new(text, mono_face(), 10.0))
    }

    fn layout() -> TextLayout {
        // "hello world" at max_width 40 -> line 0 "hello " (0..6), line 1 "world".
        let run = shape("hello world");
        TextLayout::wrap(
            &run,
            "hello world",
            40.0,
            12.0,
            9.0,
            TextAlignment::Start,
            TextOverflow::Clip,
        )
    }

    #[test]
    fn accessors() {
        let hit = TextHitTest::new(1, 6, true);
        assert_eq!(hit.line_index(), 1);
        assert_eq!(hit.byte_offset(), 6);
        assert!(hit.is_trailing());
    }

    #[test]
    fn hit_near_line_start() {
        // Point near x=0 on line 0 -> first cluster (line start), not trailing.
        let hit = TextHitTest::hit(&layout(), Point::new(0.5, 1.0));
        assert_eq!(hit.line_index(), 0);
        assert_eq!(hit.byte_offset(), 0);
        assert!(!hit.is_trailing());
    }

    #[test]
    fn hit_mid_glyph_reports_cluster_and_trailing() {
        // Glyphs are 6.0 wide. x=15 is inside the third glyph (byte 2), spanning
        // [12, 18); 15 >= midpoint 15 -> trailing.
        let hit = TextHitTest::hit(&layout(), Point::new(15.0, 1.0));
        assert_eq!(hit.line_index(), 0);
        assert_eq!(hit.byte_offset(), 2);
        assert!(hit.is_trailing());
        // x=13 is in the same glyph but before its midpoint -> not trailing.
        let leading = TextHitTest::hit(&layout(), Point::new(13.0, 1.0));
        assert_eq!(leading.byte_offset(), 2);
        assert!(!leading.is_trailing());
    }

    #[test]
    fn hit_far_right_is_trailing_at_line_end() {
        // x=35 is in the last glyph of line 0 (the space, cluster 5), spanning
        // [30, 36); 35 >= midpoint 33 -> trailing at the line end.
        let hit = TextHitTest::hit(&layout(), Point::new(35.0, 1.0));
        assert_eq!(hit.line_index(), 0);
        assert_eq!(hit.byte_offset(), 5);
        assert!(hit.is_trailing());
        // Past every glyph clamps to the line's end offset, not trailing.
        let past = TextHitTest::hit(&layout(), Point::new(1000.0, 1.0));
        assert_eq!(past.byte_offset(), 6);
        assert!(!past.is_trailing());
    }

    #[test]
    fn hit_clamps_vertically_to_the_last_line() {
        // A far-below point resolves onto the last line.
        let hit = TextHitTest::hit(&layout(), Point::new(0.5, 1000.0));
        assert_eq!(hit.line_index(), 1);
    }

    #[test]
    fn caret_rect_is_inverse_of_hit() {
        let layout = layout();
        // Boundary at byte 0 sits at x = 0 (the line start offset).
        let rect = TextHitTest::caret_rect(&layout, 0, 0, 1.0).unwrap();
        assert_eq!(rect.origin, Point::new(0.0, 0.0));
        assert_eq!(rect.size, Size::new(1.0, 12.0));
        // Boundary at byte 2 sits after two 6.0-wide glyphs.
        let mid = TextHitTest::caret_rect(&layout, 0, 2, 1.0).unwrap();
        assert_eq!(mid.origin.x, 12.0);
        // Line-end boundary sits at the trailing edge (width 36).
        let end = TextHitTest::caret_rect(&layout, 0, 6, 1.0).unwrap();
        assert_eq!(end.origin.x, 36.0);
    }

    #[test]
    fn caret_rect_out_of_range_is_none() {
        assert!(TextHitTest::caret_rect(&layout(), 9, 0, 1.0).is_none());
    }

    #[test]
    fn hit_of_empty_layout_is_origin() {
        let empty = TextLayout::new(Vec::new(), TextAlignment::Start, TextOverflow::Clip);
        let hit = TextHitTest::hit(&empty, Point::new(5.0, 5.0));
        assert_eq!(hit, TextHitTest::new(0, 0, false));
    }
}
