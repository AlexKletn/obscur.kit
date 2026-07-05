//! The full laid-out multi-font paragraph: lines stacked vertically (spec §5.10).

use obscur_geometry::Size;

use crate::font::FontId;
use crate::shaping::{GlyphRun, ShapedGlyph, ShapedRun, TextDirection};
use crate::span::TextRange;

use super::{LineBreak, ParagraphLine, TextAlignment, TextOverflow};

/// A fully laid-out multi-font paragraph: its [`ParagraphLine`]s stacked
/// vertically together with the computed total [`size`](Self::size), alignment,
/// and overflow policy (spec §5.10).
///
/// This is the multi-font companion of [`TextLayout`](super::TextLayout): where
/// that wraps a single itemized [`GlyphRun`], `ParagraphLayout` wraps a sequence
/// of font-tagged [`ShapedRun`]s (as produced by the font-fallback orchestrator)
/// into lines whose segments may each use a different font. The paragraph's
/// `size` is derived from its lines: the width is the widest line's width and the
/// height is the sum of the line heights. Because it embeds `f32` metrics and
/// non-`Eq` glyph runs it derives `PartialEq` but not `Eq`.
#[derive(Debug, Clone, PartialEq)]
pub struct ParagraphLayout {
    /// The laid-out lines, top to bottom.
    lines: Vec<ParagraphLine>,
    /// The total bounding size: max line width by summed line height.
    size: Size,
    /// The horizontal alignment applied to the lines.
    alignment: TextAlignment,
    /// The overflow policy for text exceeding the available width.
    overflow: TextOverflow,
}

/// One flattened glyph tagged with the font, direction, and font size of the
/// [`ShapedRun`] it came from. Flattening the fallback result into this stream
/// lets word-splitting and greedy packing treat the whole paragraph uniformly
/// while retaining enough per-glyph provenance to rebuild same-font segments.
#[derive(Clone, Copy)]
struct Tagged {
    font: FontId,
    direction: TextDirection,
    font_size: f32,
    glyph: ShapedGlyph,
}

/// A word: the tagged glyphs between two adjacent break opportunities, its
/// advance width, and whether a mandatory break follows it. A word may contain
/// glyphs from more than one font.
struct Word {
    glyphs: Vec<Tagged>,
    width: f32,
    mandatory_break_after: bool,
}

impl ParagraphLayout {
    /// Creates a layout from its `lines`, `alignment`, and `overflow` policy,
    /// computing the total [`size`](Self::size) from the lines (spec §5.10).
    ///
    /// The width is the widest line's width and the height is the sum of the line
    /// heights.
    #[must_use]
    pub fn new(
        lines: Vec<ParagraphLine>,
        alignment: TextAlignment,
        overflow: TextOverflow,
    ) -> Self {
        let width = lines
            .iter()
            .map(ParagraphLine::width)
            .fold(0.0_f32, f32::max);
        let height = lines.iter().map(ParagraphLine::height).sum();
        Self {
            lines,
            size: Size::new(width, height),
            alignment,
            overflow,
        }
    }

    /// Greedily wraps a sequence of font-tagged shaped `runs` of `text` into a
    /// multi-font paragraph no wider than `max_width` (spec §5.10).
    ///
    /// The `runs` are the logical-order output of the font-fallback orchestrator;
    /// their glyph clusters are byte offsets into the full `text`. The algorithm:
    ///
    /// 1. flattens `runs` into one ordered glyph stream, tagging each glyph with
    ///    its run's font, direction, and font size;
    /// 2. splits the stream into words at the [`LineBreak::opportunities`] of
    ///    `text`, mapping each break's byte offset to the glyph whose cluster
    ///    equals it — a word may therefore span several fonts;
    /// 3. greedily packs words onto lines under `max_width`, closing a line when
    ///    the next word would overflow a non-empty line and always after a
    ///    mandatory break; a word wider than `max_width` still takes its own line;
    /// 4. splits each line's glyphs into contiguous same-font segments, rebuilding
    ///    a [`ShapedRun`] per segment, and assigns the line its cumulative `top`
    ///    (spaced by `line_height`), `height`, `baseline`, and alignment offset.
    ///
    /// When `max_width <= 0` (unconstrained) the whole paragraph is one line.
    /// Empty input — no runs, or a single empty run — yields one empty line rather
    /// than panicking. This greedy wrapper performs no bidi visual reordering
    /// within a line and never splits a word, even one that spans fonts.
    #[must_use]
    pub fn wrap(
        runs: &[ShapedRun],
        text: &str,
        max_width: f32,
        line_height: f32,
        baseline: f32,
        alignment: TextAlignment,
        overflow: TextOverflow,
    ) -> Self {
        let stream = Self::flatten(runs);

        // Split the flattened stream into per-line glyph groups.
        let lines_glyphs = if stream.is_empty() {
            // Empty input still yields a single (empty) line.
            vec![Vec::new()]
        } else if max_width <= 0.0 {
            // Unconstrained width: the whole paragraph is one line.
            vec![stream]
        } else {
            Self::pack_lines(&stream, text, max_width)
        };

        // The end byte of the last line falls back to the text length.
        let text_end = text.len();

        // Build a `ParagraphLine` per group, assigning cumulative tops, byte
        // ranges, same-font segments, and alignment offsets.
        let mut top = 0.0_f32;
        let mut lines = Vec::with_capacity(lines_glyphs.len());
        for (index, glyphs) in lines_glyphs.iter().enumerate() {
            // The line's byte range runs from its first glyph's cluster to the
            // next line's first glyph cluster (or the text end).
            let start = glyphs.first().map_or(0, |t| t.glyph.cluster());
            let end = lines_glyphs
                .get(index + 1)
                .and_then(|next| next.first())
                .map_or(text_end, |t| t.glyph.cluster());
            let range = TextRange::new(start.min(end), start.max(end));

            let segments = Self::segment(glyphs, end);
            let direction = glyphs
                .first()
                .map_or(TextDirection::LeftToRight, |t| t.direction);
            let line = ParagraphLine::new(segments, range, top, line_height, baseline);
            let x_offset = alignment.x_offset(line.width(), max_width, direction);
            let line = line.with_x_offset(x_offset);
            top += line_height;
            lines.push(line);
        }

        Self::new(lines, alignment, overflow)
    }

    /// Flattens the font-tagged `runs` into one ordered stream of [`Tagged`]
    /// glyphs, preserving run order and tagging each glyph with its run's font,
    /// direction, and font size.
    fn flatten(runs: &[ShapedRun]) -> Vec<Tagged> {
        let mut stream = Vec::new();
        for shaped in runs {
            let run = shaped.run();
            let font = shaped.font();
            let direction = run.direction();
            let font_size = run.font_size();
            for glyph in run.glyphs() {
                stream.push(Tagged {
                    font,
                    direction,
                    font_size,
                    glyph: *glyph,
                });
            }
        }
        stream
    }

    /// Groups a non-empty glyph `stream` into per-line glyph vectors under a
    /// greedy `max_width` fit. `max_width` is assumed positive.
    fn pack_lines(stream: &[Tagged], text: &str, max_width: f32) -> Vec<Vec<Tagged>> {
        let words = Self::split_words(stream, text);

        let mut lines: Vec<Vec<Tagged>> = Vec::new();
        let mut current: Vec<Tagged> = Vec::new();
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

    /// Segments a glyph `stream` into [`Word`]s at the line-break opportunities of
    /// `text`. A break at byte offset `o` starts a new word at the glyph whose
    /// cluster equals `o`; a hard break marks the preceding word for a mandatory
    /// line break.
    fn split_words(stream: &[Tagged], text: &str) -> Vec<Word> {
        let breaks = LineBreak::opportunities(text);
        let mut words: Vec<Word> = Vec::new();
        let mut glyphs: Vec<Tagged> = Vec::new();
        let mut width = 0.0_f32;
        for tagged in stream {
            if let Some(brk) = breaks
                .iter()
                .find(|b| b.offset() == tagged.glyph.cluster())
                && !glyphs.is_empty()
            {
                words.push(Word {
                    glyphs: std::mem::take(&mut glyphs),
                    width,
                    mandatory_break_after: brk.is_mandatory(),
                });
                width = 0.0;
            }
            glyphs.push(*tagged);
            width += tagged.glyph.x_advance();
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

    /// Splits one line's `glyphs` into contiguous same-font [`ShapedRun`]
    /// segments, in visual order. Each segment's [`GlyphRun`] takes its direction
    /// and font size from its glyphs' source run and a `source` byte range from
    /// its glyphs' clusters (its last segment ending at `line_end`).
    fn segment(glyphs: &[Tagged], line_end: usize) -> Vec<ShapedRun> {
        let mut segments = Vec::new();
        let mut index = 0;
        while index < glyphs.len() {
            let font = glyphs[index].font;
            let direction = glyphs[index].direction;
            let font_size = glyphs[index].font_size;
            let seg_start = glyphs[index].glyph.cluster();

            let mut run_glyphs = Vec::new();
            while index < glyphs.len() && glyphs[index].font == font {
                run_glyphs.push(glyphs[index].glyph);
                index += 1;
            }

            // The segment ends where the next segment begins (or the line end).
            let seg_end = glyphs
                .get(index)
                .map_or(line_end, |t| t.glyph.cluster());
            let range = TextRange::new(seg_start.min(seg_end), seg_start.max(seg_end));
            let run = GlyphRun::new(run_glyphs, direction, font_size, range);
            segments.push(ShapedRun::new(font, run));
        }
        segments
    }

    /// Returns the laid-out lines, top to bottom.
    #[must_use]
    pub fn lines(&self) -> &[ParagraphLine] {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a font-tagged run whose glyphs have the given `(cluster, advance)`
    /// pairs and byte `source` range.
    fn run(font: u32, glyphs: &[(usize, f32)], source: TextRange) -> ShapedRun {
        let shaped = glyphs
            .iter()
            .map(|&(cluster, adv)| ShapedGlyph::new(1, cluster, adv))
            .collect();
        ShapedRun::new(
            FontId::new(font),
            GlyphRun::new(shaped, TextDirection::LeftToRight, 10.0, source),
        )
    }

    #[test]
    fn two_runs_wide_width_one_line_two_segments() {
        // "ab" in font 0 (bytes 0,1) then "ВГ" in font 1 (Cyrillic, bytes 2,4).
        let text = "abВГ";
        let runs = vec![
            run(0, &[(0, 6.0), (1, 6.0)], TextRange::new(0, 2)),
            run(1, &[(2, 8.0), (4, 8.0)], TextRange::new(2, 6)),
        ];
        let layout = ParagraphLayout::wrap(
            &runs,
            text,
            1000.0,
            12.0,
            9.0,
            TextAlignment::Start,
            TextOverflow::Clip,
        );
        assert_eq!(layout.line_count(), 1);
        let line = &layout.lines()[0];
        assert_eq!(line.segments().len(), 2);
        // First segment font 0 with two glyphs, second font 1 with two glyphs.
        assert_eq!(line.segments()[0].font(), FontId::new(0));
        assert_eq!(line.segments()[0].run().len(), 2);
        assert_eq!(line.segments()[1].font(), FontId::new(1));
        assert_eq!(line.segments()[1].run().len(), 2);
        // Total advance 6+6+8+8 = 28.
        assert_eq!(line.width(), 28.0);
        assert_eq!(layout.size().width, 28.0);
        assert_eq!(layout.size().height, 12.0);
    }

    #[test]
    fn space_forces_break_two_single_font_lines() {
        // "ab гд": font 0 covers "ab " (bytes 0,1,2=space), font 1 covers "гд"
        // (Cyrillic, bytes 3,5). A soft break sits before "гд" at byte 3.
        let text = "ab гд";
        let runs = vec![
            run(0, &[(0, 6.0), (1, 6.0), (2, 6.0)], TextRange::new(0, 3)),
            run(1, &[(3, 8.0), (5, 8.0)], TextRange::new(3, 7)),
        ];
        // max_width 20 fits "ab " (18) but not "ab " + "гд" (34).
        let layout = ParagraphLayout::wrap(
            &runs,
            text,
            20.0,
            12.0,
            9.0,
            TextAlignment::Start,
            TextOverflow::Clip,
        );
        assert_eq!(layout.line_count(), 2);
        // Each line has a single same-font segment.
        assert_eq!(layout.lines()[0].segments().len(), 1);
        assert_eq!(layout.lines()[0].segments()[0].font(), FontId::new(0));
        assert_eq!(layout.lines()[1].segments().len(), 1);
        assert_eq!(layout.lines()[1].segments()[0].font(), FontId::new(1));
        // Two lines stacked by line height.
        assert_eq!(layout.lines()[0].top(), 0.0);
        assert_eq!(layout.lines()[1].top(), 12.0);
        assert_eq!(layout.size().height, 24.0);
    }

    #[test]
    fn single_run_reproduces_one_line_one_segment() {
        let text = "abc";
        let runs = vec![run(0, &[(0, 6.0), (1, 6.0), (2, 6.0)], TextRange::new(0, 3))];
        let layout = ParagraphLayout::wrap(
            &runs,
            text,
            1000.0,
            12.0,
            9.0,
            TextAlignment::Start,
            TextOverflow::Clip,
        );
        assert_eq!(layout.line_count(), 1);
        let line = &layout.lines()[0];
        assert_eq!(line.segments().len(), 1);
        // The single segment reproduces the input run's glyphs and font.
        assert_eq!(line.segments()[0].font(), FontId::new(0));
        assert_eq!(line.segments()[0].run().glyphs(), runs[0].run().glyphs());
        assert_eq!(line.range(), TextRange::new(0, 3));
        assert_eq!(line.width(), 18.0);
    }

    #[test]
    fn empty_input_yields_one_empty_line() {
        // No runs at all.
        let none = ParagraphLayout::wrap(
            &[],
            "",
            40.0,
            12.0,
            9.0,
            TextAlignment::Start,
            TextOverflow::Clip,
        );
        assert_eq!(none.line_count(), 1);
        assert!(none.lines()[0].is_empty());
        assert_eq!(none.lines()[0].top(), 0.0);
        assert_eq!(none.lines()[0].height(), 12.0);

        // A single empty run.
        let empty_run = vec![run(0, &[], TextRange::empty(0))];
        let one = ParagraphLayout::wrap(
            &empty_run,
            "",
            40.0,
            12.0,
            9.0,
            TextAlignment::Start,
            TextOverflow::Clip,
        );
        assert_eq!(one.line_count(), 1);
        assert!(one.lines()[0].is_empty());
    }

    #[test]
    fn non_positive_max_width_is_a_single_line() {
        let text = "ab гд";
        let runs = vec![
            run(0, &[(0, 6.0), (1, 6.0), (2, 6.0)], TextRange::new(0, 3)),
            run(1, &[(3, 8.0), (5, 8.0)], TextRange::new(3, 7)),
        ];
        let layout = ParagraphLayout::wrap(
            &runs,
            text,
            0.0,
            12.0,
            9.0,
            TextAlignment::Start,
            TextOverflow::Clip,
        );
        assert_eq!(layout.line_count(), 1);
        // Both fonts survive on the one line as two segments.
        assert_eq!(layout.lines()[0].segments().len(), 2);
    }

    #[test]
    fn wrap_is_deterministic() {
        let text = "ab гд";
        let runs = vec![
            run(0, &[(0, 6.0), (1, 6.0), (2, 6.0)], TextRange::new(0, 3)),
            run(1, &[(3, 8.0), (5, 8.0)], TextRange::new(3, 7)),
        ];
        let a = ParagraphLayout::wrap(
            &runs,
            text,
            20.0,
            12.0,
            9.0,
            TextAlignment::Center,
            TextOverflow::Ellipsis,
        );
        let b = ParagraphLayout::wrap(
            &runs,
            text,
            20.0,
            12.0,
            9.0,
            TextAlignment::Center,
            TextOverflow::Ellipsis,
        );
        assert_eq!(a, b);
    }

    #[test]
    fn accessors_expose_alignment_and_overflow() {
        let text = "abc";
        let runs = vec![run(0, &[(0, 6.0)], TextRange::new(0, 1))];
        let layout = ParagraphLayout::wrap(
            &runs,
            text,
            1000.0,
            12.0,
            9.0,
            TextAlignment::Right,
            TextOverflow::Ellipsis,
        );
        assert_eq!(layout.alignment(), TextAlignment::Right);
        assert_eq!(layout.overflow(), TextOverflow::Ellipsis);
    }
}
