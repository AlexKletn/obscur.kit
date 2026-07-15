//! Drives layout over the retained tree: [`LayoutCoordinator`].

use std::collections::HashMap;

use obscur_geometry::{BoxConstraints, Insets, Point, Rect, Size};
use obscur_identity::{InternedSymbol, NodeId};
use obscur_layout::algorithm::{
    ChildMeasurer, FlexLayout, FlowLayout, GridAutoFlow, GridLayout, GridLine, GridPlacement,
    LayoutAlgorithm, LayoutChild,
};
use obscur_layout::model::{
    AlignItems, AlignSelf, BoxSizing, Dimension, FlexDirection, FlexWrap, Inset, JustifyContent,
    LayoutBox, LayoutDisplay, LayoutStyle, Margin, MarginEdges, Position,
};
use obscur_style::computed::{ComputedStyle, ComputedValue};
use obscur_style::property::{
    GridLineValue, Length, LengthContext, PropertyId, PropertyValue, layout_keyword,
};
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
use obscur_tree::tree::RetainedTree;

use super::scene_coordinator::editing_override;
use super::scroll::overflow_scrolls;
use super::{
    ComputedStyles, EditingView, GridTemplates, TabSize, alignment_for, attributes_for,
    direction_for, display_text, families_for, grid_templates_from, letter_spacing_for,
    line_height_for, overflow_for, overflow_wrap_for, resolve_named_grid_areas,
    resolve_named_grid_lines, tab_size_for, text_transform_for, white_space_for, word_break_for,
    word_spacing_for,
};

/// The initial `font-size` (spec / CSS default), used as the `em`/`rem` base
/// when a node (or the document root) has no computed `font-size`.
const DEFAULT_FONT_SIZE: f32 = 16.0;

/// The absolute [`LayoutBox`] of every node the coordinator arranged, keyed by
/// [`NodeId`].
///
/// Unlike the per-node boxes produced *inside* a [`LayoutAlgorithm`] (which are
/// parent-relative), the boxes in this map are in **absolute** viewport
/// coordinates: the coordinator accumulates ancestor offsets as it walks down
/// the tree (spec §4.2).
pub type LayoutBoxes = HashMap<NodeId, LayoutBox>;

/// The layout input [`LayoutStyle`] derived for every node, keyed by [`NodeId`].
///
/// This is the runtime's bridge from `obscur-style`'s [`ComputedStyle`] into
/// `obscur-layout`'s own vocabulary (spec §3.10, §4.2): layout consumes its own
/// [`LayoutStyle`], and the mapping from computed styles is the coordinator's
/// responsibility.
pub type LayoutStyles = HashMap<NodeId, LayoutStyle>;

/// The maximum recursion depth of the layout walk.
///
/// The retained tree is expected to be acyclic, but the coordinator never trusts
/// that: a shared or cyclic child link would otherwise recurse forever. This cap
/// bounds both `measure` and `arrange` so recursion always terminates and never
/// overflows the stack. It is deliberately generous (far deeper than any
/// realistic UI tree) so it never truncates a well-formed tree.
const MAX_DEPTH: u32 = 256;

/// Orchestrates layout over a [`RetainedTree`] (spec §3.10, §4.2).
///
/// The coordinator is **orchestration only**: it owns the tree walk and the
/// [`ComputedStyle`] → [`LayoutStyle`] mapping, but the flow/flex math lives
/// entirely in `obscur-layout`'s [`LayoutAlgorithm`] implementations
/// ([`FlowLayout`], [`FlexLayout`]). Its two responsibilities are:
///
/// 1. **Style mapping** — [`layout_styles_from`](Self::layout_styles_from)
///    derives a per-node [`LayoutStyle`] from each node's [`ComputedStyle`].
/// 2. **Recursive driving** — [`layout`](Self::layout) walks the retained tree
///    and, for each node, invokes the algorithm selected by its `display`,
///    threading a [`ChildMeasurer`] that recursively sizes children, and
///    accumulates absolute [`LayoutBox`]es.
///
/// The walk is **cycle-safe** (bounded by [`MAX_DEPTH`]) and **panic-free**:
/// out-of-range child boxes are handled defensively, and no indexing can panic.
///
/// The coordinator is a stateless zero-sized value; construct it with
/// [`new`](Self::new) or [`Default`].
#[derive(Debug, Clone, Copy, Default)]
pub struct LayoutCoordinator;

impl LayoutCoordinator {
    /// Creates a coordinator.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Derives a [`LayoutStyle`] for every node in `computed`.
    ///
    /// Each node's [`ComputedStyle`] is mapped independently via
    /// [`layout_style_from`]; nodes absent from `computed` simply produce no
    /// entry in the result.
    ///
    /// `viewport` and `root_font_size` supply the resolution context for
    /// relative and viewport-relative units and `calc()` expressions (spec
    /// §4.5): `vw`/`vh`/`vmin`/`vmax` resolve against `viewport`, `rem` against
    /// `root_font_size`, and `em` against each node's own computed `font-size`.
    /// Compute `root_font_size` with [`root_font_size`] from the tree root.
    ///
    /// `paragraph` resolves `ch`, the one unit that cannot be folded from numbers
    /// alone — it needs the `0` glyph shaped (see [`ch_width_for`], which measures
    /// only for the rare node that writes `ch`). Pass [`None`] when no shaper is
    /// available; `ch` then takes the specified `0.5em` substitute and every other
    /// unit is unaffected.
    #[must_use]
    pub fn layout_styles_from(
        &self,
        computed: &ComputedStyles,
        viewport: Size,
        root_font_size: f32,
        paragraph: Option<&ParagraphShaper>,
    ) -> LayoutStyles {
        computed
            .iter()
            .map(|(&node, style)| {
                (
                    node,
                    layout_style_from(style, viewport, root_font_size, paragraph),
                )
            })
            .collect()
    }

    /// Lays out the whole tree under `viewport` and returns each node's absolute
    /// [`LayoutBox`].
    ///
    /// Returns an empty map when the tree has no root. Otherwise the root is
    /// arranged at the viewport origin under a tight `viewport` constraint, and
    /// every reachable node's absolute box is written into the result.
    ///
    /// `styles` supplies the per-node [`LayoutStyle`] (typically produced by
    /// [`layout_styles_from`](Self::layout_styles_from)); a node missing from
    /// `styles` is laid out with [`LayoutStyle::default`]. `computed` supplies
    /// each node's [`ComputedStyle`], from which a text leaf's `font-family`
    /// fallback chain is resolved (via [`families_for`]) — the same chain the
    /// scene phase paints with.
    ///
    /// # Text is a constraint-aware leaf (spec §5.10)
    ///
    /// A [`NodeKind::Text`](obscur_tree::node::NodeKind::Text) node carries no CSS
    /// width/height, so the measure/arrange pass sizes it itself: it shapes the
    /// content through `paragraph` (the multi-font fallback orchestrator) at the
    /// node's [`font_size`](LayoutStyle::font_size) and **wraps** the resulting
    /// font-tagged runs to the finite `max.width` its container hands down,
    /// returning the wrapped paragraph's [`size`](ParagraphLayout::size) as the
    /// node's border box. A long string in a narrow container therefore wraps to
    /// *more, shorter lines* (a taller box) instead of one wide line. The **same**
    /// `paragraph` orchestrator and size drive the scene phase, which re-wraps each
    /// text node to its arranged box width, so the measured height and the painted
    /// line count always agree.
    ///
    /// # Redundant work
    ///
    /// The selected algorithm runs once per node during `arrange`, and each run
    /// re-measures that node's children (which recurses). A per-pass
    /// [`MeasureCache`] memoizes `measure(node, constraints)`, so a node sized
    /// under identical constraints is measured (and its text shaped) **once per
    /// frame** rather than once per ancestor — the dominant cost on a deep,
    /// text-heavy tree (spec §4.2). Re-measures under *different* constraints
    /// (intrinsic vs. final sizing) are still distinct; collapsing those needs an
    /// intrinsic-size cache or incremental invalidation (a later phase).
    // The tree, the per-node side-tables, the viewport + scroll offset, and the
    // shaper are the cohesive inputs of one layout entry point.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn layout(
        &self,
        tree: &RetainedTree,
        styles: &LayoutStyles,
        computed: &ComputedStyles,
        grid_templates: &GridTemplates,
        viewport: Size,
        scroll_offset: (f32, f32),
        paragraph: &ParagraphShaper,
    ) -> LayoutBoxes {
        // The default path never hyphenates: [`None`] makes text-leaf measurement
        // byte-identical to whitespace-only wrapping (no hyphen glyph is shaped).
        self.layout_with_hyphenator(
            tree,
            styles,
            computed,
            grid_templates,
            viewport,
            scroll_offset,
            paragraph,
            None,
        )
    }

    /// Derives a [`GridTemplates`] side-table for every grid node in `computed`
    /// that authored a `grid-template-columns` / `grid-template-rows`.
    ///
    /// This is the grid twin of
    /// [`layout_styles_from`](Self::layout_styles_from): the same `viewport` /
    /// `root_font_size` resolution context folds a fixed track's relative units to
    /// pixels. Nodes with no authored template produce no entry (and fall back to
    /// the default single implicit column at dispatch).
    #[must_use]
    pub fn grid_templates_from(
        &self,
        computed: &ComputedStyles,
        viewport: Size,
        root_font_size: f32,
        paragraph: Option<&ParagraphShaper>,
    ) -> GridTemplates {
        grid_templates_from(computed, viewport, root_font_size, paragraph)
    }

    /// Resolves every grid item's `grid-area: <name>` marker against its
    /// container's `grid-template-areas`, patching the item's [`LayoutStyle`]
    /// placement in `styles` (spec §4.2).
    ///
    /// Named-area placement is the one grid feature that needs the tree: a child's
    /// `grid-area: header` names a rectangle defined on its **container**'s
    /// `grid-template-areas`. This runs after
    /// [`layout_styles_from`](Self::layout_styles_from) has built `styles`; it only
    /// overwrites the `grid_placement` of items carrying a marker (looked up
    /// against the parent's parsed template), so every non-named layout is left
    /// byte-for-byte identical.
    pub fn resolve_named_grid_areas(
        &self,
        styles: &mut LayoutStyles,
        tree: &RetainedTree,
        computed: &ComputedStyles,
    ) {
        resolve_named_grid_areas(styles, tree, computed);
    }

    /// Resolves every grid item's named `grid-column` / `grid-row` line reference
    /// against its container's `grid-template-columns` / `grid-template-rows`
    /// `[name]` line names, patching the item's [`LayoutStyle`] placement in
    /// `styles` (spec §4.13).
    ///
    /// Like named-area placement, a named line (`grid-column: sidebar-start`) is
    /// declared on the item's **container**'s track list, so this runs after
    /// [`layout_styles_from`](Self::layout_styles_from) and only overwrites the
    /// named edges of items that reference one (looked up against the parent's
    /// per-axis line-name map); every non-named layout is left byte-for-byte
    /// identical.
    pub fn resolve_named_grid_lines(
        &self,
        styles: &mut LayoutStyles,
        tree: &RetainedTree,
        computed: &ComputedStyles,
    ) {
        resolve_named_grid_lines(styles, tree, computed);
    }

    /// Lays out like [`layout`](Self::layout) but wraps text-leaf nodes through an
    /// optional `hyphenator`, so an over-wide word breaks at a valid point
    /// (spec §5.10).
    ///
    /// The runtime passes the **same** hyphenator (and shapes the **same** hyphen
    /// glyph) the scene phase composes with, so the measured box height and the
    /// painted line count agree even when hyphenation splits a word. [`None`]
    /// reproduces [`layout`](Self::layout) exactly.
    ///
    /// # Scroll offset and `position: sticky`
    ///
    /// `scroll_offset` is the test-drivable viewport scroll `(x, y)` in logical
    /// pixels: the document-coordinate point currently at the top-left of the
    /// viewport. The single scroll container is the **viewport** (per-container
    /// overflow scroll containers are a follow-up); live scroll-gesture wiring is
    /// separate and simply feeds this value. It is consumed only by
    /// [`Position::Sticky`] nodes, whose used position is clamped to the scrollport
    /// edge within their containing block. With `scroll_offset == (0.0, 0.0)` (and
    /// a sticky node still below its inset threshold) every node's box is
    /// **byte-identical** to the pre-sticky walk.
    // The tree, the three per-node side-tables (styles, computed, grid templates),
    // the viewport + scroll offset, and the shaping inputs (paragraph + hyphenator)
    // are cohesive inputs of one layout entry point, not an incidental over-long
    // parameter list.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn layout_with_hyphenator(
        &self,
        tree: &RetainedTree,
        styles: &LayoutStyles,
        computed: &ComputedStyles,
        grid_templates: &GridTemplates,
        viewport: Size,
        scroll_offset: (f32, f32),
        paragraph: &ParagraphShaper,
        hyphenator: Option<&dyn Hyphenator>,
    ) -> LayoutBoxes {
        // No editing session: every text leaf measures from the retained tree,
        // byte-identical to before the editing-aware measure path existed.
        self.layout_editing(
            tree,
            styles,
            computed,
            grid_templates,
            viewport,
            scroll_offset,
            paragraph,
            hyphenator,
            None,
        )
    }

    /// Lays out like [`layout_with_hyphenator`](Self::layout_with_hyphenator) but,
    /// while a text-editing session is active, measures the **focused editable's**
    /// text from the live session buffer instead of the (possibly lagging) retained
    /// tree — the SAME string, with the SAME processing, the scene phase composes
    /// (CSS Text 3 §4.1.1).
    ///
    /// The retained tree only catches up on the app's reactive rebuild, so between
    /// a keystroke and that rebuild the tree value lags the session by a character.
    /// Sizing the box from the retained value while the scene paints the session
    /// buffer lets the arranged box (line count / width) disagree with the painted
    /// glyphs — worst on a `white-space: normal` / `pre-wrap` field that wraps to a
    /// different number of lines. Threading the same [`EditingView`] into both
    /// passes makes them shape the identical string (raw buffer + the field's real
    /// `white-space`, `process_text` skipped so caret byte offsets stay exact), so
    /// the measured box height equals the painted line count. With `editing` =
    /// [`None`] — or for any text node that is **not** the focused editable's text —
    /// this is byte-for-byte
    /// [`layout_with_hyphenator`](Self::layout_with_hyphenator).
    // The editing view is the one added optional input over `layout_with_hyphenator`;
    // the rest are the same cohesive layout inputs.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn layout_editing(
        &self,
        tree: &RetainedTree,
        styles: &LayoutStyles,
        computed: &ComputedStyles,
        grid_templates: &GridTemplates,
        viewport: Size,
        scroll_offset: (f32, f32),
        paragraph: &ParagraphShaper,
        hyphenator: Option<&dyn Hyphenator>,
        editing: Option<EditingView<'_>>,
    ) -> LayoutBoxes {
        let mut boxes = LayoutBoxes::new();
        let Some(root) = tree.root() else {
            return boxes;
        };
        // The root's containing block — and the fixed-positioning reference for
        // every `position: fixed` descendant — is the viewport rect at the origin.
        let viewport_rect = Rect::new(Point::ORIGIN, viewport);
        // One measure memoization for the whole pass (see [`MeasureCache`]).
        let cache = MeasureCache::default();
        arrange(
            tree,
            styles,
            computed,
            grid_templates,
            root,
            Point::ORIGIN,
            BoxConstraints::tight(viewport),
            &mut boxes,
            paragraph,
            hyphenator,
            viewport_rect,
            viewport_rect,
            scroll_offset,
            editing,
            MAX_DEPTH,
            &cache,
        );
        boxes
    }
}

/// Shapes and wraps a text node's `content` to its available width and returns
/// the wrapped paragraph's border-box size, constrained by `constraints`.
///
/// The content is shaped once at `style.font_size` through the multi-font
/// `paragraph` orchestrator (itemize → resolve → shape) into one or more
/// font-tagged runs, then wrapped (greedy word wrap) to the finite
/// `constraints.max.width`, minus the node's own horizontal border/padding
/// (usually zero for a text node). The returned [`size`](ParagraphLayout::size)
/// is the widest line's width by the summed line heights, so a long string in a
/// narrow width yields a taller box; the scene phase re-wraps to this box width
/// through the **same** orchestrator and so emits exactly the same lines.
///
/// `fallback` is the node's font chain resolved from its computed `font-family`
/// (via [`families_for`]); `attributes` (weight + slant, via [`attributes_for`]),
/// `direction` (the base bidi direction, via [`direction_for`]), and `alignment`
/// (the line alignment, via [`alignment_for`]) come from the node's computed
/// `font-weight` / `font-style` / `direction` / `text-align`. The scene phase
/// resolves the identical chain, attributes, direction, and alignment, so the
/// measured box and the painted glyphs use the same fonts, reordering, and
/// justification.
// The shaping inputs (fallback chain, attributes, direction, alignment) plus the
// style/constraints/shaper are cohesive positional arguments of one leaf-measure
// helper, not a public API surface. `attributes` is kept by reference to match
// the `shape_paragraph`/`families_for` convention used throughout this pass.
#[allow(clippy::too_many_arguments, clippy::trivially_copy_pass_by_ref)]
fn measure_text_node(
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

/// Shapes a single hyphen (`-`) through `paragraph` at `font_size` and returns
/// its first glyph, for [`ParagraphLayout::wrap_hyphenated`] to render at a
/// break (spec §5.10).
///
/// The hyphen is shaped with the same [`measure_face`] the scene coordinator's
/// default face mirrors, so the glyph the measure pass sizes with is identical
/// to the one the compose pass paints, keeping the box height and painted line
/// count in agreement. Returns [`None`] only if shaping yields no glyph.
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

/// Shapes a single ellipsis (`…`) through `paragraph` at `font_size` and returns
/// its first glyph, for [`ParagraphLayout::wrap_hyphenated`] to append when
/// `text-overflow: ellipsis` truncates an overflowing line (spec §5.10).
///
/// The ellipsis is shaped with the same [`measure_face`] the scene coordinator's
/// default face mirrors, so the glyph the measure pass sizes with is identical to
/// the one the compose pass paints, keeping the box width and painted lines in
/// agreement. Returns [`None`] only if shaping yields no glyph.
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

/// Resolves the `tab-size` tab-stop width in logical pixels for `content`
/// (CSS Text 3 §4.3), or `0.0` when `content` has no preserved tab.
///
/// A `<number>` tab-size is `factor` × the advance of a space shaped through the
/// same orchestrator/face the measure and compose passes use (so both agree); a
/// `<length>` tab-size needs no space and returns its fixed width. When there is
/// no tab the space is never shaped and `0.0` is returned, so the tab pass
/// early-returns and an unauthored / collapsing-mode node is unaffected.
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

/// Shapes a single space (U+0020) through `paragraph` at `font_size` and returns
/// its total advance, the unit a `<number>` `tab-size` multiplies (§4.3).
///
/// Shaped with the same [`measure_face`] the scene coordinator mirrors, so the
/// space width the measure pass resolves the tab stop from is identical to the
/// compose pass's. Returns `0.0` when shaping yields no glyph.
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

/// Shapes the `0` (ZERO, U+0030) glyph through `paragraph` at `font_size` and
/// returns its advance — the base every `ch` length resolves against (spec §4.5,
/// CSS Values 4 §6.1.2).
///
/// Returns [`None`] when shaping yields no glyph, which is the honest answer
/// rather than `0.0`: a zero-width `ch` would silently collapse `width: 60ch` to
/// nothing, while [`None`] routes the caller to
/// [`LengthContext::ch_base`]'s specified `0.5em` substitute.
///
/// # This measures the same font the text does
///
/// Shaped through the same [`measure_face`] + fallback chain + attributes as
/// [`measure_text_node`], so a `20ch` box and the twenty characters placed in it
/// are measured against one font, not two. That also means `ch` inherits this
/// pass's standing limitation: [`measure_face`] is a carrier, and the wired
/// shaper resolves the *host's* font rather than the node's `font-family`. So
/// `ch` currently follows the host font — exactly as every other text
/// measurement here does — and starts following `font-family` on the day real
/// per-node face resolution lands, with no change to this function.
// `attributes` stays by reference to match this pass's `shape_paragraph` calls.
#[allow(clippy::trivially_copy_pass_by_ref)]
fn measure_ch_width(
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

/// The `ch` base for one node, measured only when that node actually needs it.
///
/// [`ComputedStyle::uses_ch`] is an allocation-free scan and answers `false` for
/// essentially every real node; a hit is what pays for the fallback chain and the
/// shaping call. So a stylesheet that never writes `ch` costs exactly what it did
/// before `ch` existed, which is the point — `ch` is a rare unit and must not tax
/// the nodes that do not use it.
pub(super) fn ch_width_for(
    computed: &ComputedStyle,
    paragraph: Option<&ParagraphShaper>,
) -> Option<f32> {
    if !computed.uses_ch() {
        return None;
    }
    let paragraph = paragraph?;
    measure_ch_width(
        paragraph,
        &families_for(Some(computed)),
        &attributes_for(Some(computed)),
        font_size_px(computed),
    )
}

/// Maps a single [`ComputedStyle`] into layout's own [`LayoutStyle`].
///
/// This is the full computed → layout bridge (spec §3.10, §4.2): every field of
/// [`LayoutStyle`] is populated from the corresponding computed property so that
/// flex/grid formatting, the box model, and the flex item/container knobs all
/// engage directly from CSS. A property that is absent (or has the wrong value
/// kind) falls back to the matching [`LayoutStyle::default`] value.
///
/// The mapping, field by field:
///
/// * `display` ← [`PropertyId::Display`] keyword. The computed value is a
///   [`PropertyValue::Keyword`] backed by an [`InternedSymbol`]; it is compared
///   against the reserved [`layout_keyword`] handles — `DISPLAY_FLEX` → `Flex`,
///   `DISPLAY_GRID` → `Grid`, `DISPLAY_NONE` → `None`, everything else (including
///   `DISPLAY_BLOCK` and an absent property) → `Block`. Because both the CSS
///   compiler and this bridge use those reserved symbol values, the identities
///   compare equal across crates without a shared interner.
/// * `width` / `height` / `min_width` / `min_height` / `max_width` /
///   `max_height` ← the matching `PropertyId::{Width, Height, MinWidth,
///   MinHeight, MaxWidth, MaxHeight}` length, via [`dimension_of`]
///   ([`Length::Px`] → [`Dimension::Px`], [`Length::Percent`] →
///   [`Dimension::Percent`], [`Length::Auto`]/absent → [`Dimension::Auto`]).
/// * `margin` / `padding` / `border` ← the four `*-top/right/bottom/left`
///   longhands (the `border-*-width` longhands for `border`), resolved to pixel
///   [`Insets`] via [`insets_of`]. Insets are pixel-only, so a `Percent`/`Auto`
///   length contributes `0.0` on that edge. An unauthored `border-*-width`
///   computes to its `0` initial, so `border` stays zero by default.
/// * `gap` ← [`PropertyId::Gap`] length in pixels (`Percent`/`Auto` → `0.0`).
/// * `flex_grow` / `flex_shrink` ← `PropertyId::{FlexGrow, FlexShrink}` numbers,
///   defaulting to the [`LayoutStyle::default`] values (`0.0` / `1.0`).
/// * `flex_basis` ← [`PropertyId::FlexBasis`] via [`dimension_of`].
/// * `flex_direction` ← [`PropertyId::FlexDirection`] keyword against the
///   reserved `FLEX_*` handles (default `Row`).
/// * `flex_wrap` ← [`PropertyId::FlexWrap`] keyword against the reserved
///   `FLEX_WRAP_*` handles (default `NoWrap`, i.e. single-line).
/// * `justify_content` ← [`PropertyId::JustifyContent`] keyword against the
///   reserved `JUSTIFY_*` handles (default `FlexStart`).
/// * `align_content` ← [`PropertyId::AlignContent`] keyword against the same
///   reserved `JUSTIFY_*` handles (grid row-track distribution; default
///   `FlexStart`).
/// * `align_items` ← [`PropertyId::AlignItems`] keyword against the reserved
///   `ALIGN_*` handles (default `Stretch`).
fn layout_style_from(
    computed: &ComputedStyle,
    viewport: Size,
    root_font_size: f32,
    paragraph: Option<&ParagraphShaper>,
) -> LayoutStyle {
    let default = LayoutStyle::default();
    let ctx = length_context(
        computed,
        viewport,
        root_font_size,
        ch_width_for(computed, paragraph),
    );
    LayoutStyle {
        display: display_of(computed),
        // Positioning scheme + insets. An unauthored `position` stays `Static`
        // and unauthored insets stay `Auto`, so a default node contributes no
        // offset — byte-for-byte identical to before positioning was wired.
        position: position_of(computed),
        inset: Inset::new(
            dimension_of(computed, PropertyId::Top, &ctx),
            dimension_of(computed, PropertyId::Right, &ctx),
            dimension_of(computed, PropertyId::Bottom, &ctx),
            dimension_of(computed, PropertyId::Left, &ctx),
        ),
        width: dimension_of(computed, PropertyId::Width, &ctx),
        height: dimension_of(computed, PropertyId::Height, &ctx),
        // `box-sizing`: an unauthored value stays `content-box` (the CSS default),
        // so a node that never sets it is byte-for-byte identical to before.
        box_sizing: box_sizing_of(computed),
        min_width: dimension_of(computed, PropertyId::MinWidth, &ctx),
        min_height: dimension_of(computed, PropertyId::MinHeight, &ctx),
        max_width: dimension_of(computed, PropertyId::MaxWidth, &ctx),
        max_height: dimension_of(computed, PropertyId::MaxHeight, &ctx),
        margin: margin_edges_of(
            computed,
            PropertyId::MarginTop,
            PropertyId::MarginRight,
            PropertyId::MarginBottom,
            PropertyId::MarginLeft,
            &ctx,
        ),
        padding: insets_of(
            computed,
            PropertyId::PaddingTop,
            PropertyId::PaddingRight,
            PropertyId::PaddingBottom,
            PropertyId::PaddingLeft,
            &ctx,
        ),
        // Border insets from the four `border-*-width` longhands. Unauthored
        // widths compute to `0` (their initial), so a node with no border keeps
        // the zero default — byte-for-byte identical to before borders existed.
        border: insets_of(
            computed,
            PropertyId::BorderTopWidth,
            PropertyId::BorderRightWidth,
            PropertyId::BorderBottomWidth,
            PropertyId::BorderLeftWidth,
            &ctx,
        ),
        flex_direction: flex_direction_of(computed),
        // `flex-wrap`: an unauthored value stays `NoWrap` (single-line), so a
        // container that never wraps is byte-for-byte identical to before.
        flex_wrap: flex_wrap_of(computed),
        justify_content: justify_content_of(computed),
        // Grid container: block-axis (row) track distribution. Unauthored stays
        // `FlexStart` (no distribution), so a default grid packs rows at the top.
        align_content: align_content_of(computed),
        align_items: align_items_of(computed),
        // Grid container item-alignment defaults + per-item overrides. Unauthored
        // values stay `Stretch` / `Auto`, so a default grid fills each cell
        // exactly as before.
        justify_items: justify_items_of(computed),
        align_self: align_self_of(computed, PropertyId::AlignSelf),
        justify_self: align_self_of(computed, PropertyId::JustifySelf),
        gap: length_px_of(computed, PropertyId::Gap, &ctx),
        flex_grow: number_of(computed, PropertyId::FlexGrow, default.flex_grow),
        flex_shrink: number_of(computed, PropertyId::FlexShrink, default.flex_shrink),
        flex_basis: dimension_of(computed, PropertyId::FlexBasis, &ctx),
        // The node's font size, so a text leaf can shape and wrap to its
        // available width inside the constraint-aware measure/arrange pass.
        font_size: font_size_px(computed),
        // Line-based grid placement from the four `grid-column`/`grid-row`
        // longhands; unauthored edges stay `Auto`, so a non-placed item keeps the
        // all-auto default and auto-places exactly as before.
        grid_placement: grid_placement_of(computed),
        // The container's `grid-auto-flow`; an unauthored value stays `Row`
        // (sparse), so auto-placement is byte-identical to before.
        grid_auto_flow: grid_auto_flow_of(computed),
    }
}

/// Resolves the computed `grid-auto-flow` keyword to a [`GridAutoFlow`],
/// defaulting to [`GridAutoFlow::Row`].
///
/// Compares the interned keyword symbol against the reserved [`layout_keyword`]
/// `GRID_AUTO_FLOW_*` handles (each authored combination folds to one symbol);
/// an absent or unrecognized value maps to [`GridAutoFlow::Row`].
fn grid_auto_flow_of(computed: &ComputedStyle) -> GridAutoFlow {
    match keyword_sym(computed, PropertyId::GridAutoFlow) {
        Some(sym) if sym == layout_keyword::GRID_AUTO_FLOW_COLUMN => GridAutoFlow::Column,
        Some(sym) if sym == layout_keyword::GRID_AUTO_FLOW_ROW_DENSE => GridAutoFlow::RowDense,
        Some(sym) if sym == layout_keyword::GRID_AUTO_FLOW_COLUMN_DENSE => {
            GridAutoFlow::ColumnDense
        }
        _ => GridAutoFlow::Row,
    }
}

/// Assembles a [`GridPlacement`] from the four computed grid-line longhands
/// (`grid-column-start` / `-end` / `grid-row-start` / `-end`).
///
/// Each edge is read via [`grid_line_of`]; an absent edge is [`GridLine::Auto`], so
/// a node that authored no placement yields the all-[`GridLine::Auto`] default —
/// byte-for-byte the historical auto-placement behaviour.
fn grid_placement_of(computed: &ComputedStyle) -> GridPlacement {
    GridPlacement::new(
        grid_line_of(computed, PropertyId::GridColumnStart),
        grid_line_of(computed, PropertyId::GridColumnEnd),
        grid_line_of(computed, PropertyId::GridRowStart),
        grid_line_of(computed, PropertyId::GridRowEnd),
    )
}

/// Reads one grid-line longhand and converts the style-local [`GridLineValue`] into
/// an `obscur_layout` [`GridLine`], defaulting to [`GridLine::Auto`] when the
/// property is absent or is not a grid-line value.
///
/// A [`GridLineValue::Named`] line cannot be resolved on the item alone (its name
/// is defined on the container's template), so it maps to [`GridLine::Auto`] here
/// and is patched to a concrete line afterwards by
/// [`resolve_named_grid_lines`](super::resolve_named_grid_lines), mirroring how
/// `grid-area: <name>` is resolved against the parent's `grid-template-areas`.
fn grid_line_of(computed: &ComputedStyle, property: PropertyId) -> GridLine {
    match computed
        .get(property)
        .map(ComputedValue::value)
        .and_then(PropertyValue::as_grid_line)
    {
        Some(GridLineValue::Line(index)) => GridLine::Line(*index),
        Some(GridLineValue::Span(n)) => GridLine::Span(*n),
        Some(GridLineValue::Named(_) | GridLineValue::Auto) | None => GridLine::Auto,
    }
}

/// Builds the placeholder [`FontFace`] used to seed a text-measure
/// [`ShapingRequest`].
///
/// The measure pass shapes through the runtime's installed shaper: the mock is
/// font-agnostic and the wired HarfRust adapter ignores the request face in favor
/// of its host font, so a generic monospace metadata face suffices as the request
/// carrier. Real per-node face resolution (via a `FontProvider`) is a later phase.
///
/// This face is deliberately **identical** to the scene coordinator's default
/// face (same family, attributes, source, and metrics), so a text node measured
/// here and drawn by the scene phase carry the same shaping request. That keeps
/// measure and draw font-consistent and lets the runtime's shaping cache serve
/// the compose pass from the run the measure pass already shaped.
fn measure_face() -> FontFace {
    FontFace::new(
        FontFamily::generic(GenericFamily::Monospace),
        FontAttributes::default(),
        FontSource::system("mock"),
        FontMetrics::new(1000, 800.0, -200.0, 0.0),
    )
}

/// The document root's computed `font-size` in logical pixels, for `rem`.
///
/// Reads the root node's `font-size` (`Length::Px`), defaulting to
/// [`DEFAULT_FONT_SIZE`] when the root is absent or its `font-size` is not an
/// absolute pixel length. `font-size` itself expressed in relative units is not
/// resolved here (a later extension); the initial `16px` is used instead.
#[must_use]
pub fn root_font_size(computed: &ComputedStyles, root: Option<NodeId>) -> f32 {
    root.and_then(|node| computed.get(&node))
        .map_or(DEFAULT_FONT_SIZE, font_size_px)
}

/// The [`LengthContext`] for one node: its own `font-size` for `em`, the
/// document `root_font_size` for `rem`, the `viewport` for `vw`/`vh`, and
/// `ch_width` — the measured `0`-glyph advance — for `ch`.
///
/// Pass `ch_width: None` when the `0` glyph was not measured, either because the
/// node writes no `ch` (the usual case, see [`ch_width_for`]) or because no
/// shaper was in reach; `ch` then resolves through
/// [`LengthContext::ch_base`]'s `0.5em` substitute.
///
/// The percentage basis is left unknown: a bare `%` is resolved later by layout
/// against the containing block, and a `%` term inside `calc()` therefore does
/// not fold here (that value falls back to `auto`/`0`, still without an
/// unsupported-feature diagnostic).
pub(super) fn length_context(
    computed: &ComputedStyle,
    viewport: Size,
    root_font_size: f32,
    ch_width: Option<f32>,
) -> LengthContext {
    let ctx = LengthContext::new(
        font_size_px(computed),
        root_font_size,
        viewport.width,
        viewport.height,
    );
    match ch_width {
        Some(width) => ctx.with_ch_width(width),
        None => ctx,
    }
}

/// One node's computed `font-size` in logical pixels, defaulting to
/// [`DEFAULT_FONT_SIZE`] when absent or not an absolute pixel length.
fn font_size_px(computed: &ComputedStyle) -> f32 {
    computed
        .get(PropertyId::FontSize)
        .and_then(ComputedValue::as_length)
        .and_then(Length::as_px)
        .map_or(DEFAULT_FONT_SIZE, |px| px.0)
}

/// Resolves one length-valued property into a [`Dimension`], defaulting to
/// [`Dimension::Auto`] when the property is absent or cannot be resolved.
///
/// Absolute pixels and bare percentages pass through directly (percentages stay
/// symbolic for layout to resolve against the containing block); relative and
/// viewport units and `calc()` expressions are folded to pixels against `ctx`.
fn dimension_of(computed: &ComputedStyle, property: PropertyId, ctx: &LengthContext) -> Dimension {
    match computed.get(property).map(ComputedValue::value) {
        Some(&PropertyValue::Length(length)) => match length {
            Length::Px(px) => Dimension::Px(px.0),
            Length::Percent(percent) => Dimension::Percent(percent),
            Length::Auto => Dimension::Auto,
            relative => relative
                .resolve_px(ctx)
                .map_or(Dimension::Auto, Dimension::Px),
        },
        Some(PropertyValue::Calc(expr)) => expr.resolve(ctx).map_or(Dimension::Auto, Dimension::Px),
        // The intrinsic sizing keywords carry no length: they name a question
        // layout answers by measuring content (CSS Sizing 3 §4.1).
        Some(&PropertyValue::Keyword(sym)) if sym == layout_keyword::SIZING_MIN_CONTENT => {
            Dimension::MinContent
        }
        Some(&PropertyValue::Keyword(sym)) if sym == layout_keyword::SIZING_MAX_CONTENT => {
            Dimension::MaxContent
        }
        Some(&PropertyValue::Keyword(sym)) if sym == layout_keyword::SIZING_FIT_CONTENT => {
            Dimension::FitContent
        }
        _ => Dimension::Auto,
    }
}

/// Resolves one length-valued property into a pixel value, defaulting to `0.0`.
///
/// Layout insets and `gap` are pixel-only, so a bare `Percent` or `Auto` length
/// (or an absent/non-length property) contributes `0.0`; relative/viewport units
/// and `calc()` are folded to pixels against `ctx`.
fn length_px_of(computed: &ComputedStyle, property: PropertyId, ctx: &LengthContext) -> f32 {
    match computed.get(property).map(ComputedValue::value) {
        Some(&PropertyValue::Length(length)) => match length {
            Length::Px(px) => px.0,
            Length::Percent(_) | Length::Auto => 0.0,
            relative => relative.resolve_px(ctx).unwrap_or(0.0),
        },
        Some(PropertyValue::Calc(expr)) => expr.resolve(ctx).unwrap_or(0.0),
        _ => 0.0,
    }
}

/// Builds pixel [`Insets`] from the four edge longhands (top/right/bottom/left).
/// Resolves the four margin longhands into [`MarginEdges`], keeping `auto`
/// **unresolved** (CSS 2.1 §10.3.3).
///
/// This is the margin twin of [`insets_of`], and differs in the one way that
/// matters: neither `auto` nor a percentage is folded to a number here, because
/// neither *has* one yet.
///
/// * `auto` means "take the leftover", and how much is left depends on the box's
///   used width, which layout has not computed. That premature `0` is exactly what
///   used to make `margin: 0 auto` do nothing.
/// * A percentage resolves against the *containing block's* inline size (CSS 2.1
///   §8.3 — the width, on all four edges), which this node's own
///   [`LengthContext`] does not know: `ctx.percent_basis` is the node's own
///   basis, not its parent's content width. Layout resolves it where the
///   containing block is in hand.
///
/// `calc()` mixing a percentage still folds to `0` (the expression cannot be
/// resolved without a basis), as it does for every other length here.
fn margin_edges_of(
    computed: &ComputedStyle,
    top: PropertyId,
    right: PropertyId,
    bottom: PropertyId,
    left: PropertyId,
    ctx: &LengthContext,
) -> MarginEdges {
    MarginEdges {
        top: margin_of(computed, top, ctx),
        right: margin_of(computed, right, ctx),
        bottom: margin_of(computed, bottom, ctx),
        left: margin_of(computed, left, ctx),
    }
}

/// Resolves one margin longhand into a [`Margin`]: an authored `auto` stays
/// [`Margin::Auto`] and an authored `%` stays [`Margin::Percent`], while every
/// other value folds to logical pixels exactly as [`length_px_of`] does.
fn margin_of(computed: &ComputedStyle, property: PropertyId, ctx: &LengthContext) -> Margin {
    match computed.get(property).map(ComputedValue::value) {
        Some(&PropertyValue::Length(Length::Auto)) => Margin::Auto,
        Some(&PropertyValue::Length(Length::Percent(percent))) => Margin::Percent(percent),
        _ => Margin::Px(length_px_of(computed, property, ctx)),
    }
}

fn insets_of(
    computed: &ComputedStyle,
    top: PropertyId,
    right: PropertyId,
    bottom: PropertyId,
    left: PropertyId,
    ctx: &LengthContext,
) -> Insets {
    Insets {
        top: length_px_of(computed, top, ctx),
        right: length_px_of(computed, right, ctx),
        bottom: length_px_of(computed, bottom, ctx),
        left: length_px_of(computed, left, ctx),
    }
}

/// Resolves one number-valued property, defaulting to `default` when the
/// property is absent or not a bare number.
fn number_of(computed: &ComputedStyle, property: PropertyId, default: f32) -> f32 {
    match computed.get(property).map(ComputedValue::value) {
        Some(&PropertyValue::Number(value)) => value,
        _ => default,
    }
}

/// Extracts the interned keyword symbol of a keyword-valued property, or [`None`]
/// when the property is absent or is not a keyword.
fn keyword_sym(computed: &ComputedStyle, property: PropertyId) -> Option<InternedSymbol> {
    match computed.get(property).map(ComputedValue::value) {
        Some(&PropertyValue::Keyword(sym)) => Some(sym),
        _ => None,
    }
}

/// Resolves the computed `display` keyword to a [`LayoutDisplay`].
///
/// Compares the interned symbol against the reserved [`layout_keyword`] handles;
/// an absent or unrecognized value maps to [`LayoutDisplay::Block`] (which also
/// covers `DISPLAY_BLOCK`).
fn display_of(computed: &ComputedStyle) -> LayoutDisplay {
    match keyword_sym(computed, PropertyId::Display) {
        Some(sym) if sym == layout_keyword::DISPLAY_FLEX => LayoutDisplay::Flex,
        Some(sym) if sym == layout_keyword::DISPLAY_GRID => LayoutDisplay::Grid,
        Some(sym) if sym == layout_keyword::DISPLAY_NONE => LayoutDisplay::None,
        _ => LayoutDisplay::Block,
    }
}

/// Resolves the computed `box-sizing` keyword to a [`BoxSizing`], defaulting to
/// [`BoxSizing::ContentBox`] (the CSS initial).
///
/// Compares the interned symbol against the reserved [`layout_keyword`]
/// `BOX_SIZING_*` handles; an absent or unrecognized value maps to
/// [`BoxSizing::ContentBox`] (which also covers `BOX_SIZING_CONTENT_BOX`).
fn box_sizing_of(computed: &ComputedStyle) -> BoxSizing {
    match keyword_sym(computed, PropertyId::BoxSizing) {
        Some(sym) if sym == layout_keyword::BOX_SIZING_BORDER_BOX => BoxSizing::BorderBox,
        _ => BoxSizing::ContentBox,
    }
}

/// Resolves the computed `position` keyword to a [`Position`], defaulting to
/// [`Position::Static`].
///
/// Compares the interned symbol against the reserved [`layout_keyword`]
/// `POSITION_*` handles; an absent or unrecognized value maps to
/// [`Position::Static`] (which also covers `POSITION_STATIC`).
fn position_of(computed: &ComputedStyle) -> Position {
    match keyword_sym(computed, PropertyId::Position) {
        Some(sym) if sym == layout_keyword::POSITION_RELATIVE => Position::Relative,
        Some(sym) if sym == layout_keyword::POSITION_ABSOLUTE => Position::Absolute,
        Some(sym) if sym == layout_keyword::POSITION_FIXED => Position::Fixed,
        Some(sym) if sym == layout_keyword::POSITION_STICKY => Position::Sticky,
        _ => Position::Static,
    }
}

/// Resolves the computed `flex-direction` keyword to a [`FlexDirection`],
/// defaulting to [`FlexDirection::Row`].
fn flex_direction_of(computed: &ComputedStyle) -> FlexDirection {
    match keyword_sym(computed, PropertyId::FlexDirection) {
        Some(sym) if sym == layout_keyword::FLEX_ROW_REVERSE => FlexDirection::RowReverse,
        Some(sym) if sym == layout_keyword::FLEX_COLUMN => FlexDirection::Column,
        Some(sym) if sym == layout_keyword::FLEX_COLUMN_REVERSE => FlexDirection::ColumnReverse,
        _ => FlexDirection::Row,
    }
}

/// Resolves a reserved `JUSTIFY_*` keyword symbol to a [`JustifyContent`],
/// defaulting to [`JustifyContent::FlexStart`] for an absent or unrecognized
/// value. Shared by `justify-content` (flex main axis / grid columns) and
/// `align-content` (grid rows), which use the same closed keyword set.
fn justify_content_keyword(sym: Option<InternedSymbol>) -> JustifyContent {
    match sym {
        Some(sym) if sym == layout_keyword::JUSTIFY_CENTER => JustifyContent::Center,
        Some(sym) if sym == layout_keyword::JUSTIFY_END => JustifyContent::FlexEnd,
        Some(sym) if sym == layout_keyword::JUSTIFY_SPACE_BETWEEN => JustifyContent::SpaceBetween,
        Some(sym) if sym == layout_keyword::JUSTIFY_SPACE_AROUND => JustifyContent::SpaceAround,
        Some(sym) if sym == layout_keyword::JUSTIFY_SPACE_EVENLY => JustifyContent::SpaceEvenly,
        _ => JustifyContent::FlexStart,
    }
}

/// Resolves the computed `justify-content` keyword to a [`JustifyContent`],
/// defaulting to [`JustifyContent::FlexStart`]. On a grid container this is the
/// inline-axis (column) distribution of leftover track space.
fn justify_content_of(computed: &ComputedStyle) -> JustifyContent {
    justify_content_keyword(keyword_sym(computed, PropertyId::JustifyContent))
}

/// Resolves the computed `align-content` keyword to a [`JustifyContent`],
/// defaulting to [`JustifyContent::FlexStart`]. On a grid container this is the
/// block-axis (row) distribution of leftover track space; it shares the
/// `justify-content` keyword set.
fn align_content_of(computed: &ComputedStyle) -> JustifyContent {
    justify_content_keyword(keyword_sym(computed, PropertyId::AlignContent))
}

/// Resolves the computed `flex-wrap` keyword to a [`FlexWrap`], defaulting to
/// [`FlexWrap::NoWrap`] for an absent or unrecognized value.
fn flex_wrap_of(computed: &ComputedStyle) -> FlexWrap {
    match keyword_sym(computed, PropertyId::FlexWrap) {
        Some(sym) if sym == layout_keyword::FLEX_WRAP_WRAP => FlexWrap::Wrap,
        Some(sym) if sym == layout_keyword::FLEX_WRAP_WRAP_REVERSE => FlexWrap::WrapReverse,
        _ => FlexWrap::NoWrap,
    }
}

/// Resolves the computed `align-items` keyword to an [`AlignItems`], defaulting
/// to [`AlignItems::Stretch`].
fn align_items_of(computed: &ComputedStyle) -> AlignItems {
    match keyword_sym(computed, PropertyId::AlignItems) {
        Some(sym) if sym == layout_keyword::ALIGN_START => AlignItems::FlexStart,
        Some(sym) if sym == layout_keyword::ALIGN_CENTER => AlignItems::Center,
        Some(sym) if sym == layout_keyword::ALIGN_END => AlignItems::FlexEnd,
        _ => AlignItems::Stretch,
    }
}

/// Resolves the computed grid-container `justify-items` keyword to an
/// [`AlignItems`], defaulting to [`AlignItems::Stretch`].
///
/// This shares the reserved `ALIGN_*` handle set with `align-items`; the grid
/// interprets it as the default *inline-axis* cell alignment.
fn justify_items_of(computed: &ComputedStyle) -> AlignItems {
    match keyword_sym(computed, PropertyId::JustifyItems) {
        Some(sym) if sym == layout_keyword::ALIGN_START => AlignItems::Start,
        Some(sym) if sym == layout_keyword::ALIGN_CENTER => AlignItems::Center,
        Some(sym) if sym == layout_keyword::ALIGN_END => AlignItems::End,
        _ => AlignItems::Stretch,
    }
}

/// Resolves a computed grid-item `justify-self` / `align-self` keyword (named by
/// `property`) to an [`AlignSelf`], defaulting to [`AlignSelf::Auto`].
///
/// The reserved `ALIGN_AUTO` handle maps to [`AlignSelf::Auto`] (inherit the
/// container's items default); the start/end/center/stretch handles map to the
/// matching override. An absent or unrecognized value is [`AlignSelf::Auto`].
fn align_self_of(computed: &ComputedStyle, property: PropertyId) -> AlignSelf {
    match keyword_sym(computed, property) {
        Some(sym) if sym == layout_keyword::ALIGN_START => AlignSelf::Start,
        Some(sym) if sym == layout_keyword::ALIGN_CENTER => AlignSelf::Center,
        Some(sym) if sym == layout_keyword::ALIGN_END => AlignSelf::End,
        Some(sym) if sym == layout_keyword::ALIGN_STRETCH => AlignSelf::Stretch,
        _ => AlignSelf::Auto,
    }
}

/// Selects the [`LayoutAlgorithm`] for `node`'s `display` mode, or [`None`] when
/// the node generates no box ([`LayoutDisplay::None`]).
///
/// A [`LayoutDisplay::Grid`] node that authored a template (present in
/// `grid_templates`) dispatches the real track-sizing algorithm via
/// [`GridLayout::with_template`], with the node's `gap` supplying both the column
/// and row gap; a grid node with no authored template keeps the default single
/// implicit column ([`GridLayout::new(1)`]) — spec-faithful when
/// `grid-template-columns` is unspecified. The returned trait object is a
/// stateless algorithm; the caller invokes it with the node's style, children,
/// constraints, and a measurer.
fn algorithm_for(
    display: LayoutDisplay,
    node: NodeId,
    grid_templates: &GridTemplates,
    gap: f32,
) -> Option<Box<dyn LayoutAlgorithm>> {
    match display {
        LayoutDisplay::Block => Some(Box::new(FlowLayout)),
        LayoutDisplay::Flex => Some(Box::new(FlexLayout)),
        LayoutDisplay::Grid => Some(match grid_templates.get(&node) {
            Some(template) => Box::new(GridLayout::with_template(template.clone(), gap, gap)),
            None => Box::new(GridLayout::new(1)),
        }),
        LayoutDisplay::None => None,
    }
}

/// Looks up a node's [`LayoutStyle`], falling back to the default.
fn style_of(styles: &LayoutStyles, node: NodeId) -> LayoutStyle {
    styles.get(&node).copied().unwrap_or_default()
}

/// Collects a node's children as [`LayoutChild`] descriptors (identity + style).
///
/// A node absent from the tree contributes no children. Each child's style is
/// resolved via [`style_of`] so the algorithm sees a complete descriptor.
fn children_of(tree: &RetainedTree, styles: &LayoutStyles, node: NodeId) -> Vec<LayoutChild> {
    tree.get(node).map_or_else(Vec::new, |n| {
        n.children()
            .iter()
            .map(|&child| LayoutChild::new(child, style_of(styles, child)))
            .collect()
    })
}

/// Collects only a node's **in-flow** children (`Static` / `Relative`).
///
/// Out-of-flow children ([`Position::Absolute`] / [`Position::Fixed`]) are
/// dropped: they neither reach the parent's flow/flex/grid algorithm nor
/// contribute to its content size (spec §4.1). This is the child list the
/// measure pass hands the algorithm, so its indices are stable.
fn in_flow_children(tree: &RetainedTree, styles: &LayoutStyles, node: NodeId) -> Vec<LayoutChild> {
    children_of(tree, styles, node)
        .into_iter()
        .filter(|child| !child.style().position.is_out_of_flow())
        .collect()
}

/// Partitions a node's children into `(in_flow, out_of_flow)`, preserving
/// document order within each list.
///
/// The in-flow list drives the formatting-context algorithm (its indices map to
/// the algorithm's `measure_child`), and the out-of-flow list is positioned
/// separately against the containing block.
fn partition_children(
    tree: &RetainedTree,
    styles: &LayoutStyles,
    node: NodeId,
) -> (Vec<LayoutChild>, Vec<LayoutChild>) {
    children_of(tree, styles, node)
        .into_iter()
        .partition(|child| !child.style().position.is_out_of_flow())
}

/// The **padding box** of a node — its absolute border-box `border_box` inset by
/// its `border` edges — which is the containing block a positioned node
/// establishes for its absolutely-positioned descendants (spec §4.1).
///
/// Extents are clamped non-negative so a box smaller than its borders yields a
/// zero-extent (never negative) containing block. With no authored border-width,
/// `border` is zero and the padding box equals the border box.
fn padding_box(border_box: Rect, border: Insets) -> Rect {
    let x = border_box.origin.x + border.left;
    let y = border_box.origin.y + border.top;
    let width = (border_box.size.width - border.left - border.right).max(0.0);
    let height = (border_box.size.height - border.top - border.bottom).max(0.0);
    Rect::new(Point::new(x, y), Size::new(width, height))
}

/// A per-layout-pass memoization of [`measure`]: a `(node, constraints)` pair is
/// sized **once per frame**, collapsing the redundant re-measurement (and text
/// reshaping) the measure/arrange walk would otherwise do — `arrange(parent)`
/// measures a subtree, then `arrange(child)` re-measures it under the same
/// constraints, and so on down the depth. Keyed by the node and the constraint's
/// four `f32` bit patterns (exact, no hashing/collisions). `RefCell` allows the
/// short borrow for a lookup/insert to nest inside the recursive algorithm call
/// that happens between them.
type MeasureCache = std::cell::RefCell<HashMap<MeasureKey, Size>>;
type MeasureKey = (NodeId, u32, u32, u32, u32);

/// The exact cache key for `measure(node, constraints)`.
fn measure_key(node: NodeId, constraints: BoxConstraints) -> MeasureKey {
    (
        node,
        constraints.min.width.to_bits(),
        constraints.min.height.to_bits(),
        constraints.max.width.to_bits(),
        constraints.max.height.to_bits(),
    )
}

/// The [`ChildMeasurer`] the coordinator hands to an algorithm.
///
/// It measures a child purely — by recursing into [`measure`] — and never
/// touches the results map, which is what keeps [`arrange`] borrow-clean (the
/// `boxes` map is not borrowed for the duration of the algorithm call).
struct RecMeasurer<'a> {
    /// The retained tree being laid out.
    tree: &'a RetainedTree,
    /// Per-node layout styles.
    styles: &'a LayoutStyles,
    /// Per-node computed styles, for resolving a text child's `font-family`.
    computed: &'a ComputedStyles,
    /// Per-node authored grid templates, for dispatching a grid child's real
    /// track-sizing algorithm.
    grid_templates: &'a GridTemplates,
    /// The children of the node currently being laid out, parallel to the
    /// indices the algorithm passes to [`ChildMeasurer::measure_child`].
    children: &'a [LayoutChild],
    /// The paragraph orchestrator used to shape and wrap a text-leaf child.
    paragraph: &'a ParagraphShaper,
    /// The hyphenator used to break an over-wide word in a text-leaf child.
    hyphenator: Option<&'a dyn Hyphenator>,
    /// The active text-editing session view, so a text-leaf child that is the
    /// focused editable's text measures the live session buffer (the SAME string
    /// the scene phase paints); [`None`] on every non-editing frame.
    editing: Option<EditingView<'a>>,
    /// Remaining recursion budget for children measured through this measurer.
    depth: u32,
    /// The shared per-frame measure memoization (see [`MeasureCache`]).
    cache: &'a MeasureCache,
}

impl ChildMeasurer for RecMeasurer<'_> {
    fn measure_child(&mut self, index: usize, constraints: BoxConstraints) -> Size {
        match self.children.get(index) {
            Some(child) => measure(
                self.tree,
                self.styles,
                self.computed,
                self.grid_templates,
                child.node(),
                constraints,
                self.paragraph,
                self.hyphenator,
                self.editing,
                self.depth,
                self.cache,
            ),
            // Out-of-range index: contribute nothing rather than panic.
            None => Size::ZERO,
        }
    }
}

/// Pure measure pass: computes `node`'s border-box size under `constraints`
/// without writing anything.
///
/// This mirrors the algorithm selection of [`arrange`] but discards the child
/// boxes and returns only the node size. Because it never touches the results
/// map, it can be called freely from inside an algorithm (via [`RecMeasurer`])
/// while `arrange` holds a mutable borrow of that map.
///
/// `depth` bounds the recursion: at zero it returns [`Size::ZERO`], guaranteeing
/// termination on a cyclic or pathologically deep tree.
/// Resolves the used `text-overflow` for an anonymous text `node` (CSS UI 3
/// §5.2).
///
/// `text-overflow` applies to the block container and does **not** inherit, so
/// the anonymous text node's own computed value is always the initial `clip`.
/// The value that governs the node's inline content is the block container's, so
/// it is read from the text node's parent, falling back to the node itself when
/// there is none. (Nested inline boxes are not walked to the container — a
/// documented approximation matching this coordinator's flat text model.)
fn overflow_for_text(tree: &RetainedTree, computed: &ComputedStyles, node: NodeId) -> TextOverflow {
    let container = tree
        .get(node)
        .and_then(|n| n.logical_parent().parent())
        .unwrap_or(node);
    overflow_for(computed.get(&container))
}

// The tree, both style maps, the constraints, the shaper, the hyphenator, and
// the depth budget are cohesive positional arguments of one internal recursion.
#[allow(clippy::too_many_arguments)]
fn measure(
    tree: &RetainedTree,
    styles: &LayoutStyles,
    computed: &ComputedStyles,
    grid_templates: &GridTemplates,
    node: NodeId,
    constraints: BoxConstraints,
    paragraph: &ParagraphShaper,
    hyphenator: Option<&dyn Hyphenator>,
    editing: Option<EditingView<'_>>,
    depth: u32,
    cache: &MeasureCache,
) -> Size {
    if depth == 0 {
        return Size::ZERO;
    }
    // Memoize on (node, constraints): the arrange walk measures the same node
    // under the same constraints many times per frame; return the first result.
    let key = measure_key(node, constraints);
    if let Some(&hit) = cache.borrow().get(&key) {
        return hit;
    }
    let style = style_of(styles, node);
    // A text node is a constraint-aware leaf: shape + wrap to `max.width`,
    // bypassing the box algorithms entirely (spec §5.10).
    if let Some(content) = tree.get(node).and_then(|n| n.kind().text()) {
        let fallback = families_for(computed.get(&node));
        let attributes = attributes_for(computed.get(&node));
        let direction = direction_for(computed.get(&node));
        let alignment = alignment_for(computed.get(&node));
        let overflow = overflow_for_text(tree, computed, node);
        // The document root's `font-size` resolves any `rem` in the text metrics
        // below (CSS Values 4 §5.1.1); each metric's `em` / `%` resolves against
        // this node's own `style.font_size`.
        let root_font = root_font_size(computed, tree.root());
        let line_height = line_height_for(computed.get(&node), style.font_size, root_font);
        let white_space = white_space_for(computed.get(&node));
        // The string to shape, via the SAME `display_text` seam the scene phase
        // composes with (CSS Text 3 §4.1.1): while this node is the focused
        // editable's text, the live session buffer is shaped RAW (the field's real
        // `white-space`, `text-transform`/collapsing skipped for exact caret byte
        // offsets); otherwise `text-transform` (§2.1) then `white-space` collapsing
        // (§4.1.1) rewrite the retained string BEFORE shaping. Either way the box is
        // sized from the identical characters the compose pass paints, so its height
        // (line count) matches the painted lines. Letter-/word-spacing and tab stops
        // are applied post-shaping inside `measure_text_node`.
        let active = editing_override(tree, node, editing);
        let content = display_text(
            content,
            active,
            text_transform_for(computed.get(&node)),
            white_space,
        );
        let size = measure_text_node(
            content.as_ref(),
            &style,
            &fallback,
            &attributes,
            direction,
            alignment,
            overflow,
            white_space,
            overflow_wrap_for(computed.get(&node)),
            word_break_for(computed.get(&node)),
            constraints,
            paragraph,
            hyphenator,
            line_height,
            letter_spacing_for(computed.get(&node), style.font_size, root_font),
            word_spacing_for(computed.get(&node), style.font_size, root_font),
            tab_size_for(computed.get(&node), style.font_size, root_font),
        );
        cache.borrow_mut().insert(key, size);
        return size;
    }
    let Some(algorithm) = algorithm_for(style.display, node, grid_templates, style.gap) else {
        return Size::ZERO;
    };
    // Out-of-flow (absolute/fixed) children are excluded from the formatting
    // context: they contribute nothing to this node's content size. Only the
    // in-flow children reach the algorithm, and the measurer's indices are over
    // that in-flow list.
    let children = in_flow_children(tree, styles, node);
    let mut measurer = RecMeasurer {
        tree,
        styles,
        computed,
        grid_templates,
        children: &children,
        paragraph,
        hyphenator,
        editing,
        depth: depth - 1,
        cache,
    };
    let out = algorithm.layout(&style, &children, constraints, &mut measurer);
    let size = out.size();
    cache.borrow_mut().insert(key, size);
    size
}

/// Arrange pass: lays `node` out under `constraints`, writes its **absolute**
/// [`LayoutBox`] (translated by `origin`) into `boxes`, and recurses into each
/// child at its accumulated absolute origin.
///
/// The algorithm is invoked with a [`RecMeasurer`] that only *measures* (pure),
/// so `boxes` is **not** borrowed during the call — after the call returns we
/// are free to mutate `boxes`. This measure/arrange split is what keeps the
/// recursion borrow-clean: measuring never reads or writes the results map, so
/// there is no aliasing between the mutable `boxes` borrow held across the
/// recursion and the shared borrows a measurer needs.
///
/// `depth` bounds the recursion: at zero it returns immediately.
///
/// Child boxes returned by the algorithm are **parent-relative**; each child's
/// absolute origin is `origin + child_box.origin()`. The child count and the
/// returned box count are reconciled defensively so no index can panic.
///
/// # Positioning (spec §4.1)
///
/// `containing_block` is the absolute rect an [`Position::Absolute`] descendant
/// resolves its insets against (the padding box of the nearest positioned
/// ancestor); `viewport` is the fixed-positioning reference for a
/// [`Position::Fixed`] descendant. A **positioned** node
/// ([`Position::is_positioned`]) establishes a *new* containing block for its
/// descendants — its own padding box — which is threaded down in place of the
/// inherited one; a `Static` node is transparent and passes the inherited block
/// through unchanged. Out-of-flow children (partitioned out of the algorithm's
/// in-flow list) are positioned after the in-flow walk, each within its
/// containing block, and then recursed into as an independent positioned root.
// The recursive arrange walk threads the tree, both style maps, the target
// origin/constraints, the containing block + viewport for positioning, the
// output map, the shaper, and the depth budget; these are cohesive positional
// arguments of one internal recursion, not a public API.
#[allow(clippy::too_many_arguments)]
fn arrange(
    tree: &RetainedTree,
    styles: &LayoutStyles,
    computed: &ComputedStyles,
    grid_templates: &GridTemplates,
    node: NodeId,
    origin: Point,
    constraints: BoxConstraints,
    boxes: &mut LayoutBoxes,
    paragraph: &ParagraphShaper,
    hyphenator: Option<&dyn Hyphenator>,
    containing_block: Rect,
    viewport: Rect,
    scroll_offset: (f32, f32),
    editing: Option<EditingView<'_>>,
    depth: u32,
    cache: &MeasureCache,
) {
    if depth == 0 {
        return;
    }
    let style = style_of(styles, node);

    // A text node is a constraint-aware leaf: shape + wrap to `max.width` and
    // write the wrapped box, bypassing the box algorithms (spec §5.10). It has
    // no laid-out children, so there is nothing to recurse into.
    if let Some(content) = tree.get(node).and_then(|n| n.kind().text()) {
        let fallback = families_for(computed.get(&node));
        let attributes = attributes_for(computed.get(&node));
        let direction = direction_for(computed.get(&node));
        let alignment = alignment_for(computed.get(&node));
        let overflow = overflow_for_text(tree, computed, node);
        // The document root's `font-size` for any `rem` in the text metrics (CSS
        // Values 4 §5.1.1); `em` / `%` resolve against this node's own font size.
        let root_font = root_font_size(computed, tree.root());
        let line_height = line_height_for(computed.get(&node), style.font_size, root_font);
        let white_space = white_space_for(computed.get(&node));
        // The SAME string the measure pass sizes and the scene phase paints, via the
        // shared `display_text` seam (CSS Text 3 §4.1.1): the focused editable's text
        // shapes its live session buffer RAW (exact caret byte offsets), everything
        // else runs `text-transform` (§2.1) + `white-space` collapsing (§4.1.1);
        // post-shape spacing (§8.1/§8.2) + tab stops (§4.3) follow inside
        // `measure_text_node`, so the arranged box matches the painted lines.
        let active = editing_override(tree, node, editing);
        let content = display_text(
            content,
            active,
            text_transform_for(computed.get(&node)),
            white_space,
        );
        let size = measure_text_node(
            content.as_ref(),
            &style,
            &fallback,
            &attributes,
            direction,
            alignment,
            overflow,
            white_space,
            overflow_wrap_for(computed.get(&node)),
            word_break_for(computed.get(&node)),
            constraints,
            paragraph,
            hyphenator,
            line_height,
            letter_spacing_for(computed.get(&node), style.font_size, root_font),
            word_spacing_for(computed.get(&node), style.font_size, root_font),
            tab_size_for(computed.get(&node), style.font_size, root_font),
        );
        boxes.insert(node, LayoutBox::new(Rect::new(origin, size)));
        return;
    }

    let Some(algorithm) = algorithm_for(style.display, node, grid_templates, style.gap) else {
        return;
    };
    // Out-of-flow children are excluded from the formatting context (they do not
    // shift in-flow siblings or contribute to content size) and positioned
    // separately below. The in-flow list is what the algorithm sees, so its
    // indices map one-to-one to the algorithm's `measure_child(index)`.
    let (children, out_of_flow) = partition_children(tree, styles, node);

    // Scroll container (CSS Overflow Module Level 3 §scroll-container): an
    // `overflow: scroll`/`auto` box with a **definite (finitely-constrained) block
    // size** clips and scrolls its content, so its content lays out at its NATURAL
    // block size and is allowed to overflow the fixed box — the scrollport. The
    // arrange pass constrains every in-flow node to its measured size (a tight
    // block axis), so `constraints.max.height` here IS the scrollport height. Relax
    // the block-axis max to unbounded for CHILD measurement (so the algorithm lays
    // children out at their natural height instead of clamping them to the box),
    // then pin the node's OWN box back to that scrollport height. A scroll
    // container under an unbounded parent (infinite max height) has no definite
    // size — it grows to fit — so it is left unrelaxed. Only `overflow: scroll`/
    // `auto` nodes are touched; every other node is byte-for-byte identical.
    //
    // The natural content height is produced by the **single** layout pass below
    // (`out.size()`), NOT a second measure: the relaxation only changes this one
    // call's block-axis bound, and overflow is later derived from the resulting
    // absolute boxes (`scrollable_content_height`), so the common frame spends zero
    // extra measures here. The relaxation is also skipped when it provably cannot
    // change anything — a childless scroll container has no content to overflow, so
    // its box is identical relaxed or not; keeping it unrelaxed avoids forcing the
    // whole subtree onto the indefinite-height path for nothing.
    let scroll_relaxed = overflow_scrolls(computed, node)
        && constraints.max.height.is_finite()
        && !children.is_empty();
    let child_constraints = if scroll_relaxed {
        BoxConstraints {
            min: Size::new(constraints.min.width, 0.0),
            max: Size::new(constraints.max.width, f32::INFINITY),
        }
    } else {
        constraints
    };

    // Measurer is pure: it does not borrow `boxes`, so `boxes` is free after.
    let out = {
        let mut measurer = RecMeasurer {
            tree,
            styles,
            computed,
            grid_templates,
            children: &children,
            paragraph,
            hyphenator,
            editing,
            depth: depth - 1,
            cache,
        };
        algorithm.layout(&style, &children, child_constraints, &mut measurer)
    };

    // Now safe to write this node's absolute box. A relaxed scroll container keeps
    // its scrollport height (the tight block constraint) while its content
    // overflows; every other node uses the algorithm's own size verbatim.
    let node_size = if scroll_relaxed {
        Size::new(out.size().width, constraints.max.height)
    } else {
        out.size()
    };
    let border_box = Rect::new(origin, node_size);
    boxes.insert(node, LayoutBox::new(border_box));

    // The containing block this node establishes for its positioned descendants:
    // a positioned node (`relative`/`absolute`/`fixed`) hands down its own padding
    // box; a `static` node is transparent and passes the inherited block through
    // (so an absolute grandchild resolves against the nearest positioned ancestor,
    // not this static node). `fixed` descendants use `viewport` regardless.
    let descendant_cb = if style.position.is_positioned() {
        padding_box(border_box, style.border)
    } else {
        containing_block
    };

    // The containing block for a relatively-positioned child is this node's
    // content box; we approximate its size with this node's border-box size
    // (`out.size()`), which is exact when the node has no border/padding and is
    // the only containing-block size available at this point in the walk. Only a
    // definite percentage inset is affected by the approximation; a `px` inset is
    // exact.
    let containing = out.size();

    // Recurse into in-flow children, translating each parent-relative box to
    // absolute. Zipping bounds both sides, so a mismatched length never indexes
    // out of range.
    // The scroll container's visible rect in document coordinates: the viewport
    // translated by the current scroll offset. Its top-left is the doc-coord point
    // now at the top-left of the viewport, so a `position: sticky` child clamps its
    // used position against this rect's edges.
    let scrollport = Rect::new(
        Point::new(
            viewport.origin.x + scroll_offset.0,
            viewport.origin.y + scroll_offset.1,
        ),
        viewport.size,
    );
    for (child, child_rel) in children.iter().zip(out.child_boxes().iter()) {
        let rel = child_rel.origin();
        let child_style = style_of(styles, child.node());
        // The static (normal-flow) absolute origin the algorithm computed for this
        // child, before any positioning offset.
        let static_origin = Point::new(origin.x + rel.x, origin.y + rel.y);
        // A `position: sticky` child stays IN flow (its flow box, and therefore its
        // siblings, are untouched — siblings use the unchanged flow boxes), but its
        // used position is scroll-clamped to the scrollport edge within its
        // containing block; the delta rides its whole subtree, exactly like the
        // relative offset. A `position: relative` child instead offsets by its
        // resolved insets. Any other (`static`/out-of-flow) child yields `(0, 0)`,
        // keeping the walk byte-identical to before. When `scroll_offset` is
        // `(0, 0)` and a sticky child is still below its inset threshold, the sticky
        // delta is `(0, 0)` too, so the default path is unchanged.
        let (dx, dy) = if child_style.position.is_sticky() {
            child_style.inset.sticky_offset(
                static_origin,
                child_rel.size(),
                containing_block,
                scrollport,
            )
        } else {
            child_style.relative_offset(containing.width, containing.height)
        };
        let child_origin = Point::new(static_origin.x + dx, static_origin.y + dy);
        arrange(
            tree,
            styles,
            computed,
            grid_templates,
            child.node(),
            child_origin,
            BoxConstraints::tight(child_rel.size()),
            boxes,
            paragraph,
            hyphenator,
            descendant_cb,
            viewport,
            scroll_offset,
            editing,
            depth - 1,
            cache,
        );
    }

    // Position each out-of-flow child within its containing block, then recurse
    // into its subtree as an independent positioned root.
    for child in &out_of_flow {
        arrange_out_of_flow(
            tree,
            styles,
            computed,
            grid_templates,
            *child,
            descendant_cb,
            viewport,
            scroll_offset,
            boxes,
            paragraph,
            hyphenator,
            editing,
            depth - 1,
            cache,
        );
    }
}

/// Positions one out-of-flow ([`Position::Absolute`] / [`Position::Fixed`])
/// child within its containing block and recurses into its subtree (spec §4.1).
///
/// The containing block is `parent_cb` (the nearest positioned ancestor's padding
/// box) for an absolute child and `viewport` for a fixed one. The child's natural
/// size is measured under a loose constraint bounded by the containing block, then
/// [`Inset::resolve_absolute_box`] resolves its absolute border box from its insets
/// and explicit size. The subtree is arranged under a tight constraint at that box,
/// establishing the child's own padding box as the containing block for *its*
/// positioned descendants (the child is itself positioned).
// The out-of-flow placement threads the same cohesive recursion inputs as
// [`arrange`]; they are one internal positioning step, not a public API.
#[allow(clippy::too_many_arguments)]
fn arrange_out_of_flow(
    tree: &RetainedTree,
    styles: &LayoutStyles,
    computed: &ComputedStyles,
    grid_templates: &GridTemplates,
    child: LayoutChild,
    parent_cb: Rect,
    viewport: Rect,
    scroll_offset: (f32, f32),
    boxes: &mut LayoutBoxes,
    paragraph: &ParagraphShaper,
    hyphenator: Option<&dyn Hyphenator>,
    editing: Option<EditingView<'_>>,
    depth: u32,
    cache: &MeasureCache,
) {
    if depth == 0 {
        return;
    }
    let style = child.style();
    // Fixed positions against the viewport, ignoring ancestors; absolute against
    // the nearest positioned ancestor's padding box.
    let cb = if matches!(style.position, Position::Fixed) {
        viewport
    } else {
        parent_cb
    };

    // Natural (content) size: measure loosely up to the containing block. This is
    // the used size on an axis that is not pinned by both insets.
    let natural = measure(
        tree,
        styles,
        computed,
        grid_templates,
        child.node(),
        BoxConstraints::loose(cb.size),
        paragraph,
        hyphenator,
        editing,
        depth,
        cache,
    );
    let rect = style
        .inset
        .resolve_absolute_box(cb, style.width, style.height, natural);

    // Arrange the child's subtree under a tight constraint at its resolved box.
    // The recursive `arrange` re-derives and writes the child's own box (identical
    // to `rect` under the tight constraint) and, because the child is positioned,
    // establishes its padding box as the containing block for its descendants; the
    // `containing_block` argument we pass is therefore only a placeholder it
    // overrides.
    arrange(
        tree,
        styles,
        computed,
        grid_templates,
        child.node(),
        rect.origin,
        BoxConstraints::tight(rect.size),
        boxes,
        paragraph,
        hyphenator,
        cb,
        viewport,
        scroll_offset,
        editing,
        depth,
        cache,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_identity::ComponentId;
    use obscur_text::font::{FontId, MockFontResolver};
    use obscur_text::shaping::{
        CachingShaper, MockTextShaper, ScriptTag, SingleRunItemizer, TextItem, TextItemizer,
    };
    use obscur_text::span::TextRange;
    use obscur_tree::node::{LogicalParent, NodeKind};

    /// A default resolution viewport for the mapping tests.
    const TEST_VIEWPORT: Size = Size::new(800.0, 600.0);
    /// A default root font size for the mapping tests.
    const TEST_ROOT_FONT: f32 = 16.0;

    /// The default single-font paragraph orchestrator the layout tests measure
    /// through: a [`SingleRunItemizer`] + empty [`MockFontResolver`] + cached
    /// [`MockTextShaper`], reproducing the historical single-run behavior.
    fn test_paragraph() -> ParagraphShaper {
        ParagraphShaper::new(
            Box::new(SingleRunItemizer::new()),
            Box::new(MockFontResolver::new()),
            Box::new(CachingShaper::new(Box::new(MockTextShaper::new()))),
        )
    }

    /// A stub itemizer that cuts the fixed two-script string "abВГ" into a Latin
    /// run `[0..2)` and a Cyrillic run `[2..6)` (each Cyrillic char is 2 bytes).
    struct TwoScriptItemizer;

    impl TextItemizer for TwoScriptItemizer {
        fn itemize(&self, _text: &str, _base: TextDirection) -> Vec<TextItem> {
            vec![
                TextItem::new(
                    TextRange::new(0, 2),
                    ScriptTag::LATIN,
                    TextDirection::LeftToRight,
                    0,
                ),
                TextItem::new(
                    TextRange::new(2, 6),
                    ScriptTag::CYRILLIC,
                    TextDirection::LeftToRight,
                    0,
                ),
            ]
        }
    }

    /// A two-font paragraph orchestrator: the two-script itemizer plus a resolver
    /// mapping Latin→0 and Cyrillic→1, so "abВГ" shapes as two font-tagged runs.
    fn two_font_paragraph() -> ParagraphShaper {
        let mut resolver = MockFontResolver::new();
        resolver.register(ScriptTag::LATIN, FontId::new(0), measure_face());
        resolver.register(ScriptTag::CYRILLIC, FontId::new(1), measure_face());
        ParagraphShaper::new(
            Box::new(TwoScriptItemizer),
            Box::new(resolver),
            Box::new(CachingShaper::new(Box::new(MockTextShaper::new()))),
        )
    }

    /// Builds a `LayoutStyle` with an explicit block size.
    fn block(width: f32, height: f32) -> LayoutStyle {
        LayoutStyle::default()
            .with_display(LayoutDisplay::Block)
            .with_width(Dimension::Px(width))
            .with_height(Dimension::Px(height))
    }

    /// Allocates an element node under `parent`, wiring the child link and the
    /// logical parent, and returns its id.
    fn add_child(tree: &mut RetainedTree, parent: NodeId, raw_component: u64) -> NodeId {
        let id = tree.alloc(NodeKind::Element {
            component: ComponentId::from_raw(raw_component),
        });
        if let Some(node) = tree.get_mut(parent) {
            node.push_child(id);
        }
        if let Some(node) = tree.get_mut(id) {
            node.set_logical_parent(LogicalParent::of(parent));
        }
        id
    }

    #[test]
    fn empty_tree_lays_out_to_empty_boxes() {
        let coordinator = LayoutCoordinator::new();
        let tree = RetainedTree::new();
        let styles = LayoutStyles::new();
        let boxes = coordinator.layout(
            &tree,
            &styles,
            &ComputedStyles::new(),
            &GridTemplates::new(),
            Size::new(800.0, 600.0),
            (0.0, 0.0),
            &test_paragraph(),
        );
        assert!(boxes.is_empty());
    }

    #[test]
    fn mapping_reads_width_and_defaults_display_to_block() {
        let mut computed = ComputedStyle::new();
        computed.set(PropertyId::Width, ComputedValue::length(Length::px(120.0)));
        let style = layout_style_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT, None);
        assert_eq!(style.width, Dimension::Px(120.0));
        // Height absent -> Auto.
        assert_eq!(style.height, Dimension::Auto);
        // Display is always Block in Phase 1 (interner not available).
        assert_eq!(style.display, LayoutDisplay::Block);
    }

    #[test]
    fn mapping_percent_and_auto() {
        let mut computed = ComputedStyle::new();
        computed.set(
            PropertyId::Width,
            ComputedValue::length(Length::Percent(50.0)),
        );
        computed.set(PropertyId::Height, ComputedValue::length(Length::Auto));
        let style = layout_style_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT, None);
        assert_eq!(style.width, Dimension::Percent(50.0));
        assert_eq!(style.height, Dimension::Auto);
    }

    #[test]
    fn mapping_resolves_relative_units_against_context() {
        use obscur_style::property::LengthExpr;
        let mut computed = ComputedStyle::new();
        // font-size 16px -> em resolves against it; viewport 800x600 -> vw/vh.
        computed.set(
            PropertyId::FontSize,
            ComputedValue::length(Length::px(16.0)),
        );
        computed.set(PropertyId::Width, ComputedValue::length(Length::Em(3.0)));
        computed.set(PropertyId::Height, ComputedValue::length(Length::Vw(5.0)));
        // rem uses the document root font size (20), not the node font size.
        computed.set(
            PropertyId::MinWidth,
            ComputedValue::length(Length::Rem(2.0)),
        );
        // calc(10px + 2em) = 42.
        computed.set(
            PropertyId::MaxWidth,
            ComputedValue::calc(LengthExpr::Sum(
                Box::new(LengthExpr::Leaf(Length::px(10.0))),
                Box::new(LengthExpr::Leaf(Length::Em(2.0))),
            )),
        );
        let style = layout_style_from(&computed, TEST_VIEWPORT, 20.0, None);
        assert_eq!(style.width, Dimension::Px(48.0)); // 3em
        assert_eq!(style.height, Dimension::Px(40.0)); // 5vw of 800
        assert_eq!(style.min_width, Dimension::Px(40.0)); // 2rem of 20
        assert_eq!(style.max_width, Dimension::Px(42.0)); // calc(10px + 2em)
    }

    #[test]
    fn mapping_resolves_relative_gap_to_pixels() {
        let mut computed = ComputedStyle::new();
        computed.set(
            PropertyId::FontSize,
            ComputedValue::length(Length::px(10.0)),
        );
        computed.set(PropertyId::Gap, ComputedValue::length(Length::Em(2.0)));
        let style = layout_style_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT, None);
        assert_eq!(style.gap, 20.0); // 2em of 10px
    }

    #[test]
    fn root_font_size_reads_root_node_defaulting_to_sixteen() {
        let mut computed = ComputedStyles::new();
        let mut root = ComputedStyle::new();
        root.set(
            PropertyId::FontSize,
            ComputedValue::length(Length::px(24.0)),
        );
        computed.insert(NodeId::from_raw(1), root);
        assert_eq!(root_font_size(&computed, Some(NodeId::from_raw(1))), 24.0);
        // Absent root or missing font-size falls back to the initial 16px.
        assert_eq!(root_font_size(&computed, Some(NodeId::from_raw(9))), 16.0);
        assert_eq!(root_font_size(&computed, None), 16.0);
    }

    #[test]
    fn layout_styles_from_maps_every_node() {
        let coordinator = LayoutCoordinator::new();
        let mut computed = ComputedStyles::new();
        let mut a = ComputedStyle::new();
        a.set(PropertyId::Width, ComputedValue::length(Length::px(10.0)));
        computed.insert(NodeId::from_raw(1), a);
        computed.insert(NodeId::from_raw(2), ComputedStyle::new());

        let styles = coordinator.layout_styles_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT, None);
        assert_eq!(styles.len(), 2);
        assert_eq!(
            styles.get(&NodeId::from_raw(1)).map(|s| s.width),
            Some(Dimension::Px(10.0))
        );
        assert_eq!(
            styles.get(&NodeId::from_raw(2)).map(|s| s.width),
            Some(Dimension::Auto)
        );
    }

    #[test]
    fn flow_stacking_produces_absolute_boxes() {
        // Block root (width 200) with two fixed-height block children.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let child_a = add_child(&mut tree, root, 10);
        let child_b = add_child(&mut tree, root, 11);

        let mut styles = LayoutStyles::new();
        styles.insert(root, block(200.0, 100.0));
        styles.insert(child_a, block(200.0, 50.0));
        styles.insert(child_b, block(200.0, 50.0));

        let coordinator = LayoutCoordinator::new();
        let boxes = coordinator.layout(
            &tree,
            &styles,
            &ComputedStyles::new(),
            &GridTemplates::new(),
            Size::new(200.0, 100.0),
            (0.0, 0.0),
            &test_paragraph(),
        );

        // Root at the viewport origin.
        assert_eq!(boxes[&root].origin(), Point::new(0.0, 0.0));
        // First child stacks at y=0, second below it at y=50 (absolute).
        assert_eq!(boxes[&child_a].origin(), Point::new(0.0, 0.0));
        assert_eq!(boxes[&child_a].size(), Size::new(200.0, 50.0));
        assert_eq!(boxes[&child_b].origin(), Point::new(0.0, 50.0));
        assert_eq!(boxes[&child_b].size(), Size::new(200.0, 50.0));
    }

    #[test]
    fn flex_row_places_children_side_by_side() {
        // Flex root (width 200, height 100) with two flex-basis-50 children.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let child_a = add_child(&mut tree, root, 20);
        let child_b = add_child(&mut tree, root, 21);

        let flex_item = LayoutStyle::default()
            .with_display(LayoutDisplay::Flex)
            .with_height(Dimension::Px(100.0));
        let flex_item = LayoutStyle {
            flex_basis: Dimension::Px(50.0),
            flex_grow: 0.0,
            ..flex_item
        };

        let mut styles = LayoutStyles::new();
        styles.insert(
            root,
            LayoutStyle {
                display: LayoutDisplay::Flex,
                width: Dimension::Px(200.0),
                height: Dimension::Px(100.0),
                ..LayoutStyle::default()
            },
        );
        styles.insert(child_a, flex_item);
        styles.insert(child_b, flex_item);

        let coordinator = LayoutCoordinator::new();
        let boxes = coordinator.layout(
            &tree,
            &styles,
            &ComputedStyles::new(),
            &GridTemplates::new(),
            Size::new(200.0, 100.0),
            (0.0, 0.0),
            &test_paragraph(),
        );

        // Children sit side by side along the main (row) axis: x=0 then x=50.
        assert_eq!(boxes[&child_a].origin(), Point::new(0.0, 0.0));
        assert_eq!(boxes[&child_a].size().width, 50.0);
        assert_eq!(boxes[&child_b].origin().x, 50.0);
        assert_eq!(boxes[&child_b].size().width, 50.0);
    }

    #[test]
    fn nesting_accumulates_absolute_offsets() {
        // root -> container -> leaf; each level indents the child by its own box.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let container = add_child(&mut tree, root, 30);
        let leaf = add_child(&mut tree, container, 31);

        // Give the root padding so the container is offset within it, and the
        // container padding so the leaf is offset within the container. Padding
        // offsets children into the content box in flow layout.
        let mut root_style = block(200.0, 200.0);
        root_style.padding = obscur_geometry::Insets::uniform(10.0);
        let mut container_style = block(100.0, 100.0);
        container_style.padding = obscur_geometry::Insets::uniform(5.0);
        let leaf_style = block(50.0, 50.0);

        let mut styles = LayoutStyles::new();
        styles.insert(root, root_style);
        styles.insert(container, container_style);
        styles.insert(leaf, leaf_style);

        let coordinator = LayoutCoordinator::new();
        let boxes = coordinator.layout(
            &tree,
            &styles,
            &ComputedStyles::new(),
            &GridTemplates::new(),
            Size::new(200.0, 200.0),
            (0.0, 0.0),
            &test_paragraph(),
        );

        // Container offset by the root's top/left padding.
        assert_eq!(boxes[&container].origin(), Point::new(10.0, 10.0));
        // Leaf absolute origin = root padding (10) + container padding (5) = 15.
        assert_eq!(boxes[&leaf].origin(), Point::new(15.0, 15.0));
        assert_eq!(boxes[&leaf].size(), Size::new(50.0, 50.0));
    }

    #[test]
    fn relative_child_shifts_itself_and_subtree_without_moving_siblings() {
        // Block root (200x200) with two stacked block children `a` (relative) and
        // `b` (static). `a` holds a leaf grandchild so we can see the subtree shift.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 40);
        let b = add_child(&mut tree, root, 41);
        let leaf = add_child(&mut tree, a, 42);

        // `a` is `position: relative; top: 10px; left: 5px`.
        let a_relative = block(200.0, 50.0)
            .with_position(Position::Relative)
            .with_inset(Inset::new(
                Dimension::Px(10.0),
                Dimension::Auto,
                Dimension::Auto,
                Dimension::Px(5.0),
            ));

        let mut styles = LayoutStyles::new();
        styles.insert(root, block(200.0, 200.0));
        styles.insert(a, a_relative);
        styles.insert(b, block(200.0, 50.0));
        styles.insert(leaf, block(20.0, 20.0));

        let coordinator = LayoutCoordinator::new();
        let boxes = coordinator.layout(
            &tree,
            &styles,
            &ComputedStyles::new(),
            &GridTemplates::new(),
            Size::new(200.0, 200.0),
            (0.0, 0.0),
            &test_paragraph(),
        );

        // `a` is offset from its flow origin (0,0) by (left=5, top=10).
        assert_eq!(boxes[&a].origin(), Point::new(5.0, 10.0));
        // The leaf (flow-placed at a's content origin) shifts with the subtree.
        assert_eq!(boxes[&leaf].origin(), Point::new(5.0, 10.0));
        // The sibling `b` is untouched: still at the flow position (0, 50).
        assert_eq!(boxes[&b].origin(), Point::new(0.0, 50.0));
    }

    #[test]
    fn sticky_child_clamps_with_scroll_leaving_siblings_in_flow() {
        // Block root (200x1000): a 200x50 `spacer`, then a 200x50 `a` (sticky;
        // top:10) holding a leaf grandchild, then a 200x50 `b` (static). Flow:
        // spacer y=0, a y=50, b y=100. `a`'s static top (50) is below its 10px
        // inset, so at scroll 0 it sits at its static position (byte-identical).
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let spacer = add_child(&mut tree, root, 49);
        let a = add_child(&mut tree, root, 50);
        let b = add_child(&mut tree, root, 51);
        let leaf = add_child(&mut tree, a, 52);

        let a_sticky = block(200.0, 50.0)
            .with_position(Position::Sticky)
            .with_inset(Inset::new(
                Dimension::Px(10.0),
                Dimension::Auto,
                Dimension::Auto,
                Dimension::Auto,
            ));

        let mut styles = LayoutStyles::new();
        styles.insert(root, block(200.0, 1000.0));
        styles.insert(spacer, block(200.0, 50.0));
        styles.insert(a, a_sticky);
        styles.insert(b, block(200.0, 50.0));
        styles.insert(leaf, block(20.0, 20.0));

        let coordinator = LayoutCoordinator::new();
        let viewport = Size::new(200.0, 1000.0);

        // scroll (0,0): sticky_lower = 0 + 10 = 10 <= static a.y = 50 → used = 50
        // (still below its threshold) → BYTE-IDENTICAL to static.
        let boxes = coordinator.layout(
            &tree,
            &styles,
            &ComputedStyles::new(),
            &GridTemplates::new(),
            viewport,
            (0.0, 0.0),
            &test_paragraph(),
        );
        assert_eq!(
            boxes[&a].origin(),
            Point::new(0.0, 50.0),
            "scroll 0: a at static"
        );
        assert_eq!(boxes[&leaf].origin(), Point::new(0.0, 50.0));
        assert_eq!(
            boxes[&b].origin(),
            Point::new(0.0, 100.0),
            "sibling in flow"
        );

        // scroll (0,100): sticky_lower = 100 + 10 = 110 > static 50, clamp upper =
        // cb.bottom(1000) - h(50) = 950 → a sticks at y=110; its leaf rides along;
        // `b` stays at its unchanged flow position (0, 100).
        let boxes = coordinator.layout(
            &tree,
            &styles,
            &ComputedStyles::new(),
            &GridTemplates::new(),
            viewport,
            (0.0, 100.0),
            &test_paragraph(),
        );
        assert_eq!(
            boxes[&a].origin(),
            Point::new(0.0, 110.0),
            "sticks at scrollport+inset"
        );
        assert_eq!(
            boxes[&leaf].origin(),
            Point::new(0.0, 110.0),
            "subtree rides along"
        );
        assert_eq!(
            boxes[&b].origin(),
            Point::new(0.0, 100.0),
            "sibling unmoved (in flow)"
        );

        // Huge scroll: sticky_lower far below cb → pinned at cb.bottom - h = 950.
        let boxes = coordinator.layout(
            &tree,
            &styles,
            &ComputedStyles::new(),
            &GridTemplates::new(),
            viewport,
            (0.0, 5000.0),
            &test_paragraph(),
        );
        assert_eq!(
            boxes[&a].origin(),
            Point::new(0.0, 950.0),
            "pinned within CB"
        );
        assert_eq!(
            boxes[&b].origin(),
            Point::new(0.0, 100.0),
            "sibling still unmoved"
        );
    }

    #[test]
    fn static_child_is_not_offset_and_right_bottom_move_opposite() {
        // `a` static (no offset), `b` relative with `right`/`bottom` only.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 50);
        let b = add_child(&mut tree, root, 51);

        // `b` is `position: relative; right: 5px; bottom: 10px` -> moves (-5, -10).
        let b_relative = block(200.0, 50.0)
            .with_position(Position::Relative)
            .with_inset(Inset::new(
                Dimension::Auto,
                Dimension::Px(5.0),
                Dimension::Px(10.0),
                Dimension::Auto,
            ));

        let mut styles = LayoutStyles::new();
        styles.insert(root, block(200.0, 200.0));
        styles.insert(a, block(200.0, 50.0));
        styles.insert(b, b_relative);

        let coordinator = LayoutCoordinator::new();
        let boxes = coordinator.layout(
            &tree,
            &styles,
            &ComputedStyles::new(),
            &GridTemplates::new(),
            Size::new(200.0, 200.0),
            (0.0, 0.0),
            &test_paragraph(),
        );

        // Static `a` keeps its flow origin exactly.
        assert_eq!(boxes[&a].origin(), Point::new(0.0, 0.0));
        // `b` flow origin (0, 50) shifts by right=5 -> -5 x, bottom=10 -> -10 y.
        assert_eq!(boxes[&b].origin(), Point::new(-5.0, 40.0));
    }

    /// A block-only style with an explicit width and an auto (content-sized)
    /// height, for a container whose height must reflect only its in-flow content.
    fn auto_height_block(width: f32) -> LayoutStyle {
        LayoutStyle::default()
            .with_display(LayoutDisplay::Block)
            .with_width(Dimension::Px(width))
    }

    #[test]
    fn absolute_child_is_out_of_flow_and_positioned_by_insets() {
        // Scenario (a): a flow container with an in-flow `.a` and an absolute `.b`.
        // `.b` must not affect `.a` or the container's content height, and must be
        // placed at its insets within its containing block (the viewport here).
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let container = add_child(&mut tree, root, 60);
        let a = add_child(&mut tree, container, 61);
        let b = add_child(&mut tree, container, 62);

        // `.b { position: absolute; top: 10; left: 20; width: 40; height: 30 }`.
        let b_absolute = block(40.0, 30.0)
            .with_position(Position::Absolute)
            .with_inset(Inset::new(
                Dimension::Px(10.0),
                Dimension::Auto,
                Dimension::Auto,
                Dimension::Px(20.0),
            ));

        let mut styles = LayoutStyles::new();
        styles.insert(root, block(300.0, 300.0));
        // Container is auto-height so its content height is observable.
        styles.insert(container, auto_height_block(300.0));
        styles.insert(a, block(300.0, 50.0));
        styles.insert(b, b_absolute);

        let coordinator = LayoutCoordinator::new();
        let boxes = coordinator.layout(
            &tree,
            &styles,
            &ComputedStyles::new(),
            &GridTemplates::new(),
            Size::new(300.0, 300.0),
            (0.0, 0.0),
            &test_paragraph(),
        );

        // `.a` is untouched by `.b`: flow origin (0, 0), its own size.
        assert_eq!(boxes[&a].origin(), Point::new(0.0, 0.0));
        assert_eq!(boxes[&a].size(), Size::new(300.0, 50.0));
        // The container's content height reflects only `.a` (50), not `.b`.
        assert_eq!(boxes[&container].size(), Size::new(300.0, 50.0));
        // `.b` is placed at its insets (20, 10) with its explicit 40x30 size.
        assert_eq!(boxes[&b].origin(), Point::new(20.0, 10.0));
        assert_eq!(boxes[&b].size(), Size::new(40.0, 30.0));
    }

    #[test]
    fn absolute_child_resolves_against_positioned_ancestor() {
        // Scenario (b): an absolute child positioned within a `position: relative`
        // ancestor (not the root), anchored by `right` / `bottom`.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let rel = add_child(&mut tree, root, 70);
        let abs = add_child(&mut tree, rel, 71);

        // `.rel { position: relative; left: 100; top: 100; width: 200; height: 200 }`.
        let rel_style = block(200.0, 200.0)
            .with_position(Position::Relative)
            .with_inset(Inset::new(
                Dimension::Px(100.0),
                Dimension::Auto,
                Dimension::Auto,
                Dimension::Px(100.0),
            ));
        // `.abs { position: absolute; right: 0; bottom: 0; width: 20; height: 20 }`.
        let abs_style = block(20.0, 20.0)
            .with_position(Position::Absolute)
            .with_inset(Inset::new(
                Dimension::Auto,
                Dimension::Px(0.0),
                Dimension::Px(0.0),
                Dimension::Auto,
            ));

        let mut styles = LayoutStyles::new();
        styles.insert(root, block(400.0, 400.0));
        styles.insert(rel, rel_style);
        styles.insert(abs, abs_style);

        let coordinator = LayoutCoordinator::new();
        let boxes = coordinator.layout(
            &tree,
            &styles,
            &ComputedStyles::new(),
            &GridTemplates::new(),
            Size::new(400.0, 400.0),
            (0.0, 0.0),
            &test_paragraph(),
        );

        // `.rel` is offset into the flow by its insets (100, 100).
        assert_eq!(boxes[&rel].origin(), Point::new(100.0, 100.0));
        // `.abs` sits flush against `.rel`'s far corner: (100+200-20, 100+200-20).
        assert_eq!(boxes[&abs].origin(), Point::new(280.0, 280.0));
        assert_eq!(boxes[&abs].size(), Size::new(20.0, 20.0));
    }

    #[test]
    fn absolute_child_with_both_insets_stretches_to_containing_block() {
        // Scenario (c): `left` and `right` both set pin both edges, so the width
        // stretches to the containing block regardless of the explicit width.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let abs = add_child(&mut tree, root, 80);

        // Explicit 999x999 is deliberately ignored on the pinned axes.
        let abs_style = block(999.0, 999.0)
            .with_position(Position::Absolute)
            .with_inset(Inset::new(
                Dimension::Px(0.0),
                Dimension::Px(30.0),
                Dimension::Px(0.0),
                Dimension::Px(10.0),
            ));

        let mut styles = LayoutStyles::new();
        styles.insert(root, block(300.0, 300.0));
        styles.insert(abs, abs_style);

        let coordinator = LayoutCoordinator::new();
        let boxes = coordinator.layout(
            &tree,
            &styles,
            &ComputedStyles::new(),
            &GridTemplates::new(),
            Size::new(300.0, 300.0),
            (0.0, 0.0),
            &test_paragraph(),
        );

        // Horizontal: x = 10, width = 300 - 10 - 30 = 260. Vertical: y = 0,
        // height = 300 - 0 - 0 = 300.
        assert_eq!(boxes[&abs].origin(), Point::new(10.0, 0.0));
        assert_eq!(boxes[&abs].size(), Size::new(260.0, 300.0));
    }

    #[test]
    fn fixed_child_positions_against_viewport_ignoring_ancestor() {
        // Scenario (d): a `position: fixed` child ignores its positioned ancestor
        // and resolves against the viewport, so `top: 0; left: 0` lands at (0, 0)
        // rather than the ancestor's (100, 100) origin.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let rel = add_child(&mut tree, root, 90);
        let fixed = add_child(&mut tree, rel, 91);

        let rel_style = block(200.0, 200.0)
            .with_position(Position::Relative)
            .with_inset(Inset::new(
                Dimension::Px(100.0),
                Dimension::Auto,
                Dimension::Auto,
                Dimension::Px(100.0),
            ));
        let fixed_style = block(20.0, 20.0)
            .with_position(Position::Fixed)
            .with_inset(Inset::new(
                Dimension::Px(0.0),
                Dimension::Auto,
                Dimension::Auto,
                Dimension::Px(0.0),
            ));

        let mut styles = LayoutStyles::new();
        styles.insert(root, block(400.0, 400.0));
        styles.insert(rel, rel_style);
        styles.insert(fixed, fixed_style);

        let coordinator = LayoutCoordinator::new();
        let boxes = coordinator.layout(
            &tree,
            &styles,
            &ComputedStyles::new(),
            &GridTemplates::new(),
            Size::new(400.0, 400.0),
            (0.0, 0.0),
            &test_paragraph(),
        );

        // `.rel` still sits at (100, 100), but the fixed child ignores it.
        assert_eq!(boxes[&rel].origin(), Point::new(100.0, 100.0));
        assert_eq!(boxes[&fixed].origin(), Point::new(0.0, 0.0));
        assert_eq!(boxes[&fixed].size(), Size::new(20.0, 20.0));
    }

    #[test]
    fn mapping_reads_position_and_insets() {
        use obscur_style::property::layout_keyword;
        let mut computed = ComputedStyle::new();
        computed.set(
            PropertyId::Position,
            ComputedValue::from_concrete(PropertyValue::Keyword(layout_keyword::POSITION_RELATIVE))
                .expect("concrete"),
        );
        computed.set(PropertyId::Top, ComputedValue::length(Length::px(10.0)));
        computed.set(PropertyId::Left, ComputedValue::length(Length::px(5.0)));
        let style = layout_style_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT, None);
        assert_eq!(style.position, Position::Relative);
        assert_eq!(style.inset.top, Dimension::Px(10.0));
        assert_eq!(style.inset.left, Dimension::Px(5.0));
        // Unauthored edges default to `Auto`.
        assert_eq!(style.inset.right, Dimension::Auto);
        assert_eq!(style.inset.bottom, Dimension::Auto);
    }

    #[test]
    fn mapping_defaults_position_to_static() {
        // A node with no authored `position` stays `Static` with all-`auto` insets.
        let computed = ComputedStyle::new();
        let style = layout_style_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT, None);
        assert_eq!(style.position, Position::Static);
        assert_eq!(style.inset, Inset::AUTO);
    }

    #[test]
    fn mapping_builds_grid_placement_from_computed_lines() {
        let mut computed = ComputedStyle::new();
        computed.set(
            PropertyId::GridColumnStart,
            ComputedValue::from_concrete(PropertyValue::GridLine(GridLineValue::Line(1)))
                .expect("concrete"),
        );
        computed.set(
            PropertyId::GridColumnEnd,
            ComputedValue::from_concrete(PropertyValue::GridLine(GridLineValue::Line(3)))
                .expect("concrete"),
        );
        computed.set(
            PropertyId::GridRowStart,
            ComputedValue::from_concrete(PropertyValue::GridLine(GridLineValue::Span(2)))
                .expect("concrete"),
        );
        // `grid-row-end` left unset -> Auto.
        let style = layout_style_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT, None);
        let placement = style.grid_placement();
        assert_eq!(placement.column_start(), GridLine::Line(1));
        assert_eq!(placement.column_end(), GridLine::Line(3));
        assert_eq!(placement.row_start(), GridLine::Span(2));
        assert_eq!(placement.row_end(), GridLine::Auto);
        // The column axis resolves to a definite start of track 0, span 2.
        assert_eq!(placement.resolve_column(3), (Some(0), 2));
    }

    #[test]
    fn mapping_defaults_grid_placement_to_all_auto() {
        // A node with no authored line placement keeps the all-Auto default, so it
        // auto-places exactly as before.
        let computed = ComputedStyle::new();
        let style = layout_style_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT, None);
        assert_eq!(style.grid_placement(), GridPlacement::default());
    }

    #[test]
    fn mapping_reads_grid_auto_flow_keyword() {
        // Each reserved `grid-auto-flow` handle maps to its flow mode.
        for (sym, expected) in [
            (layout_keyword::GRID_AUTO_FLOW_COLUMN, GridAutoFlow::Column),
            (
                layout_keyword::GRID_AUTO_FLOW_ROW_DENSE,
                GridAutoFlow::RowDense,
            ),
            (
                layout_keyword::GRID_AUTO_FLOW_COLUMN_DENSE,
                GridAutoFlow::ColumnDense,
            ),
            (layout_keyword::GRID_AUTO_FLOW_ROW, GridAutoFlow::Row),
        ] {
            let mut computed = ComputedStyle::new();
            computed.set(PropertyId::GridAutoFlow, ComputedValue::keyword(sym));
            let style = layout_style_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT, None);
            assert_eq!(style.grid_auto_flow(), expected);
        }
        // Absent -> the `Row` (sparse) default.
        let computed = ComputedStyle::new();
        let style = layout_style_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT, None);
        assert_eq!(style.grid_auto_flow(), GridAutoFlow::Row);
    }

    #[test]
    fn mapping_reads_grid_item_alignment_keywords() {
        // Container defaults: `justify-items` / `align-items`.
        let mut computed = ComputedStyle::new();
        computed.set(
            PropertyId::JustifyItems,
            ComputedValue::keyword(layout_keyword::ALIGN_CENTER),
        );
        computed.set(
            PropertyId::AlignItems,
            ComputedValue::keyword(layout_keyword::ALIGN_END),
        );
        // Per-item overrides: `justify-self` / `align-self` (incl. `auto`).
        computed.set(
            PropertyId::JustifySelf,
            ComputedValue::keyword(layout_keyword::ALIGN_START),
        );
        computed.set(
            PropertyId::AlignSelf,
            ComputedValue::keyword(layout_keyword::ALIGN_AUTO),
        );
        let style = layout_style_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT, None);
        assert_eq!(style.justify_items, AlignItems::Center);
        assert_eq!(style.align_items, AlignItems::FlexEnd);
        assert_eq!(style.justify_self, AlignSelf::Start);
        assert_eq!(style.align_self, AlignSelf::Auto);

        // Absent -> the layout defaults (`Stretch` container, `Auto` item).
        let empty = ComputedStyle::new();
        let default_style = layout_style_from(&empty, TEST_VIEWPORT, TEST_ROOT_FONT, None);
        assert_eq!(default_style.justify_items, AlignItems::Stretch);
        assert_eq!(default_style.justify_self, AlignSelf::Auto);
        assert_eq!(default_style.align_self, AlignSelf::Auto);
    }

    #[test]
    fn mapping_populates_every_field_from_computed_style() {
        let mut computed = ComputedStyle::new();
        computed.set(
            PropertyId::Display,
            ComputedValue::keyword(layout_keyword::DISPLAY_FLEX),
        );
        // Box model.
        computed.set(
            PropertyId::MarginTop,
            ComputedValue::length(Length::px(1.0)),
        );
        computed.set(
            PropertyId::MarginRight,
            ComputedValue::length(Length::px(2.0)),
        );
        computed.set(
            PropertyId::MarginBottom,
            ComputedValue::length(Length::px(3.0)),
        );
        computed.set(
            PropertyId::MarginLeft,
            ComputedValue::length(Length::px(4.0)),
        );
        computed.set(
            PropertyId::PaddingTop,
            ComputedValue::length(Length::px(5.0)),
        );
        computed.set(
            PropertyId::PaddingRight,
            ComputedValue::length(Length::px(6.0)),
        );
        computed.set(
            PropertyId::PaddingBottom,
            ComputedValue::length(Length::px(7.0)),
        );
        computed.set(
            PropertyId::PaddingLeft,
            ComputedValue::length(Length::px(8.0)),
        );
        // Sizing constraints.
        computed.set(
            PropertyId::MinWidth,
            ComputedValue::length(Length::px(10.0)),
        );
        computed.set(
            PropertyId::MinHeight,
            ComputedValue::length(Length::px(11.0)),
        );
        computed.set(
            PropertyId::MaxWidth,
            ComputedValue::length(Length::Percent(80.0)),
        );
        computed.set(
            PropertyId::MaxHeight,
            ComputedValue::length(Length::px(13.0)),
        );
        // Flex container + item.
        computed.set(PropertyId::Gap, ComputedValue::length(Length::px(9.0)));
        computed.set(
            PropertyId::FlexDirection,
            ComputedValue::keyword(layout_keyword::FLEX_COLUMN),
        );
        computed.set(
            PropertyId::FlexWrap,
            ComputedValue::keyword(layout_keyword::FLEX_WRAP_WRAP),
        );
        computed.set(
            PropertyId::JustifyContent,
            ComputedValue::keyword(layout_keyword::JUSTIFY_CENTER),
        );
        computed.set(
            PropertyId::AlignItems,
            ComputedValue::keyword(layout_keyword::ALIGN_STRETCH),
        );
        computed.set(PropertyId::FlexGrow, ComputedValue::number(1.0));
        computed.set(PropertyId::FlexShrink, ComputedValue::number(2.0));
        computed.set(
            PropertyId::FlexBasis,
            ComputedValue::length(Length::px(50.0)),
        );

        let style = layout_style_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT, None);

        assert_eq!(style.display, LayoutDisplay::Flex);
        assert_eq!(style.margin, MarginEdges::px(1.0, 2.0, 3.0, 4.0));
        assert_eq!(
            style.padding,
            Insets {
                top: 5.0,
                right: 6.0,
                bottom: 7.0,
                left: 8.0
            }
        );
        assert_eq!(style.min_width, Dimension::Px(10.0));
        assert_eq!(style.min_height, Dimension::Px(11.0));
        assert_eq!(style.max_width, Dimension::Percent(80.0));
        assert_eq!(style.max_height, Dimension::Px(13.0));
        assert_eq!(style.gap, 9.0);
        assert_eq!(style.flex_direction, FlexDirection::Column);
        assert_eq!(style.flex_wrap, FlexWrap::Wrap);
        assert_eq!(style.justify_content, JustifyContent::Center);
        assert_eq!(style.align_items, AlignItems::Stretch);
        assert_eq!(style.flex_grow, 1.0);
        assert_eq!(style.flex_shrink, 2.0);
        assert_eq!(style.flex_basis, Dimension::Px(50.0));
    }

    #[test]
    fn mapping_display_grid_maps_to_grid_and_has_algorithm() {
        let mut computed = ComputedStyle::new();
        computed.set(
            PropertyId::Display,
            ComputedValue::keyword(layout_keyword::DISPLAY_GRID),
        );
        let style = layout_style_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT, None);
        assert_eq!(style.display, LayoutDisplay::Grid);
        assert!(
            algorithm_for(
                LayoutDisplay::Grid,
                NodeId::from_raw(1),
                &GridTemplates::new(),
                0.0
            )
            .is_some()
        );
    }

    #[test]
    fn mapping_display_none_maps_to_none() {
        let mut computed = ComputedStyle::new();
        computed.set(
            PropertyId::Display,
            ComputedValue::keyword(layout_keyword::DISPLAY_NONE),
        );
        assert_eq!(
            layout_style_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT, None).display,
            LayoutDisplay::None
        );
    }

    #[test]
    fn flex_engages_from_computed_style_end_to_end() {
        // A container with `display: flex; flex-direction: row` and two children
        // with `flex-basis: 50px` must lay the children out side by side once the
        // display keyword is mapped from computed style. Before this bridge every
        // node defaulted to Block and the children would have stacked.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let child_a = add_child(&mut tree, root, 40);
        let child_b = add_child(&mut tree, root, 41);

        let mut computed = ComputedStyles::new();

        let mut root_style = ComputedStyle::new();
        root_style.set(
            PropertyId::Display,
            ComputedValue::keyword(layout_keyword::DISPLAY_FLEX),
        );
        root_style.set(
            PropertyId::FlexDirection,
            ComputedValue::keyword(layout_keyword::FLEX_ROW),
        );
        root_style.set(PropertyId::Width, ComputedValue::length(Length::px(200.0)));
        root_style.set(PropertyId::Height, ComputedValue::length(Length::px(100.0)));
        computed.insert(root, root_style);

        for &child in &[child_a, child_b] {
            let mut item = ComputedStyle::new();
            item.set(
                PropertyId::FlexBasis,
                ComputedValue::length(Length::px(50.0)),
            );
            item.set(PropertyId::Height, ComputedValue::length(Length::px(100.0)));
            computed.insert(child, item);
        }

        let coordinator = LayoutCoordinator::new();
        let styles = coordinator.layout_styles_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT, None);
        // Sanity: the bridge produced a flex container.
        assert_eq!(styles[&root].display, LayoutDisplay::Flex);

        let boxes = coordinator.layout(
            &tree,
            &styles,
            &computed,
            &GridTemplates::new(),
            Size::new(200.0, 100.0),
            (0.0, 0.0),
            &test_paragraph(),
        );

        // Children sit side by side along the row axis: distinct x offsets.
        assert_eq!(boxes[&child_a].origin().x, 0.0);
        assert_eq!(boxes[&child_b].origin().x, 50.0);
        // Same y (a row, not a stack) — proving flex, not block, engaged.
        assert_eq!(boxes[&child_a].origin().y, boxes[&child_b].origin().y);
    }

    #[test]
    fn authored_grid_template_engages_two_columns() {
        use obscur_style::property::{GridTrack, PropertyValue};
        // A `display: grid` container with `grid-template-columns: 100px 50px`
        // and two children must place them side by side (col 0 then col 1). With
        // the default `GridLayout::new(1)` the two would stack in one column
        // (same x); the authored two-column template puts the second at x=100.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let child_a = add_child(&mut tree, root, 50);
        let child_b = add_child(&mut tree, root, 51);

        let mut computed = ComputedStyles::new();
        let mut root_style = ComputedStyle::new();
        root_style.set(
            PropertyId::Display,
            ComputedValue::keyword(layout_keyword::DISPLAY_GRID),
        );
        root_style.set(PropertyId::Width, ComputedValue::length(Length::px(150.0)));
        root_style.set(PropertyId::Height, ComputedValue::length(Length::px(100.0)));
        root_style.set(
            PropertyId::GridTemplateColumns,
            ComputedValue::from_concrete(PropertyValue::grid_tracks([
                GridTrack::Fixed(Length::px(100.0)),
                GridTrack::Fixed(Length::px(50.0)),
            ]))
            .expect("concrete"),
        );
        computed.insert(root, root_style);
        computed.insert(child_a, ComputedStyle::new());
        computed.insert(child_b, ComputedStyle::new());

        let coordinator = LayoutCoordinator::new();
        let styles = coordinator.layout_styles_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT, None);
        let templates =
            coordinator.grid_templates_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT, None);
        assert!(templates.contains_key(&root), "root authored a template");

        let boxes = coordinator.layout(
            &tree,
            &styles,
            &computed,
            &templates,
            Size::new(150.0, 100.0),
            (0.0, 0.0),
            &test_paragraph(),
        );

        // First column at x=0 (100px wide), second column at x=100 (50px wide),
        // both on the same row — proving the two-column template engaged.
        assert_eq!(boxes[&child_a].origin().x, 0.0);
        assert_eq!(boxes[&child_a].size().width, 100.0);
        assert_eq!(boxes[&child_b].origin().x, 100.0);
        assert_eq!(boxes[&child_b].size().width, 50.0);
        assert_eq!(boxes[&child_a].origin().y, boxes[&child_b].origin().y);
    }

    /// Allocates a text node carrying `content` under `parent`, wiring the child
    /// link and the logical parent, and returns its id.
    fn add_text_child(tree: &mut RetainedTree, parent: NodeId, content: &str) -> NodeId {
        let id = tree.alloc(NodeKind::Text {
            content: content.into(),
        });
        if let Some(node) = tree.get_mut(parent) {
            node.push_child(id);
        }
        if let Some(node) = tree.get_mut(id) {
            node.set_logical_parent(LogicalParent::of(parent));
        }
        id
    }

    /// Arranges a single text child under a fixed-size block `root` and returns
    /// the text node's arranged box. The text node carries no computed style, so
    /// its font size defaults to 16 (mock advance = 9.6 px/char, line_height 16).
    fn text_box_in_container(content: &str, width: f32, height: f32) -> LayoutBox {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let text = add_text_child(&mut tree, root, content);

        let mut computed = ComputedStyles::new();
        let mut root_style = ComputedStyle::new();
        root_style.set(PropertyId::Width, ComputedValue::length(Length::px(width)));
        root_style.set(
            PropertyId::Height,
            ComputedValue::length(Length::px(height)),
        );
        computed.insert(root, root_style);

        let coordinator = LayoutCoordinator::new();
        let styles = coordinator.layout_styles_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT, None);
        let boxes = coordinator.layout(
            &tree,
            &styles,
            &computed,
            &GridTemplates::new(),
            Size::new(width, height),
            (0.0, 0.0),
            &test_paragraph(),
        );
        boxes[&text]
    }

    /// Arranges a single text child through an explicit `paragraph` orchestrator
    /// and returns the text node's arranged box.
    fn text_box_with_paragraph(
        content: &str,
        width: f32,
        height: f32,
        paragraph: &ParagraphShaper,
    ) -> LayoutBox {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let text = add_text_child(&mut tree, root, content);

        let mut computed = ComputedStyles::new();
        let mut root_style = ComputedStyle::new();
        root_style.set(PropertyId::Width, ComputedValue::length(Length::px(width)));
        root_style.set(
            PropertyId::Height,
            ComputedValue::length(Length::px(height)),
        );
        computed.insert(root, root_style);

        let coordinator = LayoutCoordinator::new();
        let styles = coordinator.layout_styles_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT, None);
        let boxes = coordinator.layout(
            &tree,
            &styles,
            &computed,
            &GridTemplates::new(),
            Size::new(width, height),
            (0.0, 0.0),
            paragraph,
        );
        boxes[&text]
    }

    #[test]
    fn multi_font_text_measures_one_line_at_the_summed_run_width() {
        // "abВГ" itemizes into a Latin run and a Cyrillic run (two fonts). Each of
        // the 4 glyphs advances 9.6 (mock at 16px), so the wide-container measure
        // is one line: height == line_height (16) and width == 4·9.6 = 38.4. This
        // is the height the scene compose re-wraps to (paint line count agrees).
        let text_box = text_box_with_paragraph("abВГ", 1000.0, 200.0, &two_font_paragraph());
        assert_eq!(text_box.size().height, 16.0);
        assert!(
            (text_box.size().width - 38.4).abs() < 1e-3,
            "width {}",
            text_box.size().width
        );
    }

    #[test]
    fn long_text_in_a_narrow_container_wraps_to_multiple_lines() {
        // Requirement (a): three 4-char words (each 4·9.6 = 38.4 wide, plus a
        // trailing space → the first two words are 48.0 each) in a 50px-wide
        // container wrap to three lines, so the box is 3·line_height (48) tall —
        // strictly taller than a single line. Root is tall enough (200) that the
        // measured height is not clamped.
        let text_box = text_box_in_container("aaaa bbbb cccc", 50.0, 200.0);
        // Three lines stacked → height == 3 · 16.
        assert_eq!(text_box.size().height, 48.0);
        // Widest line is "aaaa " but its trailing space now HANGS (CSS Text 3
        // §3/§7.1), so the used width is 4·9.6 = 38.4 (was 5·9.6 = 48 when the
        // trailing space wrongly counted) — still within the 50px container.
        assert!(
            (text_box.size().width - 38.4).abs() < 1e-3,
            "width {}",
            text_box.size().width
        );
    }

    #[test]
    fn line_height_two_scales_the_wrapped_box_height() {
        // The same three-word string in a 50px container still wraps to three
        // lines, but `line-height: 2` makes each line box 2·16 = 32 tall, so the
        // measured box is 3·32 = 96 — twice the default 48 — reserving the extra
        // vertical space so following content does not overlap.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let text = add_text_child(&mut tree, root, "aaaa bbbb cccc");

        let mut computed = ComputedStyles::new();
        let mut root_style = ComputedStyle::new();
        root_style.set(PropertyId::Width, ComputedValue::length(Length::px(50.0)));
        root_style.set(PropertyId::Height, ComputedValue::length(Length::px(300.0)));
        computed.insert(root, root_style);
        let mut text_style = ComputedStyle::new();
        text_style.set(PropertyId::LineHeight, ComputedValue::number(2.0));
        computed.insert(text, text_style);

        let coordinator = LayoutCoordinator::new();
        let styles = coordinator.layout_styles_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT, None);
        let boxes = coordinator.layout(
            &tree,
            &styles,
            &computed,
            &GridTemplates::new(),
            Size::new(50.0, 300.0),
            (0.0, 0.0),
            &test_paragraph(),
        );
        // Three wrapped lines at a 32px line box → 96px tall.
        assert_eq!(boxes[&text].size().height, 96.0);
    }

    #[test]
    fn same_text_in_a_wide_container_stays_one_line() {
        // Requirement (b): the identical string in a 1000px container is a single
        // line — one line_height tall — and as wide as the whole run.
        let text_box = text_box_in_container("aaaa bbbb cccc", 1000.0, 200.0);
        assert_eq!(text_box.size().height, 16.0);
        // Full run advance: 14 chars · 9.6 = 134.4.
        assert!(
            (text_box.size().width - 134.4).abs() < 1e-3,
            "width {}",
            text_box.size().width
        );
    }

    #[test]
    fn short_single_line_text_is_sized_to_its_run() {
        // Requirement (e): short text that fits stays one line at its run advance.
        let text_box = text_box_in_container("hi", 1000.0, 200.0);
        assert_eq!(text_box.size().height, 16.0);
        // 2 chars · 9.6 = 19.2.
        assert!(
            (text_box.size().width - 19.2).abs() < 1e-3,
            "width {}",
            text_box.size().width
        );
    }

    #[test]
    fn empty_text_yields_one_empty_line_and_does_not_panic() {
        // Requirement (d): an empty string measures to a zero-width, one-line box
        // (height == line_height) without panicking.
        let text_box = text_box_in_container("", 100.0, 200.0);
        assert_eq!(text_box.size().width, 0.0);
        assert_eq!(text_box.size().height, 16.0);
    }

    #[test]
    fn text_font_size_drives_the_wrapped_box() {
        // The node's computed font-size (20px → advance 12.0, line_height 20)
        // drives the shaped/wrapped size, not the 16px default.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let text = add_text_child(&mut tree, root, "abcd");

        let mut computed = ComputedStyles::new();
        let mut root_style = ComputedStyle::new();
        root_style.set(PropertyId::Width, ComputedValue::length(Length::px(1000.0)));
        root_style.set(PropertyId::Height, ComputedValue::length(Length::px(200.0)));
        computed.insert(root, root_style);
        let mut text_computed = ComputedStyle::new();
        text_computed.set(
            PropertyId::FontSize,
            ComputedValue::length(Length::px(20.0)),
        );
        computed.insert(text, text_computed);

        let coordinator = LayoutCoordinator::new();
        let styles = coordinator.layout_styles_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT, None);
        assert_eq!(styles[&text].font_size, 20.0);
        let boxes = coordinator.layout(
            &tree,
            &styles,
            &computed,
            &GridTemplates::new(),
            Size::new(1000.0, 200.0),
            (0.0, 0.0),
            &test_paragraph(),
        );
        let text_box = boxes[&text];
        // 4 chars · (20 · 0.6) = 48.0 wide, one line 20 tall.
        assert!(
            (text_box.size().width - 48.0).abs() < 1e-3,
            "width {}",
            text_box.size().width
        );
        assert_eq!(text_box.size().height, 20.0);
    }

    #[test]
    fn display_none_generates_no_box_for_subtree() {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);

        let mut styles = LayoutStyles::new();
        styles.insert(
            root,
            LayoutStyle::default().with_display(LayoutDisplay::None),
        );

        let coordinator = LayoutCoordinator::new();
        let boxes = coordinator.layout(
            &tree,
            &styles,
            &ComputedStyles::new(),
            &GridTemplates::new(),
            Size::new(100.0, 100.0),
            (0.0, 0.0),
            &test_paragraph(),
        );
        // A `None` root produces no box at all.
        assert!(boxes.is_empty());
    }

    #[test]
    fn nested_column_flex_item_measured_at_full_intrinsic_height() {
        // Regression (the wireframe "cramping"): a flex ROW whose item is itself a
        // flex COLUMN must reserve the column's FULL stacked height (control slot +
        // margin + caption), so consecutive rows do not overlap.
        //
        //   root(block) -> row(flex row, align flex-start)
        //     -> col(flex column, width 264)
        //          -> slot(flex row) -> control(block 100x28)
        //          -> cap(block, height 14)
        //
        // The column stacks a 28px slot (with a 12px bottom margin) over a 14px
        // caption, so its intrinsic height is 28 + 12 + 14 = 54; the row reserves
        // exactly that and the caption sits at y = 40..54, clear of the slot.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let row = add_child(&mut tree, root, 100);
        let col = add_child(&mut tree, row, 101);
        let slot = add_child(&mut tree, col, 102);
        let control = add_child(&mut tree, slot, 103);
        let cap = add_child(&mut tree, col, 104);

        let mut flex_row_start = LayoutStyle::default().with_display(LayoutDisplay::Flex);
        flex_row_start.flex_direction = FlexDirection::Row;
        flex_row_start.align_items = AlignItems::FlexStart;

        let mut col_style = LayoutStyle::default()
            .with_display(LayoutDisplay::Flex)
            .with_width(Dimension::Px(264.0));
        col_style.flex_direction = FlexDirection::Column;

        let mut slot_style = flex_row_start.clone();
        slot_style.margin = MarginEdges::px(0.0, 0.0, 12.0, 0.0);

        let cap_style = LayoutStyle::default()
            .with_display(LayoutDisplay::Block)
            .with_height(Dimension::Px(14.0));

        let mut styles = LayoutStyles::new();
        styles.insert(
            root,
            LayoutStyle::default().with_display(LayoutDisplay::Block),
        );
        styles.insert(row, flex_row_start);
        styles.insert(col, col_style);
        styles.insert(slot, slot_style);
        styles.insert(control, block(100.0, 28.0));
        styles.insert(cap, cap_style);

        let coordinator = LayoutCoordinator::new();
        let boxes = coordinator.layout(
            &tree,
            &styles,
            &ComputedStyles::new(),
            &GridTemplates::new(),
            Size::new(1200.0, 4160.0),
            (0.0, 0.0),
            &test_paragraph(),
        );

        // The column child is measured at its full intrinsic height (54), and the
        // row reserves that — NOT the ~28px slot-only height that caused overlap.
        assert_eq!(boxes[&col].size().height, 54.0);
        assert_eq!(boxes[&row].size().height, 54.0);
        // Within the column the caption sits fully BELOW the slot: slot 0..28,
        // caption 40..54, no overlap.
        assert_eq!(boxes[&slot].origin().y, 0.0);
        assert_eq!(boxes[&slot].size().height, 28.0);
        assert_eq!(boxes[&cap].origin().y, 40.0);
        assert_eq!(boxes[&cap].size().height, 14.0);
    }

    #[test]
    fn placeholder_slot_is_measured_from_the_session_text_not_the_empty_value() {
        // The measure↔compose invariant for the CSS `::placeholder` slot: an
        // empty-valued field's ONLY text node is the `.placeholder` overlay. While a
        // session with typed text is active, BOTH passes shape the SESSION buffer over
        // that slot (`editing_override`), so the measured box is sized from the session
        // text — matching the glyphs the compose pass paints there (no measure/paint
        // skew). An empty session leaves the slot sized from the muted placeholder hint
        // (`:placeholder-shown` holds). Single-line mock: 9.6 px/char, so the two cases
        // differ purely by string length.
        use obscur_text::selection::TextSelection;
        let build = || {
            let mut tree = RetainedTree::new();
            let root = tree.alloc(NodeKind::Root);
            tree.set_root(root);
            let field = add_child(&mut tree, root, 700);
            let overlay = add_child(&mut tree, field, 701);
            // Mark the overlay element as the `.placeholder` pseudo-element.
            {
                use obscur_view::view::{CLASS, StyleAttributes, ViewProperty, ViewValue};
                let attrs = StyleAttributes::extract(&[ViewProperty::new(
                    CLASS,
                    ViewValue::Text("placeholder".into()),
                )]);
                if let Some(n) = tree.get_mut(overlay) {
                    n.set_attributes(attrs);
                }
            }
            let hint = add_text_child(&mut tree, overlay, "x"); // a 1-char muted hint
            let mut computed = ComputedStyles::new();
            computed.insert(root, ComputedStyle::new());
            computed.insert(field, ComputedStyle::new());
            computed.insert(overlay, ComputedStyle::new());
            computed.insert(hint, ComputedStyle::new());
            (tree, computed, field, hint)
        };
        let coordinator = LayoutCoordinator::new();
        let advance = 16.0 * 0.6; // mock advance per char

        // Typed session "abcd" over the placeholder → the slot is sized from "abcd".
        let (tree, computed, field, hint) = build();
        let styles = coordinator.layout_styles_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT, None);
        let typed = coordinator.layout_editing(
            &tree,
            &styles,
            &computed,
            &GridTemplates::new(),
            Size::new(400.0, 200.0),
            (0.0, 0.0),
            &test_paragraph(),
            None,
            Some(EditingView::new(field, "abcd", TextSelection::caret(4))),
        );
        assert!(
            (typed[&hint].size().width - advance * 4.0).abs() < 0.5,
            "placeholder slot measured from the 4-char session, got {}",
            typed[&hint].size().width,
        );

        // Empty session → the slot stays sized from the 1-char placeholder hint.
        let (tree, computed, field, hint) = build();
        let styles = coordinator.layout_styles_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT, None);
        let empty = coordinator.layout_editing(
            &tree,
            &styles,
            &computed,
            &GridTemplates::new(),
            Size::new(400.0, 200.0),
            (0.0, 0.0),
            &test_paragraph(),
            None,
            Some(EditingView::new(field, "", TextSelection::caret(0))),
        );
        assert!(
            (empty[&hint].size().width - advance).abs() < 0.5,
            "empty session leaves the slot sized from the 1-char placeholder hint, got {}",
            empty[&hint].size().width,
        );
    }
}
