#![doc = include_str!("text_leaf_measure.doc/_module.md")]

use obscur_geometry::{BoxConstraints, Size};
use obscur_layout::model::LayoutStyle;
use obscur_text::font::{
    FontAttributes, FontFace, FontFallback, FontFamily, FontId, FontMetrics, FontSource,
    GenericFamily,
};
use obscur_text::layout::{
    DefaultLineBreaker, Hyphenator, LineStrut, NullHyphenator, OverflowWrap, ParagraphLayout,
    TextAlignment, TextOverflow, WhiteSpace, WordBreak,
};
use obscur_text::shaping::{
    ParagraphShaper, ShapedGlyph, TextDirection, apply_spacing, apply_tab_stops,
};

use super::TabSize;

#[doc = include_str!("text_leaf_measure.doc/measure_text_node.md")]
// The shaping inputs (fallback chain, attributes, direction, alignment) plus the
// style/constraints/shaper are cohesive positional arguments of one leaf-measure
// helper, not a public API surface. `attributes` is kept by reference to match
// the `shape_paragraph`/`families_for` convention used throughout this pass.
#[allow(clippy::too_many_arguments, clippy::trivially_copy_pass_by_ref)]
pub(super) fn measure_text_node(
    content: &str,
    style: &LayoutStyle,
    fallback: &FontFallback,
    attributes: &FontAttributes,
    direction: TextDirection,
    alignment: TextAlignment,
    overflow: TextOverflow,
    white_space: WhiteSpace,
    overflow_wrap: OverflowWrap,
    word_break: WordBreak,
    constraints: BoxConstraints,
    paragraph: &ParagraphShaper,
    hyphenator: Option<&dyn Hyphenator>,
    line_height_override: Option<f32>,
    letter_spacing: f32,
    word_spacing: f32,
    tab_size: TabSize,
) -> Size {
    let font_size = style.font_size;
    let metrics = paragraph.line_metrics(font_size);
    // The line box height: the authored `line-height` (a multiple of `font-size`
    // or an absolute length) when present, else the font's natural metric line
    // height. A larger line height splits the extra leading half above / half
    // below the glyphs (CSS half-leading), shifting the baseline down so the text
    // centres in the taller box. With no authored value the leading is zero, so
    // the baseline is `metrics.baseline()` — byte-for-byte the previous behaviour.
    let line_height = line_height_override.unwrap_or_else(|| metrics.line_height());
    // CSS 2.1 §10.8.1 line-box strut: the element's own primary-font ascent and
    // (below-baseline) descent are the floor of every line box; each segment
    // font's metrics can grow it. The descent carries everything below the
    // baseline (incl. line-gap) so `ascent + descent == metrics.line_height()`,
    // keeping the mock (gap 0) byte-identical to the previous scalar baseline.
    let strut = LineStrut::new(
        line_height,
        metrics.ascent(),
        metrics.line_height() - metrics.ascent(),
    );
    let seg_metrics = |font: FontId| {
        let m = paragraph.line_metrics_for(font, font_size);
        (m.ascent(), m.line_height() - m.ascent())
    };
    // Shape once (cache-shared with compose), then fold CSS Text 3 letter-/word-
    // spacing into the advances through the SAME seam the compose pass uses, so
    // the width measured here matches the width painted later (§8.1 / §8.2).
    let runs = apply_spacing(
        paragraph.shape_paragraph(
            content,
            fallback,
            attributes,
            &measure_face(),
            font_size,
            direction,
        ),
        content,
        letter_spacing,
        word_spacing,
    );
    // Preserved tabs (`pre` / `pre-wrap`) advance to the next `tab-size` tab stop
    // (§4.3). Under a collapsing mode the content has no `\t`, so this is a no-op.
    // The same `tab_width` is threaded into `wrap` so a tab on a soft-wrapped line
    // can be re-measured from THAT line's start, which this pre-wrap pass — with no
    // line-break info — cannot do (it resets its pen only at a hard `\n`).
    let tab_width = tab_stop_width(
        tab_size, paragraph, fallback, attributes, direction, font_size, content,
    );
    let runs = apply_tab_stops(runs, content, tab_width);
    // A text node's own horizontal border/padding shrinks the wrap width; it is
    // almost always zero, but subtract it defensively so the content fits.
    let horizontal =
        style.padding.left + style.padding.right + style.border.left + style.border.right;
    let max_width = (constraints.max.width - horizontal).max(0.0);
    // `text-overflow: ellipsis` needs a shaped ellipsis glyph (shaped the same way
    // the scene phase does, so a truncated line measures with the identical
    // advance it paints with); `clip` / `visible` need none, so none is shaped on
    // the default path.
    let ellipsis = if overflow == TextOverflow::Ellipsis {
        measure_ellipsis_glyph(paragraph, fallback, attributes, direction, font_size)
    } else {
        None
    };
    // The replaceable line-break-opportunity port (spec §5.10, UAX #14). The
    // built-in [`DefaultLineBreaker`] supplies whitespace / forced breaks plus
    // basic inter-ideograph CJK breaks; a UAX#14 adapter would be injected here.
    let line_breaker = DefaultLineBreaker;
    let layout = match hyphenator {
        // No hyphenator and no ellipsis glyph: routed through the no-op hyphenator
        // (byte-identical to plain wrapping, proven by `wrap`≡`wrap_hyphenated`) so
        // the §5.2 emergency-break controls and the breaker port can be threaded.
        None if ellipsis.is_none() => ParagraphLayout::wrap_strut_hyphenated(
            &runs,
            content,
            max_width,
            strut,
            &seg_metrics,
            alignment,
            overflow,
            white_space,
            tab_width,
            &NullHyphenator,
            None,
            None,
            overflow_wrap,
            word_break,
            &line_breaker,
        ),
        // Ellipsis without a hyphenator: wrap through the no-op hyphenator so the
        // shaped ellipsis glyph can be threaded into truncation.
        None => ParagraphLayout::wrap_strut_hyphenated(
            &runs,
            content,
            max_width,
            strut,
            &seg_metrics,
            alignment,
            overflow,
            white_space,
            tab_width,
            &NullHyphenator,
            None,
            ellipsis,
            overflow_wrap,
            word_break,
            &line_breaker,
        ),
        // Hyphenation on: shape the hyphen the same way the scene phase does, so a
        // hyphenated break measures with the identical advance it paints with.
        Some(hyphenator) => {
            let hyphen =
                measure_hyphen_glyph(paragraph, fallback, attributes, direction, font_size);
            ParagraphLayout::wrap_strut_hyphenated(
                &runs,
                content,
                max_width,
                strut,
                &seg_metrics,
                alignment,
                overflow,
                white_space,
                tab_width,
                hyphenator,
                hyphen,
                ellipsis,
                overflow_wrap,
                word_break,
                &line_breaker,
            )
        }
    };
    // The paragraph reports the size it actually NEEDS.
    //
    // A plain `constrain` here looks defensive and is a lie: content that cannot
    // fit the offered width does not shrink, it overflows — an unbreakable word is
    // the standing example — and reporting the offered width instead of the needed
    // one hides that from every caller, exact-fit and overflow coming back
    // identical. It also makes `min-content` unanswerable: that probe offers zero
    // width, so under `constrain` every answer would be zero rather than the widest
    // unbreakable run, which is the *definition* of the value (CSS Sizing 3 §4.1).
    //
    // Wrapping already keeps the width within `max_width` whenever the content
    // *can* fit, so this differs only in the overflow case — where the old answer
    // was wrong. A tight constraint still dictates; see
    // [`BoxConstraints::constrain_allowing_overflow`].
    constraints.constrain_allowing_overflow(layout.size())
}

#[doc = include_str!("text_leaf_measure.doc/measure_hyphen_glyph.md")]
// `attributes` stays by reference to match this pass's `shape_paragraph` calls.
#[allow(clippy::trivially_copy_pass_by_ref)]
fn measure_hyphen_glyph(
    paragraph: &ParagraphShaper,
    fallback: &FontFallback,
    attributes: &FontAttributes,
    direction: TextDirection,
    font_size: f32,
) -> Option<ShapedGlyph> {
    paragraph
        .shape_paragraph(
            "-",
            fallback,
            attributes,
            &measure_face(),
            font_size,
            direction,
        )
        .iter()
        .flat_map(|run| run.run().glyphs().iter().copied())
        .next()
}

#[doc = include_str!("text_leaf_measure.doc/measure_ellipsis_glyph.md")]
// `attributes` stays by reference to match this pass's `shape_paragraph` calls.
#[allow(clippy::trivially_copy_pass_by_ref)]
fn measure_ellipsis_glyph(
    paragraph: &ParagraphShaper,
    fallback: &FontFallback,
    attributes: &FontAttributes,
    direction: TextDirection,
    font_size: f32,
) -> Option<ShapedGlyph> {
    paragraph
        .shape_paragraph(
            "…",
            fallback,
            attributes,
            &measure_face(),
            font_size,
            direction,
        )
        .iter()
        .flat_map(|run| run.run().glyphs().iter().copied())
        .next()
}

#[doc = include_str!("text_leaf_measure.doc/tab_stop_width.md")]
#[allow(clippy::too_many_arguments, clippy::trivially_copy_pass_by_ref)]
fn tab_stop_width(
    tab_size: TabSize,
    paragraph: &ParagraphShaper,
    fallback: &FontFallback,
    attributes: &FontAttributes,
    direction: TextDirection,
    font_size: f32,
    content: &str,
) -> f32 {
    if !content.contains('\t') {
        return 0.0;
    }
    let space_advance = match tab_size {
        TabSize::Length(_) => 0.0,
        TabSize::Factor(_) => {
            measure_space_advance(paragraph, fallback, attributes, direction, font_size)
        }
    };
    tab_size.resolve(space_advance)
}

#[doc = include_str!("text_leaf_measure.doc/measure_space_advance.md")]
// `attributes` stays by reference to match this pass's `shape_paragraph` calls.
#[allow(clippy::trivially_copy_pass_by_ref)]
fn measure_space_advance(
    paragraph: &ParagraphShaper,
    fallback: &FontFallback,
    attributes: &FontAttributes,
    direction: TextDirection,
    font_size: f32,
) -> f32 {
    paragraph
        .shape_paragraph(
            " ",
            fallback,
            attributes,
            &measure_face(),
            font_size,
            direction,
        )
        .iter()
        .flat_map(|run| run.run().glyphs().iter())
        .map(ShapedGlyph::x_advance)
        .sum()
}

#[doc = include_str!("text_leaf_measure.doc/measure_ch_width.md")]
// `attributes` stays by reference to match this pass's `shape_paragraph` calls.
#[allow(clippy::trivially_copy_pass_by_ref)]
pub(super) fn measure_ch_width(
    paragraph: &ParagraphShaper,
    fallback: &FontFallback,
    attributes: &FontAttributes,
    font_size: f32,
) -> Option<f32> {
    // A digit's advance does not depend on the paragraph's base direction, so
    // this probes LTR rather than threading the node's `direction` in: a run of
    // one neutral-width glyph reorders to itself.
    let advance: f32 = paragraph
        .shape_paragraph(
            "0",
            fallback,
            attributes,
            &measure_face(),
            font_size,
            TextDirection::LeftToRight,
        )
        .iter()
        .flat_map(|run| run.run().glyphs().iter())
        .map(ShapedGlyph::x_advance)
        .sum();
    (advance > 0.0).then_some(advance)
}

#[doc = include_str!("text_leaf_measure.doc/measure_face.md")]
pub(super) fn measure_face() -> FontFace {
    FontFace::new(
        FontFamily::generic(GenericFamily::Monospace),
        FontAttributes::default(),
        FontSource::system("mock"),
        FontMetrics::new(1000, 800.0, -200.0, 0.0),
    )
}
