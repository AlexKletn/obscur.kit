//! The full laid-out paragraph: lines stacked vertically (spec §5.10).

use obscur_geometry::Size;

use crate::shaping::{GlyphRun, ShapedGlyph};
use crate::span::TextRange;

use super::{LineBreak, TextAlignment, TextLine, TextOverflow};

/// A fully laid-out paragraph: its [`TextLine`]s stacked vertically together with
/// the computed total [`size`](Self::size), alignment, and overflow policy
/// (spec §5.10).
///
/// The paragraph's `size` is derived from its lines: the width is the widest
/// line's width and the height is the sum of the line heights. Because it embeds
/// `f32` metrics and non-`Eq` glyph runs it derives `PartialEq` but not `Eq`.
#[derive(Debug, Clone, PartialEq)]
pub struct TextLayout {
    /// The laid-out lines, top to bottom.
    lines: Vec<TextLine>,
    /// The total bounding size: max line width by summed line height.
    size: Size,
    /// The horizontal alignment applied to the lines.
    alignment: TextAlignment,
    /// The overflow policy for text exceeding the available width.
    overflow: TextOverflow,
}

/// A word: the glyphs between two adjacent break opportunities, its advance
/// width, and whether a mandatory break follows it.
struct Word {
    glyphs: Vec<ShapedGlyph>,
    width: f32,
    mandatory_break_after: bool,
}

impl TextLayout {
    /// Creates a layout from its `lines`, `alignment`, and `overflow` policy,
    /// computing the total [`size`](Self::size) from the lines (spec §5.10).
    ///
    /// The width is the widest line's width and the height is the sum of the
    /// line heights; an empty paragraph has a zero size.
    #[must_use]
    pub fn new(lines: Vec<TextLine>, alignment: TextAlignment, overflow: TextOverflow) -> Self {
        let width = lines.iter().map(TextLine::width).fold(0.0_f32, f32::max);
        let height = lines.iter().map(TextLine::height).sum();
        Self {
            lines,
            size: Size::new(width, height),
            alignment,
            overflow,
        }
    }

    /// Greedily wraps a single shaped `run` of `text` into a paragraph no wider
    /// than `max_width` (spec §5.10).
    ///
    /// The algorithm walks the run's glyphs, grouping them into words at the
    /// [`LineBreak::opportunities`] of `text`. Words are packed onto the current
    /// line until adding the next word would exceed `max_width` while the line is
    /// non-empty, at which point the line is closed and the word starts the next
    /// one; a mandatory (hard) break always closes the current line. A word wider
    /// than `max_width` still occupies its own line rather than being split.
    ///
    /// Each line is stacked at a cumulative `top` (spaced by `line_height`), given
    /// the shared `baseline`, and offset horizontally by
    /// [`TextAlignment::x_offset`]. When `max_width <= 0` (an unconstrained width)
    /// or `text` has no break opportunities, the whole run becomes a single line.
    ///
    /// This greedy wrapper assumes a **single itemized run** whose glyph clusters
    /// are monotonic byte offsets — as produced by the deterministic mock shaper —
    /// and does not perform bidi reordering or per-glyph justification.
    #[must_use]
    pub fn wrap(
        run: &GlyphRun,
        text: &str,
        max_width: f32,
        line_height: f32,
        baseline: f32,
        alignment: TextAlignment,
        overflow: TextOverflow,
    ) -> Self {
        let direction = run.direction();
        let font_size = run.font_size();
        let source = run.source();

        // Split the run's glyphs into per-line glyph groups.
        let lines_glyphs = if run.is_empty() {
            // An empty run still yields a single (empty) line.
            vec![Vec::new()]
        } else if max_width <= 0.0 {
            // Unconstrained width: the whole run is one line.
            vec![run.glyphs().to_vec()]
        } else {
            Self::pack_lines(run, text, max_width)
        };

        // Build a `TextLine` per group, assigning cumulative tops, byte ranges,
        // and alignment offsets.
        let mut top = 0.0_f32;
        let mut lines = Vec::with_capacity(lines_glyphs.len());
        for (index, glyphs) in lines_glyphs.iter().enumerate() {
            let start = glyphs.first().map_or(source.start, ShapedGlyph::cluster);
            let end = lines_glyphs
                .get(index + 1)
                .and_then(|next| next.first())
                .map_or(source.end, ShapedGlyph::cluster);
            let range = TextRange::new(start.min(end), end);
            let sub_run = GlyphRun::new(glyphs.clone(), direction, font_size, range);
            let line = TextLine::new(vec![sub_run], range, top, line_height, baseline);
            let x_offset = alignment.x_offset(line.width(), max_width, direction);
            let line = line.with_x_offset(x_offset);
            top += line_height;
            lines.push(line);
        }

        Self::new(lines, alignment, overflow)
    }

    /// Groups a non-empty run's glyphs into per-line glyph vectors under a greedy
    /// `max_width` fit. `max_width` is assumed positive.
    fn pack_lines(run: &GlyphRun, text: &str, max_width: f32) -> Vec<Vec<ShapedGlyph>> {
        let words = Self::split_words(run, text);

        let mut lines: Vec<Vec<ShapedGlyph>> = Vec::new();
        let mut current: Vec<ShapedGlyph> = Vec::new();
        let mut current_width = 0.0_f32;
        for word in words {
            if !current.is_empty() && current_width + word.width > max_width {
                lines.push(std::mem::take(&mut current));
                current_width = 0.0;
            }
            current.extend_from_slice(&word.glyphs);
            current_width += word.width;
            if word.mandatory_break_after {
                lines.push(std::mem::take(&mut current));
                current_width = 0.0;
            }
        }
        if !current.is_empty() {
            lines.push(current);
        }
        if lines.is_empty() {
            // All content ended on a mandatory break; keep a single empty line.
            lines.push(Vec::new());
        }
        lines
    }

    /// Segments a run's glyphs into [`Word`]s at the line-break opportunities of
    /// `text`. A break at byte offset `o` starts a new word at the glyph whose
    /// cluster equals `o`; a hard break marks the preceding word for a mandatory
    /// line break.
    fn split_words(run: &GlyphRun, text: &str) -> Vec<Word> {
        let breaks = LineBreak::opportunities(text);
        let mut words: Vec<Word> = Vec::new();
        let mut glyphs: Vec<ShapedGlyph> = Vec::new();
        let mut width = 0.0_f32;
        for glyph in run.glyphs() {
            if let Some(brk) = breaks.iter().find(|b| b.offset() == glyph.cluster())
                && !glyphs.is_empty()
            {
                words.push(Word {
                    glyphs: std::mem::take(&mut glyphs),
                    width,
                    mandatory_break_after: brk.is_mandatory(),
                });
                width = 0.0;
            }
            glyphs.push(*glyph);
            width += glyph.x_advance();
        }
        if !glyphs.is_empty() {
            words.push(Word {
                glyphs,
                width,
                mandatory_break_after: false,
            });
        }
        words
    }

    /// Returns the laid-out lines, top to bottom.
    #[must_use]
    pub fn lines(&self) -> &[TextLine] {
        &self.lines
    }

    /// Returns the total bounding size of the paragraph.
    #[must_use]
    pub const fn size(&self) -> Size {
        self.size
    }

    /// Returns the horizontal alignment applied to the lines.
    #[must_use]
    pub const fn alignment(&self) -> TextAlignment {
        self.alignment
    }

    /// Returns the overflow policy for text exceeding the available width.
    #[must_use]
    pub const fn overflow(&self) -> TextOverflow {
        self.overflow
    }

    /// Returns the number of lines.
    #[must_use]
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Returns whether the paragraph has no lines.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// Returns the line at `index`, or `None` if out of bounds.
    #[must_use]
    pub fn line_at(&self, index: usize) -> Option<&TextLine> {
        self.lines.get(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::{
        FontAttributes, FontFace, FontFamily, FontMetrics, FontSource, GenericFamily,
    };
    use crate::shaping::{MockTextShaper, ShapingRequest, TextShaper};

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

    #[test]
    fn wrap_hello_world_into_two_lines() {
        // Each glyph advance is 10.0 * 0.6 = 6.0. "hello " is 6 glyphs (36),
        // "world" is 5 glyphs (30). max_width 40 fits one word but not both.
        let run = shape("hello world");
        let layout = TextLayout::wrap(
            &run,
            "hello world",
            40.0,
            12.0,
            9.0,
            TextAlignment::Start,
            TextOverflow::Clip,
        );
        assert_eq!(layout.line_count(), 2);
        assert_eq!(layout.lines()[0].width(), 36.0);
        assert_eq!(layout.lines()[1].width(), 30.0);
        // Byte ranges: "hello " is 0..6, "world" is 6..11.
        assert_eq!(layout.lines()[0].range(), TextRange::new(0, 6));
        assert_eq!(layout.lines()[1].range(), TextRange::new(6, 11));
        // Tops stack by line height; total size is max width by summed height.
        assert_eq!(layout.lines()[0].top(), 0.0);
        assert_eq!(layout.lines()[1].top(), 12.0);
        assert_eq!(layout.size(), Size::new(36.0, 24.0));
        assert_eq!(layout.alignment(), TextAlignment::Start);
        assert_eq!(layout.overflow(), TextOverflow::Clip);
    }

    #[test]
    fn wrap_x_offsets_follow_alignment() {
        let run = shape("hello world");
        // Right alignment: offset = max_width - line_width.
        let right = TextLayout::wrap(
            &run,
            "hello world",
            40.0,
            12.0,
            9.0,
            TextAlignment::Right,
            TextOverflow::Clip,
        );
        assert_eq!(right.lines()[0].x_offset(), 4.0); // 40 - 36
        assert_eq!(right.lines()[1].x_offset(), 10.0); // 40 - 30

        // Center: half the remaining space.
        let center = TextLayout::wrap(
            &run,
            "hello world",
            40.0,
            12.0,
            9.0,
            TextAlignment::Center,
            TextOverflow::Clip,
        );
        assert_eq!(center.lines()[0].x_offset(), 2.0); // (40 - 36) / 2
        assert_eq!(center.lines()[1].x_offset(), 5.0); // (40 - 30) / 2
    }

    #[test]
    fn hard_break_closes_the_line() {
        // "ab\ncd": mandatory break after the newline -> two lines.
        let run = shape("ab\ncd");
        let layout = TextLayout::wrap(
            &run,
            "ab\ncd",
            1000.0,
            12.0,
            9.0,
            TextAlignment::Start,
            TextOverflow::Clip,
        );
        assert_eq!(layout.line_count(), 2);
        assert_eq!(layout.lines()[0].range(), TextRange::new(0, 3));
        assert_eq!(layout.lines()[1].range(), TextRange::new(3, 5));
    }

    #[test]
    fn single_line_when_it_fits() {
        let run = shape("hello world");
        let layout = TextLayout::wrap(
            &run,
            "hello world",
            1000.0,
            12.0,
            9.0,
            TextAlignment::Start,
            TextOverflow::Clip,
        );
        assert_eq!(layout.line_count(), 1);
        assert_eq!(layout.lines()[0].range(), TextRange::new(0, 11));
    }

    #[test]
    fn non_positive_max_width_is_a_single_line() {
        let run = shape("hello world");
        let layout = TextLayout::wrap(
            &run,
            "hello world",
            0.0,
            12.0,
            9.0,
            TextAlignment::Start,
            TextOverflow::Clip,
        );
        assert_eq!(layout.line_count(), 1);
    }

    #[test]
    fn empty_run_yields_a_single_empty_line() {
        let run = shape("");
        let layout = TextLayout::wrap(
            &run,
            "",
            40.0,
            12.0,
            9.0,
            TextAlignment::Start,
            TextOverflow::Clip,
        );
        assert_eq!(layout.line_count(), 1);
        assert!(layout.lines()[0].is_empty());
    }

    #[test]
    fn wrap_is_deterministic() {
        let run = shape("hello world");
        let a = TextLayout::wrap(
            &run,
            "hello world",
            40.0,
            12.0,
            9.0,
            TextAlignment::Center,
            TextOverflow::Ellipsis,
        );
        let b = TextLayout::wrap(
            &run,
            "hello world",
            40.0,
            12.0,
            9.0,
            TextAlignment::Center,
            TextOverflow::Ellipsis,
        );
        assert_eq!(a, b);
    }

    #[test]
    fn accessors_and_line_at() {
        let run = shape("hello world");
        let layout = TextLayout::wrap(
            &run,
            "hello world",
            40.0,
            12.0,
            9.0,
            TextAlignment::Start,
            TextOverflow::Clip,
        );
        assert!(!layout.is_empty());
        assert!(layout.line_at(0).is_some());
        assert!(layout.line_at(2).is_none());
    }
}
