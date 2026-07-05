//! Drives layout over the retained tree: [`LayoutCoordinator`].

use std::collections::HashMap;

use obscur_geometry::{BoxConstraints, Insets, Point, Rect, Size};
use obscur_identity::{InternedSymbol, NodeId};
use obscur_layout::algorithm::{
    ChildMeasurer, FlexLayout, FlowLayout, GridLayout, LayoutAlgorithm, LayoutChild,
};
use obscur_layout::model::{
    AlignItems, Dimension, FlexDirection, JustifyContent, LayoutBox, LayoutDisplay, LayoutStyle,
};
use obscur_style::computed::{ComputedStyle, ComputedValue};
use obscur_style::property::{layout_keyword, Length, LengthContext, PropertyId, PropertyValue};
use obscur_text::font::{
    FontAttributes, FontFace, FontFallback, FontFamily, FontMetrics, FontSource, GenericFamily,
};
use obscur_text::layout::{
    Hyphenator, NullHyphenator, ParagraphLayout, TextAlignment, TextOverflow,
};
use obscur_text::shaping::{ParagraphShaper, ShapedGlyph, TextDirection};
use obscur_tree::tree::RetainedTree;

use super::{
    alignment_for, attributes_for, direction_for, families_for, grid_templates_from, overflow_for,
    ComputedStyles, GridTemplates,
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
    #[must_use]
    pub fn layout_styles_from(
        &self,
        computed: &ComputedStyles,
        viewport: Size,
        root_font_size: f32,
    ) -> LayoutStyles {
        computed
            .iter()
            .map(|(&node, style)| (node, layout_style_from(style, viewport, root_font_size)))
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
    /// # Redundant work (Phase 1)
    ///
    /// The selected algorithm runs once per node during `arrange`, and each run
    /// re-measures that node's children (which recurses). This is deterministic
    /// and correct but does redundant measurement; a `LayoutCache` will remove
    /// the redundancy in a later phase (spec §4.2). Text is reshaped per frame.
    #[must_use]
    pub fn layout(
        &self,
        tree: &RetainedTree,
        styles: &LayoutStyles,
        computed: &ComputedStyles,
        grid_templates: &GridTemplates,
        viewport: Size,
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
    ) -> GridTemplates {
        grid_templates_from(computed, viewport, root_font_size)
    }

    /// Lays out like [`layout`](Self::layout) but wraps text-leaf nodes through an
    /// optional `hyphenator`, so an over-wide word breaks at a valid point
    /// (spec §5.10).
    ///
    /// The runtime passes the **same** hyphenator (and shapes the **same** hyphen
    /// glyph) the scene phase composes with, so the measured box height and the
    /// painted line count agree even when hyphenation splits a word. [`None`]
    /// reproduces [`layout`](Self::layout) exactly.
    // The tree, the three per-node side-tables (styles, computed, grid templates),
    // the viewport, and the shaping inputs (paragraph + hyphenator) are cohesive
    // inputs of one layout entry point, not an incidental over-long parameter list.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn layout_with_hyphenator(
        &self,
        tree: &RetainedTree,
        styles: &LayoutStyles,
        computed: &ComputedStyles,
        grid_templates: &GridTemplates,
        viewport: Size,
        paragraph: &ParagraphShaper,
        hyphenator: Option<&dyn Hyphenator>,
    ) -> LayoutBoxes {
        let mut boxes = LayoutBoxes::new();
        let Some(root) = tree.root() else {
            return boxes;
        };
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
            MAX_DEPTH,
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
    constraints: BoxConstraints,
    paragraph: &ParagraphShaper,
    hyphenator: Option<&dyn Hyphenator>,
) -> Size {
    let font_size = style.font_size;
    let metrics = paragraph.line_metrics(font_size);
    let runs = paragraph.shape_paragraph(
        content,
        fallback,
        attributes,
        &measure_face(),
        font_size,
        direction,
    );
    // A text node's own horizontal border/padding shrinks the wrap width; it is
    // almost always zero, but subtract it defensively so the content fits.
    let horizontal = style.padding.left + style.padding.right + style.border.left + style.border.right;
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
    let layout = match hyphenator {
        // No hyphenator and no ellipsis glyph: the exact pre-hyphenation path — no
        // hyphen or ellipsis glyph is shaped, so the measured size is byte-identical
        // to whitespace-only wrapping (Clip truncation still applies through
        // `overflow`, but only clips lines that actually overflow).
        None if ellipsis.is_none() => ParagraphLayout::wrap(
            &runs,
            content,
            max_width,
            metrics.line_height(),
            metrics.baseline(),
            alignment,
            overflow,
        ),
        // Ellipsis without a hyphenator: wrap through the no-op hyphenator so the
        // shaped ellipsis glyph can be threaded into truncation.
        None => ParagraphLayout::wrap_hyphenated(
            &runs,
            content,
            max_width,
            metrics.line_height(),
            metrics.baseline(),
            alignment,
            overflow,
            &NullHyphenator,
            None,
            ellipsis,
        ),
        // Hyphenation on: shape the hyphen the same way the scene phase does, so a
        // hyphenated break measures with the identical advance it paints with.
        Some(hyphenator) => {
            let hyphen =
                measure_hyphen_glyph(paragraph, fallback, attributes, direction, font_size);
            ParagraphLayout::wrap_hyphenated(
                &runs,
                content,
                max_width,
                metrics.line_height(),
                metrics.baseline(),
                alignment,
                overflow,
                hyphenator,
                hyphen,
                ellipsis,
            )
        }
    };
    constraints.constrain(layout.size())
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
        .shape_paragraph("-", fallback, attributes, &measure_face(), font_size, direction)
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
        .shape_paragraph("…", fallback, attributes, &measure_face(), font_size, direction)
        .iter()
        .flat_map(|run| run.run().glyphs().iter().copied())
        .next()
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
/// * `margin` / `padding` ← the four `*-top/right/bottom/left` longhands,
///   resolved to pixel [`Insets`] via [`insets_of`]. Insets are pixel-only, so a
///   `Percent`/`Auto` length contributes `0.0` on that edge. `border` stays at
///   its zero default (no CSS border-width property is modelled yet).
/// * `gap` ← [`PropertyId::Gap`] length in pixels (`Percent`/`Auto` → `0.0`).
/// * `flex_grow` / `flex_shrink` ← `PropertyId::{FlexGrow, FlexShrink}` numbers,
///   defaulting to the [`LayoutStyle::default`] values (`0.0` / `1.0`).
/// * `flex_basis` ← [`PropertyId::FlexBasis`] via [`dimension_of`].
/// * `flex_direction` ← [`PropertyId::FlexDirection`] keyword against the
///   reserved `FLEX_*` handles (default `Row`).
/// * `justify_content` ← [`PropertyId::JustifyContent`] keyword against the
///   reserved `JUSTIFY_*` handles (default `FlexStart`).
/// * `align_items` ← [`PropertyId::AlignItems`] keyword against the reserved
///   `ALIGN_*` handles (default `Stretch`).
fn layout_style_from(computed: &ComputedStyle, viewport: Size, root_font_size: f32) -> LayoutStyle {
    let default = LayoutStyle::default();
    let ctx = length_context(computed, viewport, root_font_size);
    LayoutStyle {
        display: display_of(computed),
        width: dimension_of(computed, PropertyId::Width, &ctx),
        height: dimension_of(computed, PropertyId::Height, &ctx),
        min_width: dimension_of(computed, PropertyId::MinWidth, &ctx),
        min_height: dimension_of(computed, PropertyId::MinHeight, &ctx),
        max_width: dimension_of(computed, PropertyId::MaxWidth, &ctx),
        max_height: dimension_of(computed, PropertyId::MaxHeight, &ctx),
        margin: insets_of(
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
        // No CSS border-width property is modelled yet; keep the zero default.
        border: default.border,
        flex_direction: flex_direction_of(computed),
        justify_content: justify_content_of(computed),
        align_items: align_items_of(computed),
        gap: length_px_of(computed, PropertyId::Gap, &ctx),
        flex_grow: number_of(computed, PropertyId::FlexGrow, default.flex_grow),
        flex_shrink: number_of(computed, PropertyId::FlexShrink, default.flex_shrink),
        flex_basis: dimension_of(computed, PropertyId::FlexBasis, &ctx),
        // The node's font size, so a text leaf can shape and wrap to its
        // available width inside the constraint-aware measure/arrange pass.
        font_size: font_size_px(computed),
        // Line-based grid placement is not yet mapped from CSS; keep the
        // all-auto default so grid children auto-place as before.
        grid_placement: default.grid_placement,
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
/// document `root_font_size` for `rem`, and the `viewport` for `vw`/`vh`.
///
/// The percentage basis is left unknown: a bare `%` is resolved later by layout
/// against the containing block, and a `%` term inside `calc()` therefore does
/// not fold here (that value falls back to `auto`/`0`, still without an
/// unsupported-feature diagnostic).
pub(super) fn length_context(
    computed: &ComputedStyle,
    viewport: Size,
    root_font_size: f32,
) -> LengthContext {
    LengthContext::new(
        font_size_px(computed),
        root_font_size,
        viewport.width,
        viewport.height,
    )
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

/// Resolves the computed `justify-content` keyword to a [`JustifyContent`],
/// defaulting to [`JustifyContent::FlexStart`].
///
/// Note: `space-evenly` has no reserved keyword handle yet, so it cannot be
/// produced here; it falls through to the default.
fn justify_content_of(computed: &ComputedStyle) -> JustifyContent {
    match keyword_sym(computed, PropertyId::JustifyContent) {
        Some(sym) if sym == layout_keyword::JUSTIFY_CENTER => JustifyContent::Center,
        Some(sym) if sym == layout_keyword::JUSTIFY_END => JustifyContent::FlexEnd,
        Some(sym) if sym == layout_keyword::JUSTIFY_SPACE_BETWEEN => JustifyContent::SpaceBetween,
        Some(sym) if sym == layout_keyword::JUSTIFY_SPACE_AROUND => JustifyContent::SpaceAround,
        _ => JustifyContent::FlexStart,
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
    /// Remaining recursion budget for children measured through this measurer.
    depth: u32,
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
                self.depth,
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
    depth: u32,
) -> Size {
    if depth == 0 {
        return Size::ZERO;
    }
    let style = style_of(styles, node);
    // A text node is a constraint-aware leaf: shape + wrap to `max.width`,
    // bypassing the box algorithms entirely (spec §5.10).
    if let Some(content) = tree.get(node).and_then(|n| n.kind().text()) {
        let fallback = families_for(computed.get(&node));
        let attributes = attributes_for(computed.get(&node));
        let direction = direction_for(computed.get(&node));
        let alignment = alignment_for(computed.get(&node));
        let overflow = overflow_for(computed.get(&node));
        return measure_text_node(
            content,
            &style,
            &fallback,
            &attributes,
            direction,
            alignment,
            overflow,
            constraints,
            paragraph,
            hyphenator,
        );
    }
    let Some(algorithm) = algorithm_for(style.display, node, grid_templates, style.gap) else {
        return Size::ZERO;
    };
    let children = children_of(tree, styles, node);
    let mut measurer = RecMeasurer {
        tree,
        styles,
        computed,
        grid_templates,
        children: &children,
        paragraph,
        hyphenator,
        depth: depth - 1,
    };
    let out = algorithm.layout(&style, &children, constraints, &mut measurer);
    out.size()
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
// The recursive arrange walk threads the tree, both style maps, the target
// origin/constraints, the output map, the shaper, and the depth budget; these
// are cohesive positional arguments of one internal recursion, not a public API.
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
    depth: u32,
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
        let overflow = overflow_for(computed.get(&node));
        let size = measure_text_node(
            content,
            &style,
            &fallback,
            &attributes,
            direction,
            alignment,
            overflow,
            constraints,
            paragraph,
            hyphenator,
        );
        boxes.insert(node, LayoutBox::new(Rect::new(origin, size)));
        return;
    }

    let Some(algorithm) = algorithm_for(style.display, node, grid_templates, style.gap) else {
        return;
    };
    let children = children_of(tree, styles, node);

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
            depth: depth - 1,
        };
        algorithm.layout(&style, &children, constraints, &mut measurer)
    };

    // Now safe to write this node's absolute box.
    boxes.insert(node, LayoutBox::new(Rect::new(origin, out.size())));

    // Recurse into children, translating each parent-relative box to absolute.
    // Zipping bounds both sides, so a mismatched length never indexes out of range.
    for (child, child_rel) in children.iter().zip(out.child_boxes().iter()) {
        let rel = child_rel.origin();
        let child_origin = Point::new(origin.x + rel.x, origin.y + rel.y);
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
            depth - 1,
        );
    }
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
                TextItem::new(TextRange::new(0, 2), ScriptTag::LATIN, TextDirection::LeftToRight, 0),
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
            &test_paragraph(),
        );
        assert!(boxes.is_empty());
    }

    #[test]
    fn mapping_reads_width_and_defaults_display_to_block() {
        let mut computed = ComputedStyle::new();
        computed.set(PropertyId::Width, ComputedValue::length(Length::px(120.0)));
        let style = layout_style_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT);
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
        let style = layout_style_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT);
        assert_eq!(style.width, Dimension::Percent(50.0));
        assert_eq!(style.height, Dimension::Auto);
    }

    #[test]
    fn mapping_resolves_relative_units_against_context() {
        use obscur_style::property::LengthExpr;
        let mut computed = ComputedStyle::new();
        // font-size 16px -> em resolves against it; viewport 800x600 -> vw/vh.
        computed.set(PropertyId::FontSize, ComputedValue::length(Length::px(16.0)));
        computed.set(PropertyId::Width, ComputedValue::length(Length::Em(3.0)));
        computed.set(PropertyId::Height, ComputedValue::length(Length::Vw(5.0)));
        // rem uses the document root font size (20), not the node font size.
        computed.set(PropertyId::MinWidth, ComputedValue::length(Length::Rem(2.0)));
        // calc(10px + 2em) = 42.
        computed.set(
            PropertyId::MaxWidth,
            ComputedValue::calc(LengthExpr::Sum(
                Box::new(LengthExpr::Leaf(Length::px(10.0))),
                Box::new(LengthExpr::Leaf(Length::Em(2.0))),
            )),
        );
        let style = layout_style_from(&computed, TEST_VIEWPORT, 20.0);
        assert_eq!(style.width, Dimension::Px(48.0)); // 3em
        assert_eq!(style.height, Dimension::Px(40.0)); // 5vw of 800
        assert_eq!(style.min_width, Dimension::Px(40.0)); // 2rem of 20
        assert_eq!(style.max_width, Dimension::Px(42.0)); // calc(10px + 2em)
    }

    #[test]
    fn mapping_resolves_relative_gap_to_pixels() {
        let mut computed = ComputedStyle::new();
        computed.set(PropertyId::FontSize, ComputedValue::length(Length::px(10.0)));
        computed.set(PropertyId::Gap, ComputedValue::length(Length::Em(2.0)));
        let style = layout_style_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT);
        assert_eq!(style.gap, 20.0); // 2em of 10px
    }

    #[test]
    fn root_font_size_reads_root_node_defaulting_to_sixteen() {
        let mut computed = ComputedStyles::new();
        let mut root = ComputedStyle::new();
        root.set(PropertyId::FontSize, ComputedValue::length(Length::px(24.0)));
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

        let styles = coordinator.layout_styles_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT);
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
            &test_paragraph(),
        );

        // Container offset by the root's top/left padding.
        assert_eq!(boxes[&container].origin(), Point::new(10.0, 10.0));
        // Leaf absolute origin = root padding (10) + container padding (5) = 15.
        assert_eq!(boxes[&leaf].origin(), Point::new(15.0, 15.0));
        assert_eq!(boxes[&leaf].size(), Size::new(50.0, 50.0));
    }

    #[test]
    fn mapping_populates_every_field_from_computed_style() {
        let mut computed = ComputedStyle::new();
        computed.set(
            PropertyId::Display,
            ComputedValue::keyword(layout_keyword::DISPLAY_FLEX),
        );
        // Box model.
        computed.set(PropertyId::MarginTop, ComputedValue::length(Length::px(1.0)));
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
        computed.set(PropertyId::MinWidth, ComputedValue::length(Length::px(10.0)));
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

        let style = layout_style_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT);

        assert_eq!(style.display, LayoutDisplay::Flex);
        assert_eq!(
            style.margin,
            Insets {
                top: 1.0,
                right: 2.0,
                bottom: 3.0,
                left: 4.0
            }
        );
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
        let style = layout_style_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT);
        assert_eq!(style.display, LayoutDisplay::Grid);
        assert!(
            algorithm_for(LayoutDisplay::Grid, NodeId::from_raw(1), &GridTemplates::new(), 0.0)
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
        assert_eq!(layout_style_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT).display, LayoutDisplay::None);
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
        let styles = coordinator.layout_styles_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT);
        // Sanity: the bridge produced a flex container.
        assert_eq!(styles[&root].display, LayoutDisplay::Flex);

        let boxes = coordinator.layout(
            &tree,
            &styles,
            &computed,
            &GridTemplates::new(),
            Size::new(200.0, 100.0),
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
        let styles = coordinator.layout_styles_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT);
        let templates = coordinator.grid_templates_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT);
        assert!(templates.contains_key(&root), "root authored a template");

        let boxes = coordinator.layout(
            &tree,
            &styles,
            &computed,
            &templates,
            Size::new(150.0, 100.0),
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
        root_style.set(PropertyId::Height, ComputedValue::length(Length::px(height)));
        computed.insert(root, root_style);

        let coordinator = LayoutCoordinator::new();
        let styles = coordinator.layout_styles_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT);
        let boxes = coordinator.layout(
            &tree,
            &styles,
            &computed,
            &GridTemplates::new(),
            Size::new(width, height),
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
        root_style.set(PropertyId::Height, ComputedValue::length(Length::px(height)));
        computed.insert(root, root_style);

        let coordinator = LayoutCoordinator::new();
        let styles = coordinator.layout_styles_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT);
        let boxes = coordinator.layout(
            &tree,
            &styles,
            &computed,
            &GridTemplates::new(),
            Size::new(width, height),
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
        // Widest line ("aaaa " = 5·9.6) fits within the 50px container width.
        assert!((text_box.size().width - 48.0).abs() < 1e-3, "width {}", text_box.size().width);
    }

    #[test]
    fn same_text_in_a_wide_container_stays_one_line() {
        // Requirement (b): the identical string in a 1000px container is a single
        // line — one line_height tall — and as wide as the whole run.
        let text_box = text_box_in_container("aaaa bbbb cccc", 1000.0, 200.0);
        assert_eq!(text_box.size().height, 16.0);
        // Full run advance: 14 chars · 9.6 = 134.4.
        assert!((text_box.size().width - 134.4).abs() < 1e-3, "width {}", text_box.size().width);
    }

    #[test]
    fn short_single_line_text_is_sized_to_its_run() {
        // Requirement (e): short text that fits stays one line at its run advance.
        let text_box = text_box_in_container("hi", 1000.0, 200.0);
        assert_eq!(text_box.size().height, 16.0);
        // 2 chars · 9.6 = 19.2.
        assert!((text_box.size().width - 19.2).abs() < 1e-3, "width {}", text_box.size().width);
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
        text_computed.set(PropertyId::FontSize, ComputedValue::length(Length::px(20.0)));
        computed.insert(text, text_computed);

        let coordinator = LayoutCoordinator::new();
        let styles = coordinator.layout_styles_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT);
        assert_eq!(styles[&text].font_size, 20.0);
        let boxes = coordinator.layout(
            &tree,
            &styles,
            &computed,
            &GridTemplates::new(),
            Size::new(1000.0, 200.0),
            &test_paragraph(),
        );
        let text_box = boxes[&text];
        // 4 chars · (20 · 0.6) = 48.0 wide, one line 20 tall.
        assert!((text_box.size().width - 48.0).abs() < 1e-3, "width {}", text_box.size().width);
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
            &test_paragraph(),
        );
        // A `None` root produces no box at all.
        assert!(boxes.is_empty());
    }
}
