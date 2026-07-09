//! The full laid-out multi-font paragraph: lines stacked vertically (spec §5.10).

use std::collections::{HashSet, VecDeque};

use obscur_geometry::Size;

use crate::font::FontId;
use crate::shaping::{GlyphRun, ShapedGlyph, ShapedRun, TextDirection};
use crate::span::TextRange;

use super::{
    Hyphenator, LineBreak, NullHyphenator, ParagraphLine, TextAlignment, TextOverflow, WhiteSpace,
};

/// The inline **line-box strut** plus the used `line-height` (CSS 2.1 §10.8.1 /
/// CSS Inline 3 §5).
///
/// Per CSS 2.1 §10.8 each line box is sized from a *strut* — an imaginary
/// zero-width inline box with the element's own (primary) font metrics — combined
/// with the inline content actually on the line. The strut is **always present**,
/// even on an empty line, so a line never collapses below the element's own font
/// height. `ascent`/`descent` are that primary face's vertical metrics (the
/// descent carries everything below the baseline, including line-gap, so
/// `ascent + descent` is the primary content box); `line_height` is the CSS
/// *used* `line-height` L, into which any excess over the line's content box is
/// distributed as half-leading (§10.8.1).
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
}

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
    /// Whether the segment this glyph belongs to is faux-sheared (spec §5.18).
    synth_oblique: bool,
    /// Whether the segment this glyph belongs to is faux-bolded (spec §5.18).
    synth_bold: bool,
    glyph: ShapedGlyph,
}

/// A word: the tagged glyphs between two adjacent break opportunities, its
/// advance width, whether a mandatory break follows it, and the interior
/// hyphenation points at which it may be split. A word may contain glyphs from
/// more than one font.
struct Word {
    glyphs: Vec<Tagged>,
    width: f32,
    /// The advance of the word's trailing run of *collapsible* whitespace glyphs
    /// (CSS Text 3 §3/§7.1). A word owns the space that follows it (the soft break
    /// sits before the next word), so this is the part of `width` that **hangs**
    /// when the word ends a line: it is excluded from the fit test and zeroed out
    /// of the line's used width. `0.0` when the word has no trailing collapsible
    /// space (or the mode preserves spaces).
    trailing_ws: f32,
    mandatory_break_after: bool,
    /// The in-word split points (from the hyphenator or author soft hyphens),
    /// ascending by [`head_width`](HyphenPoint::head_width).
    hyphens: Vec<HyphenPoint>,
}

/// One interior hyphenation opportunity inside a [`Word`]: where the head ends,
/// where the tail resumes, the head's advance width, and the source byte a
/// rendered hyphen glyph maps back to.
///
/// A break splits the word into `glyphs[0..head_end]` (the head, which may gain
/// a trailing hyphen glyph) and `glyphs[tail_start..]` (the tail). For an
/// algorithmic (interior) break `tail_start == head_end`; for an author soft
/// hyphen (`U+00AD`) `tail_start == head_end + 1`, so the invisible soft-hyphen
/// glyph is dropped at the break and replaced by the rendered hyphen.
#[derive(Clone, Copy)]
struct HyphenPoint {
    /// Exclusive glyph index the head runs up to.
    head_end: usize,
    /// Glyph index the tail resumes at (`head_end`, or `head_end + 1` for a
    /// soft hyphen whose glyph is dropped).
    tail_start: usize,
    /// The advance width of the head glyphs (`glyphs[0..head_end]`).
    head_width: f32,
    /// The source byte offset the rendered hyphen glyph carries as its cluster.
    break_cluster: usize,
}

/// The result of breaking a [`Word`] at a hyphenation point: the head glyphs
/// (with an appended hyphen when one was supplied) and the remaining tail.
struct Split {
    head: Vec<Tagged>,
    tail: Word,
}

/// The character at byte offset `cluster` in `text`, if any.
fn char_at(text: &str, cluster: usize) -> Option<char> {
    text.get(cluster..).and_then(|rest| rest.chars().next())
}

/// Whether the character at byte offset `cluster` is a CSS word separator
/// (CSS Text 3 §8.2): U+0020 SPACE or U+00A0 NO-BREAK SPACE.
fn is_word_separator(text: &str, cluster: usize) -> bool {
    matches!(char_at(text, cluster), Some(' ' | '\u{00A0}'))
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
    /// A **non-finite** `max_width` (`f32::INFINITY`) is the explicit *no
    /// constraint* sentinel (CSS 2.1 §10.8 available-width; the intrinsic /
    /// max-content measure): the whole paragraph is one line. A **finite**
    /// `max_width >= 0` is a real box, so a `0`/tiny width max-wraps toward
    /// min-content rather than overflowing on one line. The
    /// `white_space` mode (CSS Text 3 §3) selects the break opportunities: a
    /// non-wrapping mode (`nowrap` / `pre`) yields no soft breaks, so each segment
    /// between mandatory (`\n`) breaks lays out on a single line that may overflow
    /// `max_width` — which is correct (overflow/ellipsis is a separate concern).
    /// Empty input — no runs, or a single empty run — yields one empty line rather
    /// than panicking. This greedy wrapper reorders a line's *segments* into
    /// visual order (line-level bidi) but never reorders glyphs within a segment —
    /// that is the shaper's job — and never splits a word, even one that spans
    /// fonts.
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

    /// Wraps like [`wrap`](Self::wrap) but additionally breaks over-wide words at
    /// the opportunities a `hyphenator` proposes plus every author soft hyphen
    /// (`U+00AD`), optionally rendering a visible `hyphen_glyph` at each break
    /// (spec §5.10).
    ///
    /// The break opportunities are [`LineBreak::opportunities`] (whitespace and
    /// mandatory breaks — which delimit words) **plus**, for each word, the
    /// `hyphenator`'s interior byte offsets, **plus** every `U+00AD` in `text` as
    /// a soft break. When the greedy packer meets a word that alone is wider than
    /// the remaining line width, it breaks the word at the **last** hyphenation
    /// opportunity whose head (plus the hyphen glyph's advance) still fits, places
    /// the head — followed by `hyphen_glyph` when supplied — on the line, and
    /// continues the tail on the next line (which may hyphenate again). A word
    /// with no fitting opportunity still takes its own (overflowing) line, exactly
    /// as before. A `U+00AD` that is not taken as a break is left zero-advance and
    /// invisible. Determinism, cluster mapping, bidi reordering, and justification
    /// are preserved: the hyphen glyph carries the break's source cluster and
    /// inherits the font, direction, and bidi level of the glyph it follows, so it
    /// segments and reorders with the head.
    ///
    /// [`wrap`](Self::wrap) is the special case `hyphenator = NullHyphenator`,
    /// `hyphen_glyph = None`, and `ellipsis_glyph = None`.
    ///
    /// # `text-overflow` truncation
    ///
    /// After packing (and justification), any line whose content width *still*
    /// exceeds `max_width` — a single unbreakable, un-hyphenatable word is the
    /// usual cause — is truncated in place when `overflow.is_truncating()` and
    /// `max_width > 0` (spec §5.10). [`TextOverflow::Clip`] keeps the largest
    /// glyph-granular prefix that fits and drops the rest;
    /// [`TextOverflow::Ellipsis`] reserves the `ellipsis_glyph`'s advance, keeps
    /// the largest prefix that fits alongside it, then appends the ellipsis glyph
    /// (carrying the last kept glyph's cluster, font, direction, and level) — with
    /// no `ellipsis_glyph` supplied it degrades to [`Clip`](TextOverflow::Clip).
    /// [`TextOverflow::Visible`] (and any non-truncating policy) leaves overflow
    /// untouched, as does an unconstrained (non-finite `max_width`) wrap. A truncated
    /// line overflows, so justification never stretches it.
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
        let seg_metrics = move |_font: FontId| (strut.ascent, strut.descent);
        Self::wrap_core(
            runs, text, max_width, strut, &seg_metrics, alignment, overflow, white_space,
            0.0, 0.0, hyphenator, hyphen_glyph, ellipsis_glyph,
        )
    }

    /// Wraps like [`wrap`](Self::wrap) but sizes each line box from the mixed-font
    /// line-box **strut** (CSS 2.1 §10.8.1): the `strut` (the element's own
    /// primary-font metrics + used `line-height`) plus each segment font's metrics
    /// from `seg_metrics`. See
    /// [`wrap_strut_hyphenated`](Self::wrap_strut_hyphenated) for the formula.
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
    ) -> Self {
        Self::wrap_core(
            runs, text, max_width, strut, seg_metrics, alignment, overflow, white_space,
            0.0, 0.0, &NullHyphenator, None, None,
        )
    }

    /// Wraps like [`wrap_hyphenated`](Self::wrap_hyphenated) but sizes each line
    /// box from the mixed-font line-box **strut** (CSS 2.1 §10.8 / §10.8.1).
    ///
    /// For each packed line the used ascent `A = max(strut.ascent, max over
    /// segments of seg_metrics(font).0)` and descent `D = max(strut.descent, max
    /// over segments of seg_metrics(font).1)`; the strut floor is applied even on
    /// an empty line. The content box is `A + D`; when the used `line-height` L
    /// (`strut.line_height()`) exceeds it the excess splits as half-leading
    /// `(L − (A+D))/2` above and below (line box `height = L`,
    /// `baseline = A + half_leading`), else `height = A + D`, `baseline = A`. Lines
    /// stack by their own heights, so a tall fallback line pushes the rest down and
    /// the paragraph height is `Σ line heights`. Packing, hyphenation,
    /// justification, and `text-overflow` are identical to
    /// [`wrap_hyphenated`](Self::wrap_hyphenated).
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
        hyphenator: &dyn Hyphenator,
        hyphen_glyph: Option<ShapedGlyph>,
        ellipsis_glyph: Option<ShapedGlyph>,
    ) -> Self {
        Self::wrap_core(
            runs, text, max_width, strut, seg_metrics, alignment, overflow, white_space,
            0.0, 0.0, hyphenator, hyphen_glyph, ellipsis_glyph,
        )
    }

    /// The shared wrapping core: packs `runs` into lines and sizes each line box
    /// from the CSS 2.1 §10.8.1 strut (`strut` + per-font `seg_metrics`).
    ///
    /// `letter_spacing` / `word_spacing` are the resolved CSS Text 3 §8.1/§8.2
    /// tracking values that were folded into the glyph advances upstream (by
    /// [`apply_spacing`](crate::shaping::apply_spacing)); the core needs them to
    /// *hang* (trim) the trailing tracking of each line's last cluster, which §8.1
    /// says is "deferred to the line breaker". They are `0.0` on every historical
    /// path, so the trim is then a no-op and the output is byte-identical.
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
        letter_spacing: f32,
        word_spacing: f32,
        hyphenator: &dyn Hyphenator,
        hyphen_glyph: Option<ShapedGlyph>,
        ellipsis_glyph: Option<ShapedGlyph>,
    ) -> Self {
        let mut stream = Self::flatten(runs);

        // Author soft hyphens: the byte offset of every `U+00AD` char. Their
        // shaped glyphs are forced to zero advance so an untaken soft hyphen is
        // invisible mid-line (spec §5.10).
        let soft: HashSet<usize> = text
            .char_indices()
            .filter(|&(_, ch)| ch == '\u{00AD}')
            .map(|(index, _)| index)
            .collect();
        if !soft.is_empty() {
            for tagged in &mut stream {
                if soft.contains(&tagged.glyph.cluster()) {
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
                if char_at(text, tagged.glyph.cluster()) == Some('\n') {
                    tagged.glyph = tagged.glyph.with_x_advance(0.0);
                }
            }
        }

        // Byte offsets of *collapsible* trailing-hangable whitespace (CSS Text 3
        // §3/§7.1): a run of these at a line's end is removed from the line's used
        // width (it hangs). Only the collapsing modes have collapsible spaces
        // (normal / nowrap / pre-line); pre / pre-wrap preserve them, so nothing
        // hangs there and their measurement is untouched.
        let hang_ws: HashSet<usize> = if white_space.collapses_spaces() {
            text.char_indices()
                .filter(|&(_, ch)| ch == ' ' || ch == '\t')
                .map(|(index, _)| index)
                .collect()
        } else {
            HashSet::new()
        };

        // Interior break offsets proposed by the hyphenator, one word at a time.
        let hyphen_offsets = Self::hyphen_offsets(text, white_space, hyphenator);

        // A **non-finite** `max_width` (`f32::INFINITY`) is the explicit no-width
        // constraint (intrinsic / max-content measure); a finite `max_width >= 0`
        // is a real box that wraps, so a `0`/tiny width max-wraps toward min-content
        // rather than laying out one overflowing line (CSS 2.1 §10.8).
        let bounded = max_width.is_finite();

        // Split the flattened stream into per-line packed groups.
        let mut lines_glyphs = if stream.is_empty() {
            // Empty input still yields a single (empty) line.
            vec![PackedLine {
                glyphs: Vec::new(),
                word_count: 0,
                mandatory_break_after: false,
            }]
        } else if !bounded {
            // Unconstrained width: the whole paragraph is one line. A single
            // unconstrained line is never justified (it is the last line), so its
            // word count is immaterial.
            vec![PackedLine {
                glyphs: stream,
                word_count: 0,
                mandatory_break_after: false,
            }]
        } else {
            Self::pack_lines(
                &stream,
                text,
                max_width,
                white_space,
                &hyphen_offsets,
                &soft,
                &hang_ws,
                hyphen_glyph,
            )
        };

        // Hang the trailing collapsible whitespace and trailing letter-/word-spacing
        // of every line (CSS Text 3 §7.1, §8.1/§8.2): zero the advances of the
        // line's trailing collapsible-space run, then trim the trailing tracking of
        // the last remaining cluster. This is what excludes the hung advance from a
        // line's used `width()`, its alignment `x_offset`, and justification's base.
        for packed in &mut lines_glyphs {
            Self::hang_line_trailing(
                &mut packed.glyphs,
                text,
                &hang_ws,
                letter_spacing,
                word_spacing,
            );
        }

        // Justify: stretch every eligible line's inter-word gaps so it fills
        // `max_width`. Only meaningful under a positive constraint (the unbounded
        // and empty branches produce a single, non-justified last line).
        if alignment == TextAlignment::Justify && bounded {
            Self::justify(&mut lines_glyphs, text, white_space, max_width);
        }

        // Apply `text-overflow` truncation. Only lines that *still* overflow
        // `max_width` (a lone unbreakable, un-hyphenatable word) are affected, and
        // only under a truncating policy and a positive constraint. Truncation
        // runs after justify — an overflowing line was never justify-eligible
        // (justify only stretches lines with slack), so the two never conflict.
        // Truncation needs a *positive* finite box to clip against; a genuinely
        // 0-width box wraps to min-content (above) but is not clipped to nothing.
        if overflow.is_truncating() && bounded && max_width > 0.0 {
            for packed in &mut lines_glyphs {
                let width: f32 = packed.glyphs.iter().map(|t| t.glyph.x_advance()).sum();
                if width > max_width {
                    Self::truncate_line(&mut packed.glyphs, max_width, overflow, ellipsis_glyph);
                }
            }
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
            // CSS 2.1 §10.8.1: size this line box from the strut plus the fonts
            // actually on the line, then stack by its own height.
            let (height, line_baseline) = Self::strut_line_box(glyphs, strut, seg_metrics);
            let line = ParagraphLine::new(segments, range, top, height, line_baseline);
            let x_offset = alignment.x_offset(line.width(), max_width, direction);
            let line = line.with_x_offset(x_offset);
            top += height;
            lines.push(line);
        }

        Self::new(lines, alignment, overflow)
    }

    /// Computes one line box's used `(height, baseline)` from the CSS 2.1
    /// §10.8.1 strut and the fonts on the line.
    ///
    /// The line's ascent `A = max(strut.ascent, max seg ascent)` and descent
    /// `D = max(strut.descent, max seg descent)` take the strut (the element's own
    /// primary font) as an always-present floor — an empty line still gets the
    /// strut's own box. With content box `A + D` and used `line-height`
    /// `L = strut.line_height`: when `L > A + D` the excess splits as half-leading
    /// `(L − (A+D))/2` above and below, giving `height = L`,
    /// `baseline = A + half_leading`; otherwise `height = A + D`, `baseline = A`.
    /// The `max` is order-independent, so the result is deterministic.
    fn strut_line_box(
        glyphs: &[Tagged],
        strut: LineStrut,
        seg_metrics: &dyn Fn(FontId) -> (f32, f32),
    ) -> (f32, f32) {
        let mut ascent = strut.ascent;
        let mut descent = strut.descent;
        for tagged in glyphs {
            let (seg_ascent, seg_descent) = seg_metrics(tagged.font);
            ascent = ascent.max(seg_ascent);
            descent = descent.max(seg_descent);
        }
        let content = ascent + descent;
        if strut.line_height > content {
            let half_leading = (strut.line_height - content) / 2.0;
            (strut.line_height, ascent + half_leading)
        } else {
            (content, ascent)
        }
    }

    /// Hangs one line's trailing content so it is excluded from the line's used
    /// width, alignment offset, and justification base (CSS Text 3 §7.1, §8.1/§8.2).
    ///
    /// Two trims, in order:
    ///
    /// 1. **Trailing collapsible whitespace (§3/§7.1):** the maximal run of
    ///    collapsible-space glyphs (`hang_ws`) at the end of the line has its
    ///    advances zeroed — a sequence of collapsible spaces at a line's end is
    ///    *removed*. A trailing `word-spacing` folded onto that space is removed
    ///    with it.
    /// 2. **Trailing tracking (§8.1/§8.2):** the last *remaining* cluster carries
    ///    the `letter_spacing` that was added after every cluster; that trailing
    ///    tracking hangs, so it is subtracted (plus `word_spacing` when that last
    ///    cluster is itself a word separator, e.g. a preserved NBSP). `0.0`
    ///    tracking makes this a no-op.
    ///
    /// Zeroing (rather than deleting) the glyphs keeps every cluster present for
    /// caret hit-testing; only advances change.
    fn hang_line_trailing(
        glyphs: &mut [Tagged],
        text: &str,
        hang_ws: &HashSet<usize>,
        letter_spacing: f32,
        word_spacing: f32,
    ) {
        // 1. Zero the trailing collapsible-whitespace run.
        let mut last = glyphs.len();
        while last > 0 && hang_ws.contains(&glyphs[last - 1].glyph.cluster()) {
            let glyph = glyphs[last - 1].glyph;
            glyphs[last - 1].glyph = glyph.with_x_advance(0.0);
            last -= 1;
        }
        // 2. Trim the trailing letter-/word-spacing of the last remaining cluster.
        if (letter_spacing != 0.0 || word_spacing != 0.0) && last > 0 {
            let glyph = glyphs[last - 1].glyph;
            let mut advance = glyph.x_advance() - letter_spacing;
            if is_word_separator(text, glyph.cluster()) {
                advance -= word_spacing;
            }
            glyphs[last - 1].glyph = glyph.with_x_advance(advance);
        }
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
            let synth_oblique = shaped.synth_oblique();
            let synth_bold = shaped.synth_bold();
            for glyph in run.glyphs() {
                stream.push(Tagged {
                    font,
                    direction,
                    font_size,
                    level,
                    synth_oblique,
                    synth_bold,
                    glyph: *glyph,
                });
            }
        }
        stream
    }

    /// Collects the interior hyphenation offsets a `hyphenator` proposes across
    /// the words of `text`, mapped into full-text byte coordinates (spec §5.10).
    ///
    /// The text is split into the same words the greedy packer sees (the spans
    /// between adjacent [`LineBreak::opportunities`], and the leading/trailing
    /// spans); each word substring is offered to the hyphenator, and every
    /// returned offset is shifted by the word's start. Offsets that are not
    /// strictly interior to their word are dropped, so an over-eager adapter can
    /// never split at a word edge.
    fn hyphen_offsets(
        text: &str,
        white_space: WhiteSpace,
        hyphenator: &dyn Hyphenator,
    ) -> HashSet<usize> {
        let mut offsets = HashSet::new();
        let mut bounds: Vec<usize> = LineBreak::opportunities(text, white_space)
            .iter()
            .map(LineBreak::offset)
            .collect();
        bounds.push(text.len());
        let mut start = 0;
        for end in bounds {
            if end > start && end <= text.len() {
                let word = &text[start..end];
                for offset in hyphenator.hyphenate(word) {
                    let full = start + offset;
                    // Keep only strictly interior break points.
                    if full > start && full < end {
                        offsets.insert(full);
                    }
                }
            }
            start = end.max(start);
        }
        offsets
    }

    /// Groups a non-empty glyph `stream` into per-line [`PackedLine`]s under a
    /// greedy `max_width` fit, breaking an over-wide word at its last fitting
    /// hyphenation opportunity (spec §5.10). `max_width` is assumed positive.
    ///
    /// `hyphen_offsets` and `soft` are the full-text byte offsets of algorithmic
    /// and author-soft-hyphen break points; `hyphen_glyph` (when present) is
    /// appended at each taken break so a visible hyphen renders. `hang_ws` is the
    /// set of collapsible-whitespace byte offsets, so a word's trailing space can
    /// be excluded from the fit test (it hangs, CSS Text 3 §3/§7.1).
    // The stream, text, wrap width, mode, and the three break-offset sets plus the
    // hyphen glyph are cohesive positional inputs of one internal packing step.
    #[allow(clippy::too_many_arguments)]
    fn pack_lines(
        stream: &[Tagged],
        text: &str,
        max_width: f32,
        white_space: WhiteSpace,
        hyphen_offsets: &HashSet<usize>,
        soft: &HashSet<usize>,
        hang_ws: &HashSet<usize>,
        hyphen_glyph: Option<ShapedGlyph>,
    ) -> Vec<PackedLine> {
        let mut queue: VecDeque<Word> =
            Self::split_words(stream, text, white_space, hyphen_offsets, soft, hang_ws).into();

        let mut lines: Vec<PackedLine> = Vec::new();
        let mut current: Vec<Tagged> = Vec::new();
        let mut current_width = 0.0_f32;
        let mut word_count = 0_usize;
        // Whether the last word placed closed its line on a *mandatory* (forced)
        // break — used to emit a trailing empty line box after a final `\n`.
        let mut ended_on_mandatory = false;
        while let Some(word) = queue.pop_front() {
            ended_on_mandatory = false;
            // Greedy rule: a whole word that no longer fits the (non-empty) line
            // closes it. The word's own trailing collapsible whitespace **hangs**
            // (CSS Text 3 §3/§7.1), so it never counts toward the fit test —
            // otherwise a line wraps one space too early.
            if !current.is_empty()
                && (current_width + word.width - word.trailing_ws) > max_width
            {
                lines.push(PackedLine {
                    glyphs: std::mem::take(&mut current),
                    word_count,
                    mandatory_break_after: false,
                });
                current_width = 0.0;
                word_count = 0;
            }

            // The word alone is wider than the space left on the (now possibly
            // fresh) line: try to break it at the last fitting opportunity.
            let remaining = max_width - current_width;
            if word.width > remaining
                && let Some(split) =
                    Self::try_split(&word, remaining, hyphen_glyph, hyphen_offsets, soft, hang_ws)
            {
                current.extend_from_slice(&split.head);
                word_count += 1;
                lines.push(PackedLine {
                    glyphs: std::mem::take(&mut current),
                    word_count,
                    mandatory_break_after: false,
                });
                current_width = 0.0;
                word_count = 0;
                queue.push_front(split.tail);
                continue;
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
                ended_on_mandatory = true;
            }
        }
        if !current.is_empty() {
            lines.push(PackedLine {
                glyphs: current,
                word_count,
                mandatory_break_after: false,
            });
        } else if ended_on_mandatory {
            // A *trailing* forced break (a `\n` at end of text) opens a new, empty
            // line box (CSS Text 3 §4.1.1) — the caret needs a home on the last
            // line. `"ab\n"` is two lines: `ab`, then an empty one.
            lines.push(PackedLine {
                glyphs: Vec::new(),
                word_count: 0,
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

    /// Attempts to break `word` so its head (plus the hyphen glyph's advance)
    /// fits within `limit`, returning the head glyphs (with an appended
    /// `hyphen_glyph` when supplied) and the remaining tail as a fresh [`Word`].
    ///
    /// The **last** (widest) fitting opportunity is chosen. Returns [`None`] when
    /// no opportunity's head fits, so the caller falls back to placing the whole
    /// (overflowing) word.
    fn try_split(
        word: &Word,
        limit: f32,
        hyphen_glyph: Option<ShapedGlyph>,
        hyphen_offsets: &HashSet<usize>,
        soft: &HashSet<usize>,
        hang_ws: &HashSet<usize>,
    ) -> Option<Split> {
        let hyphen_adv = hyphen_glyph.map_or(0.0, |g| g.x_advance());
        // Points are ascending by head width; take the widest that still fits.
        let point = word
            .hyphens
            .iter()
            .rev()
            .find(|p| p.head_width + hyphen_adv <= limit)
            .copied()?;

        let mut head: Vec<Tagged> = word.glyphs[..point.head_end].to_vec();
        if let (Some(glyph), Some(last)) = (hyphen_glyph, head.last().copied()) {
            // The hyphen inherits the trailing head glyph's font/direction/level
            // so it segments and bidi-reorders with the head, and carries the
            // break's source cluster for hit-testing.
            let hyphen = ShapedGlyph::new(glyph.glyph_id(), point.break_cluster, glyph.x_advance())
                .with_offset(glyph.x_offset(), glyph.y_offset());
            head.push(Tagged {
                glyph: hyphen,
                ..last
            });
        }

        let tail = Self::make_word(
            word.glyphs[point.tail_start..].to_vec(),
            word.mandatory_break_after,
            hyphen_offsets,
            soft,
            hang_ws,
        );
        Some(Split { head, tail })
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
    fn justify(lines: &mut [PackedLine], text: &str, white_space: WhiteSpace, max_width: f32) {
        let Some(last) = lines.len().checked_sub(1) else {
            return;
        };
        // Byte offsets that begin a word (every break opportunity); a glyph whose
        // cluster equals one starts a new word, so the glyph before it ends a gap.
        let word_starts: std::collections::HashSet<usize> =
            LineBreak::opportunities(text, white_space)
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

    /// Truncates one overflowing line's `glyphs` in place so it fits `max_width`
    /// under the `overflow` policy, at glyph granularity (spec §5.10).
    ///
    /// [`Clip`](TextOverflow::Clip) — and [`Ellipsis`](TextOverflow::Ellipsis)
    /// with no `ellipsis_glyph` — keeps the largest prefix whose cumulative
    /// advance is `<= max_width` and drops the rest, adding nothing.
    /// [`Ellipsis`](TextOverflow::Ellipsis) with a glyph reserves that glyph's
    /// advance, keeps the largest prefix whose advance plus the ellipsis fits
    /// `max_width`, then appends the ellipsis glyph. The appended glyph carries the
    /// last kept glyph's cluster and inherits its font, direction, and bidi level
    /// (falling back to the line's first glyph when nothing fits) so it segments
    /// and reorders with the kept prefix. A partially-over glyph is dropped, so the
    /// resulting line width never exceeds `max_width`.
    fn truncate_line(
        glyphs: &mut Vec<Tagged>,
        max_width: f32,
        overflow: TextOverflow,
        ellipsis_glyph: Option<ShapedGlyph>,
    ) {
        // Only `Ellipsis` reserves and later appends a glyph; `Clip` (and
        // `Ellipsis` without a supplied glyph) truncates without adding anything.
        let ellipsis = match overflow {
            TextOverflow::Ellipsis => ellipsis_glyph,
            _ => None,
        };
        let limit = max_width - ellipsis.map_or(0.0, |g| g.x_advance());

        // Keep the largest prefix whose cumulative advance fits `limit`.
        let mut kept = 0_usize;
        let mut width = 0.0_f32;
        for tagged in glyphs.iter() {
            let next = width + tagged.glyph.x_advance();
            if next > limit {
                break;
            }
            width = next;
            kept += 1;
        }

        // The carrier lends the ellipsis its font/direction/level and cluster: the
        // last kept glyph, or the first glyph when nothing fit.
        let carrier = glyphs
            .get(kept.wrapping_sub(1))
            .or_else(|| glyphs.first())
            .copied();
        glyphs.truncate(kept);
        if let (Some(glyph), Some(carrier)) = (ellipsis, carrier) {
            let ellipsis = ShapedGlyph::new(glyph.glyph_id(), carrier.glyph.cluster(), glyph.x_advance())
                .with_offset(glyph.x_offset(), glyph.y_offset());
            glyphs.push(Tagged {
                glyph: ellipsis,
                ..carrier
            });
        }
    }

    /// Segments a glyph `stream` into [`Word`]s at the line-break opportunities of
    /// `text`. A break at byte offset `o` closes the word before the first glyph
    /// whose cluster reaches `o`; a hard break marks that word for a mandatory line
    /// break. Each word records its trailing collapsible whitespace (from
    /// `hang_ws`) and its interior hyphenation points (from `hyphen_offsets` /
    /// `soft`), see [`make_word`](Self::make_word).
    ///
    /// Breaks are consumed by offset (ascending), so a break offset that lands
    /// **inside** a ligature or **past the last glyph** (the end-of-text `\n` of
    /// `"ab\n"`, whose cluster no glyph carries) is not lost: a trailing hard break
    /// still marks the final word mandatory (CSS Text 3 §4.1.1), and a hard break
    /// with no preceding glyphs (a leading/consecutive `\n`) yields an empty
    /// mandatory word — a forced empty line box.
    fn split_words(
        stream: &[Tagged],
        text: &str,
        white_space: WhiteSpace,
        hyphen_offsets: &HashSet<usize>,
        soft: &HashSet<usize>,
        hang_ws: &HashSet<usize>,
    ) -> Vec<Word> {
        let breaks = LineBreak::opportunities(text, white_space);
        let mut words: Vec<Word> = Vec::new();
        let mut glyphs: Vec<Tagged> = Vec::new();
        // Breaks are ascending by offset; `bi` is the next unconsumed break.
        let mut bi = 0usize;
        for tagged in stream {
            let cluster = tagged.glyph.cluster();
            // Consume every break at or before this glyph; the strongest (hard)
            // kind wins for the word we are about to close.
            let mut close = false;
            let mut close_hard = false;
            while bi < breaks.len() && breaks[bi].offset() <= cluster {
                close = true;
                close_hard |= breaks[bi].is_mandatory();
                bi += 1;
            }
            if close {
                if !glyphs.is_empty() {
                    words.push(Self::make_word(
                        std::mem::take(&mut glyphs),
                        close_hard,
                        hyphen_offsets,
                        soft,
                        hang_ws,
                    ));
                } else if close_hard {
                    // A mandatory break with nothing before it (leading or repeated
                    // `\n`): emit an empty forced-break word → an empty line box.
                    words.push(Self::make_word(
                        Vec::new(),
                        true,
                        hyphen_offsets,
                        soft,
                        hang_ws,
                    ));
                }
            }
            glyphs.push(*tagged);
        }
        // Breaks after the last glyph (offset > every cluster): a trailing `\n`
        // whose cluster no glyph carries still forces a break on the final word.
        let mut trailing_hard = false;
        while bi < breaks.len() {
            trailing_hard |= breaks[bi].is_mandatory();
            bi += 1;
        }
        if !glyphs.is_empty() {
            words.push(Self::make_word(glyphs, trailing_hard, hyphen_offsets, soft, hang_ws));
        } else if trailing_hard {
            words.push(Self::make_word(Vec::new(), true, hyphen_offsets, soft, hang_ws));
        }
        words
    }

    /// Builds a [`Word`] from its `glyphs`, computing its advance width and its
    /// interior hyphenation points from `hyphen_offsets` (algorithmic breaks) and
    /// `soft` (author `U+00AD` breaks).
    ///
    /// An interior point sits **before** a glyph whose cluster is an algorithmic
    /// offset (the head runs up to it, the tail resumes at it). A soft-hyphen
    /// point sits **at** a `U+00AD` glyph: the head ends before it and the tail
    /// resumes after it, dropping the zero-advance soft-hyphen glyph. Neither
    /// point is emitted at the word's first glyph (an empty head) or where it
    /// would leave an empty tail. Points are sorted ascending by head width.
    fn make_word(
        glyphs: Vec<Tagged>,
        mandatory_break_after: bool,
        hyphen_offsets: &HashSet<usize>,
        soft: &HashSet<usize>,
        hang_ws: &HashSet<usize>,
    ) -> Word {
        let width: f32 = glyphs.iter().map(|t| t.glyph.x_advance()).sum();
        // The trailing run of collapsible-whitespace advance that hangs when this
        // word ends a line (CSS Text 3 §3/§7.1).
        let trailing_ws: f32 = glyphs
            .iter()
            .rev()
            .take_while(|t| hang_ws.contains(&t.glyph.cluster()))
            .map(|t| t.glyph.x_advance())
            .sum();
        let mut hyphens: Vec<HyphenPoint> = Vec::new();
        let mut head_width = 0.0_f32;
        for (index, tagged) in glyphs.iter().enumerate() {
            let cluster = tagged.glyph.cluster();
            // An algorithmic break resumes the tail *at* this glyph.
            if index >= 1 && hyphen_offsets.contains(&cluster) {
                hyphens.push(HyphenPoint {
                    head_end: index,
                    tail_start: index,
                    head_width,
                    break_cluster: cluster,
                });
            }
            // A soft hyphen breaks *after* this (invisible) glyph, which is
            // dropped; require a non-empty head and a non-empty tail.
            if soft.contains(&cluster) && index >= 1 && index + 1 < glyphs.len() {
                hyphens.push(HyphenPoint {
                    head_end: index,
                    tail_start: index + 1,
                    head_width,
                    break_cluster: cluster,
                });
            }
            head_width += tagged.glyph.x_advance();
        }
        hyphens.sort_by(|a, b| {
            a.head_width
                .partial_cmp(&b.head_width)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.head_end.cmp(&b.head_end))
        });
        Word {
            glyphs,
            width,
            trailing_ws,
            mandatory_break_after,
            hyphens,
        }
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
    /// glyph order is left as the shaper produced it: both HarfRust and the
    /// deterministic mock already emit an RTL run's glyphs in visual order.
    fn segment(glyphs: &[Tagged], line_end: usize) -> Vec<ShapedRun> {
        let mut segments = Vec::new();
        let mut index = 0;
        while index < glyphs.len() {
            let font = glyphs[index].font;
            let direction = glyphs[index].direction;
            let font_size = glyphs[index].font_size;
            let level = glyphs[index].level;
            let synth_oblique = glyphs[index].synth_oblique;
            let synth_bold = glyphs[index].synth_bold;
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
            segments.push(
                ShapedRun::new(font, run)
                    .with_level(level)
                    .with_synthesis(synth_oblique, synth_bold),
            );
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
        let runs = vec![run(0, &[(0, 6.0), (1, 6.0), (2, 6.0)], TextRange::new(0, 3))];
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
            &runs, text, 0.0, 12.0, 9.0, TextAlignment::Start, TextOverflow::Visible,
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
        let runs = vec![run(0, &[(0, 6.0), (1, 6.0), (2, 6.0), (3, 6.0), (4, 6.0)], TextRange::new(0, 5))];
        let layout = ParagraphLayout::wrap(
            &runs, text, 12.0, 12.0, 9.0, TextAlignment::Start, TextOverflow::Visible,
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
        let runs = vec![run(0, &[(0, 6.0), (1, 6.0), (3, 6.0), (4, 6.0)], TextRange::new(0, 5))];
        let layout = ParagraphLayout::wrap(
            &runs, text, 100.0, 12.0, 9.0, TextAlignment::Start, TextOverflow::Clip,
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
            &runs, text, 35.0, 12.0, 9.0, TextAlignment::Start, TextOverflow::Clip,
            WhiteSpace::Normal,
        );
        assert_eq!(layout.line_count(), 2);
        // Line 1 is "aa bb " (clusters 0..5) with the trailing space hung → 30.
        assert_eq!(layout.lines()[0].width(), 30.0);
        assert_eq!(line_glyphs(&layout, 0).iter().map(|g| g.0).collect::<Vec<_>>(), vec![0, 1, 2, 3, 4, 5]);
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
            &runs, text, 40.0, 12.0, 9.0, TextAlignment::Right, TextOverflow::Clip,
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
                &runs, text, 100.0, 12.0, 9.0, TextAlignment::Start, TextOverflow::Clip, mode,
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
        let runs = vec![run(0, &[(0, 6.0), (1, 6.0), (2, 6.0), (3, 6.0), (4, 6.0)], TextRange::new(0, 5))];
        let layout = ParagraphLayout::wrap(
            &runs, text, 100.0, 12.0, 9.0, TextAlignment::Start, TextOverflow::Clip,
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
            4.0,
            0.0,
            &NullHyphenator,
            None,
            None,
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
            &runs, text, 40.0, 12.0, 9.0, TextAlignment::Justify, TextOverflow::Clip,
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
            &runs, text, 50.0, 12.0, 9.0, TextAlignment::Justify, TextOverflow::Clip,
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
            &runs, text, 44.0, 12.0, 9.0, TextAlignment::Justify, TextOverflow::Clip,
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
            &runs, text, 100.0, 12.0, 9.0, TextAlignment::Justify, TextOverflow::Clip,
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
            &[(0, 6.0), (1, 6.0), (2, 6.0), (3, 6.0), (4, 6.0), (5, 6.0), (6, 6.0), (7, 6.0)],
            TextRange::new(0, 8),
        )];
        let layout = ParagraphLayout::wrap(
            &runs, text, f32::INFINITY, 12.0, 9.0, TextAlignment::Justify, TextOverflow::Clip,
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
        let runs = vec![run(0, &[(0, 6.0), (1, 6.0), (2, 6.0), (3, 6.0), (4, 6.0)], TextRange::new(0, 5))];
        let center = ParagraphLayout::wrap(
            &runs, text, 100.0, 12.0, 9.0, TextAlignment::Center, TextOverflow::Clip,
            WhiteSpace::Normal,
        );
        assert_eq!(center.lines()[0].width(), 30.0);
        assert_eq!(center.lines()[0].x_offset(), 35.0);
        let right = ParagraphLayout::wrap(
            &runs, text, 100.0, 12.0, 9.0, TextAlignment::Right, TextOverflow::Clip,
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
            &runs, text, 50.0, 12.0, 9.0, TextAlignment::Start, TextOverflow::Clip,
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
        assert_eq!(tail.iter().map(|g| g.0).collect::<Vec<_>>(), vec![5, 6, 7, 8]);
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
        assert_eq!(kept[..2].iter().map(|g| g.0).collect::<Vec<_>>(), vec![0, 1]);
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
        let unconstrained = truncated(5, f32::INFINITY, TextAlignment::Start, TextOverflow::Clip, None);
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
            &runs, text, 20.0, 12.0, 9.0, TextAlignment::Start, TextOverflow::Clip,
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
        );
        assert_eq!(legacy, strut);
    }
}
