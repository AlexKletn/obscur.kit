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

/// One flattened glyph tagged with the font, direction, font size, and bidi
/// embedding level of the [`ShapedRun`] it came from. Flattening the fallback
/// result into this stream lets word-splitting and greedy packing treat the
/// whole paragraph uniformly while retaining enough per-glyph provenance to
/// rebuild same-font, same-level segments and reorder them into visual order.
#[derive(Clone, Copy)]
struct Tagged {
    font: FontId,
    direction: TextDirection,
    font_size: f32,
    level: u8,
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

/// One packed line: its logical-order tagged glyphs, the number of words it holds
/// (so its inter-word gap count is `word_count - 1`), and whether the line was
/// closed by a *mandatory* (hard) break. Justification uses `word_count` and
/// `mandatory_break_after` to decide whether the line stretches to fill the width.
struct PackedLine {
    glyphs: Vec<Tagged>,
    word_count: usize,
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
    /// 4. splits each line's glyphs into contiguous same-font, same-level
    ///    segments, rebuilding a [`ShapedRun`] per segment, reorders those
    ///    segments into visual order via the UAX#9 L2 rule (see `segment`), and
    ///    assigns the line its cumulative `top` (spaced by `line_height`),
    ///    `height`, `baseline`, and alignment offset.
    ///
    /// When `max_width <= 0` (unconstrained) the whole paragraph is one line.
    /// Empty input — no runs, or a single empty run — yields one empty line rather
    /// than panicking. This greedy wrapper reorders a line's *segments* into
    /// visual order (line-level bidi) but never reorders glyphs within a segment —
    /// that is the shaper's job — and never splits a word, even one that spans
    /// fonts.
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

        // Split the flattened stream into per-line packed groups.
        let mut lines_glyphs = if stream.is_empty() {
            // Empty input still yields a single (empty) line.
            vec![PackedLine {
                glyphs: Vec::new(),
                word_count: 0,
                mandatory_break_after: false,
            }]
        } else if max_width <= 0.0 {
            // Unconstrained width: the whole paragraph is one line. A single
            // unconstrained line is never justified (it is the last line), so its
            // word count is immaterial.
            vec![PackedLine {
                glyphs: stream,
                word_count: 0,
                mandatory_break_after: false,
            }]
        } else {
            Self::pack_lines(&stream, text, max_width)
        };

        // Justify: stretch every eligible line's inter-word gaps so it fills
        // `max_width`. Only meaningful under a positive constraint (the unbounded
        // and empty branches produce a single, non-justified last line).
        if alignment == TextAlignment::Justify && max_width > 0.0 {
            Self::justify(&mut lines_glyphs, text, max_width);
        }

        // The end byte of the last line falls back to the text length.
        let text_end = text.len();

        // Build a `ParagraphLine` per group, assigning cumulative tops, byte
        // ranges, same-font segments, and alignment offsets.
        let mut top = 0.0_f32;
        let mut lines = Vec::with_capacity(lines_glyphs.len());
        for (index, packed) in lines_glyphs.iter().enumerate() {
            let glyphs = &packed.glyphs;
            // The line's byte range runs from its first glyph's cluster to the
            // next line's first glyph cluster (or the text end).
            let start = glyphs.first().map_or(0, |t| t.glyph.cluster());
            let end = lines_glyphs
                .get(index + 1)
                .and_then(|next| next.glyphs.first())
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
            let level = shaped.level();
            for glyph in run.glyphs() {
                stream.push(Tagged {
                    font,
                    direction,
                    font_size,
                    level,
                    glyph: *glyph,
                });
            }
        }
        stream
    }

    /// Groups a non-empty glyph `stream` into per-line [`PackedLine`]s under a
    /// greedy `max_width` fit, recording each line's word count and whether a
    /// mandatory break closed it. `max_width` is assumed positive.
    fn pack_lines(stream: &[Tagged], text: &str, max_width: f32) -> Vec<PackedLine> {
        let words = Self::split_words(stream, text);

        let mut lines: Vec<PackedLine> = Vec::new();
        let mut current: Vec<Tagged> = Vec::new();
        let mut current_width = 0.0_f32;
        let mut word_count = 0_usize;
        for word in words {
            if !current.is_empty() && current_width + word.width > max_width {
                lines.push(PackedLine {
                    glyphs: std::mem::take(&mut current),
                    word_count,
                    mandatory_break_after: false,
                });
                current_width = 0.0;
                word_count = 0;
            }
            current.extend_from_slice(&word.glyphs);
            current_width += word.width;
            word_count += 1;
            if word.mandatory_break_after {
                lines.push(PackedLine {
                    glyphs: std::mem::take(&mut current),
                    word_count,
                    mandatory_break_after: true,
                });
                current_width = 0.0;
                word_count = 0;
            }
        }
        if !current.is_empty() {
            lines.push(PackedLine {
                glyphs: current,
                word_count,
                mandatory_break_after: false,
            });
        }
        if lines.is_empty() {
            // All content ended on a mandatory break; keep a single empty line.
            lines.push(PackedLine {
                glyphs: Vec::new(),
                word_count: 0,
                mandatory_break_after: false,
            });
        }
        lines
    }

    /// Stretches every justification-eligible line so it fills `max_width`,
    /// distributing the extra space equally across the line's inter-word gaps
    /// (spec §5.10).
    ///
    /// A line is **not** justified — it keeps its natural width — when it is the
    /// paragraph's last line, when it was closed by a mandatory (hard) break, or
    /// when it holds fewer than two words (no inter-word gap to stretch). For an
    /// eligible line the extra space `max_width - line_width` is split evenly
    /// across its `g` gaps and added to the trailing glyph of each gap (typically
    /// the word-final space); glyph ids and clusters are left intact, so the line's
    /// rendered width becomes exactly `max_width`. A line already at or past
    /// `max_width` gains nothing.
    fn justify(lines: &mut [PackedLine], text: &str, max_width: f32) {
        let Some(last) = lines.len().checked_sub(1) else {
            return;
        };
        // Byte offsets that begin a word (every break opportunity); a glyph whose
        // cluster equals one starts a new word, so the glyph before it ends a gap.
        let word_starts: std::collections::HashSet<usize> = LineBreak::opportunities(text)
            .iter()
            .map(LineBreak::offset)
            .collect();
        for (index, line) in lines.iter_mut().enumerate() {
            if index == last || line.mandatory_break_after || line.word_count < 2 {
                continue;
            }
            Self::justify_line(&mut line.glyphs, &word_starts, max_width);
        }
    }

    /// Distributes the extra space that fills `max_width` across one line's
    /// inter-word gaps by widening the trailing glyph of each gap.
    ///
    /// A gap sits before every glyph (after the first) whose cluster is in
    /// `word_starts`; the glyph immediately before it is the gap's trailing glyph.
    /// The `max_width - line_width` surplus is split equally across the gaps, so a
    /// line with `g` gaps grows to exactly `max_width`. Nothing happens when the
    /// line has no interior gap or is already at/over `max_width`.
    fn justify_line(glyphs: &mut [Tagged], word_starts: &std::collections::HashSet<usize>, max_width: f32) {
        let line_width: f32 = glyphs.iter().map(|t| t.glyph.x_advance()).sum();
        let extra = max_width - line_width;
        if extra <= 0.0 {
            return;
        }
        // Trailing-glyph index of every interior gap (before a word-start glyph).
        let gap_ends: Vec<usize> = (1..glyphs.len())
            .filter(|&j| word_starts.contains(&glyphs[j].glyph.cluster()))
            .map(|j| j - 1)
            .collect();
        if gap_ends.is_empty() {
            return;
        }
        #[allow(clippy::cast_precision_loss)]
        let add = extra / gap_ends.len() as f32;
        for index in gap_ends {
            let glyph = glyphs[index].glyph;
            glyphs[index].glyph = glyph.with_x_advance(glyph.x_advance() + add);
        }
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

    /// Splits one line's logical-order `glyphs` into contiguous same-font AND
    /// same-level [`ShapedRun`] segments, then reorders those segments into visual
    /// (left-to-right) order via the UAX#9 L2 rule (see `reorder_visual`). Each
    /// segment's [`GlyphRun`] takes its direction and font size from its glyphs'
    /// source run, a `source` byte range from its glyphs' clusters (its last
    /// logical segment ending at `line_end`), and carries its bidi `level`.
    ///
    /// A level boundary ends a segment as well as a font boundary, so a run that
    /// changes embedding level splits even within one font. Within a segment the
    /// glyph order is left as the shaper produced it (HarfRust already visually
    /// orders RTL runs; the mock emits LTR order — a documented mock limitation).
    fn segment(glyphs: &[Tagged], line_end: usize) -> Vec<ShapedRun> {
        let mut segments = Vec::new();
        let mut index = 0;
        while index < glyphs.len() {
            let font = glyphs[index].font;
            let direction = glyphs[index].direction;
            let font_size = glyphs[index].font_size;
            let level = glyphs[index].level;
            let seg_start = glyphs[index].glyph.cluster();

            let mut run_glyphs = Vec::new();
            while index < glyphs.len()
                && glyphs[index].font == font
                && glyphs[index].level == level
            {
                run_glyphs.push(glyphs[index].glyph);
                index += 1;
            }

            // The segment ends where the next segment begins (or the line end).
            let seg_end = glyphs
                .get(index)
                .map_or(line_end, |t| t.glyph.cluster());
            let range = TextRange::new(seg_start.min(seg_end), seg_start.max(seg_end));
            let run = GlyphRun::new(run_glyphs, direction, font_size, range);
            segments.push(ShapedRun::new(font, run).with_level(level));
        }
        Self::reorder_visual(&mut segments);
        segments
    }

    /// Reorders a line's logical-order `segments` into visual (left-to-right)
    /// order in place, applying the UAX#9 L2 rule using each segment's bidi
    /// [`level`](ShapedRun::level).
    ///
    /// From the highest level present down to the lowest odd level, every maximal
    /// contiguous span of segments whose level is `>= lvl` is reversed. This
    /// places right-to-left runs (and nested higher-level runs within them) into
    /// their correct visual position without touching within-segment glyph order.
    /// A line that is entirely even-levelled (pure LTR) has no odd level and is
    /// left untouched. The reversal is a pure function of the levels, so the
    /// result is deterministic.
    fn reorder_visual(segments: &mut [ShapedRun]) {
        let Some(max_level) = segments.iter().map(ShapedRun::level).max() else {
            return;
        };
        let lowest_odd = segments
            .iter()
            .map(ShapedRun::level)
            .filter(|lvl| lvl % 2 == 1)
            .min();
        let Some(lowest_odd) = lowest_odd else {
            // No odd level: pure LTR line, nothing to reorder.
            return;
        };

        let mut lvl = max_level;
        while lvl >= lowest_odd {
            let mut start = 0;
            while start < segments.len() {
                if segments[start].level() >= lvl {
                    let mut end = start;
                    while end < segments.len() && segments[end].level() >= lvl {
                        end += 1;
                    }
                    segments[start..end].reverse();
                    start = end;
                } else {
                    start += 1;
                }
            }
            lvl -= 1;
        }
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

    /// Builds a font-tagged run at bidi embedding `level` whose glyphs have the
    /// given `(cluster, advance)` pairs and byte `source` range.
    fn run_lvl(font: u32, level: u8, glyphs: &[(usize, f32)], source: TextRange) -> ShapedRun {
        run(font, glyphs, source).with_level(level)
    }

    /// Wraps `runs` of `text` on one unconstrained line and returns the visual
    /// segment order the L2 reordering produced.
    fn visual_segments(runs: &[ShapedRun], text: &str) -> Vec<(u32, u8)> {
        let layout = ParagraphLayout::wrap(
            runs,
            text,
            1000.0,
            12.0,
            9.0,
            TextAlignment::Start,
            TextOverflow::Clip,
        );
        assert_eq!(layout.line_count(), 1);
        layout.lines()[0]
            .segments()
            .iter()
            .map(|s| (s.font().get(), s.level()))
            .collect()
    }

    #[test]
    fn single_ltr_run_level_zero_is_unchanged() {
        // A pure-LTR line (level 0) keeps its logical order and level.
        let text = "abc";
        let runs = vec![run_lvl(0, 0, &[(0, 6.0), (1, 6.0), (2, 6.0)], TextRange::new(0, 3))];
        assert_eq!(visual_segments(&runs, text), vec![(0, 0)]);
    }

    #[test]
    fn mixed_ltr_rtl_ltr_levels_keep_rtl_in_place() {
        // Logical segments levels [0,1,0] ("ab" LTR, "cd" RTL, "ef" LTR). Per
        // UAX#9 L2 the lone level-1 span (one segment) stays between its LTR
        // neighbors — only its interior glyphs would flip (the shaper's job).
        let text = "abcdef";
        let runs = vec![
            run_lvl(0, 0, &[(0, 6.0), (1, 6.0)], TextRange::new(0, 2)),
            run_lvl(1, 1, &[(2, 6.0), (3, 6.0)], TextRange::new(2, 4)),
            run_lvl(2, 0, &[(4, 6.0), (5, 6.0)], TextRange::new(4, 6)),
        ];
        // Visual order unchanged: LTR, RTL, LTR.
        assert_eq!(visual_segments(&runs, text), vec![(0, 0), (1, 1), (2, 0)]);

        // Left-to-right pen positions place the three segments in that order.
        let layout = ParagraphLayout::wrap(
            &runs, text, 1000.0, 12.0, 9.0, TextAlignment::Start, TextOverflow::Clip,
        );
        let segs = layout.lines()[0].segments();
        assert_eq!(segs[0].font(), FontId::new(0));
        assert_eq!(segs[1].font(), FontId::new(1));
        assert_eq!(segs[2].font(), FontId::new(2));
        assert_eq!(layout.lines()[0].width(), 36.0);
    }

    #[test]
    fn pure_rtl_line_reverses_segment_order() {
        // Two level-1 segments in logical order → reversed visually.
        let text = "abcd";
        let runs = vec![
            run_lvl(0, 1, &[(0, 6.0), (1, 6.0)], TextRange::new(0, 2)),
            run_lvl(1, 1, &[(2, 6.0), (3, 6.0)], TextRange::new(2, 4)),
        ];
        // Logical [font0, font1] → visual [font1, font0].
        assert_eq!(visual_segments(&runs, text), vec![(1, 1), (0, 1)]);
    }

    #[test]
    fn nested_levels_reorder_higher_within_lower() {
        // Logical segments levels [0,1,2,1,0] (fonts 0..4). L2 reverses the
        // level-1 span (indices 1..4) → fonts [0, 3, 2, 1, 4].
        let text = "abcde";
        let runs = vec![
            run_lvl(0, 0, &[(0, 6.0)], TextRange::new(0, 1)),
            run_lvl(1, 1, &[(1, 6.0)], TextRange::new(1, 2)),
            run_lvl(2, 2, &[(2, 6.0)], TextRange::new(2, 3)),
            run_lvl(3, 1, &[(3, 6.0)], TextRange::new(3, 4)),
            run_lvl(4, 0, &[(4, 6.0)], TextRange::new(4, 5)),
        ];
        assert_eq!(
            visual_segments(&runs, text),
            vec![(0, 0), (3, 1), (2, 2), (1, 1), (4, 0)]
        );
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
    fn justify_stretches_non_last_lines_to_max_width() {
        // "ab cd ef" all on one font. Each glyph advance 6; spaces at clusters
        // 2 and 5. Breaks sit before "cd" (byte 3) and "ef" (byte 6). Natural
        // width 8*6 = 48; on a two-line wrap the first line "ab cd " justifies.
        let text = "ab cd ef";
        let runs = vec![run(
            0,
            &[
                (0, 6.0),
                (1, 6.0),
                (2, 6.0),
                (3, 6.0),
                (4, 6.0),
                (5, 6.0),
                (6, 6.0),
                (7, 6.0),
            ],
            TextRange::new(0, 8),
        )];
        // max_width 40 fits "ab cd " (36) but not "+ef" (48), so "ef" wraps.
        let layout = ParagraphLayout::wrap(
            &runs, text, 40.0, 12.0, 9.0, TextAlignment::Justify, TextOverflow::Clip,
        );
        assert_eq!(layout.line_count(), 2);
        // The first line has a single interior gap (between "ab" and "cd") and is
        // stretched to exactly max_width.
        assert_eq!(layout.lines()[0].width(), 40.0);
        assert_eq!(layout.lines()[0].x_offset(), 0.0);
        // The paragraph's widest line is the justified one.
        assert_eq!(layout.size().width, 40.0);
        // The last line "ef" is NOT justified: it keeps its natural width (12).
        assert_eq!(layout.lines()[1].width(), 12.0);
    }

    #[test]
    fn justify_distributes_equally_across_two_gaps() {
        // "aa bb cc dd": three words fit on the first line, so it has two interior
        // gaps (the spaces at clusters 2 and 5). Every glyph advance is 5.
        let text = "aa bb cc dd";
        let runs = vec![run(
            0,
            &[
                (0, 5.0),
                (1, 5.0),
                (2, 5.0),
                (3, 5.0),
                (4, 5.0),
                (5, 5.0),
                (6, 5.0),
                (7, 5.0),
                (8, 5.0),
                (9, 5.0),
                (10, 5.0),
            ],
            TextRange::new(0, 11),
        )];
        // max_width 50 fits "aa bb cc " (45) but not "+dd" (55): "dd" wraps.
        let layout = ParagraphLayout::wrap(
            &runs, text, 50.0, 12.0, 9.0, TextAlignment::Justify, TextOverflow::Clip,
        );
        assert_eq!(layout.line_count(), 2);
        // The first line has two gaps; the 5px surplus is split as 2.5 per gap,
        // widening the two space glyphs (clusters 2 and 5) from 5 to 7.5 each.
        assert_eq!(layout.lines()[0].width(), 50.0);
        let glyphs = layout.lines()[0].segments()[0].run().glyphs();
        assert_eq!(glyphs[2].x_advance(), 7.5);
        assert_eq!(glyphs[5].x_advance(), 7.5);
        // Non-gap glyphs keep their natural advance.
        assert_eq!(glyphs[0].x_advance(), 5.0);
        assert_eq!(glyphs[3].x_advance(), 5.0);
        // The last line "dd" keeps its natural width (10).
        assert_eq!(layout.lines()[1].width(), 10.0);
    }

    #[test]
    fn justify_exempts_single_word_line() {
        // One word, no interior gap: never justified even when not the last line.
        // "abcd efgh": first line "abcd " wraps alone under a tight width.
        let text = "abcd efgh";
        let runs = vec![run(
            0,
            &[
                (0, 8.0),
                (1, 8.0),
                (2, 8.0),
                (3, 8.0),
                (4, 8.0),
                (5, 8.0),
                (6, 8.0),
                (7, 8.0),
                (8, 8.0),
            ],
            TextRange::new(0, 9),
        )];
        // max_width 44 fits "abcd " (40) but not "+efgh" (72). First line is one
        // word — not justified — so it keeps its natural width 40, not 44.
        let layout = ParagraphLayout::wrap(
            &runs, text, 44.0, 12.0, 9.0, TextAlignment::Justify, TextOverflow::Clip,
        );
        assert_eq!(layout.line_count(), 2);
        assert_eq!(layout.lines()[0].width(), 40.0);
    }

    #[test]
    fn justify_exempts_hard_broken_line() {
        // "ab cd\nef": the first line ends on a mandatory `\n` break, so it is not
        // justified even though it is not the paragraph's last line.
        let text = "ab cd\nef";
        let runs = vec![run(
            0,
            &[
                (0, 6.0),
                (1, 6.0),
                (2, 6.0),
                (3, 6.0),
                (4, 6.0),
                (6, 6.0),
                (7, 6.0),
            ],
            TextRange::new(0, 8),
        )];
        let layout = ParagraphLayout::wrap(
            &runs, text, 100.0, 12.0, 9.0, TextAlignment::Justify, TextOverflow::Clip,
        );
        assert_eq!(layout.line_count(), 2);
        // First line "ab cd" ended by a hard break: natural width 5*6 = 30.
        assert_eq!(layout.lines()[0].width(), 30.0);
    }

    #[test]
    fn justify_noop_when_unconstrained() {
        // max_width <= 0 disables justification entirely (one unconstrained line).
        let text = "ab cd ef";
        let runs = vec![run(
            0,
            &[(0, 6.0), (1, 6.0), (2, 6.0), (3, 6.0), (4, 6.0), (5, 6.0), (6, 6.0), (7, 6.0)],
            TextRange::new(0, 8),
        )];
        let layout = ParagraphLayout::wrap(
            &runs, text, 0.0, 12.0, 9.0, TextAlignment::Justify, TextOverflow::Clip,
        );
        assert_eq!(layout.line_count(), 1);
        assert_eq!(layout.lines()[0].width(), 48.0);
    }

    #[test]
    fn center_and_right_still_use_x_offset() {
        // Non-justify alignments keep offsetting via x_offset and never widen
        // glyphs, so the line width stays natural.
        let text = "ab cd";
        let runs = vec![run(0, &[(0, 6.0), (1, 6.0), (2, 6.0), (3, 6.0), (4, 6.0)], TextRange::new(0, 5))];
        let center = ParagraphLayout::wrap(
            &runs, text, 100.0, 12.0, 9.0, TextAlignment::Center, TextOverflow::Clip,
        );
        assert_eq!(center.lines()[0].width(), 30.0);
        assert_eq!(center.lines()[0].x_offset(), 35.0);
        let right = ParagraphLayout::wrap(
            &runs, text, 100.0, 12.0, 9.0, TextAlignment::Right, TextOverflow::Clip,
        );
        assert_eq!(right.lines()[0].width(), 30.0);
        assert_eq!(right.lines()[0].x_offset(), 70.0);
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
