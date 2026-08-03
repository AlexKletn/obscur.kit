#![doc = include_str!("paragraph_layout.doc/_module.md")]

use obscur_geometry::Size;

use crate::font::FontId;
use crate::shaping::{ShapedGlyph, ShapedRun, TextDirection};
use crate::span::TextRange;

use super::break_offsets::BreakOffsets;
use super::line_break::is_cjk_ideograph;
use super::line_justification::justify_lines;
use super::line_segments::line_segments;
use super::line_truncation::truncate_lines;
use super::packed_line::PackedLine;
use super::tagged_glyph::TaggedGlyph;
use super::{
    DefaultLineBreaker, Hyphenator, LineBreak, LineBreaker, LineStrut, NullHyphenator,
    OverflowWrap, ParagraphLine, TextAlignment, TextOverflow, WhiteSpace, WordBreak,
};

#[doc = include_str!("paragraph_layout.doc/ParagraphLayout.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct ParagraphLayout {
    #[doc = include_str!("paragraph_layout.doc/lines.md")]
    lines: Vec<ParagraphLine>,
    #[doc = include_str!("paragraph_layout.doc/size.md")]
    size: Size,
    #[doc = include_str!("paragraph_layout.doc/alignment.md")]
    alignment: TextAlignment,
    #[doc = include_str!("paragraph_layout.doc/overflow.md")]
    overflow: TextOverflow,
}

impl ParagraphLayout {
    #[doc = include_str!("paragraph_layout.doc/new.md")]
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

    #[doc = include_str!("paragraph_layout.doc/wrap.md")]
    // The wrap inputs plus the white-space mode are cohesive positional arguments
    // of one leaf entry point, not a public API surface.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn wrap(
        runs: &[ShapedRun],
        text: &str,
        max_width: f32,
        line_height: f32,
        baseline: f32,
        alignment: TextAlignment,
        overflow: TextOverflow,
        white_space: WhiteSpace,
    ) -> Self {
        // The historical wrapper is exactly hyphenated wrapping through the no-op
        // [`NullHyphenator`] with no hyphen glyph: no interior break is ever
        // proposed, so the output is byte-identical to whitespace-only wrapping.
        Self::wrap_hyphenated(
            runs,
            text,
            max_width,
            line_height,
            baseline,
            alignment,
            overflow,
            white_space,
            &NullHyphenator,
            None,
            None,
        )
    }

    #[doc = include_str!("paragraph_layout.doc/wrap_hyphenated.md")]
    // The wrap inputs plus the hyphenation port, hyphen glyph, and ellipsis glyph
    // are cohesive positional arguments of one leaf entry point, not a public API
    // surface.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn wrap_hyphenated(
        runs: &[ShapedRun],
        text: &str,
        max_width: f32,
        line_height: f32,
        baseline: f32,
        alignment: TextAlignment,
        overflow: TextOverflow,
        white_space: WhiteSpace,
        hyphenator: &dyn Hyphenator,
        hyphen_glyph: Option<ShapedGlyph>,
        ellipsis_glyph: Option<ShapedGlyph>,
    ) -> Self {
        // Historical entry point: a single scalar `line_height`/`baseline` is a
        // uniform strut (primary ascent = `baseline`, descent = `line_height -
        // baseline`) with font-agnostic segment metrics, so every line box equals
        // the scalar values — byte-identical to the pre-strut wrapper.
        let strut = LineStrut::new(line_height, baseline, line_height - baseline);
        let seg_metrics = move |_font: FontId| (strut.ascent(), strut.descent());
        Self::wrap_core(
            runs,
            text,
            max_width,
            strut,
            &seg_metrics,
            alignment,
            overflow,
            white_space,
            0.0,
            0.0,
            0.0,
            hyphenator,
            hyphen_glyph,
            ellipsis_glyph,
            &DefaultLineBreaker,
            OverflowWrap::Normal,
            WordBreak::Normal,
        )
    }

    #[doc = include_str!("paragraph_layout.doc/wrap_strut.md")]
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn wrap_strut(
        runs: &[ShapedRun],
        text: &str,
        max_width: f32,
        strut: LineStrut,
        seg_metrics: &dyn Fn(FontId) -> (f32, f32),
        alignment: TextAlignment,
        overflow: TextOverflow,
        white_space: WhiteSpace,
        tab_width: f32,
    ) -> Self {
        Self::wrap_core(
            runs,
            text,
            max_width,
            strut,
            seg_metrics,
            alignment,
            overflow,
            white_space,
            tab_width,
            0.0,
            0.0,
            &NullHyphenator,
            None,
            None,
            &DefaultLineBreaker,
            OverflowWrap::Normal,
            WordBreak::Normal,
        )
    }

    #[doc = include_str!("paragraph_layout.doc/wrap_strut_hyphenated.md")]
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn wrap_strut_hyphenated(
        runs: &[ShapedRun],
        text: &str,
        max_width: f32,
        strut: LineStrut,
        seg_metrics: &dyn Fn(FontId) -> (f32, f32),
        alignment: TextAlignment,
        overflow: TextOverflow,
        white_space: WhiteSpace,
        tab_width: f32,
        hyphenator: &dyn Hyphenator,
        hyphen_glyph: Option<ShapedGlyph>,
        ellipsis_glyph: Option<ShapedGlyph>,
        overflow_wrap: OverflowWrap,
        word_break: WordBreak,
        line_breaker: &dyn LineBreaker,
    ) -> Self {
        Self::wrap_core(
            runs,
            text,
            max_width,
            strut,
            seg_metrics,
            alignment,
            overflow,
            white_space,
            tab_width,
            0.0,
            0.0,
            hyphenator,
            hyphen_glyph,
            ellipsis_glyph,
            line_breaker,
            overflow_wrap,
            word_break,
        )
    }

    #[doc = include_str!("paragraph_layout.doc/wrap_core.md")]
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    fn wrap_core(
        runs: &[ShapedRun],
        text: &str,
        max_width: f32,
        strut: LineStrut,
        seg_metrics: &dyn Fn(FontId) -> (f32, f32),
        alignment: TextAlignment,
        overflow: TextOverflow,
        white_space: WhiteSpace,
        tab_width: f32,
        letter_spacing: f32,
        word_spacing: f32,
        hyphenator: &dyn Hyphenator,
        hyphen_glyph: Option<ShapedGlyph>,
        ellipsis_glyph: Option<ShapedGlyph>,
        line_breaker: &dyn LineBreaker,
        overflow_wrap: OverflowWrap,
        word_break: WordBreak,
    ) -> Self {
        let mut stream = TaggedGlyph::flatten(runs);

        // CSS Text 3 §4.1.1: in a collapsing white-space mode a collapsible space
        // "immediately following another collapsible space … is collapsed to have
        // zero advance width. (It is invisible, but retains its soft wrap
        // opportunity, if any.)" Only the FIRST space of a run keeps its advance;
        // every following collapsible space in the run is zeroed IN PLACE — its glyph
        // and source cluster (byte offset) are kept so caret hit-testing and
        // selection still see every original character, and deleting a collapsed
        // space from the buffer still shifts every downstream offset. The preserving
        // modes (`pre` / `pre-wrap`) keep every space, so this is inert for them; on
        // the runtime path the pre-shape seam has usually already deleted the run to
        // a single space, making this an idempotent no-op there.
        if white_space.collapses_spaces() {
            let mut prev_collapsible = false;
            for tagged in &mut stream {
                let is_collapsible = matches!(tagged.source_char(text), Some(' ' | '\t'));
                if is_collapsible && prev_collapsible {
                    tagged.glyph = tagged.glyph.with_x_advance(0.0);
                }
                prev_collapsible = is_collapsible;
            }
        }

        // The ordinary break opportunities from the replaceable port (spec §5.10,
        // UAX #14). `word-break: keep-all` (CSS Text 3 §5.2) then suppresses any
        // soft break sitting between two adjacent CJK ideographs, so the letters of
        // CJK-context text are kept together even with the built-in breaker.
        let mut breaks = line_breaker.opportunities(text, white_space);
        if word_break.keeps_all() {
            breaks.retain(|b| !Self::is_keep_all_suppressed(*b, text));
        }

        // What the source text says about each byte offset: where a word may be
        // hyphenated, where the author wrote a soft hyphen, and which collapsible
        // spaces hang at a line's end. Collected once, after the break list is
        // final, and consulted per glyph from here on.
        let offsets = BreakOffsets::collect(text, &breaks, hyphenator, white_space);

        // An author soft hyphen's shaped glyph is forced to zero advance so an
        // untaken soft hyphen is invisible mid-line (spec §5.10).
        if offsets.has_soft_hyphens() {
            for tagged in &mut stream {
                if offsets.is_soft_hyphen(tagged.glyph.cluster()) {
                    tagged.glyph = tagged.glyph.with_x_advance(0.0);
                }
            }
        }

        // A preserved segment break (`\n` under pre / pre-wrap / pre-line, CSS Text
        // 3 §4.1.1) is a *forced* break and is zero-width — like a soft hyphen. If
        // the shaper emitted a glyph for it, zero that glyph's advance so a `\n`
        // trailing on its line never inflates the line width (spec §4.1.1/§7.1).
        if white_space.preserves_segment_breaks() {
            for tagged in &mut stream {
                if tagged.source_char(text) == Some('\n') {
                    tagged.glyph = tagged.glyph.with_x_advance(0.0);
                }
            }
        }

        // A **non-finite** `max_width` (`f32::INFINITY`) is the explicit no-width
        // constraint (intrinsic / max-content measure); a finite `max_width >= 0`
        // is a real box that wraps, so a `0`/tiny width max-wraps toward min-content
        // rather than laying out one overflowing line (CSS 2.1 §10.8).
        let bounded = max_width.is_finite();

        let mut packed_lines = if stream.is_empty() {
            vec![PackedLine::empty()]
        } else if bounded {
            PackedLine::pack(
                &stream,
                max_width,
                &breaks,
                &offsets,
                hyphen_glyph,
                overflow_wrap,
                word_break,
            )
        } else {
            vec![PackedLine::whole(stream)]
        };

        // Re-expand preserved tabs from each visual line's own left edge (CSS Text
        // 3 §4.3): a tab stop is measured from the start of the LINE the tab is on,
        // but the pre-wrap `apply_tab_stops` pass had no line-break info and could
        // only reset its pen at a hard `\n`, so a tab at the start of a
        // soft-wrapped 2nd+ line kept an advance measured from the paragraph
        // origin. This corrects every line's tabs from the line's left edge; a
        // line-0 tab is unchanged (its line start is the paragraph origin). A no-op
        // unless a real stop width is supplied and `text` still holds a `\t`.
        if tab_width > 0.0 && text.contains('\t') {
            PackedLine::retab(&mut packed_lines, text, tab_width);
        }

        // Hang the trailing collapsible whitespace and trailing letter-/word-spacing
        // of every line (CSS Text 3 §7.1, §8.1/§8.2): zero the advances of the
        // line's trailing collapsible-space run, then trim the trailing tracking of
        // the last remaining cluster. This is what excludes the hung advance from a
        // line's used `width()`, its alignment `x_offset`, and justification's base.
        for packed in &mut packed_lines {
            packed.hang_trailing(text, &offsets, letter_spacing, word_spacing);
        }

        // Justify: stretch every eligible line's inter-word gaps so it fills
        // `max_width`. Only meaningful under a positive constraint (the unbounded
        // and empty branches produce a single, non-justified last line).
        if alignment == TextAlignment::Justify && bounded {
            justify_lines(&mut packed_lines, &breaks, max_width);
        }

        // Apply `text-overflow` truncation. Only lines that *still* overflow
        // `max_width` (a lone unbreakable, un-hyphenatable word) are affected, and
        // only under a truncating policy and a positive constraint. Truncation
        // runs after justify — an overflowing line was never justify-eligible
        // (justify only stretches lines with slack), so the two never conflict.
        // Truncation needs a *positive* finite box to clip against; a genuinely
        // 0-width box wraps to min-content (above) but is not clipped to nothing.
        if overflow.is_truncating() && bounded && max_width > 0.0 {
            truncate_lines(&mut packed_lines, max_width, overflow, ellipsis_glyph);
        }

        let text_end = text.len();

        let mut top = 0.0_f32;
        let mut lines = Vec::with_capacity(packed_lines.len());
        for (index, packed) in packed_lines.iter().enumerate() {
            let glyphs = &packed.glyphs;
            let start = glyphs.first().map_or(0, |t| t.glyph.cluster());
            let end = Self::next_line_start(&packed_lines, index, text_end);
            let range = TextRange::new(start.min(end), start.max(end));

            let segments = line_segments(glyphs, end);
            let direction = glyphs
                .first()
                .map_or(TextDirection::LeftToRight, |t| t.direction);
            // CSS 2.1 §10.8.1: size this line box from the strut plus the fonts
            // actually on the line, then stack by its own height.
            let (height, line_baseline) = strut.line_box(glyphs, seg_metrics);
            let line = ParagraphLine::new(segments, range, top, height, line_baseline);
            let x_offset = alignment.x_offset(line.width(), max_width, direction);
            let line = line.with_x_offset(x_offset);
            top += height;
            lines.push(line);
        }

        Self::new(lines, alignment, overflow)
    }

    #[doc = include_str!("paragraph_layout.doc/is_keep_all_suppressed.md")]
    fn is_keep_all_suppressed(break_point: LineBreak, text: &str) -> bool {
        if break_point.is_mandatory() {
            return false;
        }
        let offset = break_point.offset();
        let before = text.get(..offset).and_then(|head| head.chars().next_back());
        let after = text.get(offset..).and_then(|tail| tail.chars().next());
        matches!((before, after), (Some(a), Some(b)) if is_cjk_ideograph(a) && is_cjk_ideograph(b))
    }

    fn next_line_start(packed_lines: &[PackedLine], index: usize, text_end: usize) -> usize {
        packed_lines
            .get(index + 1)
            .and_then(|next| next.glyphs.first())
            .map_or(text_end, |tagged| tagged.glyph.cluster())
    }

    #[doc = include_str!("paragraph_layout.doc/aligned_to.md")]
    #[must_use]
    pub fn aligned_to(self, width: f32) -> Self {
        let alignment = self.alignment;
        let overflow = self.overflow;
        let lines = self
            .lines
            .into_iter()
            .map(|line| {
                let offset = alignment.x_offset(line.width(), width, line.direction());
                line.with_x_offset(offset)
            })
            .collect();
        Self::new(lines, alignment, overflow)
    }

    #[doc = include_str!("paragraph_layout.doc/lines_2.md")]
    #[must_use]
    pub fn lines(&self) -> &[ParagraphLine] {
        &self.lines
    }

    #[doc = include_str!("paragraph_layout.doc/size_2.md")]
    #[must_use]
    pub const fn size(&self) -> Size {
        self.size
    }

    #[doc = include_str!("paragraph_layout.doc/alignment_2.md")]
    #[must_use]
    pub const fn alignment(&self) -> TextAlignment {
        self.alignment
    }

    #[doc = include_str!("paragraph_layout.doc/overflow_2.md")]
    #[must_use]
    pub const fn overflow(&self) -> TextOverflow {
        self.overflow
    }

    #[doc = include_str!("paragraph_layout.doc/line_count.md")]
    #[must_use]
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::shaping::GlyphRun;

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
            WhiteSpace::Normal,
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
        let runs = vec![run_lvl(
            0,
            0,
            &[(0, 6.0), (1, 6.0), (2, 6.0)],
            TextRange::new(0, 3),
        )];
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
            &runs,
            text,
            1000.0,
            12.0,
            9.0,
            TextAlignment::Start,
            TextOverflow::Clip,
            WhiteSpace::Normal,
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
            WhiteSpace::Normal,
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
            WhiteSpace::Normal,
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
        let runs = vec![run(
            0,
            &[(0, 6.0), (1, 6.0), (2, 6.0)],
            TextRange::new(0, 3),
        )];
        let layout = ParagraphLayout::wrap(
            &runs,
            text,
            1000.0,
            12.0,
            9.0,
            TextAlignment::Start,
            TextOverflow::Clip,
            WhiteSpace::Normal,
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
            WhiteSpace::Normal,
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
            WhiteSpace::Normal,
        );
        assert_eq!(one.line_count(), 1);
        assert!(one.lines()[0].is_empty());
    }

    #[test]
    fn unbounded_max_width_is_a_single_line() {
        // The no-constraint sentinel is now a NON-FINITE max_width (`INFINITY`),
        // the intrinsic / max-content measure (CSS 2.1 §10.8) — the whole
        // paragraph is one line. (This test used to pass `0.0` for "unbounded";
        // `0.0` is now a real 0-width box that max-wraps — see `finite_zero_or_tiny
        // _width_wraps_to_min_content`.)
        let text = "ab гд";
        let runs = vec![
            run(0, &[(0, 6.0), (1, 6.0), (2, 6.0)], TextRange::new(0, 3)),
            run(1, &[(3, 8.0), (5, 8.0)], TextRange::new(3, 7)),
        ];
        let layout = ParagraphLayout::wrap(
            &runs,
            text,
            f32::INFINITY,
            12.0,
            9.0,
            TextAlignment::Start,
            TextOverflow::Clip,
            WhiteSpace::Normal,
        );
        assert_eq!(layout.line_count(), 1);
        // Both fonts survive on the one line as two segments.
        assert_eq!(layout.lines()[0].segments().len(), 2);
    }

    #[test]
    fn finite_zero_or_tiny_width_wraps_to_min_content() {
        // Fix 5 (CSS 2.1 §10.8): a *finite* `0` width is a real (empty) box, NOT
        // the no-constraint sentinel, so it max-wraps toward min-content — every
        // word on its own line — instead of one overflowing line. `"a b c"` → 3.
        let text = "a b c";
        let runs = vec![run(
            0,
            &[(0, 6.0), (1, 6.0), (2, 6.0), (3, 6.0), (4, 6.0)],
            TextRange::new(0, 5),
        )];
        let layout = ParagraphLayout::wrap(
            &runs,
            text,
            0.0,
            12.0,
            9.0,
            TextAlignment::Start,
            TextOverflow::Visible,
            WhiteSpace::Normal,
        );
        assert_eq!(layout.line_count(), 3);
    }

    #[test]
    fn nowrap_keeps_one_line_past_max_width() {
        // `nowrap` (CSS Text 3 §3): no soft wrap, so a phrase far wider than the
        // box stays on ONE line (overflow is a separate concern). "ab cd" (five
        // 6px glyphs = 30) in a 12px box.
        let text = "ab cd";
        let runs = vec![run(
            0,
            &[(0, 6.0), (1, 6.0), (2, 6.0), (3, 6.0), (4, 6.0)],
            TextRange::new(0, 5),
        )];
        let layout = ParagraphLayout::wrap(
            &runs,
            text,
            12.0,
            12.0,
            9.0,
            TextAlignment::Start,
            TextOverflow::Visible,
            WhiteSpace::Nowrap,
        );
        assert_eq!(layout.line_count(), 1);
        assert_eq!(layout.lines()[0].width(), 30.0);
    }

    #[test]
    fn pre_forces_two_lines_at_newline_ignoring_width() {
        // `pre` (§3): preserve the segment break as a forced break (two lines) but
        // never soft-wrap. "ab\ncd" in a wide box → two lines split at the `\n`.
        // Clusters skip the `\n` byte (2): 0,1,3,4.
        let text = "ab\ncd";
        let runs = vec![run(
            0,
            &[(0, 6.0), (1, 6.0), (3, 6.0), (4, 6.0)],
            TextRange::new(0, 5),
        )];
        let layout = ParagraphLayout::wrap(
            &runs,
            text,
            100.0,
            12.0,
            9.0,
            TextAlignment::Start,
            TextOverflow::Clip,
            WhiteSpace::Pre,
        );
        assert_eq!(layout.line_count(), 2);
        assert_eq!(layout.lines()[0].top(), 0.0);
        assert_eq!(layout.lines()[1].top(), 12.0);
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
            WhiteSpace::Normal,
        );
        let b = ParagraphLayout::wrap(
            &runs,
            text,
            20.0,
            12.0,
            9.0,
            TextAlignment::Center,
            TextOverflow::Ellipsis,
            WhiteSpace::Normal,
        );
        assert_eq!(a, b);
    }

    // --- CSS Text 3 line-wrap correctness (trailing hang / forced break) --------

    #[test]
    fn trailing_collapsible_space_hangs_and_never_wraps_early() {
        // Fix 1 (CSS Text 3 §3/§7.1): "aa bb cc", 6px glyphs, max_width 35. "aa bb"
        // is 30 wide once the space between "bb" and "cc" hangs, so it FITS line 1 —
        // no premature wrap after "aa". Line 1's used width excludes the hung space.
        let text = "aa bb cc";
        let runs = vec![run(
            0,
            &(0..8).map(|i| (i, 6.0)).collect::<Vec<_>>(),
            TextRange::new(0, 8),
        )];
        let layout = ParagraphLayout::wrap(
            &runs,
            text,
            35.0,
            12.0,
            9.0,
            TextAlignment::Start,
            TextOverflow::Clip,
            WhiteSpace::Normal,
        );
        assert_eq!(layout.line_count(), 2);
        // Line 1 is "aa bb " (clusters 0..5) with the trailing space hung → 30.
        assert_eq!(layout.lines()[0].width(), 30.0);
        assert_eq!(
            line_glyphs(&layout, 0)
                .iter()
                .map(|g| g.0)
                .collect::<Vec<_>>(),
            vec![0, 1, 2, 3, 4, 5]
        );
        assert_eq!(layout.lines()[1].width(), 12.0);
    }

    #[test]
    fn trailing_space_hangs_for_alignment_offset() {
        // Fix 1(c): a right-aligned "aa bb " in a 40px box. The trailing space
        // hangs, so the used width is 30 and x_offset = 40 − 30 = 10 (NOT 4, which
        // is what counting the trailing space would give).
        let text = "aa bb ";
        let runs = vec![run(
            0,
            &(0..6).map(|i| (i, 6.0)).collect::<Vec<_>>(),
            TextRange::new(0, 6),
        )];
        let layout = ParagraphLayout::wrap(
            &runs,
            text,
            40.0,
            12.0,
            9.0,
            TextAlignment::Right,
            TextOverflow::Clip,
            WhiteSpace::Normal,
        );
        assert_eq!(layout.line_count(), 1);
        assert_eq!(layout.lines()[0].width(), 30.0);
        assert_eq!(layout.lines()[0].x_offset(), 10.0);
    }

    #[test]
    fn trailing_forced_break_opens_a_final_empty_line() {
        // Fix 2 (CSS Text 3 §4.1.1): "ab\n" under `pre` is TWO lines — "ab" then an
        // empty last line box — even though the `\n`'s break offset (byte 3) is at
        // text end and no glyph carries it. The empty line is where a caret lives.
        let text = "ab\n";
        // Glyphs for "ab" only; the shaper carried no glyph for the final `\n`.
        let runs = vec![run(0, &[(0, 6.0), (1, 6.0)], TextRange::new(0, 3))];
        for mode in [WhiteSpace::Pre, WhiteSpace::PreWrap] {
            let layout = ParagraphLayout::wrap(
                &runs,
                text,
                100.0,
                12.0,
                9.0,
                TextAlignment::Start,
                TextOverflow::Clip,
                mode,
            );
            assert_eq!(layout.line_count(), 2, "{mode:?}");
            assert_eq!(layout.lines()[0].width(), 12.0);
            assert!(layout.lines()[1].is_empty(), "{mode:?}");
            assert_eq!(layout.lines()[1].top(), 12.0);
        }
    }

    #[test]
    fn preserved_newline_glyph_is_zero_width() {
        // Fix 3: under `pre` the shaped `\n` glyph (cluster 2) keeps a non-zero
        // advance from shaping; a forced break is zero-width, so it is zeroed and
        // never inflates its line. "ab\ncd" (6px) → line 0 width 12, not 18.
        let text = "ab\ncd";
        // Clusters 0,1,2(\n),3,4 — the `\n` glyph IS present, advance 6.
        let runs = vec![run(
            0,
            &[(0, 6.0), (1, 6.0), (2, 6.0), (3, 6.0), (4, 6.0)],
            TextRange::new(0, 5),
        )];
        let layout = ParagraphLayout::wrap(
            &runs,
            text,
            100.0,
            12.0,
            9.0,
            TextAlignment::Start,
            TextOverflow::Clip,
            WhiteSpace::Pre,
        );
        assert_eq!(layout.line_count(), 2);
        // Line 0 is "ab" + the zeroed `\n` → 12, not 18.
        assert_eq!(layout.lines()[0].width(), 12.0);
        assert_eq!(layout.lines()[1].width(), 12.0);
    }

    #[test]
    fn trailing_letter_spacing_hangs_from_line_width() {
        // Fix 4 (CSS Text 3 §8.1): `apply_spacing` adds letter-spacing after EVERY
        // cluster incl. the last, so "ab" with letter-spacing 4 shapes to a=10,
        // b=10 (total 20). The trailing tracking of the last cluster HANGS: the line
        // breaker trims it, so the used line width is 16 (6 + 4 + 6), not 20.
        let text = "ab";
        // Pre-spaced advances (6 + 4 letter-spacing each), as `apply_spacing` emits.
        let runs = vec![run(0, &[(0, 10.0), (1, 10.0)], TextRange::new(0, 2))];
        let strut = LineStrut::new(12.0, 9.0, 3.0);
        let layout = ParagraphLayout::wrap_core(
            &runs,
            text,
            100.0,
            strut,
            &|_| (9.0, 3.0),
            TextAlignment::Start,
            TextOverflow::Clip,
            WhiteSpace::Normal,
            0.0,
            4.0,
            0.0,
            &NullHyphenator,
            None,
            None,
            &DefaultLineBreaker,
            OverflowWrap::Normal,
            WordBreak::Normal,
        );
        assert_eq!(layout.line_count(), 1);
        assert_eq!(layout.lines()[0].width(), 16.0);
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
            &runs,
            text,
            40.0,
            12.0,
            9.0,
            TextAlignment::Justify,
            TextOverflow::Clip,
            WhiteSpace::Normal,
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
            &runs,
            text,
            50.0,
            12.0,
            9.0,
            TextAlignment::Justify,
            TextOverflow::Clip,
            WhiteSpace::Normal,
        );
        assert_eq!(layout.line_count(), 2);
        // The first line "aa bb cc " has two interior gaps AND a trailing space
        // that now HANGS (CSS Text 3 §3/§7.1): its base width is 40 (not 45), so
        // the surplus is 10 split as 5 per gap — widening the two interior space
        // glyphs (clusters 2 and 5) from 5 to 10 each (was 7.5 when the trailing
        // space wrongly counted against the box).
        assert_eq!(layout.lines()[0].width(), 50.0);
        let glyphs = layout.lines()[0].segments()[0].run().glyphs();
        assert_eq!(glyphs[2].x_advance(), 10.0);
        assert_eq!(glyphs[5].x_advance(), 10.0);
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
        // max_width 44 fits "abcd " (32 hung, 40 with the trailing space) but not
        // "+efgh" (72). First line is one word — not justified — so it keeps its
        // natural used width. Its trailing space now HANGS (CSS Text 3 §3/§7.1),
        // so the used width is 32 ("abcd"), not the old 40 that counted the space.
        let layout = ParagraphLayout::wrap(
            &runs,
            text,
            44.0,
            12.0,
            9.0,
            TextAlignment::Justify,
            TextOverflow::Clip,
            WhiteSpace::Normal,
        );
        assert_eq!(layout.line_count(), 2);
        assert_eq!(layout.lines()[0].width(), 32.0);
    }

    #[test]
    fn justify_exempts_hard_broken_line() {
        // "ab cd\nef" under `pre-line` (segment breaks preserved): the first line
        // ends on a mandatory `\n` break, so it is not justified even though it is
        // not the paragraph's last line.
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
            &runs,
            text,
            100.0,
            12.0,
            9.0,
            TextAlignment::Justify,
            TextOverflow::Clip,
            WhiteSpace::PreLine,
        );
        assert_eq!(layout.line_count(), 2);
        // First line "ab cd" ended by a hard break: natural width 5*6 = 30.
        assert_eq!(layout.lines()[0].width(), 30.0);
    }

    #[test]
    fn justify_noop_when_unconstrained() {
        // A non-finite (`INFINITY`) max_width is the no-constraint sentinel and
        // disables justification entirely (one unconstrained last line). Was
        // written with `0.0`, which now means a real 0-width box that wraps.
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
        let layout = ParagraphLayout::wrap(
            &runs,
            text,
            f32::INFINITY,
            12.0,
            9.0,
            TextAlignment::Justify,
            TextOverflow::Clip,
            WhiteSpace::Normal,
        );
        assert_eq!(layout.line_count(), 1);
        assert_eq!(layout.lines()[0].width(), 48.0);
    }

    #[test]
    fn center_and_right_still_use_x_offset() {
        // Non-justify alignments keep offsetting via x_offset and never widen
        // glyphs, so the line width stays natural.
        let text = "ab cd";
        let runs = vec![run(
            0,
            &[(0, 6.0), (1, 6.0), (2, 6.0), (3, 6.0), (4, 6.0)],
            TextRange::new(0, 5),
        )];
        let center = ParagraphLayout::wrap(
            &runs,
            text,
            100.0,
            12.0,
            9.0,
            TextAlignment::Center,
            TextOverflow::Clip,
            WhiteSpace::Normal,
        );
        assert_eq!(center.lines()[0].width(), 30.0);
        assert_eq!(center.lines()[0].x_offset(), 35.0);
        let right = ParagraphLayout::wrap(
            &runs,
            text,
            100.0,
            12.0,
            9.0,
            TextAlignment::Right,
            TextOverflow::Clip,
            WhiteSpace::Normal,
        );
        assert_eq!(right.lines()[0].width(), 30.0);
        assert_eq!(right.lines()[0].x_offset(), 70.0);
    }

    /// A stub [`Hyphenator`] that proposes a single fixed byte `offset` in any
    /// word long enough to contain it.
    struct StubHyphenator {
        offset: usize,
    }

    impl Hyphenator for StubHyphenator {
        fn hyphenate(&self, word: &str) -> Vec<usize> {
            if self.offset < word.len() {
                vec![self.offset]
            } else {
                Vec::new()
            }
        }
    }

    /// Flattens a layout's lines back into `(cluster, advance)` per glyph so a
    /// test can inspect what each wrapped line carries.
    fn line_glyphs(layout: &ParagraphLayout, line: usize) -> Vec<(usize, f32, u32)> {
        layout.lines()[line]
            .segments()
            .iter()
            .flat_map(|s| s.run().glyphs().iter())
            .map(|g| (g.cluster(), g.x_advance(), g.glyph_id()))
            .collect()
    }

    #[test]
    fn null_hyphenator_wrap_hyphenated_equals_wrap() {
        // Requirement (a): the no-op hyphenator with no hyphen glyph reproduces
        // plain `wrap` exactly, across a multi-line greedy wrap.
        let text = "aaaa bbbb cccc";
        let runs = vec![run(
            0,
            &(0..14).map(|i| (i, 6.0)).collect::<Vec<_>>(),
            TextRange::new(0, 14),
        )];
        let plain = ParagraphLayout::wrap(
            &runs,
            text,
            50.0,
            12.0,
            9.0,
            TextAlignment::Start,
            TextOverflow::Clip,
            WhiteSpace::Normal,
        );
        let hyphenated = ParagraphLayout::wrap_hyphenated(
            &runs,
            text,
            50.0,
            12.0,
            9.0,
            TextAlignment::Start,
            TextOverflow::Clip,
            WhiteSpace::Normal,
            &NullHyphenator,
            None,
            None,
        );
        assert_eq!(plain, hyphenated);
    }

    #[test]
    fn stub_hyphenator_breaks_over_wide_word_with_visible_hyphen() {
        // Requirement (b): "hyphenate" (nine 6px glyphs = 54) in a 40px box, with
        // a stub proposing a break before byte 5 ("hyphe|nate") and a 4px hyphen
        // glyph (id 999). The head "hyphe" + hyphen (34px) fits 40; the tail
        // "nate" starts the next line.
        let text = "hyphenate";
        let runs = vec![run(
            0,
            &(0..9).map(|i| (i, 6.0)).collect::<Vec<_>>(),
            TextRange::new(0, 9),
        )];
        let hyphen = ShapedGlyph::new(999, 0, 4.0);
        let layout = ParagraphLayout::wrap_hyphenated(
            &runs,
            text,
            40.0,
            12.0,
            9.0,
            TextAlignment::Start,
            TextOverflow::Clip,
            WhiteSpace::Normal,
            &StubHyphenator { offset: 5 },
            Some(hyphen),
            None,
        );
        assert_eq!(layout.line_count(), 2);

        // Head line: five source glyphs plus the hyphen, width 34 ≤ 40.
        let head = line_glyphs(&layout, 0);
        assert_eq!(head.len(), 6);
        assert_eq!(layout.lines()[0].width(), 34.0);
        assert!(layout.lines()[0].width() <= 40.0);
        // The head ends with the hyphen glyph, which carries the break cluster (5).
        assert_eq!(head[5], (5, 4.0, 999));
        // The five source glyphs precede it, clusters 0..4.
        assert_eq!(
            head[..5].iter().map(|g| g.0).collect::<Vec<_>>(),
            vec![0, 1, 2, 3, 4]
        );

        // Tail line: "nate" (clusters 5..8), no hyphen, width 24.
        let tail = line_glyphs(&layout, 1);
        assert_eq!(
            tail.iter().map(|g| g.0).collect::<Vec<_>>(),
            vec![5, 6, 7, 8]
        );
        assert_eq!(layout.lines()[1].width(), 24.0);
    }

    #[test]
    fn soft_hyphen_breaks_when_over_wide_and_is_invisible_otherwise() {
        // Requirement (c): "aa\u{AD}bb" carries an author soft hyphen at byte 2.
        // Glyph clusters are 0,1,2(soft),4,5 (the soft hyphen is two bytes).
        let text = "aa\u{AD}bb";
        let runs = vec![run(
            0,
            &[(0, 6.0), (1, 6.0), (2, 6.0), (4, 6.0), (5, 6.0)],
            TextRange::new(0, 6),
        )];

        // Wide box: no break taken, the soft-hyphen glyph is zero-advance and no
        // hyphen renders. One line of width 24 (four visible 6px glyphs).
        let wide = ParagraphLayout::wrap_hyphenated(
            &runs,
            text,
            100.0,
            12.0,
            9.0,
            TextAlignment::Start,
            TextOverflow::Clip,
            WhiteSpace::Normal,
            &NullHyphenator,
            Some(ShapedGlyph::new(999, 0, 4.0)),
            None,
        );
        assert_eq!(wide.line_count(), 1);
        let one = line_glyphs(&wide, 0);
        // The soft-hyphen glyph (cluster 2) survives but is invisible (0 advance).
        assert_eq!(one.iter().find(|g| g.0 == 2).map(|g| g.1), Some(0.0));
        // No hyphen glyph (id 999) is present mid-line.
        assert!(!one.iter().any(|g| g.2 == 999));
        assert_eq!(wide.lines()[0].width(), 24.0);

        // Narrow box (20px): the soft hyphen is taken as the break. Head "aa" +
        // hyphen (16px), tail "bb" on the next line; the soft glyph is dropped.
        let narrow = ParagraphLayout::wrap_hyphenated(
            &runs,
            text,
            20.0,
            12.0,
            9.0,
            TextAlignment::Start,
            TextOverflow::Clip,
            WhiteSpace::Normal,
            &NullHyphenator,
            Some(ShapedGlyph::new(999, 0, 4.0)),
            None,
        );
        assert_eq!(narrow.line_count(), 2);
        let head = line_glyphs(&narrow, 0);
        // Head: two source glyphs (0,1) then the hyphen (id 999, cluster 2).
        assert_eq!(head.iter().map(|g| g.0).collect::<Vec<_>>(), vec![0, 1, 2]);
        assert_eq!(head[2], (2, 4.0, 999));
        assert_eq!(narrow.lines()[0].width(), 16.0);
        // Tail: "bb" (clusters 4,5); the invisible soft-hyphen glyph was dropped.
        let tail = line_glyphs(&narrow, 1);
        assert_eq!(tail.iter().map(|g| g.0).collect::<Vec<_>>(), vec![4, 5]);
    }

    #[test]
    fn over_wide_word_without_fitting_break_takes_its_own_line() {
        // A stub whose only break (byte 1) leaves too wide a head still overflows
        // gracefully: no split fits, so the whole word takes its own line as
        // before hyphenation existed.
        let text = "wwwww";
        let runs = vec![run(
            0,
            &[(0, 20.0), (1, 20.0), (2, 20.0), (3, 20.0), (4, 20.0)],
            TextRange::new(0, 5),
        )];
        // Only break is before byte 1: head "w" (20) + hyphen (4) = 24 > 10, so
        // no opportunity fits and the word is placed whole. `Visible` overflow
        // isolates the packing fallback (a truncating policy would clip it).
        let layout = ParagraphLayout::wrap_hyphenated(
            &runs,
            text,
            10.0,
            12.0,
            9.0,
            TextAlignment::Start,
            TextOverflow::Visible,
            WhiteSpace::Normal,
            &StubHyphenator { offset: 1 },
            Some(ShapedGlyph::new(999, 0, 4.0)),
            None,
        );
        assert_eq!(layout.line_count(), 1);
        assert_eq!(line_glyphs(&layout, 0).len(), 5);
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
            WhiteSpace::Normal,
        );
        assert_eq!(layout.alignment(), TextAlignment::Right);
        assert_eq!(layout.overflow(), TextOverflow::Ellipsis);
    }

    /// Wraps a single unbreakable `n`-glyph word (each 10px, no break
    /// opportunities) at `max_width` under `overflow`, with an optional ellipsis
    /// glyph, and no hyphenation.
    fn truncated(
        n: usize,
        max_width: f32,
        alignment: TextAlignment,
        overflow: TextOverflow,
        ellipsis: Option<ShapedGlyph>,
    ) -> ParagraphLayout {
        // "wwww…" — one word, so `LineBreak` proposes no interior opportunity and
        // the word cannot be packed onto multiple lines.
        let text: String = "w".repeat(n);
        let runs = vec![run(
            0,
            &(0..n).map(|i| (i, 10.0)).collect::<Vec<_>>(),
            TextRange::new(0, n),
        )];
        ParagraphLayout::wrap_hyphenated(
            &runs,
            &text,
            max_width,
            12.0,
            9.0,
            alignment,
            overflow,
            WhiteSpace::Normal,
            &NullHyphenator,
            None,
            ellipsis,
        )
    }

    #[test]
    fn clip_drops_overflowing_glyphs_so_line_fits() {
        // Five 10px glyphs (50px) in a 25px box: Clip keeps the largest prefix
        // whose width ≤ 25 (two glyphs = 20) and drops the rest, adding nothing.
        let layout = truncated(5, 25.0, TextAlignment::Start, TextOverflow::Clip, None);
        assert_eq!(layout.line_count(), 1);
        assert!(layout.lines()[0].width() <= 25.0);
        let kept = line_glyphs(&layout, 0);
        assert_eq!(kept.iter().map(|g| g.0).collect::<Vec<_>>(), vec![0, 1]);
    }

    #[test]
    fn ellipsis_keeps_prefix_plus_ellipsis_and_ends_with_it() {
        // A 5px ellipsis glyph (id 888) reserves its advance, so the prefix must
        // fit 25 − 5 = 20 (two glyphs); the appended ellipsis carries the last
        // kept glyph's cluster (1) and the total width (25) fits the box.
        let ellipsis = ShapedGlyph::new(888, 0, 5.0);
        let layout = truncated(
            5,
            25.0,
            TextAlignment::Start,
            TextOverflow::Ellipsis,
            Some(ellipsis),
        );
        assert_eq!(layout.line_count(), 1);
        assert!(layout.lines()[0].width() <= 25.0);
        let kept = line_glyphs(&layout, 0);
        // Two source glyphs (clusters 0,1) then the ellipsis (id 888, cluster 1).
        assert_eq!(kept.len(), 3);
        assert_eq!(
            kept[..2].iter().map(|g| g.0).collect::<Vec<_>>(),
            vec![0, 1]
        );
        assert_eq!(kept[2], (1, 5.0, 888));
    }

    #[test]
    fn ellipsis_without_glyph_behaves_like_clip() {
        // No ellipsis glyph supplied: Ellipsis degrades to Clip (no glyph added).
        let layout = truncated(5, 25.0, TextAlignment::Start, TextOverflow::Ellipsis, None);
        let kept = line_glyphs(&layout, 0);
        assert_eq!(kept.iter().map(|g| g.0).collect::<Vec<_>>(), vec![0, 1]);
        assert!(layout.lines()[0].width() <= 25.0);
    }

    #[test]
    fn within_width_line_is_untouched_by_truncation() {
        // Two 10px glyphs (20px) in a 100px box: nothing overflows, so Clip and
        // Ellipsis leave the line exactly as wrapped.
        let clip = truncated(2, 100.0, TextAlignment::Start, TextOverflow::Clip, None);
        assert_eq!(line_glyphs(&clip, 0).len(), 2);
        assert_eq!(clip.lines()[0].width(), 20.0);
        let ellipsis = truncated(
            2,
            100.0,
            TextAlignment::Start,
            TextOverflow::Ellipsis,
            Some(ShapedGlyph::new(888, 0, 5.0)),
        );
        assert_eq!(line_glyphs(&ellipsis, 0).len(), 2);
        assert_eq!(ellipsis.lines()[0].width(), 20.0);
    }

    #[test]
    fn non_truncating_and_unconstrained_do_not_truncate() {
        // Visible policy leaves an over-wide line intact (all five glyphs).
        let visible = truncated(5, 25.0, TextAlignment::Start, TextOverflow::Visible, None);
        assert_eq!(line_glyphs(&visible, 0).len(), 5);
        assert_eq!(visible.lines()[0].width(), 50.0);
        // Unconstrained width (the non-finite `INFINITY` sentinel) is a single,
        // never-truncated line even under a truncating policy. (Was `0.0`, which is
        // now a finite 0-width box that clips.)
        let unconstrained = truncated(
            5,
            f32::INFINITY,
            TextAlignment::Start,
            TextOverflow::Clip,
            None,
        );
        assert_eq!(line_glyphs(&unconstrained, 0).len(), 5);
    }

    #[test]
    fn a_truncated_line_is_not_justified() {
        // Even under Justify, a truncated (overflowing) line keeps its glyphs'
        // natural advances — justification never stretches it, because an
        // overflowing line has no slack to distribute.
        let layout = truncated(5, 25.0, TextAlignment::Justify, TextOverflow::Clip, None);
        assert!(layout.lines()[0].width() <= 25.0);
        let kept = line_glyphs(&layout, 0);
        assert_eq!(kept.iter().map(|g| g.0).collect::<Vec<_>>(), vec![0, 1]);
        // No advance was widened by justification.
        assert!(kept.iter().all(|g| (g.1 - 10.0).abs() < f32::EPSILON));
    }

    // --- CSS 2.1 §10.8.1 mixed-font line-box strut ---------------------------

    /// A test-double per-font metrics lookup: font 1 is a TALLER fallback face
    /// (ascent 18, descent 6 → content 24); every other font is the primary
    /// (ascent 9, descent 3 → content 12). Stands in for the uniform mock, which
    /// cannot exercise the max-over-fonts strut aggregation.
    fn tall_fallback(font: FontId) -> (f32, f32) {
        if font.get() == 1 {
            (18.0, 6.0)
        } else {
            (9.0, 3.0)
        }
    }

    #[test]
    fn strut_uses_content_box_when_line_height_is_smaller() {
        // L < A+D: the content box wins (no leading). A tall fallback segment
        // (font 1: 18/6) with a short used line-height (12): A=max(9,18)=18,
        // D=max(3,6)=6, content=24 > L=12 → height=24, baseline=A=18.
        let text = "ab";
        let runs = vec![
            run(0, &[(0, 6.0)], TextRange::new(0, 1)),
            run(1, &[(1, 6.0)], TextRange::new(1, 2)),
        ];
        let layout = ParagraphLayout::wrap_strut(
            &runs,
            text,
            1000.0,
            LineStrut::new(12.0, 9.0, 3.0),
            &tall_fallback,
            TextAlignment::Start,
            TextOverflow::Clip,
            WhiteSpace::Normal,
            0.0,
        );
        assert_eq!(layout.line_count(), 1);
        assert_eq!(layout.lines()[0].height(), 24.0);
        assert_eq!(layout.lines()[0].baseline(), 18.0);
    }

    #[test]
    fn strut_distributes_half_leading_when_line_height_is_larger() {
        // L > A+D: the excess splits as half-leading. Primary-only line, strut
        // ascent 9/descent 3 (content 12), used line-height 20: half=(20-12)/2=4,
        // height=20, baseline=9+4=13.
        let text = "a";
        let runs = vec![run(0, &[(0, 6.0)], TextRange::new(0, 1))];
        let layout = ParagraphLayout::wrap_strut(
            &runs,
            text,
            1000.0,
            LineStrut::new(20.0, 9.0, 3.0),
            &|_| (9.0, 3.0),
            TextAlignment::Start,
            TextOverflow::Clip,
            WhiteSpace::Normal,
            0.0,
        );
        assert_eq!(layout.lines()[0].height(), 20.0);
        assert_eq!(layout.lines()[0].baseline(), 13.0);
    }

    #[test]
    fn tall_fallback_segment_grows_only_its_line_and_the_paragraph() {
        // "ab гд": font 0 "ab " (primary, bytes 0,1,2=space) then font 1 "гд"
        // (Cyrillic fallback, bytes 3,5). max_width 20 fits "ab " (18) but not
        // "+гд" (34), so it wraps: line 0 primary-only, line 1 the tall fallback.
        let text = "ab гд";
        let runs = vec![
            run(0, &[(0, 6.0), (1, 6.0), (2, 6.0)], TextRange::new(0, 3)),
            run(1, &[(3, 8.0), (5, 8.0)], TextRange::new(3, 7)),
        ];
        let strut = LineStrut::new(12.0, 9.0, 3.0);
        let layout = ParagraphLayout::wrap_strut(
            &runs,
            text,
            20.0,
            strut,
            &tall_fallback,
            TextAlignment::Start,
            TextOverflow::Clip,
            WhiteSpace::Normal,
            0.0,
        );
        assert_eq!(layout.line_count(), 2);
        // Primary-only line: the strut box is unchanged (height 12, baseline 9).
        assert_eq!(layout.lines()[0].height(), 12.0);
        assert_eq!(layout.lines()[0].baseline(), 9.0);
        // The fallback line grows: A=18, D=6, content=24 > L=12 → height 24,
        // baseline 18 (lower than the primary line's 9).
        assert_eq!(layout.lines()[1].height(), 24.0);
        assert_eq!(layout.lines()[1].baseline(), 18.0);
        assert!(layout.lines()[1].baseline() > layout.lines()[0].baseline());
        // Lines stack by their OWN heights: the tall line sits below the short one
        // and the paragraph height is the sum, not N·line_height.
        assert_eq!(layout.lines()[0].top(), 0.0);
        assert_eq!(layout.lines()[1].top(), 12.0);
        assert_eq!(layout.size().height, 36.0);

        // Contrast: a font-agnostic (uniform) lookup — the mock's own case —
        // leaves BOTH lines at the strut box, so the paragraph stays 24 tall. The
        // growth is entirely the fallback face's taller metrics.
        let uniform = ParagraphLayout::wrap_strut(
            &runs,
            text,
            20.0,
            strut,
            &|_| (9.0, 3.0),
            TextAlignment::Start,
            TextOverflow::Clip,
            WhiteSpace::Normal,
            0.0,
        );
        assert_eq!(uniform.lines()[0].height(), 12.0);
        assert_eq!(uniform.lines()[1].height(), 12.0);
        assert_eq!(uniform.size().height, 24.0);
    }

    #[test]
    fn empty_line_still_gets_the_strut_box() {
        // The strut is always present: an empty paragraph keeps the element's own
        // primary-font box (height 12, baseline 9) even with a tall fallback in
        // the lookup (nothing on the line to trigger it).
        let layout = ParagraphLayout::wrap_strut(
            &[],
            "",
            40.0,
            LineStrut::new(12.0, 9.0, 3.0),
            &tall_fallback,
            TextAlignment::Start,
            TextOverflow::Clip,
            WhiteSpace::Normal,
            0.0,
        );
        assert_eq!(layout.line_count(), 1);
        assert!(layout.lines()[0].is_empty());
        assert_eq!(layout.lines()[0].height(), 12.0);
        assert_eq!(layout.lines()[0].baseline(), 9.0);
    }

    #[test]
    fn legacy_wrap_equals_uniform_strut_wrap_byte_identical() {
        // The byte-identical guard: the historical scalar `wrap` (line_height 12,
        // baseline 9) is exactly `wrap_strut` with the equivalent uniform strut
        // (ascent 9, descent 12-9=3) and a font-agnostic lookup — for a MULTI-font,
        // multi-line input, proving the seam adds no drift under the uniform mock.
        let text = "ab гд";
        let runs = vec![
            run(0, &[(0, 6.0), (1, 6.0), (2, 6.0)], TextRange::new(0, 3)),
            run(1, &[(3, 8.0), (5, 8.0)], TextRange::new(3, 7)),
        ];
        let legacy = ParagraphLayout::wrap(
            &runs,
            text,
            20.0,
            12.0,
            9.0,
            TextAlignment::Start,
            TextOverflow::Clip,
            WhiteSpace::Normal,
        );
        let strut = ParagraphLayout::wrap_strut(
            &runs,
            text,
            20.0,
            LineStrut::new(12.0, 9.0, 3.0),
            &|_| (9.0, 3.0),
            TextAlignment::Start,
            TextOverflow::Clip,
            WhiteSpace::Normal,
            0.0,
        );
        assert_eq!(legacy, strut);
    }

    /// Flattens every glyph advance of one laid-out line, in visual segment order.
    fn line_advances(layout: &ParagraphLayout, line: usize) -> Vec<f32> {
        layout.lines()[line]
            .segments()
            .iter()
            .flat_map(|s| s.run().glyphs().iter())
            .map(ShapedGlyph::x_advance)
            .collect()
    }

    #[test]
    fn preserved_tab_on_a_wrapped_line_measures_from_that_line_start() {
        // CSS Text 3 §4.3: a tab stop is measured from the start of the LINE the tab
        // is on. "aa bb\t" (pre-wrap) soft-wraps to "aa " / "bb\t"; the pre-shape
        // `apply_tab_stops` pass measures the tab from the PARAGRAPH origin (pen 30
        // = a+a+space+b+b) → advance 40−30 = 10, which is what the input runs carry.
        // After wrapping puts the tab on the 2nd line, its stop must be re-measured
        // from that line's left edge (pen 12 = b+b) → advance 40−12 = 28.
        let text = "aa bb\t";
        // Runs exactly as `apply_tab_stops` (paragraph origin) would hand them to the
        // wrapper: the tab already carries its 10px paragraph-origin advance.
        let runs = vec![run(
            0,
            &[(0, 6.0), (1, 6.0), (2, 6.0), (3, 6.0), (4, 6.0), (5, 10.0)],
            TextRange::new(0, 6),
        )];
        let strut = LineStrut::new(12.0, 9.0, 3.0);

        // max_width 30 fits "aa " (18) but not the whole run, so "bb\t" wraps to
        // line 1 with the tab following "bb".
        let wrapped = ParagraphLayout::wrap_strut(
            &runs,
            text,
            30.0,
            strut,
            &|_| (9.0, 3.0),
            TextAlignment::Start,
            // `Visible`: the re-expanded tab makes line 1 (40) overflow the 30 box;
            // under `Clip` that would drop the over-wide tab, hiding the advance we
            // assert. Visible keeps it (overflow/clip is a separate, tested concern).
            TextOverflow::Visible,
            WhiteSpace::PreWrap,
            40.0,
        );
        assert_eq!(wrapped.line_count(), 2);
        // Line 0 "aa " is untouched (no tab, and it is line 0).
        assert_eq!(line_advances(&wrapped, 0), vec![6.0, 6.0, 6.0]);
        // Line 1 "bb\t": the tab is re-measured from the line start (pen 12) → 28,
        // NOT the 10 it carried from the paragraph origin.
        assert_eq!(line_advances(&wrapped, 1), vec![6.0, 6.0, 28.0]);
        assert_eq!(wrapped.lines()[1].width(), 40.0);

        // Byte-identical guard: unwrapped (wide box, one line) the SAME tab stays on
        // line 0, whose start IS the paragraph origin, so the re-expansion reproduces
        // the pre-wrap advance (10) exactly — the common single-line case is unchanged.
        let one_line = ParagraphLayout::wrap_strut(
            &runs,
            text,
            1000.0,
            strut,
            &|_| (9.0, 3.0),
            TextAlignment::Start,
            TextOverflow::Visible,
            WhiteSpace::PreWrap,
            40.0,
        );
        assert_eq!(one_line.line_count(), 1);
        assert_eq!(
            line_advances(&one_line, 0),
            vec![6.0, 6.0, 6.0, 6.0, 6.0, 10.0]
        );
    }

    // --- CSS Text 3 §5.2 overflow-wrap / word-break emergency breaks -----------

    /// Wraps a single-font `runs`/`text` at `max_width` with the given §5.2
    /// controls (through the default breaker, no hyphen/ellipsis), for the
    /// emergency-break tests.
    fn wrap_break(
        runs: &[ShapedRun],
        text: &str,
        max_width: f32,
        overflow: TextOverflow,
        overflow_wrap: OverflowWrap,
        word_break: WordBreak,
    ) -> ParagraphLayout {
        ParagraphLayout::wrap_strut_hyphenated(
            runs,
            text,
            max_width,
            LineStrut::new(12.0, 9.0, 3.0),
            &|_| (9.0, 3.0),
            TextAlignment::Start,
            overflow,
            WhiteSpace::Normal,
            0.0,
            &NullHyphenator,
            None,
            None,
            overflow_wrap,
            word_break,
            &DefaultLineBreaker,
        )
    }

    #[test]
    fn overflow_wrap_break_word_wraps_a_long_token_at_grapheme_boundaries() {
        // Requirement (a): a 12-glyph no-space token (advance 1 each) in a 5-wide
        // box wraps across THREE lines at grapheme boundaries — 5 + 5 + 2 —
        // instead of overflowing on one line, because `overflow-wrap: break-word`
        // permits a last-resort break of a word that does not fit its own line.
        let text = "aaaaaaaaaaaa";
        let runs = vec![run(
            0,
            &(0..12).map(|i| (i, 1.0)).collect::<Vec<_>>(),
            TextRange::new(0, 12),
        )];
        let layout = wrap_break(
            &runs,
            text,
            5.0,
            TextOverflow::Visible,
            OverflowWrap::BreakWord,
            WordBreak::Normal,
        );
        assert_eq!(layout.line_count(), 3);
        assert_eq!(
            line_glyphs(&layout, 0)
                .iter()
                .map(|g| g.0)
                .collect::<Vec<_>>(),
            vec![0, 1, 2, 3, 4]
        );
        assert_eq!(
            line_glyphs(&layout, 1)
                .iter()
                .map(|g| g.0)
                .collect::<Vec<_>>(),
            vec![5, 6, 7, 8, 9]
        );
        assert_eq!(
            line_glyphs(&layout, 2)
                .iter()
                .map(|g| g.0)
                .collect::<Vec<_>>(),
            vec![10, 11]
        );
        // No hyphen glyph is inserted by an emergency break.
        assert!(
            layout
                .lines()
                .iter()
                .flat_map(|l| l.segments())
                .flat_map(|s| s.run().glyphs())
                .all(|g| g.glyph_id() == 1)
        );
    }

    #[test]
    fn word_break_break_all_breaks_a_latin_word_mid_word_greedily() {
        // Requirement (b): `word-break: break-all` fills each line by splitting a
        // Latin word mid-word at any grapheme boundary. Eight 2px glyphs in a 6px
        // box → [0,1,2] / [3,4,5] / [6,7].
        let text = "abcdefgh";
        let runs = vec![run(
            0,
            &(0..8).map(|i| (i, 2.0)).collect::<Vec<_>>(),
            TextRange::new(0, 8),
        )];
        let layout = wrap_break(
            &runs,
            text,
            6.0,
            TextOverflow::Visible,
            OverflowWrap::Normal,
            WordBreak::BreakAll,
        );
        assert_eq!(layout.line_count(), 3);
        assert_eq!(
            line_glyphs(&layout, 0)
                .iter()
                .map(|g| g.0)
                .collect::<Vec<_>>(),
            vec![0, 1, 2]
        );
        assert_eq!(
            line_glyphs(&layout, 1)
                .iter()
                .map(|g| g.0)
                .collect::<Vec<_>>(),
            vec![3, 4, 5]
        );
        assert_eq!(
            line_glyphs(&layout, 2)
                .iter()
                .map(|g| g.0)
                .collect::<Vec<_>>(),
            vec![6, 7]
        );
    }

    #[test]
    fn break_all_fills_the_remaining_space_on_a_partial_line() {
        // `break-all` (§5.2) fills the CURRENT line mid-word rather than moving the
        // whole word down: "ab" (2 glyphs) then a 4-glyph word in a 4px box (1px
        // glyphs). Line 0 packs "ab" + the first 2 graphemes of the next word.
        let text = "ab cdef";
        let runs = vec![run(
            0,
            &[
                (0, 1.0),
                (1, 1.0),
                (2, 1.0),
                (3, 1.0),
                (4, 1.0),
                (5, 1.0),
                (6, 1.0),
            ],
            TextRange::new(0, 7),
        )];
        let layout = wrap_break(
            &runs,
            text,
            4.0,
            TextOverflow::Visible,
            OverflowWrap::Normal,
            WordBreak::BreakAll,
        );
        // Line 0 = "ab c" (clusters 0,1,2=space,3): the space hangs, so "ab" + "c"
        // fill the 4px line with the second word starting mid-line.
        assert_eq!(
            line_glyphs(&layout, 0)
                .iter()
                .map(|g| g.0)
                .collect::<Vec<_>>(),
            vec![0, 1, 2, 3]
        );
        assert!(layout.line_count() >= 2);
    }

    #[test]
    fn default_normal_lets_an_over_wide_token_overflow_one_line() {
        // Requirement (c): with the defaults (`overflow-wrap: normal` +
        // `word-break: normal`) the same 12-glyph token stays on ONE overflowing
        // line — byte-identical to the historical behaviour.
        let text = "aaaaaaaaaaaa";
        let runs = vec![run(
            0,
            &(0..12).map(|i| (i, 1.0)).collect::<Vec<_>>(),
            TextRange::new(0, 12),
        )];
        let layout = wrap_break(
            &runs,
            text,
            5.0,
            TextOverflow::Visible,
            OverflowWrap::Normal,
            WordBreak::Normal,
        );
        assert_eq!(layout.line_count(), 1);
        assert_eq!(layout.lines()[0].width(), 12.0);
        // Byte-identical to plain `wrap` (which also uses the default breaker/modes).
        let plain = ParagraphLayout::wrap(
            &runs,
            text,
            5.0,
            12.0,
            9.0,
            TextAlignment::Start,
            TextOverflow::Visible,
            WhiteSpace::Normal,
        );
        assert_eq!(layout, plain);
    }

    #[test]
    fn a_single_grapheme_wider_than_the_box_still_overflows() {
        // Emergency breaking guarantees progress: a lone 8px glyph in a 5px box
        // overflows by that one grapheme rather than looping. Two such glyphs → two
        // one-glyph lines under `break-word`.
        let text = "WW";
        let runs = vec![run(0, &[(0, 8.0), (1, 8.0)], TextRange::new(0, 2))];
        let layout = wrap_break(
            &runs,
            text,
            5.0,
            TextOverflow::Visible,
            OverflowWrap::BreakWord,
            WordBreak::Normal,
        );
        assert_eq!(layout.line_count(), 2);
        assert_eq!(layout.lines()[0].width(), 8.0);
        assert_eq!(layout.lines()[1].width(), 8.0);
    }

    #[test]
    fn default_breaker_wraps_unspaced_cjk_between_ideographs() {
        // Basic CJK out of the box: "中文字对" (4 ideographs, 3 bytes each,
        // clusters 0,3,6,9; advance 6) in a 13px box wraps to "中文" / "字对" via
        // the default breaker's inter-ideograph soft breaks — no adapter needed.
        let text = "中文字对";
        let runs = vec![run(
            0,
            &[(0, 6.0), (3, 6.0), (6, 6.0), (9, 6.0)],
            TextRange::new(0, 12),
        )];
        let layout = ParagraphLayout::wrap(
            &runs,
            text,
            13.0,
            12.0,
            9.0,
            TextAlignment::Start,
            TextOverflow::Visible,
            WhiteSpace::Normal,
        );
        assert_eq!(layout.line_count(), 2);
        assert_eq!(
            line_glyphs(&layout, 0)
                .iter()
                .map(|g| g.0)
                .collect::<Vec<_>>(),
            vec![0, 3]
        );
        assert_eq!(
            line_glyphs(&layout, 1)
                .iter()
                .map(|g| g.0)
                .collect::<Vec<_>>(),
            vec![6, 9]
        );
    }

    #[test]
    fn keep_all_suppresses_cjk_breaks_so_ideographs_stay_together() {
        // `word-break: keep-all` (§5.2) drops the inter-ideograph opportunities, so
        // the same CJK run stays on ONE (overflowing) line.
        let text = "中文字对";
        let runs = vec![run(
            0,
            &[(0, 6.0), (3, 6.0), (6, 6.0), (9, 6.0)],
            TextRange::new(0, 12),
        )];
        let layout = wrap_break(
            &runs,
            text,
            13.0,
            TextOverflow::Visible,
            OverflowWrap::Normal,
            WordBreak::KeepAll,
        );
        assert_eq!(layout.line_count(), 1);
        assert_eq!(layout.lines()[0].width(), 24.0);
    }
}
