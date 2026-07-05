//! Composes a paint-ordered display list from the styled + laid-out tree:
//! [`SceneCoordinator`].

use std::collections::HashSet;
use std::fmt;

use obscur_geometry::Point;
use obscur_identity::NodeId;
use obscur_scene::color::Color;
use obscur_scene::command::{DisplayList, DisplayListBuilder};
use obscur_scene::paint::Brush;
use obscur_scene::text::{GlyphRun, PositionedGlyph};
use obscur_style::computed::ComputedValue;
use obscur_style::property::{Length, PropertyId, StyleColor};
use obscur_text::font::{
    FontAttributes, FontFace, FontFallback, FontFamily, FontMetrics, FontResolver, FontSource,
    GenericFamily, MockFontResolver,
};
use obscur_text::layout::{
    Hyphenator, NullHyphenator, ParagraphLayout, TextAlignment, TextOverflow,
};
use obscur_text::shaping::{
    CachingShaper, MockTextShaper, ParagraphShaper, ShapedGlyph, ShapedRun, SingleRunItemizer,
    TextDirection, TextItemizer, TextShaper,
};
use obscur_tree::tree::RetainedTree;

use super::{
    alignment_for, attributes_for, direction_for, families_for, overflow_for, ComputedStyles,
    LayoutBoxes,
};

/// Default font size (in pixels) used to emit text runs.
///
/// A text node with no computed `font-size` (or one that is not an absolute
/// pixel length) is shaped at this fallback; a node that carries `font-size: Npx`
/// is shaped at that value (see [`font_size_of`]).
const DEFAULT_FONT_SIZE: f32 = 16.0;

/// Orchestrates the scene phase: turns the styled + laid-out tree into a
/// Scene/Paint [`DisplayList`] with a deterministic paint order (spec §5.1,
/// §3.10, §3.4).
///
/// The coordinator is **orchestration only**: it walks the retained tree and
/// emits paint commands through `obscur-scene`'s [`DisplayListBuilder`], but it
/// performs no rasterization — turning the display list into pixels is a
/// renderer concern in a later phase.
///
/// # Paint order
///
/// The tree is walked **pre-order** (a parent is emitted before its children,
/// children left-to-right), which is exactly the scene's deterministic paint
/// order: a parent paints first and its children paint on top (spec §5.1). The
/// walk is **cycle-safe** — a visited-set guard means a shared or cyclic child
/// link terminates instead of looping — and **panic-free**: every lookup is
/// fallible and no indexing is performed.
///
/// # Phase-1 scope
///
/// The coordinator emits **background fills** and **shaped text runs**: a node
/// whose computed `background-color` is present and not fully transparent fills
/// its arranged layout rectangle, and a text node with an arranged box emits a
/// [`draw_text`](DisplayListBuilder::draw_text) run at that box's origin.
///
/// Text is **shaped once, here in the pipeline**, through an `obscur-text`
/// [`TextShaper`] port (the deterministic [`MockTextShaper`] by default): the
/// coordinator turns the node's string into a [`GlyphRun`] of positioned glyphs
/// and carries those glyphs in the [`DrawText`](obscur_scene::command::PaintCommand::DrawText)
/// command, so every renderer consumes an identical run and only rasterization
/// differs (spec §5.10, §5.2). Each run is shaped at the node's computed
/// `font-size` (falling back to [`DEFAULT_FONT_SIZE`]) and its glyphs sit on the
/// font's real baseline (the shaper's [`line_metrics`](TextShaper::line_metrics)
/// ascent), the same size and metrics the layout phase measured the box with.
/// Opacity/clip layers, borders, and images are
/// deferred to later phases. Because only leaf
/// [`fill_rect`](DisplayListBuilder::fill_rect) and
/// [`draw_text`](DisplayListBuilder::draw_text) commands are emitted, the
/// resulting [`DisplayList`] is always well-formed (balanced) and can be consumed
/// by a renderer or `Scene` directly.
///
/// The coordinator owns its paragraph orchestrator and a default resolved face;
/// construct it with [`new`](Self::new) or [`Default`].
///
/// Text is shaped through a [`ParagraphShaper`] — the multi-font fallback
/// orchestrator composing an itemizer, a font resolver, and an inner
/// [`TextShaper`]. The default composition (a [`SingleRunItemizer`], an empty
/// [`MockFontResolver`], and a cache-wrapped [`MockTextShaper`]) reproduces the
/// historical single-font behavior exactly, so headless determinism is preserved;
/// a host installs a real engine, a Unicode itemizer, and a coverage registry via
/// [`set_text_shaper`](Self::set_text_shaper),
/// [`set_itemizer`](Self::set_itemizer), and
/// [`set_font_resolver`](Self::set_font_resolver). Because `ParagraphShaper` holds
/// `dyn` ports it is neither `Clone` nor `Debug`, so the coordinator is not
/// `Clone` and carries a manual [`fmt::Debug`].
pub struct SceneCoordinator {
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
    /// The hyphenation port used to break an over-wide word at a valid point
    /// (spec §5.10), or [`None`] when no hyphenator is installed.
    ///
    /// The default is [`None`]: text wraps exactly as it did before hyphenation
    /// existed (no hyphen glyph is even shaped), which is what the cross-backend
    /// determinism boundary relies on. A host installs a real adapter via
    /// [`set_hyphenator`](Self::set_hyphenator); this is the
    /// [`NullHyphenator`](obscur_text::layout::NullHyphenator)-equivalent default
    /// made explicit so the byte-identical path takes no extra shaping.
    hyphenator: Option<Box<dyn Hyphenator>>,
}

impl SceneCoordinator {
    /// Creates a coordinator whose default paragraph orchestrator reproduces the
    /// historical single-font behavior (single run, primary face, deterministic
    /// [`MockTextShaper`]) with a generic monospace default face.
    #[must_use]
    pub fn new() -> Self {
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

    /// Installs `hyphenator` as the hyphenation port used to break an over-wide
    /// word, replacing the default (no hyphenation) (spec §5.10).
    ///
    /// This is the seam the runtime's
    /// [`set_hyphenator`](crate::runtime::UiRuntime::set_hyphenator) drives. The
    /// same hyphenator sizes text in the measure pass and paints it in the
    /// compose pass, so a hyphenated line's measured height and painted line count
    /// agree. Leaving it unset preserves headless determinism.
    pub fn set_hyphenator(&mut self, hyphenator: Box<dyn Hyphenator>) {
        self.hyphenator = Some(hyphenator);
    }

    /// Returns the installed hyphenation port, or [`None`] when hyphenation is
    /// off (the default).
    ///
    /// The layout phase measures text through this so it wraps (and hyphenates)
    /// identically to the compose phase, keeping measured box height and painted
    /// line count in agreement.
    #[must_use]
    pub fn hyphenator(&self) -> Option<&dyn Hyphenator> {
        self.hyphenator.as_deref()
    }

    /// Installs `shaper` as the inner text shaper subsequent shaping runs through,
    /// replacing the previous one and keeping the itemizer and resolver in place.
    ///
    /// The engine is wrapped in a **fresh** shaping cache so every shape call
    /// (measure, arrange, compose) is memoized behind one seam and no run shaped
    /// by the previous engine survives. Installing a different shaper changes the
    /// emitted [`GlyphRun`] — that is the seam the runtime's `set_text_shaper`
    /// drives (e.g. the HarfRust adapter for real, font-specific glyph indices).
    pub fn set_text_shaper(&mut self, shaper: Box<dyn TextShaper>) {
        self.paragraph
            .set_shaper(Box::new(CachingShaper::new(shaper)));
    }

    /// Installs `itemizer` as the paragraph itemizer, replacing the default
    /// [`SingleRunItemizer`] (e.g. a real Unicode script/bidi itemizer) so a text
    /// node splits into multiple script runs.
    pub fn set_itemizer(&mut self, itemizer: Box<dyn TextItemizer>) {
        self.paragraph.set_itemizer(itemizer);
    }

    /// Installs `resolver` as the font resolver, replacing the default empty
    /// [`MockFontResolver`] (e.g. a coverage registry) so each script run resolves
    /// to a covering font.
    pub fn set_font_resolver(&mut self, resolver: Box<dyn FontResolver>) {
        self.paragraph.set_resolver(resolver);
    }

    /// Returns the paragraph orchestrator as a shared borrow.
    ///
    /// This is the seam the layout phase measures text through: `SceneCoordinator`
    /// owns the orchestrator, so the layout coordinator borrows it from here to
    /// size text nodes with the **same** itemizer/resolver/shaper (and therefore
    /// the same runs, advances, and line metrics) the scene phase later draws
    /// with. Because `layout` and `scene_composer` are disjoint fields of the
    /// runtime, this shared borrow coexists with a mutable borrow of the layout
    /// coordinator.
    #[must_use]
    pub fn paragraph(&self) -> &ParagraphShaper {
        &self.paragraph
    }

    /// Shapes `content` at `font_size` through the fallback orchestrator and
    /// **wraps** the font-tagged runs to `max_width` into a multi-line, multi-font
    /// [`ParagraphLayout`] (spec §5.10).
    ///
    /// Shaping runs **once**, here: [`shape_paragraph`](ParagraphShaper::shape_paragraph)
    /// produces one [`ShapedRun`] per script/font run, which
    /// [`ParagraphLayout::wrap`] greedily breaks into lines no wider than
    /// `max_width` (the arranged box width), each line split into contiguous
    /// same-font segments. Because the layout phase measured the box by wrapping
    /// the **same** orchestrator's runs to the **same** width, the wrapped line
    /// count here matches the box height layout produced. A `max_width <= 0` yields
    /// a single line.
    ///
    /// `fallback` is the node's resolved font chain (from its computed
    /// `font-family` via [`families_for`], or the default chain when unauthored);
    /// `attributes` (weight + slant, via [`attributes_for`]), `direction` (the
    /// base bidi direction, via [`direction_for`]), and `alignment` (the line
    /// alignment, via [`alignment_for`]) come from the node's computed
    /// `font-weight` / `font-style` / `direction` / `text-align`. The layout phase
    /// measures the box with the identical chain, attributes, direction, and
    /// alignment, so the shaped runs — and therefore the wrapped and justified
    /// lines — agree.
    // The shaping inputs (fallback chain, attributes, direction, alignment) plus
    // the font size and wrap width are cohesive positional arguments of one leaf
    // shaping helper, not a public API surface. `attributes` is kept by reference
    // to match the `shape_paragraph` convention used throughout this pass.
    #[allow(clippy::too_many_arguments, clippy::trivially_copy_pass_by_ref)]
    fn shape_lines(
        &self,
        content: &str,
        fallback: &FontFallback,
        attributes: &FontAttributes,
        direction: TextDirection,
        alignment: TextAlignment,
        overflow: TextOverflow,
        font_size: f32,
        max_width: f32,
    ) -> ParagraphLayout {
        let runs = self.paragraph.shape_paragraph(
            content,
            fallback,
            attributes,
            &self.default_face,
            font_size,
            direction,
        );
        let metrics = self.paragraph.line_metrics(font_size);
        // `text-overflow: ellipsis` needs a shaped ellipsis glyph to append at a
        // cut; `clip` / `visible` need none, so none is shaped on the default path
        // (Clip truncation works from advances alone).
        let ellipsis = if overflow == TextOverflow::Ellipsis {
            self.shape_ellipsis(fallback, attributes, direction, font_size)
        } else {
            None
        };
        match self.hyphenator.as_deref() {
            // No hyphenator and no ellipsis glyph: the exact pre-hyphenation path —
            // no hyphen or ellipsis glyph is shaped, so the output (and the
            // shaping-cache call count) is byte-identical to today. Clip truncation
            // still applies through `overflow`, but only clips lines that overflow.
            None if ellipsis.is_none() => ParagraphLayout::wrap(
                &runs,
                content,
                max_width,
                metrics.line_height(),
                metrics.baseline(),
                alignment,
                overflow,
            ),
            // Ellipsis without a hyphenator: wrap through the no-op hyphenator so
            // the shaped ellipsis glyph can be threaded into truncation.
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
            // Hyphenation on: shape the hyphen glyph so a break renders (and
            // measures) with a real advance, and wrap through the installed port.
            Some(hyphenator) => {
                let hyphen = self.shape_hyphen(fallback, attributes, direction, font_size);
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
        }
    }

    /// Shapes a single ellipsis (`…`) at `font_size` through the paragraph
    /// orchestrator and returns its first glyph, for
    /// [`ParagraphLayout::wrap_hyphenated`] to append when `text-overflow:
    /// ellipsis` truncates an overflowing line (spec §5.10).
    ///
    /// Shaped with the coordinator's default face — identical to the layout
    /// coordinator's `measure_face` — so the glyph the compose pass paints matches
    /// the one the measure pass sized with. Returns [`None`] only if shaping yields
    /// no glyph.
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
            .shape_paragraph("…", fallback, attributes, &self.default_face, font_size, direction)
            .iter()
            .flat_map(|run| run.run().glyphs().iter().copied())
            .next()
    }

    /// Shapes a single hyphen (`-`) at `font_size` through the paragraph
    /// orchestrator and returns its first glyph, for
    /// [`ParagraphLayout::wrap_hyphenated`] to render at a break (spec §5.10).
    ///
    /// The hyphen is shaped with the coordinator's default face — identical to the
    /// layout coordinator's `measure_face` — so the glyph the compose pass paints
    /// matches the one the measure pass sized with. Returns [`None`] only if
    /// shaping yields no glyph.
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
            .shape_paragraph("-", fallback, attributes, &self.default_face, font_size, direction)
            .iter()
            .flat_map(|run| run.run().glyphs().iter().copied())
            .next()
    }

    /// Composes a paint-ordered [`DisplayList`] of background fills from the
    /// styled + laid-out tree.
    ///
    /// Returns an empty (and balanced) display list when the tree has no root.
    /// Otherwise the tree is walked pre-order from the root (parent before
    /// children, children left-to-right, cycle-safe); for each visited node that
    /// has a non-transparent computed `background-color` **and** an arranged box
    /// in `boxes`, a fill of that box's rectangle is appended in paint order.
    ///
    /// `computed` supplies each node's [`ComputedStyle`](obscur_style::computed::ComputedStyle)
    /// (typically produced by the style coordinator) and `boxes` supplies each
    /// node's absolute layout rectangle (produced by the layout coordinator). A
    /// node missing from either map simply contributes no fill.
    #[must_use]
    pub fn compose(
        &self,
        tree: &RetainedTree,
        computed: &ComputedStyles,
        boxes: &LayoutBoxes,
    ) -> DisplayList {
        let mut builder = DisplayListBuilder::new();
        let Some(root) = tree.root() else {
            return builder.build();
        };

        // Pre-order walk (parent before children). Children are pushed reversed
        // so the leftmost child is popped first, preserving left-to-right paint
        // order. The visited set makes the walk cycle-safe and panic-free.
        let mut visited: HashSet<NodeId> = HashSet::new();
        let mut stack: Vec<NodeId> = vec![root];
        while let Some(node) = stack.pop() {
            if !visited.insert(node) {
                // Already painted (cycle or shared child): do not revisit.
                continue;
            }

            if let Some(color) = background_color_of(computed, node) {
                // Skip fully transparent backgrounds: they paint nothing.
                if color.a > 0.0
                    && let Some(layout_box) = boxes.get(&node)
                {
                    builder.fill_rect(layout_box.bounds, Brush::solid(scene_color_from(color)));
                }
            }

            if let Some(retained) = tree.get(node) {
                // Text nodes emit one text run **per wrapped line** at their
                // arranged box. A node is either a text node or an element with a
                // background, so this never double-emits with the background fill
                // above. The content is wrapped to the box width the layout phase
                // measured with — so the emitted line count matches the box height
                // — and each line is drawn at `(origin.x + x_offset, origin.y +
                // top)` with its glyphs on that line's baseline. A text node with
                // no arranged box is skipped.
                if let Some(content) = retained.kind().text()
                    && let Some(layout_box) = boxes.get(&node)
                {
                    let brush = Brush::solid(text_color_of(computed, node));
                    // Same per-node font size the layout phase measured with, so
                    // the drawn advances and baseline match the sized box.
                    let font_size = font_size_of(computed, node);
                    // The node's authored `font-family` drives the fallback chain,
                    // and its `font-weight`/`font-style`/`direction` drive the
                    // shaping attributes and base direction — the SAME inputs the
                    // layout phase measured this box with.
                    let fallback = families_for(computed.get(&node));
                    let attributes = attributes_for(computed.get(&node));
                    let direction = direction_for(computed.get(&node));
                    let alignment = alignment_for(computed.get(&node));
                    let overflow = overflow_for(computed.get(&node));
                    let origin = layout_box.origin();
                    let layout = self.shape_lines(
                        content,
                        &fallback,
                        &attributes,
                        direction,
                        alignment,
                        overflow,
                        font_size,
                        layout_box.size().width,
                    );
                    for line in layout.lines() {
                        let baseline = line.baseline();
                        let line_top = origin.y + line.top();
                        let line_left = origin.x + line.x_offset();
                        if line.segments().is_empty() {
                            // An empty line (e.g. empty text) still emits a single
                            // empty run at the line origin so the painted line count
                            // matches the box height (one line per wrapped line).
                            let range = line.range();
                            let line_text =
                                content.get(range.start..range.end).unwrap_or_default();
                            let run = GlyphRun::new(Vec::new(), font_size, line_text);
                            builder.draw_text(Point::new(line_left, line_top), run, brush.clone());
                            continue;
                        }
                        // Walk the line's contiguous same-font segments left to
                        // right, emitting one DrawText per segment at the running
                        // pen origin so a multi-font line paints as several
                        // font-tagged runs (spec §5.10).
                        let mut seg_x = 0.0_f32;
                        for segment in line.segments() {
                            let range = segment.run().source();
                            let seg_text =
                                content.get(range.start..range.end).unwrap_or_default();
                            let run = positioned_run(segment, baseline, font_size, seg_text);
                            let seg_origin = Point::new(line_left + seg_x, line_top);
                            builder.draw_text(seg_origin, run, brush.clone());
                            seg_x += segment.run().advance();
                        }
                    }
                }

                for &child in retained.children().iter().rev() {
                    if !visited.contains(&child) {
                        stack.push(child);
                    }
                }
            }
        }

        builder.build()
    }
}

impl Default for SceneCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for SceneCoordinator {
    /// `ParagraphShaper` holds `dyn` ports and is not `Debug`, so it is shown as an
    /// opaque placeholder; the resolved default face is printed normally.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SceneCoordinator")
            .field("paragraph", &"<ParagraphShaper>")
            .field("default_face", &self.default_face)
            .field("hyphenator", &"<dyn Hyphenator>")
            .finish()
    }
}

/// Converts one line [`segment`](ShapedRun) into a font-tagged scene [`GlyphRun`],
/// laying its glyphs out left-to-right into pen positions on the line's baseline.
///
/// The segment's glyphs carry only per-glyph advances, so the pen accumulates
/// them from the segment's left edge (`0`, relative to the segment origin the
/// caller places); each glyph's `y` is the line `baseline` (the ascent within the
/// line box), and the shaping `x_offset`/`y_offset` are added on top. `text` is
/// the source substring the segment covers, and the run is tagged with the
/// segment's [`font`](ShapedRun::font) index so the renderer can pick the right
/// face. For a single-font line (one segment) this reproduces the original
/// single-run behavior exactly, with font `0`.
fn positioned_run(segment: &ShapedRun, baseline: f32, font_size: f32, text: &str) -> GlyphRun {
    let mut pen_x = 0.0_f32;
    let mut glyphs: Vec<PositionedGlyph> = Vec::new();
    for glyph in segment.run().glyphs() {
        glyphs.push(PositionedGlyph::new(
            glyph.glyph_id(),
            glyph.cluster(),
            pen_x + glyph.x_offset(),
            baseline + glyph.y_offset(),
            glyph.x_advance(),
        ));
        pen_x += glyph.x_advance();
    }
    GlyphRun::new(glyphs, font_size, text).with_font(segment.font().get())
}

/// Reads a node's computed `background-color`, if present and color-valued.
///
/// Returns [`None`] when the node has no computed style, no `background-color`
/// property, or a non-color value for it.
fn background_color_of(computed: &ComputedStyles, node: NodeId) -> Option<StyleColor> {
    computed
        .get(&node)
        .and_then(|style| style.get(PropertyId::BackgroundColor))
        .and_then(ComputedValue::as_color)
}

/// Reads a node's computed text `color` as a scene [`Color`], defaulting to
/// [`Color::WHITE`] when absent.
///
/// Mirrors [`background_color_of`] but reads [`PropertyId::Color`]. When the node
/// has no computed style, no `color` property, or a non-color value, the run
/// falls back to opaque white (visible on the example's colored bars).
fn text_color_of(computed: &ComputedStyles, node: NodeId) -> Color {
    computed
        .get(&node)
        .and_then(|style| style.get(PropertyId::Color))
        .and_then(ComputedValue::as_color)
        .map_or(Color::WHITE, scene_color_from)
}

/// Reads a node's computed `font-size` in logical pixels, defaulting to
/// [`DEFAULT_FONT_SIZE`] when absent or not an absolute pixel length.
///
/// This mirrors the layout coordinator's font-size resolution so the scene draws
/// each text run at the same size the layout phase measured its box with (only
/// absolute `px` is honored here; relative `font-size` units are a later
/// extension, matching layout).
fn font_size_of(computed: &ComputedStyles, node: NodeId) -> f32 {
    computed
        .get(&node)
        .and_then(|style| style.get(PropertyId::FontSize))
        .and_then(ComputedValue::as_length)
        .and_then(Length::as_px)
        .map_or(DEFAULT_FONT_SIZE, |px| px.0)
}

/// Converts an authored style color into a scene paint color.
///
/// [`StyleColor`] is the authored style color (straight, non-premultiplied sRGB
/// components); the scene [`Color`] is the paint color a renderer consumes.
/// Phase 1 copies the components verbatim — the linear/premultiplied conversion
/// the GPU needs is a renderer concern applied later, not in the coordinator.
fn scene_color_from(c: StyleColor) -> Color {
    Color::rgba(c.r, c.g, c.b, c.a)
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_geometry::{Point, Rect, Size};
    use obscur_identity::ComponentId;
    use obscur_layout::model::LayoutBox;
    use obscur_scene::command::PaintCommand;
    use obscur_style::computed::ComputedStyle;
    use obscur_tree::node::{LogicalParent, NodeKind};

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

    /// Builds a `ComputedStyle` carrying a single text `color`.
    fn with_color(color: StyleColor) -> ComputedStyle {
        let mut style = ComputedStyle::new();
        style.set(PropertyId::Color, ComputedValue::color(color));
        style
    }

    /// Collects the `DrawText` runs (origin, glyph run, brush) in a display list.
    fn text_runs(list: &DisplayList) -> Vec<(Point, GlyphRun, Brush)> {
        list.commands()
            .iter()
            .filter_map(|c| match c {
                PaintCommand::DrawText { origin, run, brush } => {
                    Some((*origin, run.clone(), brush.clone()))
                }
                _ => None,
            })
            .collect()
    }

    /// Builds a `ComputedStyle` carrying a single `background-color`.
    fn with_background(color: StyleColor) -> ComputedStyle {
        let mut style = ComputedStyle::new();
        style.set(PropertyId::BackgroundColor, ComputedValue::color(color));
        style
    }

    /// A layout box at the origin with the given size.
    fn box_at(x: f32, y: f32, w: f32, h: f32) -> LayoutBox {
        LayoutBox::new(Rect::new(Point::new(x, y), Size::new(w, h)))
    }

    /// Counts the `DrawPath` (fill) commands in a display list.
    fn fill_count(list: &DisplayList) -> usize {
        list.commands()
            .iter()
            .filter(|c| matches!(c, PaintCommand::DrawPath { .. }))
            .count()
    }

    #[test]
    fn empty_tree_composes_empty_balanced_list() {
        let coordinator = SceneCoordinator::new();
        let tree = RetainedTree::new();
        let computed = ComputedStyles::new();
        let boxes = LayoutBoxes::new();

        let list = coordinator.compose(&tree, &computed, &boxes);
        assert!(list.is_empty());
        assert!(list.is_balanced());
    }

    #[test]
    fn root_and_two_children_fill_in_preorder() {
        // root (bg) -> [child_a (bg), child_b (no bg)].
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let child_a = add_child(&mut tree, root, 10);
        let child_b = add_child(&mut tree, root, 11);

        let root_bg = StyleColor::rgba(1.0, 0.0, 0.0, 1.0);
        let child_a_bg = StyleColor::rgba(0.0, 1.0, 0.0, 1.0);

        let mut computed = ComputedStyles::new();
        computed.insert(root, with_background(root_bg));
        computed.insert(child_a, with_background(child_a_bg));
        // child_b has a style but no background-color.
        computed.insert(child_b, ComputedStyle::new());

        let mut boxes = LayoutBoxes::new();
        boxes.insert(root, box_at(0.0, 0.0, 100.0, 100.0));
        boxes.insert(child_a, box_at(0.0, 0.0, 100.0, 50.0));
        boxes.insert(child_b, box_at(0.0, 50.0, 100.0, 50.0));

        let coordinator = SceneCoordinator::new();
        let list = coordinator.compose(&tree, &computed, &boxes);

        // Two nodes have a non-transparent background -> two fills.
        assert_eq!(fill_count(&list), 2);
        assert_eq!(list.len(), 2);
        assert!(list.is_balanced());

        // Paint order is pre-order: the root's fill is emitted first, then the
        // child's. Assert the rectangles in order.
        let rects: Vec<Rect> = list
            .commands()
            .iter()
            .filter_map(|c| match c {
                PaintCommand::DrawPath { path, .. } => path.bounds(),
                _ => None,
            })
            .collect();
        assert_eq!(rects.len(), 2);
        assert_eq!(
            rects[0],
            Rect::new(Point::new(0.0, 0.0), Size::new(100.0, 100.0))
        );
        assert_eq!(
            rects[1],
            Rect::new(Point::new(0.0, 0.0), Size::new(100.0, 50.0))
        );
    }

    #[test]
    fn fully_transparent_background_emits_no_fill() {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);

        let transparent = StyleColor::rgba(1.0, 0.0, 0.0, 0.0);
        let mut computed = ComputedStyles::new();
        computed.insert(root, with_background(transparent));

        let mut boxes = LayoutBoxes::new();
        boxes.insert(root, box_at(0.0, 0.0, 10.0, 10.0));

        let coordinator = SceneCoordinator::new();
        let list = coordinator.compose(&tree, &computed, &boxes);

        assert!(list.is_empty());
        assert!(list.is_balanced());
    }

    #[test]
    fn node_without_a_box_emits_no_fill() {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);

        let mut computed = ComputedStyles::new();
        computed.insert(root, with_background(StyleColor::rgba(0.0, 0.0, 1.0, 1.0)));

        // No arranged box supplied for the root.
        let boxes = LayoutBoxes::new();

        let coordinator = SceneCoordinator::new();
        let list = coordinator.compose(&tree, &computed, &boxes);
        assert!(list.is_empty());
    }

    #[test]
    fn text_node_emits_run_at_box_origin_with_default_white() {
        // root (bg) -> text child (no color set -> default white).
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let text = add_text_child(&mut tree, root, "hello");

        let mut computed = ComputedStyles::new();
        computed.insert(root, with_background(StyleColor::rgba(1.0, 0.0, 0.0, 1.0)));
        // Text node has no computed color property.
        computed.insert(text, ComputedStyle::new());

        let mut boxes = LayoutBoxes::new();
        boxes.insert(root, box_at(0.0, 0.0, 100.0, 100.0));
        // Box wide enough for all five 9.6px glyphs (48px) so the default `clip`
        // policy has nothing to truncate — this test exercises shaping, not overflow.
        boxes.insert(text, box_at(5.0, 12.0, 60.0, 16.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes);

        // Background element still fills.
        assert_eq!(fill_count(&list), 1);

        let runs = text_runs(&list);
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].0, Point::new(5.0, 12.0));
        // The run was shaped in the domain: five chars -> five positioned glyphs.
        assert_eq!(runs[0].1.text(), "hello");
        assert_eq!(runs[0].1.len(), 5);
        assert_eq!(runs[0].2, Brush::solid(Color::WHITE));

        // Leaf-only list stays balanced.
        assert!(list.is_balanced());
    }

    #[test]
    fn text_node_shapes_one_positioned_glyph_per_char_in_order() {
        // A four-char run yields four glyphs, laid out left-to-right at the
        // mock's fixed advance, on the real baseline (the ascent below origin,
        // 0.8·size for the mock — NOT the full em).
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let text = add_text_child(&mut tree, root, "abcd");

        let mut computed = ComputedStyles::new();
        computed.insert(text, ComputedStyle::new());
        let mut boxes = LayoutBoxes::new();
        boxes.insert(text, box_at(0.0, 0.0, 40.0, 16.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes);
        let runs = text_runs(&list);
        assert_eq!(runs.len(), 1);
        let run = &runs[0].1;
        assert_eq!(run.len(), 4);
        let advance = DEFAULT_FONT_SIZE * 0.6;
        // The mock's idealized ascent baseline: 0.8 * 16 = 12.8, below the em.
        let baseline = DEFAULT_FONT_SIZE * 0.8;
        assert_ne!(baseline, DEFAULT_FONT_SIZE);
        for (i, glyph) in run.glyphs().iter().enumerate() {
            assert_eq!(glyph.glyph_id(), u32::from(['a', 'b', 'c', 'd'][i]));
            assert_eq!(glyph.cluster(), i);
            assert_eq!(glyph.x(), advance * i as f32);
            assert_eq!(glyph.y(), baseline);
            assert_eq!(glyph.advance(), advance);
        }
    }

    #[test]
    fn composed_glyphs_sit_on_the_ascent_baseline_not_the_full_em() {
        // Requirement (d): the baseline is the font's ascent (0.8·size for the
        // idealized mock), strictly less than the full em `DEFAULT_FONT_SIZE`.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let text = add_text_child(&mut tree, root, "hi");

        let mut computed = ComputedStyles::new();
        computed.insert(text, ComputedStyle::new());
        let mut boxes = LayoutBoxes::new();
        boxes.insert(text, box_at(0.0, 0.0, 40.0, 16.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes);
        let runs = text_runs(&list);
        let run = &runs[0].1;
        let ascent = DEFAULT_FONT_SIZE * 0.8; // 12.8
        for glyph in run.glyphs() {
            assert_eq!(glyph.y(), ascent);
            assert!(glyph.y() < DEFAULT_FONT_SIZE);
        }
    }

    #[test]
    fn text_shaping_is_deterministic_across_composes() {
        // The same input tree must produce byte-identical positioned glyphs on
        // every compose (the property both backends' agreement rests on).
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let text = add_text_child(&mut tree, root, "Count: 7");

        let mut computed = ComputedStyles::new();
        computed.insert(text, ComputedStyle::new());
        let mut boxes = LayoutBoxes::new();
        boxes.insert(text, box_at(2.0, 3.0, 80.0, 16.0));

        let coordinator = SceneCoordinator::new();
        let first = text_runs(&coordinator.compose(&tree, &computed, &boxes));
        let second = text_runs(&coordinator.compose(&tree, &computed, &boxes));
        assert_eq!(first, second);
    }

    #[test]
    fn text_node_uses_its_computed_color() {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let text = add_text_child(&mut tree, root, "tinted");

        let color = StyleColor::rgba(0.1, 0.2, 0.3, 1.0);
        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(text, with_color(color));

        let mut boxes = LayoutBoxes::new();
        boxes.insert(text, box_at(1.0, 2.0, 10.0, 10.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes);

        let runs = text_runs(&list);
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].2, Brush::solid(Color::rgba(0.1, 0.2, 0.3, 1.0)));
    }

    #[test]
    fn zero_size_text_box_still_emits_a_run() {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let text = add_text_child(&mut tree, root, "z");

        let mut computed = ComputedStyles::new();
        computed.insert(text, ComputedStyle::new());

        let mut boxes = LayoutBoxes::new();
        boxes.insert(text, box_at(3.0, 4.0, 0.0, 0.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes);

        let runs = text_runs(&list);
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].0, Point::new(3.0, 4.0));
        assert_eq!(runs[0].1.text(), "z");
        assert_eq!(runs[0].1.len(), 1);
    }

    #[test]
    fn text_node_without_a_box_emits_no_run() {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let text = add_text_child(&mut tree, root, "orphan");

        let mut computed = ComputedStyles::new();
        computed.insert(text, ComputedStyle::new());

        // No arranged box for the text node.
        let boxes = LayoutBoxes::new();

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes);
        assert!(text_runs(&list).is_empty());
        assert!(list.is_empty());
    }

    #[test]
    fn long_text_emits_one_run_per_wrapped_line_agreeing_with_box_height() {
        // Requirement (c): three 4-char words in a 50px-wide box wrap to three
        // lines, so the scene emits exactly three `DrawText` runs — one per line —
        // at cumulative tops (0, 16, 32) and a shared x_offset of 0. The box
        // height (48) equals N · line_height, so layout and scene AGREE.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let text = add_text_child(&mut tree, root, "aaaa bbbb cccc");

        let mut computed = ComputedStyles::new();
        computed.insert(text, ComputedStyle::new());

        let mut boxes = LayoutBoxes::new();
        // Box as the layout phase would size it: 50px wide, 3·16 = 48 tall.
        boxes.insert(text, box_at(10.0, 20.0, 50.0, 48.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes);
        let runs = text_runs(&list);
        assert_eq!(runs.len(), 3, "one DrawText per wrapped line");
        // Line count · line_height (16) equals the arranged box height (48).
        assert_eq!(runs.len() as f32 * 16.0, boxes[&text].size().height);

        // Origins stack by line height, all at the box's x (x_offset 0 for Start).
        assert_eq!(runs[0].0, Point::new(10.0, 20.0));
        assert_eq!(runs[1].0, Point::new(10.0, 36.0));
        assert_eq!(runs[2].0, Point::new(10.0, 52.0));

        // Glyph counts per line: "aaaa " (5), "bbbb " (5), "cccc" (4) = 14 total.
        assert_eq!(runs[0].1.len(), 5);
        assert_eq!(runs[1].1.len(), 5);
        assert_eq!(runs[2].1.len(), 4);

        // Every glyph sits on the line baseline (ascent 0.8·16 = 12.8), and each
        // line lays its glyphs out from pen x = 0.
        for (_, run, _) in &runs {
            for (i, glyph) in run.glyphs().iter().enumerate() {
                assert_eq!(glyph.y(), 12.8);
                assert_eq!(glyph.x(), 9.6 * i as f32);
            }
        }
        assert!(list.is_balanced());
    }

    #[test]
    fn empty_text_emits_a_single_empty_run_without_panicking() {
        // Requirement (d): an empty string yields one line with zero glyphs; the
        // scene still emits a single (empty) run at the box origin and never panics.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let text = add_text_child(&mut tree, root, "");

        let mut computed = ComputedStyles::new();
        computed.insert(text, ComputedStyle::new());
        let mut boxes = LayoutBoxes::new();
        boxes.insert(text, box_at(3.0, 4.0, 0.0, 16.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes);
        let runs = text_runs(&list);
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].0, Point::new(3.0, 4.0));
        assert_eq!(runs[0].1.len(), 0);
        assert!(list.is_balanced());
    }

    #[test]
    fn scene_color_from_copies_components() {
        let style_color = StyleColor::rgba(0.25, 0.5, 0.75, 0.9);
        let color = scene_color_from(style_color);
        assert_eq!(color, Color::rgba(0.25, 0.5, 0.75, 0.9));
    }

    /// A stub [`TextShaper`] that emits a fixed glyph id per source char,
    /// independent of the character, so an installed shaper is observable in the
    /// emitted [`GlyphRun`].
    struct FixedIdShaper(u32);

    impl obscur_text::shaping::TextShaper for FixedIdShaper {
        fn shape(
            &self,
            request: &obscur_text::shaping::ShapingRequest,
        ) -> obscur_text::shaping::GlyphRun {
            let glyphs = request
                .text()
                .char_indices()
                .map(|(byte, _)| obscur_text::shaping::ShapedGlyph::new(self.0, byte, 8.0))
                .collect();
            obscur_text::shaping::GlyphRun::new(
                glyphs,
                request.direction(),
                request.font_size(),
                obscur_text::span::TextRange::new(0, request.text().len()),
            )
        }

        fn can_shape(&self, _face: &FontFace) -> bool {
            true
        }
    }

    #[test]
    fn installed_shaper_changes_the_emitted_glyph_run() {
        // A single text node with an arranged box, so a run is emitted.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let text = add_text_child(&mut tree, root, "ab");

        let mut computed = ComputedStyles::new();
        computed.insert(text, ComputedStyle::new());
        let mut boxes = LayoutBoxes::new();
        boxes.insert(text, box_at(0.0, 0.0, 40.0, 16.0));

        // Default mock shaper: glyph ids are the Unicode scalars, one segment on
        // the primary face (font 0).
        let mut coordinator = SceneCoordinator::new();
        let mock_runs = text_runs(&coordinator.compose(&tree, &computed, &boxes));
        assert_eq!(mock_runs.len(), 1);
        assert_eq!(mock_runs[0].1.font(), 0);
        let mock_ids: Vec<u32> = mock_runs[0].1.glyphs().iter().map(|g| g.glyph_id()).collect();
        assert_eq!(mock_ids, vec![u32::from('a'), u32::from('b')]);

        // Install a fixed-id shaper through the runtime seam: the emitted run
        // reflects it (ids all 42), still one primary-face segment.
        coordinator.set_text_shaper(Box::new(FixedIdShaper(42)));
        let stub_runs = text_runs(&coordinator.compose(&tree, &computed, &boxes));
        assert_eq!(stub_runs.len(), 1);
        assert_eq!(stub_runs[0].1.font(), 0);
        let stub_ids: Vec<u32> = stub_runs[0].1.glyphs().iter().map(|g| g.glyph_id()).collect();
        assert_eq!(stub_ids, vec![42, 42]);
        assert_ne!(mock_ids, stub_ids);
    }

    /// A stub itemizer that cuts the fixed two-script string "abВГ" into a Latin
    /// run `[0..2)` and a Cyrillic run `[2..6)` (each Cyrillic char is 2 bytes).
    struct TwoScriptItemizer;

    impl TextItemizer for TwoScriptItemizer {
        fn itemize(
            &self,
            _text: &str,
            _base: TextDirection,
        ) -> Vec<obscur_text::shaping::TextItem> {
            use obscur_text::shaping::{ScriptTag, TextItem};
            use obscur_text::span::TextRange;
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

    #[test]
    fn multi_font_text_emits_one_font_tagged_run_per_segment() {
        use obscur_text::font::FontId;
        use obscur_text::shaping::ScriptTag;

        // A two-script text node "abВГ" in a wide box: one line, two same-font
        // segments (Latin→font 0, Cyrillic→font 1), so compose emits two DrawText
        // runs carrying the two distinct fonts.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let text = add_text_child(&mut tree, root, "abВГ");

        let mut computed = ComputedStyles::new();
        computed.insert(text, ComputedStyle::new());
        let mut boxes = LayoutBoxes::new();
        // Wide box, one line tall (line_height 16 at the default 16px font size).
        boxes.insert(text, box_at(3.0, 4.0, 200.0, 16.0));

        let mut coordinator = SceneCoordinator::new();
        coordinator.set_itemizer(Box::new(TwoScriptItemizer));
        let mut resolver = MockFontResolver::new();
        let face = coordinator.default_face.clone();
        resolver.register(ScriptTag::LATIN, FontId::new(0), face.clone());
        resolver.register(ScriptTag::CYRILLIC, FontId::new(1), face);
        coordinator.set_font_resolver(Box::new(resolver));

        let runs = text_runs(&coordinator.compose(&tree, &computed, &boxes));
        // Two font-tagged runs on a single line: fonts 0 then 1.
        assert_eq!(runs.len(), 2);
        assert_eq!(runs[0].1.font(), 0);
        assert_eq!(runs[1].1.font(), 1);
        // Both segments paint on the SAME line (same y origin) — one wrapped line,
        // so box height (16) == painted line count (1) × line_height (16).
        assert_eq!(runs[0].0.y, runs[1].0.y);
        assert_eq!(boxes[&text].size().height, 16.0);
        // The second segment starts to the right of the first by the first
        // segment's advance (2 glyphs × 9.6), continuing the pen across fonts.
        assert!((runs[1].0.x - (runs[0].0.x + 2.0 * 9.6)).abs() < 1e-3);
        // Each segment carries its own script's glyphs.
        assert_eq!(runs[0].1.text(), "ab");
        assert_eq!(runs[1].1.text(), "ВГ");
        assert!(coordinator.compose(&tree, &computed, &boxes).is_balanced());
    }

    /// A stub [`Hyphenator`] that proposes one fixed byte offset in any word long
    /// enough to hold it.
    struct StubHyphenator {
        offset: usize,
    }

    impl obscur_text::layout::Hyphenator for StubHyphenator {
        fn hyphenate(&self, word: &str) -> Vec<usize> {
            if self.offset < word.len() {
                vec![self.offset]
            } else {
                Vec::new()
            }
        }
    }

    #[test]
    fn installed_hyphenator_breaks_over_wide_word_and_renders_a_hyphen() {
        // "hyphenation" (eleven 9.6px mock glyphs = 105.6) in a 60px box. Without
        // a hyphenator the whole word overflows on one line; with a stub proposing
        // a break before byte 5 ("hyphe|nation"), compose emits two lines and the
        // head line ends with a rendered hyphen glyph (mock id 45 = '-').
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let text = add_text_child(&mut tree, root, "hyphenation");

        let mut computed = ComputedStyles::new();
        computed.insert(text, ComputedStyle::new());
        let mut boxes = LayoutBoxes::new();
        boxes.insert(text, box_at(0.0, 0.0, 60.0, 16.0));

        let mut coordinator = SceneCoordinator::new();
        // Default (no hyphenator): the over-wide word stays on one line.
        assert_eq!(text_runs(&coordinator.compose(&tree, &computed, &boxes)).len(), 1);

        coordinator.set_hyphenator(Box::new(StubHyphenator { offset: 5 }));
        let runs = text_runs(&coordinator.compose(&tree, &computed, &boxes));
        assert_eq!(runs.len(), 2, "the word breaks into a head and a tail line");
        // The head line ("hyphe" + hyphen) ends with the mock hyphen glyph (id 45)
        // and fits the 60px box.
        let head = &runs[0].1;
        assert_eq!(head.glyphs().last().map(PositionedGlyph::glyph_id), Some(45));
        let head_width: f32 = head.glyphs().iter().map(PositionedGlyph::advance).sum();
        assert!(head_width <= 60.0, "head width {head_width} must fit 60");
        // The tail line ("nation") starts on the next line, no hyphen.
        assert!(runs[1].0.y > runs[0].0.y);
        assert!(!runs[1].1.glyphs().iter().any(|g| g.glyph_id() == 45));
        assert!(coordinator.compose(&tree, &computed, &boxes).is_balanced());
    }
}
