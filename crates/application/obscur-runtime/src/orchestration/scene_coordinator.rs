//! Composes a paint-ordered display list from the styled + laid-out tree:
//! [`SceneCoordinator`].

use std::collections::{HashMap, HashSet};
use std::fmt;

use obscur_geometry::{AffineTransform, CornerRadii, Point, Rect, Size};
use obscur_identity::NodeId;
use obscur_scene::clip::Clip;
use obscur_scene::color::Color;
use obscur_scene::command::{DisplayList, DisplayListBuilder, PaintCommand};
use obscur_scene::layer::SceneLayer;
use obscur_scene::paint::{
    Brush, ConicGradient as SceneConicGradient, GradientStop as SceneGradientStop,
    LinearGradient as SceneLinearGradient, RadialGradient as SceneRadialGradient, SceneImage,
};
use obscur_scene::path::Path;
use obscur_scene::text::{GlyphRun, PositionedGlyph};
use obscur_style::computed::{ComputedStyle, ComputedValue};
use obscur_style::property::{
    layout_keyword, BoxShadow, ConicGradient, Length, LinearGradient, PropertyId, PropertyValue,
    RadialGradient, RadialSize, StyleColor, Transform, TransformOrigin,
};
use obscur_text::font::{
    FontAttributes, FontFace, FontFallback, FontFamily, FontMetrics, FontResolver, FontSource,
    GenericFamily, MockFontResolver,
};
use obscur_text::layout::{
    Hyphenator, NullHyphenator, ParagraphLayout, TextAlignment, TextOverflow,
};
use obscur_text::selection::TextSelection;
use obscur_text::shaping::{
    CachingShaper, MockTextShaper, ParagraphShaper, ShapedGlyph, ShapedRun, SingleRunItemizer,
    TextDirection, TextItemizer, TextShaper,
};
use obscur_tree::tree::RetainedTree;

use super::stacking::{effective_stacking_z, PaintUnit};
use super::{
    alignment_for, attributes_for, direction_for, families_for, overflow_for, ComputedStyles,
    EditingView, LayoutBoxes,
};

/// The width, in logical pixels, of the caret rectangle painted at the collapsed
/// selection of the focused editable (a thin vertical bar over the text baseline).
const CARET_WIDTH: f32 = 1.0;

/// The thickness, in logical pixels, of the underline painted beneath an active
/// IME preedit (spec §5.12) — the visual mark distinguishing the uncommitted
/// composing text from committed glyphs.
const PREEDIT_UNDERLINE_THICKNESS: f32 = 1.0;

/// The default selection-highlight color: a semi-transparent blue painted behind
/// the selected span so the glyphs show through it (spec §5.11). A host has no
/// hook to override this yet; it is a documented fixed default.
const SELECTION_COLOR: Color = Color::rgba(0.20, 0.45, 0.90, 0.35);

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
/// children left-to-right); the walk is **cycle-safe** — a visited-set guard
/// means a shared or cyclic child link terminates instead of looping — and
/// **panic-free**: every lookup is fallible and no indexing is performed.
///
/// Paint order is then the scene's `z-index` **stacking** order (spec §5.1): each
/// node's own paint is captured as a unit keyed `(effective z, pre-order index)`
/// and stable-sorted, so a positioned element with a higher `z-index` paints on
/// top of a lower one, and a positioned subtree stacks together (see the
/// [`stacking`](super::stacking) module for the bounded, flat model and its
/// documented simplifications). When no `z-index` is authored every effective z
/// is `0` and the order is exactly pre-order (parent, then children) — the
/// pre-stacking behaviour, byte-for-byte.
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

    /// Returns the caret's pen x (logical pixels) at byte `index` in `content`:
    /// the sum of the shaped glyph advances whose source **cluster** falls before
    /// `index` — i.e. the pen x just left of the glyph the caret sits before (spec
    /// §5.11). A caret at `0` yields `0`; a caret at the text length yields the
    /// whole run advance.
    ///
    /// The text is shaped through the **same** paragraph orchestrator (and default
    /// face) the runs are drawn with, unwrapped, so a caret x on the first line is
    /// exact. Multi-line wrapping and RTL reordering are not modeled here — the
    /// caret is placed by the logical (source-order) advance sum, a documented
    /// single-line, left-to-right approximation.
    // The shaping inputs mirror `shape_lines`/`shape_hyphen`: cohesive positional
    // arguments of a leaf helper, `attributes` by reference to match the
    // `shape_paragraph` convention this pass uses throughout.
    #[allow(clippy::too_many_arguments, clippy::trivially_copy_pass_by_ref)]
    fn caret_offset_x(
        &self,
        content: &str,
        fallback: &FontFallback,
        attributes: &FontAttributes,
        direction: TextDirection,
        font_size: f32,
        index: usize,
    ) -> f32 {
        self.paragraph
            .shape_paragraph(content, fallback, attributes, &self.default_face, font_size, direction)
            .iter()
            .flat_map(|run| run.run().glyphs().iter().copied())
            .filter(|glyph| glyph.cluster() < index)
            .map(|glyph| glyph.x_advance())
            .sum()
    }

    /// Resolves a run-local pen x (`local_x`, logical pixels measured from the
    /// text run's left edge) to the nearest cluster byte offset in `content` —
    /// the **inverse** of [`caret_offset_x`](Self::caret_offset_x), used to place
    /// the caret at a pointer click within the focused editable (spec §5.11).
    ///
    /// `style` is the text node's computed style: its `font-family` /
    /// `font-weight` / `font-style` / `direction` / `font-size` drive the fallback
    /// chain, shaping attributes, base direction, and size — the **same** inputs
    /// the compose pass shapes the drawn run with (and the same
    /// [`caret_offset_x`](Self::caret_offset_x) measures with), so a click at the
    /// rendered caret x of offset `i` resolves back to `i`.
    ///
    /// The text is shaped **once** through the same paragraph orchestrator (and
    /// default face) the runs are drawn with, then the glyph advances are walked:
    /// the boundary whose horizontal midpoint `local_x` falls at or before is
    /// chosen (so a tie resolves to the earlier boundary). A `local_x` at or before
    /// the run start clamps to `0`; a `local_x` past the last glyph clamps to the
    /// content length. Single-line, left-to-right — the same documented
    /// approximation as [`caret_offset_x`](Self::caret_offset_x).
    #[must_use]
    pub(crate) fn caret_offset_at_x(
        &self,
        content: &str,
        style: Option<&ComputedStyle>,
        local_x: f32,
    ) -> usize {
        let fallback = families_for(style);
        let attributes = attributes_for(style);
        let direction = direction_for(style);
        let font_size = font_size_of_style(style);
        let mut pen = 0.0_f32;
        for glyph in self
            .paragraph
            .shape_paragraph(content, &fallback, &attributes, &self.default_face, font_size, direction)
            .iter()
            .flat_map(|run| run.run().glyphs().iter().copied())
        {
            let advance = glyph.x_advance();
            // The boundary just left of this glyph sits at `pen`; cross to the next
            // boundary only once `local_x` is past this glyph's midpoint.
            if local_x <= pen + advance / 2.0 {
                return glyph.cluster();
            }
            pen += advance;
        }
        // Past every glyph: the caret lands at the end of the text.
        content.len()
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
    ///
    /// `editing` is the runtime's active text-editing session, or [`None`] when no
    /// editable is focused. For the editing field's text (its
    /// [`node`](EditingView::node) or that field's `Text` descendant) the compose
    /// draws the session [`text`](EditingView::text) instead of the retained
    /// content — so a just-typed character is visible immediately — and paints a
    /// caret line (collapsed selection) on top of the glyphs plus a selection
    /// highlight (non-empty selection) behind them. With `editing` = [`None`] every
    /// text node paints its retained content with no caret or highlight, exactly as
    /// before (the byte-identical regression path).
    #[must_use]
    pub fn compose(
        &self,
        tree: &RetainedTree,
        computed: &ComputedStyles,
        boxes: &LayoutBoxes,
        editing: Option<EditingView<'_>>,
    ) -> DisplayList {
        // The image-free path: no `background-image: url(...)` resolves to a
        // painted image, so this is byte-identical to before image backgrounds.
        self.compose_with_images(tree, computed, boxes, editing, &HashMap::new())
    }

    /// Composes a paint-ordered [`DisplayList`] like [`compose`](Self::compose),
    /// additionally painting `background-image: url(...)` images that have already
    /// been resolved into the `images` lookup.
    ///
    /// `images` maps a computed `background-image` URL (the raw authored string) to
    /// its decoded [`SceneImage`]. The runtime resolves URLs to images *outside*
    /// this pure coordinator (through a resource loader/decoder) and hands the
    /// ready map in, so composition stays deterministic and IO-free: a node whose
    /// `background-image` is a URL present in `images` emits a
    /// [`DrawImage`](obscur_scene::command::PaintCommand::DrawImage) stretched over
    /// its box (painted OVER `background-color`, UNDER its border and content); a
    /// URL absent from the map (not yet loaded, or failed) paints nothing — exactly
    /// the [`compose`](Self::compose) behaviour. An empty map reproduces
    /// [`compose`](Self::compose) byte-for-byte.
    #[must_use]
    pub fn compose_with_images(
        &self,
        tree: &RetainedTree,
        computed: &ComputedStyles,
        boxes: &LayoutBoxes,
        editing: Option<EditingView<'_>>,
        images: &HashMap<Box<str>, SceneImage>,
    ) -> DisplayList {
        let Some(root) = tree.root() else {
            return DisplayListBuilder::new().build();
        };

        // Each node's effective stacking z (a positioned element's numeric
        // `z-index`, else its nearest z-index ancestor's, root = 0). Computed once
        // up front so the emission walk can key each paint unit by it.
        let zmap = effective_stacking_z(tree, computed);

        // Pre-order walk (parent before children). Children are pushed reversed
        // so the leftmost child is popped first, preserving left-to-right paint
        // order. The visited set makes the walk cycle-safe and panic-free.
        //
        // Instead of emitting directly, each node's own paint (background / text —
        // not its descendants') is captured as a `PaintUnit` tagged with its
        // stacking key `(z, tree_index)`, where `tree_index` is the node's
        // pre-order position. The units are then STABLE-SORTED by `(z, tree_index)`
        // so higher-z positioned elements paint last (on top) while equal-z units
        // keep tree order. When no `z-index` is authored every z is 0 and the sort
        // reproduces the exact pre-order sequence (the regression guard).
        let mut visited: HashSet<NodeId> = HashSet::new();
        // Each stack entry carries the node's parent and its tree depth, so a
        // `border-radius` element can bracket its subtree with a clip whose
        // `PopClip` sorts after the last descendant (see `clip_brackets`).
        let mut stack: Vec<(NodeId, Option<NodeId>, u32)> = vec![(root, None, 0)];
        let mut units: Vec<PaintUnit> = Vec::new();
        let mut next_index: usize = 0;
        // Per-node bookkeeping for the clip brackets: the node's pre-order enter
        // index, its parent (to propagate subtree exit indices), its depth, the
        // pre-order sequence, and — for the nodes that carry a radius — the clip
        // shape to bracket their subtree with.
        let mut enter_index: std::collections::HashMap<NodeId, usize> =
            std::collections::HashMap::new();
        let mut parent_of: std::collections::HashMap<NodeId, Option<NodeId>> =
            std::collections::HashMap::new();
        let mut depth_of: std::collections::HashMap<NodeId, u32> =
            std::collections::HashMap::new();
        let mut preorder: Vec<NodeId> = Vec::new();
        let mut clip_brackets: Vec<(NodeId, Clip)> = Vec::new();
        // A CSS 2D `transform` brackets a node's WHOLE subtree paint with a
        // `PushTransform` … `PopTransform` around (outside) any rounded clip. The
        // affine is resolved about the node's transform origin; the bracket is
        // recorded here and sorted into place in the same second pass as clips.
        let mut transform_brackets: Vec<(NodeId, AffineTransform)> = Vec::new();
        // A CSS `opacity` below `1.0` establishes a stacking context and composites
        // the node's WHOLE subtree as one group: the subtree paint is bracketed by
        // a `PushLayer(SceneLayer::with_opacity(a))` … `PopLayer`, nested *inside*
        // any transform and *outside* the rounded clip (transform → layer → clip).
        // A fully opaque (or absent) opacity records nothing — byte-for-byte
        // identical to before opacity layers existed.
        let mut opacity_brackets: Vec<(NodeId, f32)> = Vec::new();
        while let Some((node, parent, depth)) = stack.pop() {
            if !visited.insert(node) {
                // Already painted (cycle or shared child): do not revisit.
                continue;
            }
            // Assign the node's pre-order document position (the stacking
            // tie-breaker) at pop time, matching the historical emission order.
            let tree_index = next_index;
            next_index += 1;
            enter_index.insert(node, tree_index);
            parent_of.insert(node, parent);
            depth_of.insert(node, depth);
            preorder.push(node);

            // A node's per-corner `border-radius` (all four corners), read off its
            // computed style. Non-zero radii round its background fill and clip its
            // subtree to the rounded border box.
            let radii = corner_radii_of(computed, node);
            let has_radius = corner_radii_present(radii);
            // The rounded clip shape (border box) bracketing this node's subtree,
            // recorded for a second pass once every descendant's exit index is known.
            if has_radius && let Some(layout_box) = boxes.get(&node) {
                clip_brackets.push((node, Clip::rounded_rect(layout_box.bounds, radii)));
            }

            // A non-identity CSS 2D `transform` brackets this node's subtree. The
            // affine is resolved about the node's transform origin (border-box
            // position + `transform-origin`, default center). No transform (or an
            // identity one) records nothing — byte-for-byte identical to before.
            if let Some(layout_box) = boxes.get(&node)
                && let Some(affine) = resolved_transform(computed, node, layout_box.bounds)
            {
                transform_brackets.push((node, affine));
            }

            // A CSS `opacity` below `1.0` brackets this node's subtree with an
            // opacity group layer. It needs no geometry (the layer applies to the
            // group as a whole), so — unlike the transform — it is recorded even
            // for a node with no arranged box. `opacity: 1` or an absent opacity
            // records nothing (byte-for-byte identical to before).
            if let Some(alpha) = opacity_of(computed, node) {
                opacity_brackets.push((node, alpha));
            }

            // This node's own paint accumulates into a private builder so it can be
            // reordered as one unit; only leaf commands are emitted, so it stays
            // balanced.
            let mut node_paint = DisplayListBuilder::new();

            // CSS `box-shadow`: a **drop** shadow paints BEHIND the node's
            // background (emitted first), an **inset** shadow ON TOP of it
            // (emitted after the background fill, below). CSS paints the
            // first-listed shadow on top, so each group is emitted in reverse
            // order. The shadow color falls back to the computed `color`
            // (`currentColor`) when the author omitted it. No `box-shadow` emits
            // nothing — byte-for-byte identical to before.
            let shadows = box_shadows_of(computed, node);
            if let (Some(shadows), Some(layout_box)) = (shadows, boxes.get(&node)) {
                let current = current_color_of(computed, node);
                for shadow in shadows.iter().rev().filter(|s| !s.is_inset()) {
                    emit_box_shadow(&mut node_paint, layout_box.bounds, radii, shadow, current);
                }
            }

            // CSS `filter: drop-shadow(...)`: each drop-shadow paints BEHIND the
            // node like a `box-shadow` DROP shadow (spread 0, never inset), reusing
            // the same `DrawShadow` path — the GPU backend blurs it with a real
            // gaussian, the CPU backend approximates a flat offset rect. In real
            // CSS `filter` applies to the already-rendered element (including its
            // `box-shadow`); we approximate the shadow as the element's border box,
            // and emit the filter drop-shadows AFTER the `box-shadow` drop shadows
            // (so they sit just in front of them) but still behind the element's own
            // background / border / text. Only the drop-shadow filter functions
            // reach here; `blur()` / color-matrix / `backdrop-filter` are dropped at
            // CSS lower time (out of scope on vello 0.9). No filter emits nothing —
            // byte-for-byte identical to before.
            let drop_shadows = filter_drop_shadows_of(computed, node);
            if let (Some(drop_shadows), Some(layout_box)) = (drop_shadows, boxes.get(&node)) {
                let current = current_color_of(computed, node);
                for shadow in drop_shadows.iter().rev() {
                    emit_box_shadow(&mut node_paint, layout_box.bounds, radii, shadow, current);
                }
            }

            // The background fill brush: an authored `background-image` gradient
            // (linear / radial / conic) wins over `background-color` (the gradient
            // paints over the color), otherwise a non-transparent
            // `background-color` fills solid. With neither authored the block emits
            // nothing — byte-for-byte identical to before backgrounds existed.
            if let Some(layout_box) = boxes.get(&node) {
                let brush = if let Some(brush) =
                    background_gradient_brush(computed, node, layout_box.bounds)
                {
                    Some(brush)
                } else {
                    background_color_of(computed, node)
                        // Skip fully transparent backgrounds: they paint nothing.
                        .filter(|color| color.a > 0.0)
                        .map(|color| Brush::solid(scene_color_from(color)))
                };
                if let Some(brush) = brush {
                    if has_radius {
                        // A rounded element fills a rounded-rect outline of its
                        // border box instead of a plain rectangle.
                        node_paint.fill(Path::rounded_rect(layout_box.bounds, radii), brush);
                    } else {
                        node_paint.fill_rect(layout_box.bounds, brush);
                    }
                }

                // An authored `background-image: url(...)` whose image has been
                // resolved paints OVER the `background-color` fill and UNDER the
                // border/content — matching CSS background paint order. The image
                // is stretched over the node's border box (sub-step 1's model;
                // `background-size`/`repeat`/`position` and multiple layers are
                // future work). A URL not present in `images` (unloaded or failed)
                // emits nothing, so the color background shows through — the
                // regression-safe path (byte-identical when `images` is empty).
                if let Some(image) = background_image_url_of(computed, node)
                    .and_then(|url| images.get(url))
                {
                    node_paint.draw_image(image.clone(), layout_box.bounds);
                }
            }

            // Inset shadows paint on top of the background (clipped to the box,
            // approximated by the renderer), under the border and text.
            if let (Some(shadows), Some(layout_box)) = (shadows, boxes.get(&node)) {
                let current = current_color_of(computed, node);
                for shadow in shadows.iter().rev().filter(|s| s.is_inset()) {
                    emit_box_shadow(&mut node_paint, layout_box.bounds, radii, shadow, current);
                }
            }

            // Border edges paint over the background, under the text: one filled
            // rectangle per visible side (style `solid`, width > 0), covering the
            // border box's outer ring. An unauthored border emits nothing. When the
            // element has a radius the edges are clipped to the rounded border box
            // (the surrounding `PushClip`), so a uniform border follows the rounded
            // outline; non-uniform rounded borders are approximate (documented).
            if let Some(layout_box) = boxes.get(&node) {
                for (rect, color) in border_fills(computed, node, layout_box.bounds) {
                    node_paint.fill_rect(rect, Brush::solid(scene_color_from(color)));
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
                if let Some(retained_content) = retained.kind().text()
                    && let Some(layout_box) = boxes.get(&node)
                {
                    // Is this text node the focused editable's text? Then paint the
                    // live SESSION text (not the possibly-lagging retained content)
                    // and, below, the caret + selection highlight. `editing` is
                    // `None` on every non-editing frame, so this whole override
                    // path is inert and the retained content is drawn unchanged.
                    let active = editing.filter(|view| is_editing_text_node(tree, node, view.node));
                    // The displayed text. While composing, the IME preedit is
                    // spliced into the session text at its anchor for DISPLAY only
                    // (the buffer is untouched, spec §5.12); otherwise it is the
                    // session text (editing) or the retained content (idle). The
                    // spliced string is owned here so it outlives the shaping below.
                    let mut spliced = String::new();
                    let content: &str = match active {
                        Some(view) => match view.preedit {
                            Some(preedit) => {
                                let at = preedit.at.min(view.text.len());
                                spliced.push_str(&view.text[..at]);
                                spliced.push_str(preedit.text);
                                spliced.push_str(&view.text[at..]);
                                &spliced
                            }
                            None => view.text,
                        },
                        None => retained_content,
                    };
                    // The rendered caret/selection: while composing, the caret sits
                    // at the preedit's caret within the SPLICED display text; with
                    // no preedit it is the session's own selection (buffer offsets
                    // equal display offsets). `None` on a non-editing text node.
                    let display_selection = active.map(|view| match view.preedit {
                        Some(preedit) => {
                            let at = preedit.at.min(view.text.len());
                            TextSelection::caret(at + preedit.caret)
                        }
                        None => view.selection,
                    });
                    let text_color = text_color_of(computed, node);
                    let brush = Brush::solid(text_color);
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
                    // Selection highlight, painted BEHIND the glyphs (emitted before
                    // the runs so the text shows over it): a rect from x(start) to
                    // x(end) spanning the first line's height, in the documented
                    // semi-transparent selection color. Only for the editing field
                    // and only when the selection actually covers a span (a
                    // collapsed caret paints no highlight). Single-line only — a
                    // multi-line span highlights just the first line's extent
                    // (documented limitation).
                    if let Some(selection) = display_selection
                        && !selection.is_empty()
                    {
                        let line_height = self.paragraph.line_metrics(font_size).line_height();
                        let x0 = self.caret_offset_x(
                            content,
                            &fallback,
                            &attributes,
                            direction,
                            font_size,
                            selection.start(),
                        );
                        let x1 = self.caret_offset_x(
                            content,
                            &fallback,
                            &attributes,
                            direction,
                            font_size,
                            selection.end(),
                        );
                        node_paint.fill_rect(
                            Rect::new(
                                Point::new(origin.x + x0, origin.y),
                                Size::new(x1 - x0, line_height),
                            ),
                            Brush::solid(SELECTION_COLOR),
                        );
                    }
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
                            node_paint.draw_text(
                                Point::new(line_left, line_top),
                                run,
                                brush.clone(),
                            );
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
                            node_paint.draw_text(seg_origin, run, brush.clone());
                            seg_x += segment.run().advance();
                        }
                    }
                    // IME preedit underline (spec §5.12): a thin bar beneath the
                    // composing text, distinguishing the uncommitted preedit from
                    // committed glyphs. The preedit's x-range is bounded with the
                    // SAME caret advance-walk the caret/selection use, over the
                    // spliced display `content`: from x(at) to x(at + preedit len).
                    // Only for the editing field with a live, non-empty preedit.
                    if let Some(preedit) = active.and_then(|view| view.preedit)
                        && !preedit.text.is_empty()
                    {
                        let at = preedit.at.min(content.len());
                        let line_height = self.paragraph.line_metrics(font_size).line_height();
                        let x0 = self.caret_offset_x(
                            content,
                            &fallback,
                            &attributes,
                            direction,
                            font_size,
                            at,
                        );
                        let x1 = self.caret_offset_x(
                            content,
                            &fallback,
                            &attributes,
                            direction,
                            font_size,
                            at + preedit.text.len(),
                        );
                        node_paint.fill_rect(
                            Rect::new(
                                Point::new(
                                    origin.x + x0,
                                    origin.y + line_height - PREEDIT_UNDERLINE_THICKNESS,
                                ),
                                Size::new(x1 - x0, PREEDIT_UNDERLINE_THICKNESS),
                            ),
                            Brush::solid(text_color),
                        );
                    }
                    // Caret, painted ON TOP of the glyphs (emitted after the runs):
                    // a thin vertical bar at the caret's pen x, spanning the first
                    // line's height, in the text color. Only for the editing field
                    // and only when the (display) selection is collapsed (a caret) —
                    // a non-empty selection shows the highlight above instead. While
                    // composing, the display selection is the preedit's caret.
                    if let Some(selection) = display_selection
                        && selection.is_collapsed()
                    {
                        let line_height = self.paragraph.line_metrics(font_size).line_height();
                        let caret_x = self.caret_offset_x(
                            content,
                            &fallback,
                            &attributes,
                            direction,
                            font_size,
                            selection.focus(),
                        );
                        node_paint.fill_rect(
                            Rect::new(
                                Point::new(origin.x + caret_x, origin.y),
                                Size::new(CARET_WIDTH, line_height),
                            ),
                            Brush::solid(text_color),
                        );
                    }
                }

                for &child in retained.children().iter().rev() {
                    if !visited.contains(&child) {
                        stack.push((child, Some(node), depth.saturating_add(1)));
                    }
                }
            }

            // A node with no own paint (no background, no text) contributes no
            // unit — exactly as the historical walk emitted nothing for it.
            let commands = node_paint.build().commands().to_vec();
            if !commands.is_empty() {
                let z = zmap.get(&node).copied().unwrap_or(0);
                units.push(PaintUnit {
                    z,
                    tree_index,
                    order: PaintUnit::PAINT_ORDER,
                    commands,
                });
            }
        }

        // Bracket each rounded element's subtree with a clip. The `PushClip` sorts
        // at the node's enter index (before its own paint); the `PopClip` at the
        // subtree's *exit* index — the largest enter index within the subtree — so
        // it sorts after every descendant. The exit index is the running max of
        // each node's own enter index propagated up to its parent, walking the
        // pre-order sequence in reverse (descendants before ancestors).
        if !clip_brackets.is_empty()
            || !transform_brackets.is_empty()
            || !opacity_brackets.is_empty()
        {
            let mut exit_index = enter_index.clone();
            for &node in preorder.iter().rev() {
                if let (Some(&enter), Some(Some(parent))) =
                    (exit_index.get(&node), parent_of.get(&node))
                {
                    let parent = *parent;
                    if let Some(slot) = exit_index.get_mut(&parent) {
                        *slot = (*slot).max(enter);
                    }
                }
            }
            for (node, clip) in clip_brackets {
                let z = zmap.get(&node).copied().unwrap_or(0);
                let enter = enter_index.get(&node).copied().unwrap_or(0);
                let exit = exit_index.get(&node).copied().unwrap_or(enter);
                let depth = depth_of.get(&node).copied().unwrap_or(0);
                units.push(PaintUnit {
                    z,
                    tree_index: enter,
                    order: PaintUnit::CLIP_PUSH_ORDER,
                    commands: vec![PaintCommand::PushClip(clip)],
                });
                units.push(PaintUnit {
                    z,
                    tree_index: exit,
                    order: PaintUnit::clip_pop_order(depth),
                    commands: vec![PaintCommand::PopClip],
                });
            }
            // Opacity group layers are emitted AFTER the clip brackets and BEFORE
            // the transform brackets. On enter the `PushLayer` sorts between the
            // `PushTransform` (LAYER_PUSH_ORDER > TRANSFORM_PUSH_ORDER) and the
            // `PushClip` (LAYER_PUSH_ORDER < CLIP_PUSH_ORDER), so the layer wraps
            // the subtree inside the transform and outside the rounded clip. On
            // exit the `PopLayer` ties with the clip's/transform's pop at the same
            // `(z, exit, MAX - depth)` key; emitting it after the clip bracket but
            // before the transform bracket makes the stable sort close the clip
            // first (innermost), then the layer, then the transform (outermost).
            for (node, alpha) in opacity_brackets {
                let z = zmap.get(&node).copied().unwrap_or(0);
                let enter = enter_index.get(&node).copied().unwrap_or(0);
                let exit = exit_index.get(&node).copied().unwrap_or(enter);
                let depth = depth_of.get(&node).copied().unwrap_or(0);
                units.push(PaintUnit {
                    z,
                    tree_index: enter,
                    order: PaintUnit::LAYER_PUSH_ORDER,
                    commands: vec![PaintCommand::PushLayer(
                        SceneLayer::new().with_opacity(alpha),
                    )],
                });
                units.push(PaintUnit {
                    z,
                    tree_index: exit,
                    order: PaintUnit::layer_pop_order(depth),
                    commands: vec![PaintCommand::PopLayer],
                });
            }
            // Transform brackets are emitted AFTER the clip brackets so that when
            // a node carries both, the tie at the shared `(z, exit, MAX - depth)`
            // key resolves — via the stable sort — with the clip's `PopClip`
            // before the transform's `PopTransform` (the clip closes inside the
            // transform). The `PushTransform` sorts before the `PushClip`
            // (TRANSFORM_PUSH_ORDER < CLIP_PUSH_ORDER), so the transform is the
            // outermost wrapper of the subtree.
            for (node, affine) in transform_brackets {
                let z = zmap.get(&node).copied().unwrap_or(0);
                let enter = enter_index.get(&node).copied().unwrap_or(0);
                let exit = exit_index.get(&node).copied().unwrap_or(enter);
                let depth = depth_of.get(&node).copied().unwrap_or(0);
                units.push(PaintUnit {
                    z,
                    tree_index: enter,
                    order: PaintUnit::TRANSFORM_PUSH_ORDER,
                    commands: vec![PaintCommand::PushTransform(affine)],
                });
                units.push(PaintUnit {
                    z,
                    tree_index: exit,
                    order: PaintUnit::transform_pop_order(depth),
                    commands: vec![PaintCommand::PopTransform],
                });
            }
        }

        // Stable sort by `(z ascending, tree_index ascending, order ascending)`:
        // higher z paints last (on top); equal-z units keep pre-order (document)
        // order; and `order` phases a subtree's `PushClip` before the node's paint
        // and its `PopClip` after the whole subtree. With no radius every `order`
        // is `PAINT_ORDER`, so the key reduces to `(z, tree_index)`, reproducing the
        // pre-radius pre-order sequence byte-for-byte.
        units.sort_by_key(|unit| (unit.z, unit.tree_index, unit.order));

        let mut ordered: Vec<PaintCommand> = Vec::new();
        for unit in units {
            ordered.extend(unit.commands);
        }
        DisplayList::from_commands(ordered)
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
/// face, plus the segment's synthetic-styling hints
/// ([`synth_oblique`](ShapedRun::synth_oblique) / [`synth_bold`](ShapedRun::synth_bold))
/// so a faux italic/bold run is sheared/thickened at raster time (spec §5.18).
/// For a single-font line (one segment) this reproduces the original single-run
/// behavior exactly, with font `0` and no synthesis.
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
    GlyphRun::new(glyphs, font_size, text)
        .with_font(segment.font().get())
        .with_synthesis(segment.synth_oblique(), segment.synth_bold())
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

/// Builds the fill [`Brush`] for a node's computed `background-image` gradient
/// (linear / radial / conic), resolved over the box `bounds`, or [`None`] when the
/// node has no gradient background — the common (unauthored) case, which falls
/// back to the solid `background-color` path.
///
/// A stop-less gradient (never produced by the CSS lowering, but guarded here)
/// paints nothing. Exactly one of the three gradient kinds is present at a time
/// (`background-image` holds a single computed value).
fn background_gradient_brush(
    computed: &ComputedStyles,
    node: NodeId,
    bounds: Rect,
) -> Option<Brush> {
    let value = computed
        .get(&node)
        .and_then(|style| style.get(PropertyId::BackgroundImage))?;
    if let Some(gradient) = value.as_linear_gradient() {
        return (!gradient.stops().is_empty())
            .then(|| Brush::Linear(linear_gradient_brush(gradient, bounds)));
    }
    if let Some(gradient) = value.as_radial_gradient() {
        return (!gradient.stops().is_empty())
            .then(|| Brush::Radial(radial_gradient_brush(gradient, bounds)));
    }
    if let Some(gradient) = value.as_conic_gradient() {
        return (!gradient.stops().is_empty())
            .then(|| Brush::Conic(conic_gradient_brush(gradient, bounds)));
    }
    None
}

/// Reads a node's computed `background-image: url(...)` raw URL/path string, if
/// present.
///
/// Returns [`None`] when the node has no computed style, no `background-image`, or
/// a non-`url()` `background-image` (a gradient) — none of which paints an image.
/// The URL is the key the runtime resolves against its image cache.
fn background_image_url_of(computed: &ComputedStyles, node: NodeId) -> Option<&str> {
    computed
        .get(&node)
        .and_then(|style| style.get(PropertyId::BackgroundImage))
        .and_then(ComputedValue::as_background_image_url)
}

/// Reads a node's computed `box-shadow` list, if present.
///
/// Returns [`None`] when the node has no computed style, no `box-shadow`
/// property, or a non-shadow value — the common (unauthored) case, which emits
/// no shadows. `box-shadow: none` is an **empty** list (also nothing to emit).
fn box_shadows_of(computed: &ComputedStyles, node: NodeId) -> Option<&[BoxShadow]> {
    computed
        .get(&node)
        .and_then(|style| style.get(PropertyId::BoxShadow))
        .and_then(ComputedValue::as_box_shadows)
}

/// Reads a node's computed `filter` drop-shadow list, if present.
///
/// Returns [`None`] when the node has no computed style, no `filter` property, or
/// a non-filter value — the common (unauthored) case, which emits no shadows.
/// `filter: none` (and a filter with only out-of-scope functions) is an **empty**
/// list (also nothing to emit). Each entry is a drop [`BoxShadow`] (`spread == 0`,
/// not inset) painted behind the node like a `box-shadow` drop shadow.
fn filter_drop_shadows_of(computed: &ComputedStyles, node: NodeId) -> Option<&[BoxShadow]> {
    computed
        .get(&node)
        .and_then(|style| style.get(PropertyId::Filter))
        .and_then(ComputedValue::as_drop_shadows)
}

/// Emits one [`BoxShadow`] as a scene
/// [`DrawShadow`](obscur_scene::command::PaintCommand::DrawShadow) over the border
/// box `bounds` with corner `radii` (spec §5.2).
///
/// The shadow's lengths are resolved to logical pixels (only absolute `px` is
/// honored, mirroring [`border_width_of`]); its color falls back to `current` (the
/// node's computed `color`, i.e. CSS `currentColor`) when the author omitted it.
/// The renderer applies the offset / spread / blur when it rasterizes.
fn emit_box_shadow(
    builder: &mut DisplayListBuilder,
    bounds: Rect,
    radii: CornerRadii,
    shadow: &BoxShadow,
    current: StyleColor,
) {
    let px = |length: Length| length.as_px().map_or(0.0, |px| px.0);
    let color = scene_color_from(shadow.color().unwrap_or(current));
    builder.draw_shadow(
        bounds,
        radii,
        color,
        px(shadow.blur()),
        px(shadow.spread()),
        (px(shadow.offset_x()), px(shadow.offset_y())),
        shadow.is_inset(),
    );
}

/// Resolves a node's CSS 2D `transform` into the absolute-space affine that
/// brackets its subtree paint (spec §5.2), or [`None`] when there is no authored
/// transform (or it is the identity — the byte-identical no-op path).
///
/// The authored functions are already composed into a single matrix `M` by the CSS
/// lowering; here it is wrapped about the element's **transform origin**: the
/// border-box position plus the resolved `transform-origin` (defaulting to the box
/// center). The returned affine is `translate(origin) · M · translate(-origin)`, so
/// a point in the box's untransformed absolute space is mapped about the origin —
/// exactly the CSS `transform` behaviour. A renderer multiplies it into its current
/// transform for the bracketed region.
fn resolved_transform(
    computed: &ComputedStyles,
    node: NodeId,
    bounds: Rect,
) -> Option<AffineTransform> {
    let transform = transform_of(computed, node)?;
    if transform.is_identity() {
        return None;
    }
    let origin = transform_origin_of(computed, node);
    let ox = bounds.origin.x + resolve_origin_length(origin.x(), bounds.size.width);
    let oy = bounds.origin.y + resolve_origin_length(origin.y(), bounds.size.height);
    let matrix = AffineTransform {
        coeffs: transform.matrix(),
    };
    Some(
        AffineTransform::translate(ox, oy)
            .multiply(matrix)
            .multiply(AffineTransform::translate(-ox, -oy)),
    )
}

/// Reads a node's effective CSS `opacity` as a group-layer alpha, or [`None`]
/// when the subtree needs no opacity layer (spec §5.1).
///
/// Reads [`PropertyId::Opacity`] (via [`ComputedStyle::opacity`]) and clamps it to
/// `0.0..=1.0`. Returns `Some(a)` **only** when `a < 1.0`: a fully-opaque (or
/// absent) opacity returns [`None`] so nothing is bracketed and the emitted
/// command stream is byte-for-byte identical to before opacity layers existed. A
/// partially-transparent value brackets the node's whole subtree with a
/// `PushLayer(SceneLayer::with_opacity(a))` … `PopLayer`, compositing the group as
/// one (CSS `opacity` establishes a stacking context and composites the group,
/// not each descendant independently).
fn opacity_of(computed: &ComputedStyles, node: NodeId) -> Option<f32> {
    let alpha = computed.get(&node).and_then(ComputedStyle::opacity)?;
    let alpha = alpha.clamp(0.0, 1.0);
    (alpha < 1.0).then_some(alpha)
}

/// Reads a node's computed `transform`, if present and a resolved affine.
fn transform_of(computed: &ComputedStyles, node: NodeId) -> Option<Transform> {
    computed
        .get(&node)
        .and_then(|style| style.get(PropertyId::Transform))
        .and_then(ComputedValue::as_transform)
}

/// Reads a node's computed `transform-origin`, defaulting to the box center
/// (`50% 50%`) when unauthored (the CSS initial).
fn transform_origin_of(computed: &ComputedStyles, node: NodeId) -> TransformOrigin {
    computed
        .get(&node)
        .and_then(|style| style.get(PropertyId::TransformOrigin))
        .and_then(ComputedValue::as_transform_origin)
        .unwrap_or(TransformOrigin::CENTER)
}

/// Resolves one `transform-origin` component to a logical-pixel offset within a
/// box axis of length `basis`: an absolute `px` directly, a `Percent` as a
/// fraction of `basis`. Any other (context-dependent) length falls back to the
/// center of the axis (a documented approximation).
fn resolve_origin_length(length: Length, basis: f32) -> f32 {
    match length {
        Length::Px(px) => px.0,
        Length::Percent(pct) => pct / 100.0 * basis,
        _ => basis / 2.0,
    }
}

/// Builds a scene [`SceneLinearGradient`] fill brush for a style
/// [`LinearGradient`] over the box `bounds` (spec §5.2).
///
/// The CSS `angle_deg` (0 = to top, increasing clockwise) plus the box size define
/// the gradient *line*: a line through the box center at that angle, with its `0%`
/// / `100%` endpoints at the projections of the box that the line covers. The
/// endpoints are `center ∓ dir · half_len`, where `dir = (sin θ, −cos θ)` (screen
/// space, y down) and `half_len = (|w·sin θ| + |h·cos θ|) / 2`. Each style stop's
/// authored (resolved) `0.0..=1.0` position becomes the scene stop's offset, and
/// its [`StyleColor`] becomes the scene paint [`Color`]; an unpositioned stop
/// (never produced by the CSS lowering, which resolves all positions) falls back
/// to its index-proportional offset. The brush keeps the default
/// [`ExtendMode::Pad`](obscur_scene::paint::ExtendMode::Pad).
fn linear_gradient_brush(gradient: &LinearGradient, bounds: Rect) -> SceneLinearGradient {
    let theta = gradient.angle_deg().to_radians();
    let (sin, cos) = (theta.sin(), theta.cos());
    let w = bounds.size.width;
    let h = bounds.size.height;
    let cx = bounds.origin.x + w / 2.0;
    let cy = bounds.origin.y + h / 2.0;
    // Direction of increasing gradient offset (0% → 100%). y is down, so `to top`
    // (θ = 0) points to smaller y.
    let dir_x = sin;
    let dir_y = -cos;
    let half_len = (w * sin).abs().mul_add(0.5, (h * cos).abs() * 0.5);
    let start = Point::new(cx - dir_x * half_len, cy - dir_y * half_len);
    let end = Point::new(cx + dir_x * half_len, cy + dir_y * half_len);
    SceneLinearGradient::new(start, end, gradient_stops(gradient.stops()))
}

/// Builds a scene [`SceneRadialGradient`] fill brush for a style
/// [`RadialGradient`] over the box `bounds` (spec §5.2).
///
/// The box-relative center (`center_x` / `center_y`) resolves to an absolute point
/// against the box; the ending [`RadialSize`] resolves to a single radius —
/// `farthest-corner` (the default) is the maximum distance from the center to the
/// four box corners, and the other extents / an explicit length follow suit (see
/// [`resolve_radius`]). The scene models a concentric single-radius circle, so an
/// `ellipse` shape is collapsed to that radius (a documented approximation). Each
/// style stop's resolved `0.0..=1.0` position becomes the scene stop's offset; the
/// brush keeps the default [`ExtendMode::Pad`](obscur_scene::paint::ExtendMode::Pad).
fn radial_gradient_brush(gradient: &RadialGradient, bounds: Rect) -> SceneRadialGradient {
    let cx = resolve_center(gradient.center_x(), bounds.origin.x, bounds.size.width);
    let cy = resolve_center(gradient.center_y(), bounds.origin.y, bounds.size.height);
    let radius = resolve_radius(gradient.size(), cx, cy, bounds);
    SceneRadialGradient::new(Point::new(cx, cy), radius, gradient_stops(gradient.stops()))
}

/// Builds a scene [`SceneConicGradient`] fill brush for a style
/// [`ConicGradient`] over the box `bounds` (spec §5.2).
///
/// The box-relative center resolves to an absolute point against the box. CSS's
/// `from` angle is measured from *up* (12 o'clock) increasing clockwise, while the
/// scene sweep's `0` offset lies along `+x` (3 o'clock) advancing clockwise in
/// screen space (y down), so the start angle is `from − 90°` in radians. Each
/// style stop's resolved `0.0..=1.0` position becomes the scene stop's offset; the
/// brush keeps the default [`ExtendMode::Pad`](obscur_scene::paint::ExtendMode::Pad).
fn conic_gradient_brush(gradient: &ConicGradient, bounds: Rect) -> SceneConicGradient {
    let cx = resolve_center(gradient.center_x(), bounds.origin.x, bounds.size.width);
    let cy = resolve_center(gradient.center_y(), bounds.origin.y, bounds.size.height);
    let start_angle = (gradient.from_angle_deg() - 90.0).to_radians();
    SceneConicGradient::new(
        Point::new(cx, cy),
        start_angle,
        gradient_stops(gradient.stops()),
    )
}

/// Maps a style gradient's color stops to scene [`SceneGradientStop`]s: each stop's
/// resolved `0.0..=1.0` position becomes the scene offset and its [`StyleColor`]
/// the scene paint color. An unpositioned stop (never produced by the CSS lowering,
/// which resolves every position) falls back to its index-proportional offset.
fn gradient_stops(stops: &[obscur_style::property::GradientStop]) -> Vec<SceneGradientStop> {
    stops
        .iter()
        .enumerate()
        .map(|(index, stop)| {
            let offset = stop.position().unwrap_or_else(|| {
                let last = stops.len().saturating_sub(1).max(1);
                #[allow(clippy::cast_precision_loss)]
                {
                    index as f32 / last as f32
                }
            });
            SceneGradientStop::new(offset, scene_color_from(stop.color()))
        })
        .collect()
}

/// Resolves a gradient's box-relative center [`Length`] on one axis to an absolute
/// coordinate: an absolute `px` offset from `origin`, a `Percent` as a fraction of
/// the axis `extent` from `origin`. Any other (context-dependent) length falls back
/// to the axis center (a documented approximation).
fn resolve_center(length: Length, origin: f32, extent: f32) -> f32 {
    match length {
        Length::Px(px) => origin + px.0,
        Length::Percent(pct) => origin + pct / 100.0 * extent,
        _ => origin + extent / 2.0,
    }
}

/// Resolves a [`RadialGradient`] ending [`RadialSize`] to a single scene radius,
/// given the gradient's absolute center `(cx, cy)` and the box `bounds`.
///
/// The extent keywords use the center's distances to the box sides (near/far per
/// axis) and corners (hypotenuse of the per-axis distances). An explicit `px`
/// length is used directly; a percentage radius resolves against the box's smaller
/// half-extent (a documented approximation, since the scene models one radius);
/// any other length falls back to `farthest-corner`.
fn resolve_radius(size: RadialSize, cx: f32, cy: f32, bounds: Rect) -> f32 {
    let (left, top, right, bottom) = (bounds.origin.x, bounds.origin.y, bounds.right(), bounds.bottom());
    let dx_near = (cx - left).abs().min((right - cx).abs());
    let dx_far = (cx - left).abs().max((right - cx).abs());
    let dy_near = (cy - top).abs().min((bottom - cy).abs());
    let dy_far = (cy - top).abs().max((bottom - cy).abs());
    match size {
        RadialSize::ClosestSide => dx_near.min(dy_near),
        RadialSize::FarthestSide => dx_far.max(dy_far),
        RadialSize::ClosestCorner => dx_near.hypot(dy_near),
        RadialSize::FarthestCorner => dx_far.hypot(dy_far),
        RadialSize::Length(len) => match len {
            Length::Px(px) => px.0,
            Length::Percent(pct) => {
                pct / 100.0 * bounds.size.width.min(bounds.size.height) / 2.0
            }
            _ => dx_far.hypot(dy_far),
        },
    }
}

/// Computes the filled rectangles painting a node's border, in top/right/bottom/
/// left paint order, each paired with its side color (spec CSS Backgrounds and
/// Borders §4).
///
/// A side contributes a rectangle only when its computed `border-*-style` is
/// `solid` **and** its `border-*-width` is a positive pixel length; every other
/// side (unauthored, `none`, or zero-width) paints nothing, so a node with no
/// authored border yields an empty list (byte-identical to before borders were
/// painted). The four rectangles tile the border box's outer ring: the top and
/// bottom edges span the full width, and the left and right edges fill the
/// remaining height between them, so per-side widths and colors compose without
/// overlap (corner mitres are assigned to the top / bottom edges — a documented
/// simplification). A side's color is its `border-*-color`, falling back to the
/// node's computed `color` (CSS `currentColor`) when the color is unset.
fn border_fills(computed: &ComputedStyles, node: NodeId, bounds: Rect) -> Vec<(Rect, StyleColor)> {
    let top = border_width_of(computed, node, PropertyId::BorderTopWidth);
    let right = border_width_of(computed, node, PropertyId::BorderRightWidth);
    let bottom = border_width_of(computed, node, PropertyId::BorderBottomWidth);
    let left = border_width_of(computed, node, PropertyId::BorderLeftWidth);
    // Fast path: no positive border width ⇒ nothing to paint (the common case).
    if top <= 0.0 && right <= 0.0 && bottom <= 0.0 && left <= 0.0 {
        return Vec::new();
    }
    let current = current_color_of(computed, node);
    let x = bounds.origin.x;
    let y = bounds.origin.y;
    let w = bounds.size.width;
    let h = bounds.size.height;
    // The height of the left / right edges: the border box height minus the top
    // and bottom edges (clamped non-negative), so the four rects never overlap.
    let mid_h = (h - top - bottom).max(0.0);
    let mut out = Vec::new();
    if top > 0.0 && border_is_solid(computed, node, PropertyId::BorderTopStyle) {
        let color = border_color_of(computed, node, PropertyId::BorderTopColor, current);
        out.push((Rect::new(Point::new(x, y), Size::new(w, top)), color));
    }
    if right > 0.0 && border_is_solid(computed, node, PropertyId::BorderRightStyle) {
        let color = border_color_of(computed, node, PropertyId::BorderRightColor, current);
        out.push((
            Rect::new(Point::new(x + w - right, y + top), Size::new(right, mid_h)),
            color,
        ));
    }
    if bottom > 0.0 && border_is_solid(computed, node, PropertyId::BorderBottomStyle) {
        let color = border_color_of(computed, node, PropertyId::BorderBottomColor, current);
        out.push((
            Rect::new(Point::new(x, y + h - bottom), Size::new(w, bottom)),
            color,
        ));
    }
    if left > 0.0 && border_is_solid(computed, node, PropertyId::BorderLeftStyle) {
        let color = border_color_of(computed, node, PropertyId::BorderLeftColor, current);
        out.push((
            Rect::new(Point::new(x, y + top), Size::new(left, mid_h)),
            color,
        ));
    }
    out
}

/// Reads a node's computed `border-*-width` (`id`) in logical pixels, defaulting
/// to `0.0` when absent or not an absolute pixel length.
///
/// Only absolute `px` is honored here (mirroring [`font_size_of`]); a relative or
/// percentage border width contributes no painted edge even though the layout
/// phase resolves it into the box model — a documented limitation.
fn border_width_of(computed: &ComputedStyles, node: NodeId, id: PropertyId) -> f32 {
    computed
        .get(&node)
        .and_then(|style| style.get(id))
        .and_then(ComputedValue::as_length)
        .and_then(Length::as_px)
        .map_or(0.0, |px| px.0)
}

/// Reads a node's four computed `border-*-radius` corner lengths into a
/// [`CornerRadii`], in top-left / top-right / bottom-right / bottom-left order.
///
/// Only absolute `px` is honored (mirroring [`border_width_of`]); an absent or
/// non-pixel corner radius contributes `0` (a square corner), so a node with no
/// authored radius yields all-zero radii — the pre-radius behaviour.
fn corner_radii_of(computed: &ComputedStyles, node: NodeId) -> CornerRadii {
    CornerRadii {
        top_left: border_width_of(computed, node, PropertyId::BorderTopLeftRadius),
        top_right: border_width_of(computed, node, PropertyId::BorderTopRightRadius),
        bottom_right: border_width_of(computed, node, PropertyId::BorderBottomRightRadius),
        bottom_left: border_width_of(computed, node, PropertyId::BorderBottomLeftRadius),
    }
}

/// Whether any of `radii`'s four corners is a positive radius, i.e. the element
/// actually rounds (and therefore needs a rounded fill + subtree clip). All-zero
/// radii (the common, unauthored case) return `false` for a byte-identical path.
fn corner_radii_present(radii: CornerRadii) -> bool {
    radii.top_left > 0.0
        || radii.top_right > 0.0
        || radii.bottom_right > 0.0
        || radii.bottom_left > 0.0
}

/// Whether a node's computed `border-*-style` (`id`) is the reserved `solid`
/// handle. An absent style (the deferred `none` initial) or any other value is
/// treated as not solid, so no edge is painted.
fn border_is_solid(computed: &ComputedStyles, node: NodeId, id: PropertyId) -> bool {
    matches!(
        computed
            .get(&node)
            .and_then(|style| style.get(id))
            .map(ComputedValue::value),
        Some(&PropertyValue::Keyword(sym)) if sym == layout_keyword::BORDER_STYLE_SOLID
    )
}

/// Reads a node's computed `border-*-color` (`id`), falling back to `current`
/// (the node's computed `color`, i.e. CSS `currentColor`) when the color is unset.
fn border_color_of(
    computed: &ComputedStyles,
    node: NodeId,
    id: PropertyId,
    current: StyleColor,
) -> StyleColor {
    computed
        .get(&node)
        .and_then(|style| style.get(id))
        .and_then(ComputedValue::as_color)
        .unwrap_or(current)
}

/// Reads a node's computed `color` as a [`StyleColor`] for the `currentColor`
/// border fallback, defaulting to [`StyleColor::BLACK`] (the CSS `color` initial)
/// when absent.
fn current_color_of(computed: &ComputedStyles, node: NodeId) -> StyleColor {
    computed
        .get(&node)
        .and_then(|style| style.get(PropertyId::Color))
        .and_then(ComputedValue::as_color)
        .unwrap_or(StyleColor::BLACK)
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
    font_size_of_style(computed.get(&node))
}

/// Reads a computed `font-size` in logical pixels from a single node's style,
/// defaulting to [`DEFAULT_FONT_SIZE`] when absent or not an absolute pixel
/// length — the by-style form [`font_size_of`] delegates to, shared with the
/// click-caret hit-test so it sizes text identically to the drawn run.
fn font_size_of_style(style: Option<&ComputedStyle>) -> f32 {
    style
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

/// Whether `node` is the text painted for the editing field `field`: either the
/// field node itself (if it directly carries the text) or any descendant of it,
/// found by walking `node`'s logical-parent chain toward the root.
///
/// The editing session is bound to the editable *field* (the ancestor that binds
/// `EVENT_INPUT`), but the glyphs live on a `Text` descendant; this maps that
/// descendant back to its field so the compose phase can override its content and
/// paint the caret/selection there. The walk is bounded by the tree size, so a
/// malformed (cyclic) parent chain terminates deterministically instead of
/// looping.
fn is_editing_text_node(tree: &RetainedTree, node: NodeId, field: NodeId) -> bool {
    let mut current = node;
    // At most `len` hops reach the root from any node in a well-formed tree; the
    // inclusive bound tolerates a self-parent without looping.
    for _ in 0..=tree.len() {
        if current == field {
            return true;
        }
        match tree.get(current).and_then(|n| n.logical_parent().parent()) {
            Some(parent) => current = parent,
            None => return false,
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_geometry::{Point, Rect, Size};
    use obscur_identity::ComponentId;
    use obscur_layout::model::LayoutBox;
    use obscur_scene::command::PaintCommand;
    use obscur_style::computed::ComputedStyle;
    use obscur_text::selection::TextSelection;
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

    /// Collects the `DrawPath` (fill) brushes of a display list, in paint order.
    fn fill_brushes(list: &DisplayList) -> Vec<Brush> {
        list.commands()
            .iter()
            .filter_map(|c| match c {
                PaintCommand::DrawPath { paint, .. } => Some(paint.brush().clone()),
                _ => None,
            })
            .collect()
    }

    /// A `ComputedStyle` carrying a single `background-image` linear gradient.
    fn with_background_gradient(gradient: LinearGradient) -> ComputedStyle {
        let mut style = ComputedStyle::new();
        style.set(
            PropertyId::BackgroundImage,
            ComputedValue::from_concrete(PropertyValue::LinearGradient(gradient))
                .expect("a gradient is a concrete computed value"),
        );
        style
    }

    #[test]
    fn linear_gradient_background_emits_gradient_brush() {
        use obscur_style::property::GradientStop as StyleStop;

        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);

        // `linear-gradient(90deg, red, blue)` = to right.
        let gradient = LinearGradient::new(
            90.0,
            vec![
                StyleStop::new(StyleColor::rgba(1.0, 0.0, 0.0, 1.0), 0.0),
                StyleStop::new(StyleColor::rgba(0.0, 0.0, 1.0, 1.0), 1.0),
            ],
        );
        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(a, with_background_gradient(gradient));

        let mut boxes = LayoutBoxes::new();
        boxes.insert(a, box_at(0.0, 0.0, 100.0, 50.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        let brushes = fill_brushes(&list);
        assert_eq!(brushes.len(), 1, "one gradient fill");
        let Brush::Linear(g) = &brushes[0] else {
            panic!("expected a linear gradient brush, got {:?}", brushes[0]);
        };
        // 90deg (to right): the line runs left→right through the box center.
        let near = |a: Point, b: Point| (a.x - b.x).abs() < 1e-3 && (a.y - b.y).abs() < 1e-3;
        assert!(near(g.start(), Point::new(0.0, 25.0)), "start {:?}", g.start());
        assert!(near(g.end(), Point::new(100.0, 25.0)), "end {:?}", g.end());
        // The authored stops, in order, with their colors and offsets.
        assert_eq!(g.stops().len(), 2);
        assert_eq!(g.stops()[0].offset(), 0.0);
        assert_eq!(g.stops()[0].color(), Color::rgba(1.0, 0.0, 0.0, 1.0));
        assert_eq!(g.stops()[1].offset(), 1.0);
        assert_eq!(g.stops()[1].color(), Color::rgba(0.0, 0.0, 1.0, 1.0));
        assert!(list.is_balanced());
    }

    /// A solid 2×2 opaque RGBA image, for the `background-image: url(...)` tests.
    fn canned_image() -> SceneImage {
        use obscur_scene::color::AlphaMode;
        let pixels: std::sync::Arc<[u8]> = std::sync::Arc::from(vec![0u8; 2 * 2 * 4]);
        SceneImage::new(2, 2, pixels, AlphaMode::Straight).expect("2x2 rgba is valid")
    }

    /// Collects the `DrawImage` destination rects of a display list, in paint order.
    fn image_dsts(list: &DisplayList) -> Vec<Rect> {
        list.commands()
            .iter()
            .filter_map(|c| match c {
                PaintCommand::DrawImage { dst, .. } => Some(*dst),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn url_background_image_emits_draw_image_over_the_box() {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);

        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(
            a,
            with_background_image(PropertyValue::BackgroundImageUrl("x.png".into())),
        );

        let mut boxes = LayoutBoxes::new();
        let node_box = box_at(5.0, 7.0, 100.0, 50.0);
        boxes.insert(a, node_box);

        // Without the URL in the map: no DrawImage (regression-safe).
        let empty = HashMap::new();
        let bare = SceneCoordinator::new().compose_with_images(&tree, &computed, &boxes, None, &empty);
        assert!(image_dsts(&bare).is_empty(), "no image resolved → no DrawImage");
        assert!(bare.is_balanced());

        // With the URL resolved: one DrawImage stretched over the node's box.
        let mut images = HashMap::new();
        images.insert(Box::<str>::from("x.png"), canned_image());
        let list =
            SceneCoordinator::new().compose_with_images(&tree, &computed, &boxes, None, &images);
        let dsts = image_dsts(&list);
        assert_eq!(dsts.len(), 1, "one resolved image paints one DrawImage");
        assert_eq!(dsts[0], node_box.bounds, "image stretches over the node box");
        assert!(list.is_balanced());
    }

    #[test]
    fn url_background_image_paints_over_background_color() {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);

        // A node with BOTH a background-color and a background-image url.
        let mut style = with_background_image(PropertyValue::BackgroundImageUrl("x.png".into()));
        style.set(
            PropertyId::BackgroundColor,
            ComputedValue::color(StyleColor::rgba(1.0, 0.0, 0.0, 1.0)),
        );
        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(a, style);

        let mut boxes = LayoutBoxes::new();
        boxes.insert(a, box_at(0.0, 0.0, 20.0, 20.0));

        let mut images = HashMap::new();
        images.insert(Box::<str>::from("x.png"), canned_image());
        let list =
            SceneCoordinator::new().compose_with_images(&tree, &computed, &boxes, None, &images);

        // Paint order: the background-color fill (DrawPath) comes BEFORE the image
        // (DrawImage) so the image sits on top of the color.
        let kinds: Vec<&str> = list
            .commands()
            .iter()
            .filter_map(|c| match c {
                PaintCommand::DrawPath { .. } => Some("fill"),
                PaintCommand::DrawImage { .. } => Some("image"),
                _ => None,
            })
            .collect();
        assert_eq!(kinds, vec!["fill", "image"], "color fills, image paints over it");
    }

    /// A `ComputedStyle` carrying a single `background-image` computed value.
    fn with_background_image(value: PropertyValue) -> ComputedStyle {
        let mut style = ComputedStyle::new();
        style.set(
            PropertyId::BackgroundImage,
            ComputedValue::from_concrete(value).expect("a gradient is a concrete computed value"),
        );
        style
    }

    #[test]
    fn radial_gradient_background_emits_radial_brush() {
        use obscur_style::property::{
            GradientStop as StyleStop, RadialGradient, RadialShape, RadialSize,
        };

        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);

        // `radial-gradient(circle farthest-corner at center, red, blue)`.
        let gradient = RadialGradient::new(
            RadialShape::Circle,
            RadialSize::FarthestCorner,
            Length::Percent(50.0),
            Length::Percent(50.0),
            vec![
                StyleStop::new(StyleColor::rgba(1.0, 0.0, 0.0, 1.0), 0.0),
                StyleStop::new(StyleColor::rgba(0.0, 0.0, 1.0, 1.0), 1.0),
            ],
        );
        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(
            a,
            with_background_image(PropertyValue::RadialGradient(gradient)),
        );

        let mut boxes = LayoutBoxes::new();
        boxes.insert(a, box_at(0.0, 0.0, 100.0, 50.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        let brushes = fill_brushes(&list);
        assert_eq!(brushes.len(), 1, "one radial gradient fill");
        let Brush::Radial(g) = &brushes[0] else {
            panic!("expected a radial gradient brush, got {:?}", brushes[0]);
        };
        // Center at the box center; farthest-corner radius = distance to a corner.
        let near = |a: f32, b: f32| (a - b).abs() < 1e-3;
        assert!(near(g.center().x, 50.0), "center x {:?}", g.center());
        assert!(near(g.center().y, 25.0), "center y {:?}", g.center());
        assert!(near(g.radius(), 50.0_f32.hypot(25.0)), "radius {}", g.radius());
        assert_eq!(g.stops().len(), 2);
        assert_eq!(g.stops()[0].color(), Color::rgba(1.0, 0.0, 0.0, 1.0));
        assert_eq!(g.stops()[1].offset(), 1.0);
        assert!(list.is_balanced());
    }

    #[test]
    fn conic_gradient_background_emits_conic_brush() {
        use obscur_style::property::{ConicGradient, GradientStop as StyleStop};

        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);

        // `conic-gradient(from 90deg at center, red, blue)`.
        let gradient = ConicGradient::new(
            90.0,
            Length::Percent(50.0),
            Length::Percent(50.0),
            vec![
                StyleStop::new(StyleColor::rgba(1.0, 0.0, 0.0, 1.0), 0.0),
                StyleStop::new(StyleColor::rgba(0.0, 0.0, 1.0, 1.0), 1.0),
            ],
        );
        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(
            a,
            with_background_image(PropertyValue::ConicGradient(gradient)),
        );

        let mut boxes = LayoutBoxes::new();
        boxes.insert(a, box_at(0.0, 0.0, 100.0, 50.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        let brushes = fill_brushes(&list);
        assert_eq!(brushes.len(), 1, "one conic gradient fill");
        let Brush::Conic(g) = &brushes[0] else {
            panic!("expected a conic gradient brush, got {:?}", brushes[0]);
        };
        let near = |a: f32, b: f32| (a - b).abs() < 1e-3;
        assert!(near(g.center().x, 50.0), "center x {:?}", g.center());
        assert!(near(g.center().y, 25.0), "center y {:?}", g.center());
        // from 90deg → start_angle = (90 − 90)° = 0 rad (scene +x reference).
        assert!(near(g.start_angle(), 0.0), "start_angle {}", g.start_angle());
        assert_eq!(g.stops().len(), 2);
        assert_eq!(g.stops()[0].color(), Color::rgba(1.0, 0.0, 0.0, 1.0));
        assert!(list.is_balanced());
    }

    /// A `ComputedStyle` carrying a `background-color` and a resolved `transform`.
    fn with_background_and_transform(bg: StyleColor, matrix: [f32; 6]) -> ComputedStyle {
        let mut style = ComputedStyle::new();
        style.set(PropertyId::BackgroundColor, ComputedValue::color(bg));
        style.set(
            PropertyId::Transform,
            ComputedValue::from_concrete(PropertyValue::Transform(Transform::new(matrix)))
                .expect("a transform is a concrete computed value"),
        );
        style
    }

    #[test]
    fn transformed_node_brackets_subtree_with_push_pop_transform() {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);

        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        // `transform: translate(10px, 0)` over a background box.
        computed.insert(
            a,
            with_background_and_transform(
                StyleColor::rgba(0.0, 1.0, 0.0, 1.0),
                [1.0, 0.0, 0.0, 1.0, 10.0, 0.0],
            ),
        );

        let mut boxes = LayoutBoxes::new();
        boxes.insert(a, box_at(0.0, 0.0, 40.0, 20.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        assert!(list.is_balanced(), "the transform bracket is well-formed");
        let commands = list.commands();

        // The transform opens first (outermost) and closes last, bracketing the
        // node's whole subtree paint.
        let Some(PaintCommand::PushTransform(affine)) = commands.first() else {
            panic!("expected a leading PushTransform, got {:?}", commands.first());
        };
        // A pure translation commutes with the origin wrapping, so the affine is
        // exactly translate(10, 0) regardless of the (default center) origin.
        assert_eq!(affine.coeffs, [1.0, 0.0, 0.0, 1.0, 10.0, 0.0]);
        assert!(
            matches!(commands.last(), Some(PaintCommand::PopTransform)),
            "the subtree closes with a PopTransform"
        );
        // The background fill still paints inside the bracket.
        assert_eq!(fill_count(&list), 1);
    }

    #[test]
    fn untransformed_node_emits_no_transform_bracket() {
        // Regression: a plain background (no transform) emits no PushTransform.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);

        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(a, with_background(StyleColor::rgba(0.0, 1.0, 0.0, 1.0)));

        let mut boxes = LayoutBoxes::new();
        boxes.insert(a, box_at(0.0, 0.0, 10.0, 10.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        assert!(
            !list
                .commands()
                .iter()
                .any(|c| matches!(
                    c,
                    PaintCommand::PushTransform(_) | PaintCommand::PopTransform
                )),
            "no transform authored ⇒ no transform bracket"
        );
    }

    #[test]
    fn identity_transform_emits_no_bracket() {
        // An explicit identity transform (e.g. `transform: none`) brackets nothing.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);

        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(
            a,
            with_background_and_transform(
                StyleColor::rgba(0.0, 1.0, 0.0, 1.0),
                [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            ),
        );

        let mut boxes = LayoutBoxes::new();
        boxes.insert(a, box_at(0.0, 0.0, 10.0, 10.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        assert!(
            !list
                .commands()
                .iter()
                .any(|c| matches!(c, PaintCommand::PushTransform(_))),
            "an identity transform is a no-op bracket"
        );
    }

    #[test]
    fn solid_background_still_emits_solid_brush() {
        // Regression: a plain `background-color` (no image) still emits a solid
        // brush, byte-identical to before background images existed.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);

        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(a, with_background(StyleColor::rgba(0.0, 1.0, 0.0, 1.0)));

        let mut boxes = LayoutBoxes::new();
        boxes.insert(a, box_at(0.0, 0.0, 10.0, 10.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        let brushes = fill_brushes(&list);
        assert_eq!(brushes, vec![Brush::solid(Color::rgba(0.0, 1.0, 0.0, 1.0))]);
    }

    /// Collects the `DrawPath` (fill) rectangles of a display list, in paint order.
    fn fill_rects(list: &DisplayList) -> Vec<Rect> {
        list.commands()
            .iter()
            .filter_map(|c| match c {
                PaintCommand::DrawPath { path, .. } => path.bounds(),
                _ => None,
            })
            .collect()
    }

    /// Collects the `DrawShadow` commands in a display list, in paint order.
    fn shadow_commands(list: &DisplayList) -> Vec<&PaintCommand> {
        list.commands()
            .iter()
            .filter(|c| matches!(c, PaintCommand::DrawShadow { .. }))
            .collect()
    }

    /// A `ComputedStyle` carrying a `background-color` and a `box-shadow` list.
    fn with_background_and_shadow(bg: StyleColor, shadows: Vec<BoxShadow>) -> ComputedStyle {
        let mut style = ComputedStyle::new();
        style.set(PropertyId::BackgroundColor, ComputedValue::color(bg));
        style.set(
            PropertyId::BoxShadow,
            ComputedValue::from_concrete(PropertyValue::BoxShadow(shadows.into_boxed_slice()))
                .expect("a box-shadow list is a concrete computed value"),
        );
        style
    }

    #[test]
    fn box_shadow_emits_draw_shadow_behind_background() {
        // A node with `box-shadow: 4px 4px 8px black` over its background.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);

        let shadow = BoxShadow::new(
            Length::px(4.0),
            Length::px(4.0),
            Length::px(8.0),
            Length::px(0.0),
            Some(StyleColor::BLACK),
            false,
        );
        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(
            a,
            with_background_and_shadow(StyleColor::rgba(0.0, 1.0, 0.0, 1.0), vec![shadow]),
        );

        let mut boxes = LayoutBoxes::new();
        boxes.insert(a, box_at(0.0, 0.0, 20.0, 10.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);

        // Exactly one DrawShadow, carrying the authored params.
        let shadows = shadow_commands(&list);
        assert_eq!(shadows.len(), 1);
        let PaintCommand::DrawShadow {
            rect,
            color,
            blur,
            spread,
            offset,
            inset,
            ..
        } = shadows[0]
        else {
            panic!("expected a DrawShadow");
        };
        assert_eq!(*rect, Rect::new(Point::new(0.0, 0.0), Size::new(20.0, 10.0)));
        assert_eq!(*color, obscur_scene::color::Color::BLACK);
        assert_eq!(*blur, 8.0);
        assert_eq!(*spread, 0.0);
        assert_eq!(*offset, (4.0, 4.0));
        assert!(!*inset);

        // The shadow paints BEFORE (behind) the background fill.
        let shadow_pos = list
            .commands()
            .iter()
            .position(|c| matches!(c, PaintCommand::DrawShadow { .. }))
            .expect("shadow present");
        let fill_pos = list
            .commands()
            .iter()
            .position(|c| matches!(c, PaintCommand::DrawPath { .. }))
            .expect("background present");
        assert!(shadow_pos < fill_pos, "drop shadow paints behind the background");
        assert!(list.is_balanced());
    }

    #[test]
    fn no_box_shadow_emits_no_draw_shadow() {
        // Regression: a node with a background but no `box-shadow` emits no shadow.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);

        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(a, with_background(StyleColor::rgba(0.0, 1.0, 0.0, 1.0)));

        let mut boxes = LayoutBoxes::new();
        boxes.insert(a, box_at(0.0, 0.0, 10.0, 10.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        assert!(shadow_commands(&list).is_empty());
    }

    /// A `ComputedStyle` carrying a `background-color` and a `filter` drop-shadow
    /// list.
    fn with_background_and_filter(bg: StyleColor, shadows: Vec<BoxShadow>) -> ComputedStyle {
        let mut style = ComputedStyle::new();
        style.set(PropertyId::BackgroundColor, ComputedValue::color(bg));
        style.set(
            PropertyId::Filter,
            ComputedValue::from_concrete(PropertyValue::Filter(shadows.into_boxed_slice()))
                .expect("a filter drop-shadow list is a concrete computed value"),
        );
        style
    }

    #[test]
    fn filter_drop_shadow_emits_draw_shadow_behind_background() {
        // A node with `filter: drop-shadow(4px 4px 8px black)` over its background.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);

        let shadow = BoxShadow::new(
            Length::px(4.0),
            Length::px(4.0),
            Length::px(8.0),
            Length::px(0.0),
            Some(StyleColor::BLACK),
            false,
        );
        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(
            a,
            with_background_and_filter(StyleColor::rgba(0.0, 1.0, 0.0, 1.0), vec![shadow]),
        );

        let mut boxes = LayoutBoxes::new();
        boxes.insert(a, box_at(0.0, 0.0, 20.0, 10.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);

        // Exactly one DrawShadow, carrying the authored params (spread 0, not inset).
        let shadows = shadow_commands(&list);
        assert_eq!(shadows.len(), 1);
        let PaintCommand::DrawShadow {
            rect,
            color,
            blur,
            spread,
            offset,
            inset,
            ..
        } = shadows[0]
        else {
            panic!("expected a DrawShadow");
        };
        assert_eq!(*rect, Rect::new(Point::new(0.0, 0.0), Size::new(20.0, 10.0)));
        assert_eq!(*color, obscur_scene::color::Color::BLACK);
        assert_eq!(*blur, 8.0);
        assert_eq!(*spread, 0.0);
        assert_eq!(*offset, (4.0, 4.0));
        assert!(!*inset, "a drop-shadow is never inset");

        // The filter drop-shadow paints BEFORE (behind) the background fill.
        let shadow_pos = list
            .commands()
            .iter()
            .position(|c| matches!(c, PaintCommand::DrawShadow { .. }))
            .expect("shadow present");
        let fill_pos = list
            .commands()
            .iter()
            .position(|c| matches!(c, PaintCommand::DrawPath { .. }))
            .expect("background present");
        assert!(shadow_pos < fill_pos, "filter drop-shadow paints behind the background");
        assert!(list.is_balanced());
    }

    #[test]
    fn no_filter_emits_no_draw_shadow() {
        // Regression: a node with a background but no `filter` emits no shadow.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);

        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(a, with_background(StyleColor::rgba(0.0, 1.0, 0.0, 1.0)));

        let mut boxes = LayoutBoxes::new();
        boxes.insert(a, box_at(0.0, 0.0, 10.0, 10.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        assert!(shadow_commands(&list).is_empty());
    }

    #[test]
    fn box_shadow_and_filter_drop_shadow_both_emit() {
        // A node carrying BOTH a `box-shadow` and a `filter: drop-shadow(...)`
        // emits two DrawShadows, both behind the background.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);

        let box_shadow = BoxShadow::new(
            Length::px(2.0),
            Length::px(2.0),
            Length::px(4.0),
            Length::px(0.0),
            Some(StyleColor::BLACK),
            false,
        );
        let filter_shadow = BoxShadow::new(
            Length::px(6.0),
            Length::px(6.0),
            Length::px(12.0),
            Length::px(0.0),
            Some(StyleColor::rgba(1.0, 0.0, 0.0, 1.0)),
            false,
        );
        let mut style =
            with_background_and_shadow(StyleColor::rgba(0.0, 1.0, 0.0, 1.0), vec![box_shadow]);
        style.set(
            PropertyId::Filter,
            ComputedValue::from_concrete(PropertyValue::Filter(
                vec![filter_shadow].into_boxed_slice(),
            ))
            .expect("a filter drop-shadow list is a concrete computed value"),
        );

        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(a, style);

        let mut boxes = LayoutBoxes::new();
        boxes.insert(a, box_at(0.0, 0.0, 20.0, 10.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);

        let shadows = shadow_commands(&list);
        assert_eq!(shadows.len(), 2, "box-shadow and filter drop-shadow both paint");
        // Both shadows paint before the background fill.
        let fill_pos = list
            .commands()
            .iter()
            .position(|c| matches!(c, PaintCommand::DrawPath { .. }))
            .expect("background present");
        for (i, c) in list.commands().iter().enumerate() {
            if matches!(c, PaintCommand::DrawShadow { .. }) {
                assert!(i < fill_pos, "shadows paint behind the background");
            }
        }
        assert!(list.is_balanced());
    }

    /// A `ComputedStyle` with a `background-color`, `position: absolute`, and an
    /// optional numeric `z-index` — the shape the stacking sort keys on.
    fn positioned_bg(color: StyleColor, z: Option<f32>) -> ComputedStyle {
        let mut style = ComputedStyle::new();
        style.set(PropertyId::BackgroundColor, ComputedValue::color(color));
        style.set(
            PropertyId::Position,
            ComputedValue::keyword(obscur_style::property::layout_keyword::POSITION_ABSOLUTE),
        );
        if let Some(z) = z {
            style.set(PropertyId::ZIndex, ComputedValue::number(z));
        }
        style
    }

    #[test]
    fn z_index_reorders_positioned_fills_above_tree_order() {
        // root -> [a (z=2), b (no z ⇒ 0), c (z=-1)] in tree order a, b, c. Despite
        // c being LAST in the tree, its z=-1 paints it FIRST; a's z=2 paints it
        // LAST (on top), overriding pure tree order.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);
        let b = add_child(&mut tree, root, 11);
        let c = add_child(&mut tree, root, 12);

        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new()); // root: no background.
        computed.insert(a, positioned_bg(StyleColor::rgba(1.0, 0.0, 0.0, 1.0), Some(2.0)));
        computed.insert(b, positioned_bg(StyleColor::rgba(0.0, 1.0, 0.0, 1.0), None));
        computed.insert(c, positioned_bg(StyleColor::rgba(0.0, 0.0, 1.0, 1.0), Some(-1.0)));

        let mut boxes = LayoutBoxes::new();
        boxes.insert(a, box_at(0.0, 0.0, 10.0, 10.0));
        boxes.insert(b, box_at(1.0, 1.0, 10.0, 10.0));
        boxes.insert(c, box_at(2.0, 2.0, 10.0, 10.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        let rects = fill_rects(&list);
        assert_eq!(rects.len(), 3);
        // c (z=-1) first, b (z=0) middle, a (z=2) last (topmost).
        assert_eq!(rects[0], Rect::new(Point::new(2.0, 2.0), Size::new(10.0, 10.0)));
        assert_eq!(rects[1], Rect::new(Point::new(1.0, 1.0), Size::new(10.0, 10.0)));
        assert_eq!(rects[2], Rect::new(Point::new(0.0, 0.0), Size::new(10.0, 10.0)));
        assert!(list.is_balanced());
    }

    #[test]
    fn no_z_index_keeps_tree_paint_order() {
        // Regression guard: with no authored `z-index`, three positioned fills
        // paint in exact pre-order (a, b, c) — byte-identical to pre-stacking.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);
        let b = add_child(&mut tree, root, 11);
        let c = add_child(&mut tree, root, 12);

        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(a, positioned_bg(StyleColor::rgba(1.0, 0.0, 0.0, 1.0), None));
        computed.insert(b, positioned_bg(StyleColor::rgba(0.0, 1.0, 0.0, 1.0), None));
        computed.insert(c, positioned_bg(StyleColor::rgba(0.0, 0.0, 1.0, 1.0), None));

        let mut boxes = LayoutBoxes::new();
        boxes.insert(a, box_at(0.0, 0.0, 10.0, 10.0));
        boxes.insert(b, box_at(1.0, 1.0, 10.0, 10.0));
        boxes.insert(c, box_at(2.0, 2.0, 10.0, 10.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        let rects = fill_rects(&list);
        assert_eq!(
            rects,
            vec![
                Rect::new(Point::new(0.0, 0.0), Size::new(10.0, 10.0)),
                Rect::new(Point::new(1.0, 1.0), Size::new(10.0, 10.0)),
                Rect::new(Point::new(2.0, 2.0), Size::new(10.0, 10.0)),
            ]
        );
    }

    #[test]
    fn positioned_subtree_stacks_together_above_lower_siblings() {
        // root -> [ sib (z=0, bg), panel (z=1, positioned, bg) -> text ]. The panel
        // and its text share stacking level 1 (the text inherits the panel's z), so
        // both paint above the z=0 sibling, and the text paints above the panel's
        // OWN background (equal z, later in tree).
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let sib = add_child(&mut tree, root, 20);
        let panel = add_child(&mut tree, root, 21);
        let text = add_text_child(&mut tree, panel, "hi");

        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(sib, positioned_bg(StyleColor::rgba(0.0, 1.0, 0.0, 1.0), None));
        computed.insert(panel, positioned_bg(StyleColor::rgba(1.0, 0.0, 0.0, 1.0), Some(1.0)));
        computed.insert(text, ComputedStyle::new());

        let mut boxes = LayoutBoxes::new();
        boxes.insert(sib, box_at(0.0, 0.0, 20.0, 20.0));
        boxes.insert(panel, box_at(5.0, 5.0, 20.0, 20.0));
        boxes.insert(text, box_at(6.0, 6.0, 20.0, 16.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);

        // Command kinds in paint order: sib fill, panel fill, then the text run.
        let kinds: Vec<&str> = list
            .commands()
            .iter()
            .map(|c| match c {
                PaintCommand::DrawPath { .. } => "fill",
                PaintCommand::DrawText { .. } => "text",
                _ => "other",
            })
            .collect();
        assert_eq!(kinds, vec!["fill", "fill", "text"]);

        // The two fills: the z=0 sibling first, the z=1 panel above it.
        let rects = fill_rects(&list);
        assert_eq!(rects[0], Rect::new(Point::new(0.0, 0.0), Size::new(20.0, 20.0)));
        assert_eq!(rects[1], Rect::new(Point::new(5.0, 5.0), Size::new(20.0, 20.0)));
        // The text run sits at its box origin, painted last (on top of everything).
        let runs = text_runs(&list);
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].0, Point::new(6.0, 6.0));
        assert!(list.is_balanced());
    }

    #[test]
    fn empty_tree_composes_empty_balanced_list() {
        let coordinator = SceneCoordinator::new();
        let tree = RetainedTree::new();
        let computed = ComputedStyles::new();
        let boxes = LayoutBoxes::new();

        let list = coordinator.compose(&tree, &computed, &boxes, None);
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
        let list = coordinator.compose(&tree, &computed, &boxes, None);

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
        let list = coordinator.compose(&tree, &computed, &boxes, None);

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
        let list = coordinator.compose(&tree, &computed, &boxes, None);
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

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);

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

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
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

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
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
        let first = text_runs(&coordinator.compose(&tree, &computed, &boxes, None));
        let second = text_runs(&coordinator.compose(&tree, &computed, &boxes, None));
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

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);

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

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);

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

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
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

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
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

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        let runs = text_runs(&list);
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].0, Point::new(3.0, 4.0));
        assert_eq!(runs[0].1.len(), 0);
        assert!(list.is_balanced());
    }

    /// Builds a `ComputedStyle` with a uniform solid border of `width` px and the
    /// given per-side `color`.
    fn with_border(width: f32, color: StyleColor) -> ComputedStyle {
        let mut style = ComputedStyle::new();
        for id in [
            PropertyId::BorderTopWidth,
            PropertyId::BorderRightWidth,
            PropertyId::BorderBottomWidth,
            PropertyId::BorderLeftWidth,
        ] {
            style.set(id, ComputedValue::length(Length::px(width)));
        }
        for id in [
            PropertyId::BorderTopStyle,
            PropertyId::BorderRightStyle,
            PropertyId::BorderBottomStyle,
            PropertyId::BorderLeftStyle,
        ] {
            style.set(id, ComputedValue::keyword(layout_keyword::BORDER_STYLE_SOLID));
        }
        for id in [
            PropertyId::BorderTopColor,
            PropertyId::BorderRightColor,
            PropertyId::BorderBottomColor,
            PropertyId::BorderLeftColor,
        ] {
            style.set(id, ComputedValue::color(color));
        }
        style
    }

    /// Collects the `DrawPath` fills of a display list as `(bounds, solid color)`
    /// pairs, in paint order.
    fn path_fills(list: &DisplayList) -> Vec<(Rect, Color)> {
        list.commands()
            .iter()
            .filter_map(|c| match c {
                PaintCommand::DrawPath { path, paint } => {
                    Some((path.bounds()?, paint.brush().as_solid()?.color()))
                }
                _ => None,
            })
            .collect()
    }

    #[test]
    fn border_paints_four_side_rects_covering_the_edges() {
        // A 100×50 box at (10,20) with `border: 2px solid red` paints four edge
        // rectangles in top/right/bottom/left order, each red, tiling the ring.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);

        let red = StyleColor::rgba(1.0, 0.0, 0.0, 1.0);
        let mut computed = ComputedStyles::new();
        computed.insert(root, with_border(2.0, red));

        let mut boxes = LayoutBoxes::new();
        boxes.insert(root, box_at(10.0, 20.0, 100.0, 50.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        let fills = path_fills(&list);
        assert_eq!(fills.len(), 4, "four side rects, no background");
        // Every side is red.
        for (_, color) in &fills {
            assert_eq!(*color, Color::rgba(1.0, 0.0, 0.0, 1.0));
        }
        // top, right, bottom, left — covering the 2px outer ring.
        assert_eq!(fills[0].0, Rect::new(Point::new(10.0, 20.0), Size::new(100.0, 2.0)));
        assert_eq!(fills[1].0, Rect::new(Point::new(108.0, 22.0), Size::new(2.0, 46.0)));
        assert_eq!(fills[2].0, Rect::new(Point::new(10.0, 68.0), Size::new(100.0, 2.0)));
        assert_eq!(fills[3].0, Rect::new(Point::new(10.0, 22.0), Size::new(2.0, 46.0)));
        assert!(list.is_balanced());
    }

    #[test]
    fn border_paints_over_background_under_no_op() {
        // Background + border on the same node: the background fills first, then
        // the four border edges paint over it (all within one node's paint unit).
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);

        let mut style = with_border(1.0, StyleColor::rgba(0.0, 0.0, 1.0, 1.0));
        style.set(
            PropertyId::BackgroundColor,
            ComputedValue::color(StyleColor::rgba(0.0, 1.0, 0.0, 1.0)),
        );
        let mut computed = ComputedStyles::new();
        computed.insert(root, style);

        let mut boxes = LayoutBoxes::new();
        boxes.insert(root, box_at(0.0, 0.0, 20.0, 20.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        let fills = path_fills(&list);
        assert_eq!(fills.len(), 5, "one background + four border edges");
        // The background is emitted first, covering the whole border box.
        assert_eq!(fills[0].0, Rect::new(Point::new(0.0, 0.0), Size::new(20.0, 20.0)));
        assert_eq!(fills[0].1, Color::rgba(0.0, 1.0, 0.0, 1.0));
        // The remaining four are the blue border edges.
        for (_, color) in &fills[1..] {
            assert_eq!(*color, Color::rgba(0.0, 0.0, 1.0, 1.0));
        }
    }

    #[test]
    fn no_authored_border_emits_no_border_paint() {
        // Regression guard: a node with a background but no border paints exactly
        // one fill (the background) — byte-identical to before borders existed.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);

        let mut computed = ComputedStyles::new();
        computed.insert(root, with_background(StyleColor::rgba(1.0, 0.0, 0.0, 1.0)));

        let mut boxes = LayoutBoxes::new();
        boxes.insert(root, box_at(0.0, 0.0, 30.0, 30.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        assert_eq!(fill_count(&list), 1, "only the background, no border rects");
    }

    #[test]
    fn border_color_falls_back_to_current_color_when_unset() {
        // Width + solid style but NO border color: each edge paints in the node's
        // computed `color` (CSS `currentColor`), here blue.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);

        let blue = StyleColor::rgba(0.0, 0.0, 1.0, 1.0);
        let mut style = ComputedStyle::new();
        style.set(PropertyId::Color, ComputedValue::color(blue));
        for id in [
            PropertyId::BorderTopWidth,
            PropertyId::BorderRightWidth,
            PropertyId::BorderBottomWidth,
            PropertyId::BorderLeftWidth,
        ] {
            style.set(id, ComputedValue::length(Length::px(2.0)));
        }
        for id in [
            PropertyId::BorderTopStyle,
            PropertyId::BorderRightStyle,
            PropertyId::BorderBottomStyle,
            PropertyId::BorderLeftStyle,
        ] {
            style.set(id, ComputedValue::keyword(layout_keyword::BORDER_STYLE_SOLID));
        }
        let mut computed = ComputedStyles::new();
        computed.insert(root, style);

        let mut boxes = LayoutBoxes::new();
        boxes.insert(root, box_at(0.0, 0.0, 40.0, 40.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        let fills = path_fills(&list);
        assert_eq!(fills.len(), 4);
        for (_, color) in &fills {
            assert_eq!(*color, Color::rgba(0.0, 0.0, 1.0, 1.0), "border uses currentColor");
        }
    }

    #[test]
    fn border_style_none_paints_nothing_even_with_width() {
        // A width with no solid style (style unset ⇒ `none`) paints no edge, even
        // though the width still insets the box model in the layout phase.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);

        let mut style = ComputedStyle::new();
        style.set(PropertyId::BorderTopWidth, ComputedValue::length(Length::px(4.0)));
        // No border-top-style ⇒ treated as `none`.
        let mut computed = ComputedStyles::new();
        computed.insert(root, style);

        let mut boxes = LayoutBoxes::new();
        boxes.insert(root, box_at(0.0, 0.0, 10.0, 10.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        assert_eq!(fill_count(&list), 0, "no solid style ⇒ no border painted");
    }

    /// A `ComputedStyle` with a `background-color` and a uniform `border-radius`.
    fn with_background_and_radius(color: StyleColor, radius: f32) -> ComputedStyle {
        let mut style = with_background(color);
        for id in [
            PropertyId::BorderTopLeftRadius,
            PropertyId::BorderTopRightRadius,
            PropertyId::BorderBottomRightRadius,
            PropertyId::BorderBottomLeftRadius,
        ] {
            style.set(id, ComputedValue::length(Length::px(radius)));
        }
        style
    }

    /// Whether a `DrawPath` path has any curved (rounded) corner segment.
    fn path_has_curve(path: &obscur_scene::path::Path) -> bool {
        path.segments()
            .iter()
            .any(|s| matches!(s, obscur_scene::path::PathSegment::CubicTo { .. }))
    }

    #[test]
    fn border_radius_emits_rounded_fill_and_clip_bracketing_children() {
        // root (bg + radius 8) -> child (bg, no radius). The rounded root fills a
        // curved path and its subtree is bracketed by a clip; the child paints
        // inside the clip.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let child = add_child(&mut tree, root, 10);

        let mut computed = ComputedStyles::new();
        computed.insert(
            root,
            with_background_and_radius(StyleColor::rgba(1.0, 0.0, 0.0, 1.0), 8.0),
        );
        computed.insert(child, with_background(StyleColor::rgba(0.0, 1.0, 0.0, 1.0)));

        let mut boxes = LayoutBoxes::new();
        boxes.insert(root, box_at(0.0, 0.0, 100.0, 100.0));
        boxes.insert(child, box_at(10.0, 10.0, 20.0, 20.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        let cmds = list.commands();
        // PushClip, rounded root fill, child fill, PopClip — the clip brackets the
        // whole subtree.
        assert!(matches!(cmds[0], PaintCommand::PushClip(_)), "clip opens first");
        assert!(matches!(cmds.last(), Some(PaintCommand::PopClip)), "clip closes last");
        assert!(list.is_balanced(), "the clip bracket is balanced");

        // The rounded root background is a curved path (not a plain rectangle),
        // and its bounds still equal the border box.
        match &cmds[1] {
            PaintCommand::DrawPath { path, .. } => {
                assert!(path_has_curve(path), "rounded background uses a curved path");
                assert_eq!(
                    path.bounds(),
                    Some(Rect::new(Point::new(0.0, 0.0), Size::new(100.0, 100.0)))
                );
            }
            other => panic!("expected the rounded background fill, got {other:?}"),
        }
        // The child's plain-rect fill sits between the push and the pop.
        match &cmds[2] {
            PaintCommand::DrawPath { path, .. } => {
                assert!(!path_has_curve(path), "child has no radius ⇒ plain rect");
            }
            other => panic!("expected the child fill, got {other:?}"),
        }
    }

    #[test]
    fn no_radius_emits_plain_fill_and_no_clip() {
        // Regression guard: a background with no authored radius paints exactly one
        // plain-rectangle fill and emits NO clip — byte-identical to pre-radius.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);

        let mut computed = ComputedStyles::new();
        computed.insert(root, with_background(StyleColor::rgba(1.0, 0.0, 0.0, 1.0)));

        let mut boxes = LayoutBoxes::new();
        boxes.insert(root, box_at(0.0, 0.0, 30.0, 30.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        let cmds = list.commands();
        assert_eq!(cmds.len(), 1, "just the background, no clip");
        match &cmds[0] {
            PaintCommand::DrawPath { path, .. } => {
                assert!(!path_has_curve(path), "no radius ⇒ plain rectangle");
                assert_eq!(
                    path,
                    &obscur_scene::path::Path::rect(Rect::new(
                        Point::new(0.0, 0.0),
                        Size::new(30.0, 30.0)
                    )),
                    "byte-identical plain-rect fill"
                );
            }
            other => panic!("expected a plain fill, got {other:?}"),
        }
        assert!(
            !cmds.iter().any(|c| matches!(
                c,
                PaintCommand::PushClip(_) | PaintCommand::PopClip
            )),
            "no clip commands with no radius"
        );
    }

    /// A `ComputedStyle` carrying a `background-color` and an `opacity`.
    fn with_background_and_opacity(color: StyleColor, opacity: f32) -> ComputedStyle {
        let mut style = with_background(color);
        style.set(PropertyId::Opacity, ComputedValue::number(opacity));
        style
    }

    #[test]
    fn opacity_below_one_brackets_subtree_with_push_pop_layer() {
        // root (bg + opacity 0.5) -> child (bg). The subtree is bracketed by a
        // PushLayer(opacity=0.5) … PopLayer wrapping BOTH fills as one group.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let child = add_child(&mut tree, root, 10);

        let mut computed = ComputedStyles::new();
        computed.insert(
            root,
            with_background_and_opacity(StyleColor::rgba(1.0, 0.0, 0.0, 1.0), 0.5),
        );
        computed.insert(child, with_background(StyleColor::rgba(0.0, 1.0, 0.0, 1.0)));

        let mut boxes = LayoutBoxes::new();
        boxes.insert(root, box_at(0.0, 0.0, 100.0, 100.0));
        boxes.insert(child, box_at(10.0, 10.0, 20.0, 20.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        let cmds = list.commands();
        assert!(list.is_balanced(), "the layer bracket is balanced");
        // PushLayer opens first, PopLayer closes last, with both fills between.
        match &cmds[0] {
            PaintCommand::PushLayer(layer) => {
                assert!(
                    (layer.opacity() - 0.5).abs() < 1e-6,
                    "layer opacity is 0.5, got {}",
                    layer.opacity()
                );
            }
            other => panic!("expected a leading PushLayer, got {other:?}"),
        }
        assert!(
            matches!(cmds.last(), Some(PaintCommand::PopLayer)),
            "layer closes last"
        );
        // Exactly one layer pair wraps the whole group (not one per descendant).
        assert_eq!(
            cmds.iter()
                .filter(|c| matches!(c, PaintCommand::PushLayer(_)))
                .count(),
            1,
            "one group layer for the subtree"
        );
        assert_eq!(fill_count(&list), 2, "both fills are inside the one group");
    }

    #[test]
    fn opacity_one_and_absent_emit_no_layer() {
        // Regression: opacity 1.0 (and an absent opacity) emit NO PushLayer, so the
        // command stream is byte-identical to before opacity layers existed.
        for opacity in [None, Some(1.0_f32)] {
            let mut tree = RetainedTree::new();
            let root = tree.alloc(NodeKind::Root);
            tree.set_root(root);

            let mut computed = ComputedStyles::new();
            let style = match opacity {
                Some(a) => with_background_and_opacity(StyleColor::rgba(1.0, 0.0, 0.0, 1.0), a),
                None => with_background(StyleColor::rgba(1.0, 0.0, 0.0, 1.0)),
            };
            computed.insert(root, style);

            let mut boxes = LayoutBoxes::new();
            boxes.insert(root, box_at(0.0, 0.0, 30.0, 30.0));

            let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
            let cmds = list.commands();
            assert!(
                !cmds.iter().any(|c| matches!(
                    c,
                    PaintCommand::PushLayer(_) | PaintCommand::PopLayer
                )),
                "opacity {opacity:?} emits no layer commands"
            );
            assert_eq!(cmds.len(), 1, "just the one background fill for {opacity:?}");
        }
    }

    #[test]
    fn opacity_nests_transform_outer_layer_middle_clip_inner() {
        // A single node carrying transform + border-radius + opacity 0.5. The three
        // wrappers must nest transform (outer) → opacity layer → rounded clip
        // (inner) on enter, and unwind clip → layer → transform on exit.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);

        let mut style = with_background_and_radius(StyleColor::rgba(1.0, 0.0, 0.0, 1.0), 8.0);
        // translate(10, 0) — a non-identity transform.
        style.set(
            PropertyId::Transform,
            ComputedValue::from_concrete(PropertyValue::Transform(Transform::new([
                1.0, 0.0, 0.0, 1.0, 10.0, 0.0,
            ])))
            .expect("a transform is a concrete computed value"),
        );
        style.set(PropertyId::Opacity, ComputedValue::number(0.5));

        let mut computed = ComputedStyles::new();
        computed.insert(root, style);

        let mut boxes = LayoutBoxes::new();
        boxes.insert(root, box_at(0.0, 0.0, 100.0, 100.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        let cmds = list.commands();
        assert!(list.is_balanced(), "the three brackets are balanced");

        // Enter order: transform (outer), layer (middle), clip (inner).
        assert!(
            matches!(cmds[0], PaintCommand::PushTransform(_)),
            "transform opens outermost, got {:?}",
            cmds[0]
        );
        assert!(
            matches!(cmds[1], PaintCommand::PushLayer(_)),
            "layer opens inside the transform, got {:?}",
            cmds[1]
        );
        assert!(
            matches!(cmds[2], PaintCommand::PushClip(_)),
            "clip opens innermost, got {:?}",
            cmds[2]
        );
        // Exit order (reversed): clip closes first, then layer, then transform.
        let n = cmds.len();
        assert!(
            matches!(cmds[n - 3], PaintCommand::PopClip),
            "clip closes first, got {:?}",
            cmds[n - 3]
        );
        assert!(
            matches!(cmds[n - 2], PaintCommand::PopLayer),
            "layer closes after the clip, got {:?}",
            cmds[n - 2]
        );
        assert!(
            matches!(cmds[n - 1], PaintCommand::PopTransform),
            "transform closes outermost, got {:?}",
            cmds[n - 1]
        );
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
        let mock_runs = text_runs(&coordinator.compose(&tree, &computed, &boxes, None));
        assert_eq!(mock_runs.len(), 1);
        assert_eq!(mock_runs[0].1.font(), 0);
        let mock_ids: Vec<u32> = mock_runs[0].1.glyphs().iter().map(|g| g.glyph_id()).collect();
        assert_eq!(mock_ids, vec![u32::from('a'), u32::from('b')]);

        // Install a fixed-id shaper through the runtime seam: the emitted run
        // reflects it (ids all 42), still one primary-face segment.
        coordinator.set_text_shaper(Box::new(FixedIdShaper(42)));
        let stub_runs = text_runs(&coordinator.compose(&tree, &computed, &boxes, None));
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

        let runs = text_runs(&coordinator.compose(&tree, &computed, &boxes, None));
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
        assert!(coordinator.compose(&tree, &computed, &boxes, None).is_balanced());
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
        assert_eq!(text_runs(&coordinator.compose(&tree, &computed, &boxes, None)).len(), 1);

        coordinator.set_hyphenator(Box::new(StubHyphenator { offset: 5 }));
        let runs = text_runs(&coordinator.compose(&tree, &computed, &boxes, None));
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
        assert!(coordinator.compose(&tree, &computed, &boxes, None).is_balanced());
    }

    // ---- Text caret + selection highlight (this step) ------------------------

    /// Collects the fill rects (path bounds + brush) of a display list, in paint
    /// order — the caret and selection highlight are the fills over a text node
    /// that carries no background.
    fn caret_fills(list: &DisplayList) -> Vec<(Rect, Brush)> {
        list.commands()
            .iter()
            .filter_map(|c| match c {
                PaintCommand::DrawPath { path, paint } => {
                    path.bounds().map(|r| (r, paint.brush().clone()))
                }
                _ => None,
            })
            .collect()
    }

    /// A tree with a single text node carrying `content` under the root, its box
    /// at the origin (a wide 200x16 line so the text never wraps), and a red text
    /// color so the caret brush is identifiable — the caret/selection fixture.
    fn editing_fixture(content: &str) -> (RetainedTree, ComputedStyles, LayoutBoxes, NodeId) {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let text = add_text_child(&mut tree, root, content);
        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(text, with_color(StyleColor::rgba(1.0, 0.0, 0.0, 1.0)));
        let mut boxes = LayoutBoxes::new();
        boxes.insert(text, box_at(0.0, 0.0, 200.0, 16.0));
        (tree, computed, boxes, text)
    }

    #[test]
    fn collapsed_caret_paints_a_thin_bar_at_the_pen_x() {
        // (a) A collapsed caret over "hi" paints one thin vertical bar at the sum
        // of the advances before the caret index: 0 at index 0, one advance at 1,
        // two at 2 (the end). Advance is the mock's fixed 16·0.6 = 9.6.
        let advance = 16.0 * MockTextShaper::ADVANCE_RATIO;
        for (index, expected_x) in [(0_usize, 0.0_f32), (1, advance), (2, advance * 2.0)] {
            let (tree, computed, boxes, text) = editing_fixture("hi");
            let view = EditingView::new(text, "hi", TextSelection::caret(index));
            let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, Some(view));

            let fills = caret_fills(&list);
            assert_eq!(fills.len(), 1, "a collapsed caret paints exactly one fill");
            let (rect, brush) = &fills[0];
            assert!(
                (rect.origin.x - expected_x).abs() < 1e-4,
                "caret x at index {index}: {} != {expected_x}",
                rect.origin.x
            );
            assert_eq!(rect.size.width, CARET_WIDTH, "the caret is a thin bar");
            assert_eq!(rect.size.height, 16.0, "the caret spans the line height");
            assert_eq!(
                *brush,
                Brush::solid(Color::rgba(1.0, 0.0, 0.0, 1.0)),
                "the caret is painted in the text color"
            );
            // The caret paints ON TOP: it is the last command (after the DrawText).
            assert!(
                matches!(list.commands().last(), Some(PaintCommand::DrawPath { .. })),
                "the caret is the last (topmost) command"
            );
            assert_eq!(text_runs(&list).len(), 1, "the glyphs still render");
        }
    }

    #[test]
    fn nonempty_selection_paints_a_highlight_behind_the_text() {
        // (b) A non-empty selection [0,1) paints one highlight rect spanning x
        // 0..advance behind the glyphs (no caret).
        let advance = 16.0 * MockTextShaper::ADVANCE_RATIO;
        let (tree, computed, boxes, text) = editing_fixture("hi");
        let view = EditingView::new(text, "hi", TextSelection::new(0, 1));
        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, Some(view));

        let fills = caret_fills(&list);
        assert_eq!(fills.len(), 1, "a non-empty selection paints one highlight, no caret");
        let (rect, brush) = &fills[0];
        assert!((rect.origin.x - 0.0).abs() < 1e-4, "highlight starts at x(start)=0");
        assert!(
            (rect.size.width - advance).abs() < 1e-4,
            "highlight spans x(start)..x(end) = one advance"
        );
        assert_eq!(rect.size.height, 16.0, "highlight spans the line height");
        assert_eq!(*brush, Brush::solid(SELECTION_COLOR), "documented selection color");

        // The highlight paints BEHIND the text: its fill is emitted before the run.
        let fill_idx = list
            .commands()
            .iter()
            .position(|c| matches!(c, PaintCommand::DrawPath { .. }))
            .expect("a highlight fill");
        let text_idx = list
            .commands()
            .iter()
            .position(|c| matches!(c, PaintCommand::DrawText { .. }))
            .expect("a text run");
        assert!(fill_idx < text_idx, "the highlight is emitted before the glyphs");
    }

    #[test]
    fn active_preedit_splices_display_text_and_underlines_it() {
        // (IME §5.12) A composition over "ab" (caret at end, at=2) with preedit
        // "ni": compose SPLICES the preedit for display ("abni"), underlines the
        // preedit's x-range with a thin (height 1) horizontal fill spanning two
        // advances at x∈[2·adv, 4·adv), and places the caret at the preedit caret
        // (end of "ni" → display offset 4). The buffer text ("ab") is untouched.
        let advance = 16.0 * MockTextShaper::ADVANCE_RATIO;
        let (tree, computed, boxes, text) = editing_fixture("ab");
        let view = EditingView::new(text, "ab", TextSelection::caret(2))
            .with_preedit(crate::orchestration::PreeditView::new("ni", 2, 2));
        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, Some(view));

        // The displayed text is the spliced "abni", not the buffer "ab".
        let drawn: String = text_runs(&list)
            .iter()
            .map(|(_, run, _)| run.text().to_owned())
            .collect();
        assert_eq!(drawn, "abni", "the preedit is spliced into the display text");

        let fills = caret_fills(&list);
        // Exactly two fills: the preedit underline and the caret.
        let underline = fills
            .iter()
            .find(|(r, _)| (r.size.height - PREEDIT_UNDERLINE_THICKNESS).abs() < 1e-4 && r.size.width > 1.0)
            .expect("a thin preedit underline");
        assert!(
            (underline.0.origin.x - advance * 2.0).abs() < 1e-4,
            "underline starts at the preedit left edge (2·advance)"
        );
        assert!(
            (underline.0.size.width - advance * 2.0).abs() < 1e-4,
            "underline spans the two preedit glyphs"
        );
        let caret = fills
            .iter()
            .find(|(r, _)| (r.size.width - CARET_WIDTH).abs() < 1e-4)
            .expect("a caret bar");
        assert!(
            (caret.0.origin.x - advance * 4.0).abs() < 1e-4,
            "caret sits at the preedit caret (display offset 4)"
        );
        assert!(list.is_balanced(), "the display list stays balanced");
    }

    #[test]
    fn session_text_overrides_retained_content() {
        // (c) The retained node carries "old" but the session says "new"; compose
        // draws the session text so a just-typed character is visible immediately.
        let (tree, computed, boxes, text) = editing_fixture("old");
        let view = EditingView::new(text, "new", TextSelection::caret(3));
        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, Some(view));

        let drawn: String = text_runs(&list)
            .iter()
            .map(|(_, run, _)| run.text().to_owned())
            .collect();
        assert_eq!(drawn, "new", "the scene shows the session text, not the retained content");
    }

    #[test]
    fn no_editing_session_shows_retained_content_with_no_caret() {
        // (d) Regression: with `editing: None` the retained content renders and no
        // caret/highlight fill is emitted — byte-identical to the pre-caret path.
        let (tree, computed, boxes, _text) = editing_fixture("old");
        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);

        assert!(caret_fills(&list).is_empty(), "no caret or highlight without a session");
        let drawn: String = text_runs(&list)
            .iter()
            .map(|(_, run, _)| run.text().to_owned())
            .collect();
        assert_eq!(drawn, "old", "the retained content renders unchanged");
    }

    #[test]
    fn caret_only_paints_on_the_editing_field_not_other_text() {
        // A second, non-editing text node emits no caret even while a session is
        // active for a different node.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let editing_text = add_text_child(&mut tree, root, "hi");
        let other_text = add_text_child(&mut tree, root, "yo");

        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(editing_text, with_color(StyleColor::rgba(1.0, 0.0, 0.0, 1.0)));
        computed.insert(other_text, with_color(StyleColor::rgba(0.0, 1.0, 0.0, 1.0)));
        let mut boxes = LayoutBoxes::new();
        boxes.insert(editing_text, box_at(0.0, 0.0, 200.0, 16.0));
        boxes.insert(other_text, box_at(0.0, 20.0, 200.0, 16.0));

        let view = EditingView::new(editing_text, "hi", TextSelection::caret(1));
        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, Some(view));
        // Exactly one caret, over the editing node only.
        assert_eq!(caret_fills(&list).len(), 1, "only the editing field gets a caret");
        assert_eq!(text_runs(&list).len(), 2, "both text nodes still render");
    }
}
