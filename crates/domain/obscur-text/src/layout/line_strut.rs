//! The inline line-box strut: the floor every line box is sized from
//! (CSS 2.1 §10.8.1 / CSS Inline 3 §5).

use crate::font::FontId;

use super::tagged_glyph::TaggedGlyph;

#[doc = include_str!("../../docs/api/layout/line_strut.doc/LineStrut.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineStrut {
    /// The CSS *used* `line-height` L, in logical pixels.
    line_height: f32,
    /// The primary font's ascent above the baseline, in logical pixels.
    ascent: f32,
    /// The primary font's descent below the baseline (magnitude, incl. line-gap),
    /// in logical pixels.
    descent: f32,
}

impl LineStrut {
    /// Creates a strut from the used `line_height` L and the primary font's
    /// `ascent` and `descent` (both logical pixels, `descent` a positive
    /// magnitude below the baseline).
    #[must_use]
    pub const fn new(line_height: f32, ascent: f32, descent: f32) -> Self {
        Self {
            line_height,
            ascent,
            descent,
        }
    }

    /// The used `line-height` L, in logical pixels.
    #[must_use]
    pub const fn line_height(&self) -> f32 {
        self.line_height
    }

    /// The primary font's ascent above the baseline, in logical pixels.
    #[must_use]
    pub const fn ascent(&self) -> f32 {
        self.ascent
    }

    /// The primary font's descent below the baseline (magnitude), in logical
    /// pixels.
    #[must_use]
    pub const fn descent(&self) -> f32 {
        self.descent
    }

    #[doc = include_str!("../../docs/api/layout/line_strut.doc/line_box.md")]
    pub(super) fn line_box(
        self,
        glyphs: &[TaggedGlyph],
        seg_metrics: &dyn Fn(FontId) -> (f32, f32),
    ) -> (f32, f32) {
        let mut ascent = self.ascent;
        let mut descent = self.descent;
        for tagged in glyphs {
            let (seg_ascent, seg_descent) = seg_metrics(tagged.font);
            ascent = ascent.max(seg_ascent);
            descent = descent.max(seg_descent);
        }
        let content = ascent + descent;
        if self.line_height > content {
            let half_leading = (self.line_height - content) / 2.0;
            (self.line_height, ascent + half_leading)
        } else {
            (content, ascent)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::shaping::{GlyphRun, ShapedGlyph, ShapedRun, TextDirection};
    use crate::span::TextRange;

    /// One tagged glyph in `font`, so a line can be given content without a
    /// paragraph around it.
    fn glyph_in(font: u32) -> TaggedGlyph {
        let run = ShapedRun::new(
            FontId::new(font),
            GlyphRun::new(
                vec![ShapedGlyph::new(1, 0, 6.0)],
                TextDirection::LeftToRight,
                10.0,
                TextRange::new(0, 1),
            ),
        );
        TaggedGlyph::flatten(&[run])[0]
    }

    #[test]
    fn an_empty_line_still_gets_the_struts_own_content_box() {
        // No content, and L below the content box: the strut is the floor.
        let strut = LineStrut::new(10.0, 8.0, 2.0);
        assert_eq!(strut.line_box(&[], &|_| (0.0, 0.0)), (10.0, 8.0));
    }

    #[test]
    fn excess_line_height_splits_as_half_leading_above_and_below() {
        // L = 20, content box A + D = 10 → half-leading 5 above and below.
        let strut = LineStrut::new(20.0, 8.0, 2.0);
        assert_eq!(strut.line_box(&[], &|_| (0.0, 0.0)), (20.0, 13.0));
    }

    #[test]
    fn a_taller_segment_font_raises_the_line_box_above_the_strut() {
        // The strut is a floor, not a cap: a taller fallback face wins the max.
        let strut = LineStrut::new(10.0, 8.0, 2.0);
        let line = [glyph_in(1)];
        assert_eq!(strut.line_box(&line, &|_| (12.0, 4.0)), (16.0, 12.0));
    }

    #[test]
    fn a_shorter_segment_font_never_shrinks_the_line_box() {
        let strut = LineStrut::new(10.0, 8.0, 2.0);
        let line = [glyph_in(1)];
        assert_eq!(strut.line_box(&line, &|_| (3.0, 1.0)), (10.0, 8.0));
    }
}
