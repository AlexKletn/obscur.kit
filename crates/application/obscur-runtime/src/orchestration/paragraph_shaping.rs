//! The text engine the compose phase shapes every paragraph through:
//! [`ParagraphShaping`].

use std::fmt;

use obscur_identity::NodeId;
use obscur_style::computed::{ComputedStyle, ComputedValue};
use obscur_style::property::{Length, PropertyId};
use obscur_text::font::{
    FontAttributes, FontFace, FontFallback, FontFamily, FontId, FontMetrics, FontResolver,
    FontSource, GenericFamily, MockFontResolver,
};
use obscur_text::layout::{
    DefaultLineBreaker, Hyphenator, LineStrut, NullHyphenator, OverflowWrap, ParagraphLayout,
    TextAlignment, TextOverflow, WhiteSpace, WordBreak,
};
use obscur_text::shaping::{
    CachingShaper, MockTextShaper, ParagraphShaper, ShapedGlyph, SingleRunItemizer, TextDirection,
    TextItemizer, TextShaper, apply_spacing, apply_tab_stops,
};
use obscur_tree::tree::RetainedTree;

use super::{
    ComputedStyles, LayoutBoxes, TabSize, alignment_for, attributes_for, direction_for,
    families_for, letter_spacing_for, line_height_for, overflow_for, overflow_wrap_for,
    root_font_size, tab_size_for, white_space_for, word_break_for, word_spacing_for,
};

#[doc = include_str!("../../docs/api/orchestration/paragraph_shaping.doc/DEFAULT_FONT_SIZE.md")]
pub(super) const DEFAULT_FONT_SIZE: f32 = 16.0;

#[doc = include_str!("../../docs/api/orchestration/paragraph_shaping.doc/ParagraphShaping.md")]
pub(crate) struct ParagraphShaping {
    /// The multi-font orchestrator turning text-node strings into font-tagged
    /// runs. The default composition is deterministic (single run, primary face,
    /// mock glyphs), which is what the cross-backend determinism boundary relies
    /// on. A host swaps its ports via the setters.
    paragraph: ParagraphShaper,
    /// The default fallback face requests resolve to when the resolver covers no
    /// font for a run. The mock shaper is font-agnostic (it needs no real
    /// metrics), so a generic monospace face suffices as the default; a real
    /// pipeline would resolve this per node via a `FontProvider`.
    default_face: FontFace,
    #[doc = include_str!("../../docs/api/orchestration/paragraph_shaping.doc/hyphenator.md")]
    hyphenator: Option<Box<dyn Hyphenator>>,
}

impl ParagraphShaping {
    /// Creates the default engine: a single-run itemizer, an empty resolver, a
    /// cache-wrapped deterministic [`MockTextShaper`], a generic monospace default
    /// face, and no hyphenation.
    pub(super) fn new() -> Self {
        Self {
            // Single-run itemizer + empty resolver ⇒ one PRIMARY run per text; the
            // inner mock is cache-wrapped so measure/arrange/compose share one
            // shaped run per unique request within (and across) frames.
            paragraph: ParagraphShaper::new(
                Box::new(SingleRunItemizer::new()),
                Box::new(MockFontResolver::new()),
                Box::new(CachingShaper::new(Box::new(MockTextShaper::new()))),
            ),
            default_face: FontFace::new(
                FontFamily::generic(GenericFamily::Monospace),
                FontAttributes::default(),
                FontSource::system("mock"),
                FontMetrics::new(1000, 800.0, -200.0, 0.0),
            ),
            // No hyphenation by default: byte-identical to pre-hyphenation
            // wrapping, and no hyphen glyph is shaped on the default path.
            hyphenator: None,
        }
    }

    /// The paragraph orchestrator as a shared borrow — the seam the layout phase
    /// measures text through, so measure and compose shape the identical runs.
    pub(super) fn paragraph(&self) -> &ParagraphShaper {
        &self.paragraph
    }

    #[doc = include_str!("../../docs/api/orchestration/paragraph_shaping.doc/default_face_2.md")]
    #[cfg(test)]
    pub(super) fn default_face(&self) -> &FontFace {
        &self.default_face
    }

    /// Installs `shaper` as the inner engine, wrapped in a **fresh** shaping cache
    /// so no run shaped by the previous engine survives it.
    pub(super) fn set_shaper(&mut self, shaper: Box<dyn TextShaper>) {
        self.paragraph
            .set_shaper(Box::new(CachingShaper::new(shaper)));
    }

    /// Installs `itemizer` as the paragraph itemizer, keeping the resolver and the
    /// inner shaper (and its cache) in place.
    pub(super) fn set_itemizer(&mut self, itemizer: Box<dyn TextItemizer>) {
        self.paragraph.set_itemizer(itemizer);
    }

    /// Installs `resolver` as the font resolver, keeping the itemizer and the inner
    /// shaper (and its cache) in place.
    pub(super) fn set_resolver(&mut self, resolver: Box<dyn FontResolver>) {
        self.paragraph.set_resolver(resolver);
    }

    /// Installs the BCP-47 `language` hint carried by every shaping request this
    /// orchestrator issues.
    pub(super) fn set_language(&mut self, language: Option<Box<str>>) {
        self.paragraph.set_language(language);
    }

    /// Installs `hyphenator` as the hyphenation port, replacing the default (no
    /// hyphenation). The same port is used by the measure and the compose pass.
    pub(super) fn set_hyphenator(&mut self, hyphenator: Box<dyn Hyphenator>) {
        self.hyphenator = Some(hyphenator);
    }

    /// The installed hyphenation port, or [`None`] when hyphenation is off (the
    /// default).
    pub(super) fn hyphenator(&self) -> Option<&dyn Hyphenator> {
        self.hyphenator.as_deref()
    }

    #[doc = include_str!("../../docs/api/orchestration/paragraph_shaping.doc/shape_lines.md")]
    // The shaping inputs (fallback chain, attributes, direction, alignment) plus
    // the font size and wrap width are cohesive positional arguments of one leaf
    // shaping helper, not a public API surface. `attributes` is kept by reference
    // to match the `shape_paragraph` convention used throughout this pass.
    #[allow(clippy::too_many_arguments, clippy::trivially_copy_pass_by_ref)]
    pub(super) fn shape_lines(
        &self,
        content: &str,
        fallback: &FontFallback,
        attributes: &FontAttributes,
        direction: TextDirection,
        alignment: TextAlignment,
        overflow: TextOverflow,
        white_space: WhiteSpace,
        overflow_wrap: OverflowWrap,
        word_break: WordBreak,
        font_size: f32,
        max_width: f32,
        line_height_override: Option<f32>,
        letter_spacing: f32,
        word_spacing: f32,
        tab_size: TabSize,
    ) -> ParagraphLayout {
        // Shape once (cache-shared with the measure pass), then fold CSS Text 3
        // letter-/word-spacing into the advances through the SAME `apply_spacing`
        // seam the measure pass uses, so the painted advances match the measured
        // box (§8.1 / §8.2). `content` is already `text-transform`ed + white-space
        // collapsed by the caller.
        let runs = apply_spacing(
            self.paragraph.shape_paragraph_shared(
                content,
                fallback,
                attributes,
                &self.default_face,
                font_size,
                direction,
            ),
            content,
            letter_spacing,
            word_spacing,
        );
        // Preserved tabs advance to the next `tab-size` stop (§4.3); a no-op unless
        // `content` still holds a `\t` (only `pre` / `pre-wrap` keep tabs). The same
        // `tab_width` is threaded into `wrap` so a tab on a soft-wrapped line is
        // re-measured from THAT line's start (this pre-wrap pass has no line-break
        // info and can only reset its pen at a hard `\n`).
        let tab_width = self.tab_stop_width(
            tab_size, fallback, attributes, direction, font_size, content,
        );
        let runs = apply_tab_stops(runs, content, tab_width);
        let metrics = self.paragraph.line_metrics(font_size);
        // The line box height: the authored `line-height` when present, else the
        // font's natural metric line height. A larger line height splits the extra
        // leading half above / half below (CSS half-leading), shifting the baseline
        // down so the text centres in the taller box. With no authored value the
        // leading is zero, so `line_height`/`baseline` equal the metric values —
        // byte-for-byte the single-line, default-spacing behaviour.
        let line_height = line_height_override.unwrap_or_else(|| metrics.line_height());
        // CSS 2.1 §10.8.1 line-box strut: the element's own primary-font metrics
        // are the floor of every line box; a taller fallback segment grows it. The
        // descent carries everything below the baseline (incl. line-gap) so
        // `ascent + descent == metrics.line_height()`, keeping the mock (gap 0)
        // byte-identical to the previous scalar baseline — and IDENTICAL to the
        // measure pass, which builds the same strut and lookup.
        let strut = LineStrut::new(
            line_height,
            metrics.ascent(),
            metrics.line_height() - metrics.ascent(),
        );
        let seg_metrics = |font: FontId| {
            let m = self.paragraph.line_metrics_for(font, font_size);
            (m.ascent(), m.line_height() - m.ascent())
        };
        // `text-overflow: ellipsis` needs a shaped ellipsis glyph to append at a
        // cut; `clip` / `visible` need none, so none is shaped on the default path
        // (Clip truncation works from advances alone).
        let ellipsis = if overflow == TextOverflow::Ellipsis {
            self.shape_ellipsis(fallback, attributes, direction, font_size)
        } else {
            None
        };
        // The replaceable line-break-opportunity port (spec §5.10, UAX #14): the
        // built-in [`DefaultLineBreaker`], matching the measure pass exactly so the
        // painted lines agree with the sized box. A UAX#14 adapter would inject here.
        let line_breaker = DefaultLineBreaker;
        match self.hyphenator.as_deref() {
            // No hyphenator and no ellipsis glyph: routed through the no-op
            // hyphenator (byte-identical to plain wrapping) so the §5.2 controls and
            // the breaker port thread through; no hyphen/ellipsis glyph is shaped, so
            // the shaping-cache call count is unchanged. Clip truncation still
            // applies through `overflow`, but only clips lines that overflow.
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
            // Ellipsis without a hyphenator: wrap through the no-op hyphenator so
            // the shaped ellipsis glyph can be threaded into truncation.
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
            // Hyphenation on: shape the hyphen glyph so a break renders (and
            // measures) with a real advance, and wrap through the installed port.
            Some(hyphenator) => {
                let hyphen = self.shape_hyphen(fallback, attributes, direction, font_size);
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
        }
    }

    #[doc = include_str!("../../docs/api/orchestration/paragraph_shaping.doc/shape_ellipsis.md")]
    // `attributes` stays by reference to match this pass's `shape_paragraph` calls.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn shape_ellipsis(
        &self,
        fallback: &FontFallback,
        attributes: &FontAttributes,
        direction: TextDirection,
        font_size: f32,
    ) -> Option<ShapedGlyph> {
        self.paragraph
            .shape_paragraph_shared(
                "…",
                fallback,
                attributes,
                &self.default_face,
                font_size,
                direction,
            )
            .iter()
            .flat_map(|run| run.run().glyphs().iter().copied())
            .next()
    }

    #[doc = include_str!("../../docs/api/orchestration/paragraph_shaping.doc/shape_hyphen.md")]
    // `attributes` stays by reference to match this pass's `shape_paragraph` calls.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn shape_hyphen(
        &self,
        fallback: &FontFallback,
        attributes: &FontAttributes,
        direction: TextDirection,
        font_size: f32,
    ) -> Option<ShapedGlyph> {
        self.paragraph
            .shape_paragraph_shared(
                "-",
                fallback,
                attributes,
                &self.default_face,
                font_size,
                direction,
            )
            .iter()
            .flat_map(|run| run.run().glyphs().iter().copied())
            .next()
    }

    #[doc = include_str!("../../docs/api/orchestration/paragraph_shaping.doc/tab_stop_width.md")]
    // `attributes` stays by reference to match this pass's `shape_paragraph` calls.
    #[allow(clippy::too_many_arguments, clippy::trivially_copy_pass_by_ref)]
    fn tab_stop_width(
        &self,
        tab_size: TabSize,
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
            TabSize::Factor(_) => self
                .paragraph
                .shape_paragraph_shared(
                    " ",
                    fallback,
                    attributes,
                    &self.default_face,
                    font_size,
                    direction,
                )
                .iter()
                .flat_map(|run| run.run().glyphs().iter())
                .map(ShapedGlyph::x_advance)
                .sum(),
        };
        tab_size.resolve(space_advance)
    }

    #[doc = include_str!("../../docs/api/orchestration/paragraph_shaping.doc/align_width_for.md")]
    pub(super) fn align_width_for(
        tree: &RetainedTree,
        computed: &ComputedStyles,
        boxes: &LayoutBoxes,
        node: NodeId,
        box_width: f32,
    ) -> f32 {
        let Some(parent) = tree.get(node).and_then(|n| n.logical_parent().parent()) else {
            return box_width;
        };
        let Some(parent_box) = boxes.get(&parent) else {
            return box_width;
        };
        if let Some(content) = parent_box.used_content_rect() {
            return content.size.width.max(box_width);
        }
        let style = computed.get(&parent);
        let px = |property: PropertyId| -> f32 {
            style
                .and_then(|style| style.get(property))
                .and_then(ComputedValue::as_length)
                .and_then(|length| match length {
                    Length::Px(px) => Some(px.0),
                    _ => None,
                })
                .unwrap_or(0.0)
        };
        let content = parent_box.size().width
            - px(PropertyId::PaddingLeft)
            - px(PropertyId::PaddingRight)
            - px(PropertyId::BorderLeftWidth)
            - px(PropertyId::BorderRightWidth);
        content.max(box_width)
    }

    #[doc = include_str!("../../docs/api/orchestration/paragraph_shaping.doc/shape_text_lines.md")]
    pub(super) fn shape_text_lines(
        &self,
        tree: &RetainedTree,
        computed: &ComputedStyles,
        boxes: &LayoutBoxes,
        node: NodeId,
        content: &str,
        box_width: f32,
    ) -> ParagraphLayout {
        self.shape_text_lines_with_overflow(
            tree,
            computed,
            boxes,
            node,
            content,
            box_width,
            overflow_for_text(tree, computed, node),
        )
    }

    // The field clips the translated full line at its scrollport.
    pub(crate) fn shape_editable_lines(
        &self,
        tree: &RetainedTree,
        computed: &ComputedStyles,
        boxes: &LayoutBoxes,
        node: NodeId,
        content: &str,
        box_width: f32,
    ) -> ParagraphLayout {
        let overflow = if white_space_for(computed.get(&node)).wraps() {
            overflow_for_text(tree, computed, node)
        } else {
            TextOverflow::Visible
        };
        self.shape_text_lines_with_overflow(
            tree, computed, boxes, node, content, box_width, overflow,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn shape_text_lines_with_overflow(
        &self,
        tree: &RetainedTree,
        computed: &ComputedStyles,
        boxes: &LayoutBoxes,
        node: NodeId,
        content: &str,
        box_width: f32,
        overflow: TextOverflow,
    ) -> ParagraphLayout {
        let style = computed.get(&node);
        let font_size = font_size_of(computed, node);
        // Root `font-size` resolves any `rem` in the text metrics (CSS Values 4
        // §5.1.1); `em` / `%` resolve against this node's `font_size`. Resolved
        // identically to the layout phase so caret/hit-test lines match the box.
        let root_font = root_font_size(computed, tree.root());
        let layout = self.shape_lines(
            content,
            &families_for(style),
            &attributes_for(style),
            direction_for(style),
            alignment_for(style),
            overflow,
            white_space_for(style),
            overflow_wrap_for(style),
            word_break_for(style),
            font_size,
            box_width,
            line_height_for(style, font_size, root_font),
            letter_spacing_for(style, font_size, root_font),
            word_spacing_for(style, font_size, root_font),
            tab_size_for(style, font_size, root_font),
        );
        // Wrapping answered to the box; alignment answers to the containing
        // block. Only an explicitly non-default alignment re-aligns, so a
        // `start`-aligned paragraph — which is every paragraph until an author
        // says otherwise, in either direction — takes the identical path it
        // always did.
        let alignment = alignment_for(style);
        if matches!(
            alignment,
            TextAlignment::Center
                | TextAlignment::Right
                | TextAlignment::End
                | TextAlignment::Justify
        ) {
            let width = Self::align_width_for(tree, computed, boxes, node, box_width);
            if width > box_width {
                return layout.aligned_to(width);
            }
        }
        layout
    }
}

impl fmt::Debug for ParagraphShaping {
    /// `ParagraphShaper` holds `dyn` ports and is not `Debug`, so it is shown as an
    /// opaque placeholder; the resolved default face is printed normally.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ParagraphShaping")
            .field("paragraph", &"<ParagraphShaper>")
            .field("default_face", &self.default_face)
            .field("hyphenator", &"<dyn Hyphenator>")
            .finish()
    }
}

#[doc = include_str!("../../docs/api/orchestration/paragraph_shaping.doc/font_size_of.md")]
pub(super) fn font_size_of(computed: &ComputedStyles, node: NodeId) -> f32 {
    font_size_of_style(computed.get(&node))
}

/// Reads a computed `font-size` in logical pixels from a single node's style,
/// defaulting to [`DEFAULT_FONT_SIZE`] when absent or not an absolute pixel
/// length — the by-style form [`font_size_of`] delegates to, shared with the
/// click-caret hit-test so it sizes text identically to the drawn run.
pub(super) fn font_size_of_style(style: Option<&ComputedStyle>) -> f32 {
    style
        .and_then(|style| style.get(PropertyId::FontSize))
        .and_then(ComputedValue::as_length)
        .and_then(Length::as_px)
        .map_or(DEFAULT_FONT_SIZE, |px| px.0)
}

#[doc = include_str!("../../docs/api/orchestration/paragraph_shaping.doc/overflow_for_text.md")]
pub(super) fn overflow_for_text(
    tree: &RetainedTree,
    computed: &ComputedStyles,
    node: NodeId,
) -> TextOverflow {
    let container = tree
        .get(node)
        .and_then(|n| n.logical_parent().parent())
        .unwrap_or(node);
    overflow_for(computed.get(&container))
}
