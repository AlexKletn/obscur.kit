//! Composes a paint-ordered display list from the styled + laid-out tree:
//! [`SceneCoordinator`].

use std::collections::{HashMap, HashSet};
use std::fmt;

use obscur_geometry::{AffineTransform, CornerRadii, Point, Rect, Size};
use obscur_identity::NodeId;
use obscur_scene::clip::Clip;
use obscur_scene::color::Color;
use obscur_scene::command::{DisplayList, DisplayListBuilder, PaintCommand};
use obscur_scene::layer::{LayerEffect, SceneLayer};
use obscur_scene::paint::{
    Brush, ConicGradient as SceneConicGradient, ExtendMode, GradientStop as SceneGradientStop,
    LinearGradient as SceneLinearGradient, Paint, RadialGradient as SceneRadialGradient, SceneImage,
    Stroke,
};
use obscur_scene::path::Path;
use obscur_scene::text::{GlyphRun, PositionedGlyph};
use obscur_style::computed::{ComputedStyle, ComputedValue};
use obscur_style::property::{
    layout_keyword, BackgroundLayer, BackgroundPosition, BackgroundRepeat, BackgroundSize,
    BoxShadow, ConicGradient, Corner, FilterOp, Length, LinearGradient, ObjectFit, PropertyId,
    PropertyValue, RadialGradient, RadialShape, RadialSize, StyleColor, Transform, TransformOrigin,
};
use obscur_text::font::{
    FontAttributes, FontFace, FontFallback, FontFamily, FontId, FontMetrics, FontResolver,
    FontSource, GenericFamily, MockFontResolver,
};
use obscur_text::layout::{
    Hyphenator, LineStrut, NullHyphenator, ParagraphLayout, ParagraphLine, TextAlignment,
    TextOverflow, WhiteSpace,
};
use obscur_text::selection::TextSelection;
use obscur_text::shaping::{
    apply_spacing, apply_tab_stops, CachingShaper, MockTextShaper, ParagraphShaper, ShapedGlyph,
    ShapedRun, SingleRunItemizer, TextDirection, TextItemizer, TextShaper,
};
use obscur_text::selection::{CaretAffinity, SelectionGeometry};
use obscur_tree::tree::RetainedTree;

use super::stacking::{effective_paint_layer, effective_stacking_z, PaintLayer, PaintUnit};
use super::{
    alignment_for, attributes_for, direction_for, display_text, families_for, letter_spacing_for,
    line_height_for, overflow_for, root_font_size, tab_size_for, text_transform_for,
    white_space_for, word_spacing_for, ComputedStyles, EditingView, LayoutBoxes, TabSize,
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

/// The width, in logical pixels, of a scroll container's vertical scrollbar
/// (track and thumb) painted in the right gutter of the scrollport.
const SCROLLBAR_WIDTH: f32 = 8.0;

/// The smallest a scrollbar thumb is allowed to shrink to (logical px), so a very
/// tall content still leaves a grabbable-sized thumb.
const SCROLLBAR_MIN_THUMB: f32 = 16.0;

/// The scrollbar track color: a faint translucent wash so it reads as a gutter
/// without dominating. A host has no hook to override this yet (a documented
/// fixed default, like the selection highlight).
const SCROLLBAR_TRACK_COLOR: Color = Color::rgba(0.0, 0.0, 0.0, 0.06);

/// The scrollbar thumb color: a darker translucent grey over the track.
const SCROLLBAR_THUMB_COLOR: Color = Color::rgba(0.0, 0.0, 0.0, 0.35);

/// Whether the wrapped-line boundary at byte `index` follows a **forced** break —
/// a mandatory line-break character consumed by the earlier line (`\n`, `\r`, and
/// the other Unicode segment-break characters, CSS Text 3 §4.1) — rather than a
/// **soft** wrap where no character is consumed.
///
/// Caret [`CaretAffinity`] only disambiguates SOFT boundaries: at a forced break
/// the two visual positions straddle the break character and are therefore
/// *different* bytes, never the same offset, so affinity is inert. The soft case
/// (this returns `false`) is the one where `index` is simultaneously the end of one
/// line and the start of the next with the preceding character being ordinary
/// content. A caret offset is always a char boundary; a non-boundary or
/// out-of-range `index` conservatively reports `false`.
fn boundary_after_forced_break(content: &str, index: usize) -> bool {
    let index = index.min(content.len());
    if !content.is_char_boundary(index) {
        return false;
    }
    content[..index].chars().next_back().is_some_and(|c| {
        matches!(
            c,
            '\n' | '\r' | '\u{000B}' | '\u{000C}' | '\u{0085}' | '\u{2028}' | '\u{2029}'
        )
    })
}

/// The live scroll state a scroll container is painted with (CSS Overflow 3
/// §scroll-container): its current vertical scroll position and the scrollable
/// content height that sizes the scrollbar thumb.
///
/// The runtime owns the authoritative per-node scroll position + content height
/// (see [`scroll`](super::scroll)); it hands a map of these to the coordinator so
/// composition stays a **pure** function of its inputs. A node **absent** from
/// that map is not a scroll container (or does not overflow), so no scroll
/// transform and no scrollbar are emitted — byte-for-byte identical to before
/// scroll containers existed (the regression guard).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollPaint {
    /// The current vertical scroll position (logical px), already clamped by the
    /// runtime to `0 ..= content_height − scrollport_height`.
    pub scroll_top: f32,
    /// The scrollable content height (logical px), `> scrollport_height`.
    pub content_height: f32,
}

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
        white_space: WhiteSpace,
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
            self.paragraph.shape_paragraph(
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
        // `content` still holds a `\t` (only `pre` / `pre-wrap` keep tabs).
        let runs = apply_tab_stops(
            runs,
            content,
            self.tab_stop_width(tab_size, fallback, attributes, direction, font_size, content),
        );
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
        match self.hyphenator.as_deref() {
            // No hyphenator and no ellipsis glyph: the exact pre-hyphenation path —
            // no hyphen or ellipsis glyph is shaped, so the output (and the
            // shaping-cache call count) is byte-identical to today. Clip truncation
            // still applies through `overflow`, but only clips lines that overflow.
            None if ellipsis.is_none() => ParagraphLayout::wrap_strut(
                &runs,
                content,
                max_width,
                strut,
                &seg_metrics,
                alignment,
                overflow,
                white_space,
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
                &NullHyphenator,
                None,
                ellipsis,
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

    /// Resolves the `tab-size` tab-stop width in logical pixels for `content`
    /// (CSS Text 3 §4.3), or `0.0` when `content` holds no preserved tab.
    ///
    /// A `<number>` tab-size multiplies the advance of a space shaped through the
    /// coordinator's default face — identical to the layout coordinator's — so both
    /// passes resolve the same stop; a `<length>` needs no space. The space is only
    /// shaped when a tab is present, keeping the tab-free path allocation-free.
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
                .shape_paragraph(" ", fallback, attributes, &self.default_face, font_size, direction)
                .iter()
                .flat_map(|run| run.run().glyphs().iter())
                .map(ShapedGlyph::x_advance)
                .sum(),
        };
        tab_size.resolve(space_advance)
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

    /// Returns the caret's pen x (logical pixels) at byte `index` in `content`,
    /// resolving the shaping inputs from the text node's computed `style` — the
    /// **style-driven** wrapper over [`caret_offset_x`](Self::caret_offset_x) the
    /// runtime uses to keep the caret in view (single-line horizontal scroll, CSS
    /// Overflow 3 §3.1). It resolves `font-family` / `font-weight` / `font-style`
    /// / `direction` / `font-size` **identically** to
    /// [`caret_offset_at_x`](Self::caret_offset_at_x) (the click inverse) and the
    /// compose caret. `index == content.len()` yields the whole run advance (the
    /// field's content width).
    ///
    /// This single-line scroll measure sums the **un-spaced** shaped advances; the
    /// compose caret + click inverse resolve their within-line x from the composed,
    /// spaced/tabbed line ([`line_caret_x`](Self::line_caret_x)) instead. The two
    /// agree exactly for the default field (no letter/word-spacing, no tab) that
    /// horizontal scroll applies to; folding spacing into the scroll measure is
    /// deferred to the horizontal-scroll item.
    #[must_use]
    pub(crate) fn caret_advance_x(
        &self,
        content: &str,
        style: Option<&ComputedStyle>,
        index: usize,
    ) -> f32 {
        let fallback = families_for(style);
        let attributes = attributes_for(style);
        let direction = direction_for(style);
        let font_size = font_size_of_style(style);
        self.caret_offset_x(content, &fallback, &attributes, direction, font_size, index)
    }

    /// The line index a caret byte `index` sits on within a wrapped `layout`
    /// (spec §5.10): the first line whose byte range **ends past** `index`, else
    /// the last line — so a caret at a soft-wrap/`\n` boundary sits at the start of
    /// the *next* line and a caret at the text end sits on the final line. A
    /// single-line layout always resolves to line `0`.
    fn line_of_byte(layout: &ParagraphLayout, index: usize) -> usize {
        let lines = layout.lines();
        lines
            .iter()
            .position(|line| index < line.range().end)
            .unwrap_or_else(|| lines.len().saturating_sub(1))
    }

    /// The byte a wrapped line's content **starts** at (spec §5.10). For a line
    /// with glyphs this is its first glyph's cluster (`range().start`); for an
    /// **empty** line box — a trailing/leading/consecutive forced break (`\n`)
    /// leaves one, whose [`ParagraphLine`] defaults its start to `0` — the true
    /// start is the *previous* line's end (or `0` for a leading empty line), so a
    /// caret on the empty line after a final `\n` resolves to just past it.
    fn line_start_byte(layout: &ParagraphLayout, li: usize) -> usize {
        let lines = layout.lines();
        match lines.get(li) {
            Some(line) if line.glyph_count() > 0 => line.range().start,
            Some(_) if li > 0 => lines[li - 1].range().end,
            _ => 0,
        }
    }

    /// The caret's pen x **within** a wrapped `line`: the sum of the line's own
    /// spaced/tabbed glyph advances whose source **cluster** falls before `index`
    /// (CSS Text 3 §8 letter-/word-spacing + §4.3 tab-size), measured from the
    /// line's content start.
    ///
    /// These are the SAME advances the segments are painted from — the line's
    /// glyphs already carry the `apply_spacing` / `apply_tab_stops` adjustments the
    /// compose pass wrapped them with, and any per-line trailing-tracking hang and
    /// justification the wrapper applied (CSS Text 3 §8.1 / §7.1) — so a caret x
    /// derived here lands exactly on the painted glyph edge, not on a fresh
    /// spacing-blind re-shape. Because the line holds only glyphs with clusters in
    /// its own byte range, filtering `cluster < index` yields the line-relative pen
    /// x directly (no `\n`/preceding-line subtraction). A caret at the line's end
    /// byte sums every glyph → the line's used content width. Single-line,
    /// left-to-right: RTL reordering is not modeled (a documented approximation).
    fn line_caret_x(line: &ParagraphLine, index: usize) -> f32 {
        line.segments()
            .iter()
            .flat_map(|segment| segment.run().glyphs().iter().copied())
            .filter(|glyph| glyph.cluster() < index)
            .map(|glyph| glyph.x_advance())
            .sum()
    }

    /// Resolves a within-line pen x (`local_x`, measured from the `line`'s content
    /// start with any alignment offset already undone) to the nearest cluster byte
    /// on that line — the per-line **inverse** of [`line_caret_x`](Self::line_caret_x)
    /// over the SAME spaced/tabbed glyph advances (spec §5.11). The boundary whose
    /// horizontal midpoint `local_x` falls at or before is chosen (a tie resolves to
    /// the earlier boundary); a `local_x` past the last glyph clamps to `line_end`.
    fn byte_at_line_local_x(line: &ParagraphLine, line_end: usize, local_x: f32) -> usize {
        let mut pen = 0.0_f32;
        for glyph in line
            .segments()
            .iter()
            .flat_map(|segment| segment.run().glyphs().iter())
        {
            let advance = glyph.x_advance();
            // The boundary just left of this glyph sits at `pen`; cross to the next
            // boundary only once `local_x` is past this glyph's midpoint.
            if local_x <= pen + advance / 2.0 {
                return glyph.cluster();
            }
            pen += advance;
        }
        // Past every glyph on this line: clamp to the line's end byte.
        line_end
    }

    /// Resolves a caret byte `index` to its painted geometry against the wrapped
    /// `layout` (spec §5.10/§5.11): `(line_top, line_height, line_x_offset, x)`,
    /// where `line_top` is the caret's line box top measured **down from the text
    /// box origin** (add `origin.y`), `line_x_offset` is that line's alignment
    /// offset, and `x` is the caret pen x measured from the line's content start
    /// (so the painted x is `origin.x + line_x_offset + x − scroll_left`).
    ///
    /// The pen x is the line-relative [`line_caret_x`](Self::line_caret_x): the sum
    /// of THIS line's spaced/tabbed glyph advances before `index`, i.e. the same
    /// advances (incl. any per-line hang / justification, CSS Text 3 §8) the
    /// segments paint from — so a caret on line 2 lands at that line's own x, not
    /// the summed width of both lines, AND a field with `letter-spacing` /
    /// `word-spacing` / `tab` places the caret on the spaced glyph edge rather than
    /// a spacing-blind re-shape. A single-line, default-spacing, left-aligned field
    /// → line 0, top 0, `x_offset` 0, and the same spacing-0 advance sum the flat
    /// re-shape produced — byte-identical.
    fn caret_line_geometry(
        &self,
        layout: &ParagraphLayout,
        font_size: f32,
        index: usize,
    ) -> (f32, f32, f32, f32) {
        let li = Self::line_of_byte(layout, index);
        let Some(line) = layout.lines().get(li) else {
            return (0.0, self.paragraph.line_metrics(font_size).line_height(), 0.0, 0.0);
        };
        (line.top(), line.height(), line.x_offset(), Self::line_caret_x(line, index))
    }

    /// Resolves a caret byte `index` to its painted geometry like
    /// [`caret_line_geometry`](Self::caret_line_geometry) but honoring the caret's
    /// wrap-boundary [`CaretAffinity`] (spec §5.11).
    ///
    /// A byte at a **soft**-wrap boundary is shared by two visual positions — the
    /// end of the earlier line and the start of the later one. The default
    /// [`Downstream`](CaretAffinity::Downstream) binds it to the later line's
    /// leading edge (exactly what `caret_line_geometry` does), so a default-affinity
    /// caret — and every caret NOT on a soft boundary — is byte-identical.
    /// [`Upstream`](CaretAffinity::Upstream) instead binds it to the earlier line's
    /// trailing edge: its box top/height, its alignment `x_offset`, and its full
    /// used content width (every glyph on that line has a cluster before `index`).
    /// A **forced** break (`\n`, spec §4.1) is NOT a soft boundary — its two sides
    /// are different bytes — so affinity is inert there (`content` distinguishes the
    /// two cases).
    fn caret_line_geometry_affinity(
        &self,
        layout: &ParagraphLayout,
        content: &str,
        font_size: f32,
        index: usize,
        affinity: CaretAffinity,
    ) -> (f32, f32, f32, f32) {
        let lines = layout.lines();
        let downstream = Self::line_of_byte(layout, index);
        if affinity.is_upstream()
            && downstream > 0
            && lines[downstream - 1].range().end == index
            && Self::line_start_byte(layout, downstream) == index
            && !boundary_after_forced_break(content, index)
        {
            let line = &lines[downstream - 1];
            return (line.top(), line.height(), line.x_offset(), Self::line_caret_x(line, index));
        }
        self.caret_line_geometry(layout, font_size, index)
    }

    /// Shapes the wrapped multi-line [`ParagraphLayout`] for `content` on `node`
    /// using the SAME style inputs and `box_width` (the text node's content-box
    /// width, its wrap scrollport) the compose pass wraps the drawn runs with
    /// (spec §5.10) — so a caret / hit-test resolved against this layout lands on
    /// the same line the glyphs paint on. The runtime uses it for vertical caret
    /// motion and 2D click→byte hit-testing over the live editing text.
    pub(crate) fn shape_text_lines(
        &self,
        tree: &RetainedTree,
        computed: &ComputedStyles,
        node: NodeId,
        content: &str,
        box_width: f32,
    ) -> ParagraphLayout {
        let style = computed.get(&node);
        let font_size = font_size_of(computed, node);
        // Root `font-size` resolves any `rem` in the text metrics (CSS Values 4
        // §5.1.1); `em` / `%` resolve against this node's `font_size`. Resolved
        // identically to the layout phase so caret/hit-test lines match the box.
        let root_font = root_font_size(computed, tree.root());
        self.shape_lines(
            content,
            &families_for(style),
            &attributes_for(style),
            direction_for(style),
            alignment_for(style),
            overflow_for_text(tree, computed, node),
            white_space_for(style),
            font_size,
            box_width,
            line_height_for(style, font_size, root_font),
            letter_spacing_for(style, font_size, root_font),
            word_spacing_for(style, font_size, root_font),
            tab_size_for(style, font_size, root_font),
        )
    }

    /// Resolves a pointer at text-box-local `(local_x, local_y)` (already
    /// scroll-adjusted; measured from the text node's box origin) to a byte offset
    /// in `content`, picking the wrapped line by `local_y` then the byte by
    /// `local_x` on that line — the 2D inverse of the line-aware caret paint
    /// (spec §5.11). The line is the first whose box contains `local_y` (else the
    /// last); within it the byte is the nearest cluster boundary to `local_x`
    /// (`caret_offset_at_x`'s midpoint walk), clamped to the line's byte span so a
    /// click never crosses onto an adjacent line, and kept before a trailing `\n`.
    /// A single-line field → line 0 and the identical `caret_offset_at_x` byte
    /// (byte-identical).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn caret_offset_at_point(
        &self,
        tree: &RetainedTree,
        computed: &ComputedStyles,
        node: NodeId,
        content: &str,
        box_width: f32,
        local_x: f32,
        local_y: f32,
    ) -> usize {
        let style = computed.get(&node);
        let layout = self.shape_text_lines(tree, computed, node, content, box_width);
        let lines = layout.lines();
        if lines.is_empty() {
            return self.caret_offset_at_x(content, style, local_x);
        }
        // The line whose box contains `local_y` (top ≤ y < top+height), else the
        // last line (a click below the text lands on the final line).
        let li = lines
            .iter()
            .position(|line| local_y < line.top() + line.height())
            .unwrap_or(lines.len() - 1);
        self.byte_at_line_x(&layout, li, content, style, local_x)
    }

    /// The byte on wrapped line `li` nearest an x of `target_x` (measured from the
    /// text box origin), clamped to that line's byte span (spec §5.11).
    ///
    /// The line's content starts at pen x `line.x_offset()` from the box origin, so
    /// the target is turned into a within-line pen x by undoing the alignment offset
    /// (`target_x − line.x_offset()`) and resolved over the line's OWN spaced/tabbed
    /// glyph advances with the SAME midpoint walk
    /// [`byte_at_line_local_x`](Self::byte_at_line_local_x) uses — the exact inverse
    /// of the [`line_caret_x`](Self::line_caret_x) the caret paints with, so a click
    /// at a spaced glyph edge (or on a `text-align: right`/`center` line) resolves
    /// back to that glyph's byte. The result is clamped to `[line start, line end']`,
    /// where `end'` excludes a trailing `\n` so a click past the line's end sits
    /// before the newline (on this line), not at the start of the next.
    fn byte_at_line_x(
        &self,
        layout: &ParagraphLayout,
        li: usize,
        content: &str,
        style: Option<&ComputedStyle>,
        target_x: f32,
    ) -> usize {
        let Some(line) = layout.lines().get(li) else {
            return self.caret_offset_at_x(content, style, target_x);
        };
        let line_start = Self::line_start_byte(layout, li);
        // Undo the line's alignment offset, then resolve against this line's own
        // spaced/tabbed advances (the exact inverse of the painted caret x).
        let byte =
            Self::byte_at_line_local_x(line, line.range().end, target_x - line.x_offset());
        // The maximum caret byte that stays on this line: the line's end, minus a
        // trailing `\n` (whose caret belongs before it, on this line).
        let end = line.range().end.max(line_start).min(content.len());
        let line_max = if content.get(line_start..end).is_some_and(|s| s.ends_with('\n')) {
            end - 1
        } else {
            end
        };
        byte.clamp(line_start, line_max)
    }

    /// Resolves the byte a **vertical** caret move (ArrowUp/ArrowDown) lands on
    /// from `index`, moving to the line above (`down = false`) or below
    /// (`down = true`) at roughly the same pen x — standard caret vertical motion
    /// (spec §5.11). Returns [`None`] when there is no line in that direction (the
    /// caret is already on the first line moving up, or the last line moving down),
    /// so the caller can clamp to the text start/end. A single-line field has one
    /// line, so it always returns [`None`].
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn caret_line_move(
        &self,
        tree: &RetainedTree,
        computed: &ComputedStyles,
        node: NodeId,
        content: &str,
        box_width: f32,
        index: usize,
        down: bool,
    ) -> Option<usize> {
        let style = computed.get(&node);
        let layout = self.shape_text_lines(tree, computed, node, content, box_width);
        let cur = Self::line_of_byte(&layout, index);
        let target = if down {
            let next = cur + 1;
            (next < layout.lines().len()).then_some(next)?
        } else {
            cur.checked_sub(1)?
        };
        // The caret's current pen x within its line (over the line's own
        // spaced/tabbed advances), re-expressed from the text box origin by adding
        // the TARGET line's alignment offset so `byte_at_line_x` maps it back
        // consistently (it undoes that same offset).
        let within_line =
            layout.lines().get(cur).map_or(0.0, |line| Self::line_caret_x(line, index));
        let target_x = layout.lines()[target].x_offset() + within_line;
        Some(self.byte_at_line_x(&layout, target, content, style, target_x))
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
        // No scroll containers: byte-identical to before per-element scroll existed.
        self.compose_with_images_scrolled(tree, computed, boxes, editing, images, &HashMap::new())
    }

    /// Composes like [`compose_with_images`](Self::compose_with_images), additionally
    /// applying each scroll container's live scroll offset and painting its
    /// scrollbar (CSS Overflow Module Level 3 §scroll-container, §scrollport).
    ///
    /// `scroll` maps a scroll-container [`NodeId`] to its
    /// [`ScrollPaint`] (current `scroll_top` + scrollable `content_height`). For a
    /// node present in the map the coordinator brackets its descendants — inside
    /// the node's own `overflow` clip and outside the node's own paint — with a
    /// `PushTransform(translate(0, −scroll_top))`, so the content scrolls within
    /// the (clipped) scrollport while the container's own background/border stay
    /// put; and paints a vertical scrollbar (track + thumb) on the right, on top of
    /// and unaffected by the scroll. A node **absent** from the map contributes
    /// neither, so an **empty** map reproduces
    /// [`compose_with_images`](Self::compose_with_images) byte-for-byte. The scroll
    /// transform is elided at `scroll_top == 0` (it reduces to the identity), so an
    /// unscrolled container differs from the pre-scroll list only by its scrollbar.
    #[must_use]
    pub fn compose_with_images_scrolled(
        &self,
        tree: &RetainedTree,
        computed: &ComputedStyles,
        boxes: &LayoutBoxes,
        editing: Option<EditingView<'_>>,
        images: &HashMap<Box<str>, SceneImage>,
        scroll: &HashMap<NodeId, ScrollPaint>,
    ) -> DisplayList {
        let Some(root) = tree.root() else {
            return DisplayListBuilder::new().build();
        };

        // Each node's effective stacking z (a positioned element's numeric
        // `z-index`, else its nearest z-index ancestor's, root = 0). Computed once
        // up front so the emission walk can key each paint unit by it.
        let zmap = effective_stacking_z(tree, computed);
        // Each node's CSS 2.1 §9.9 paint-layer rank, keyed like `zmap`. Sorted
        // between `z` and `tree_index`, it lifts a positioned `z:auto`/`0` box (and
        // its inherited subtree) above non-positioned siblings within the same z
        // bucket (catalog bug B2). Uniform (`NonPositioned`) when nothing is
        // positioned, so the sort then reduces to the pre-fix `(z, tree_index)`.
        let layermap = effective_paint_layer(tree, computed);

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
        // A CSS `filter` blurs and/or color-transforms a node's WHOLE composited
        // subtree: it brackets the subtree with a
        // `PushLayer(SceneLayer::with_clip(border_box).add_effect(...))` …
        // `PopLayer` carrying a `blur(r)` effect (r > 0), a `ColorMatrix` effect
        // (the folded color-op chain), or both. The clip carries the element's
        // border box so the renderer knows the region. This filter layer is the
        // OUTERMOST subtree wrapper (outside the transform / opacity / clip
        // brackets), so the Vello renderer's owned wgpu passes operate on the
        // fully-composited element. Each tuple is `(node, border_box, blur_radius
        // (0 = none), color_matrix)`; no blur AND no color op records nothing —
        // byte-for-byte identical to before.
        let mut blur_brackets: Vec<(NodeId, Rect, f32, Option<[f32; 20]>)> = Vec::new();
        // A CSS `outline` (a focus ring) paints as ONE stroked path OUTSIDE the
        // node's border box, offset by `outline-offset`. It reserves no layout
        // space and — unlike the background/border — must NOT be clipped by the
        // node's own rounded clip, so it is recorded here and emitted in the second
        // pass as its own unit at the subtree's EXIT index (on top of, and outside,
        // the node's whole subtree). No outline records nothing — byte-for-byte
        // identical to before outlines existed.
        let mut outline_brackets: Vec<(NodeId, PaintCommand)> = Vec::new();
        // A scroll container (a node present in `scroll` with a non-zero
        // `scroll_top`) brackets its DESCENDANTS with a
        // `PushTransform(translate(0, −scroll_top))` … `PopTransform` nested INSIDE
        // its overflow clip and OUTSIDE its own paint, so the content scrolls within
        // the clipped scrollport while the container's own background stays fixed
        // (CSS Overflow 3 §scrollport). `(node, scroll_top)`; a zero offset records
        // nothing (the identity), so an unscrolled container is byte-identical save
        // for its scrollbar.
        let mut scroll_brackets: Vec<(NodeId, f32)> = Vec::new();
        // A scroll container's vertical scrollbar: `(node, scrollport, scroll_top,
        // content_height)`. Painted as its own unit at the subtree exit index, on
        // top of and outside the scrolled content (CSS Overflow 3 §scrollbars).
        let mut scrollbar_units: Vec<(NodeId, Rect, f32, f32)> = Vec::new();
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
            // subtree to the rounded border box. The authored radii are put through
            // the CSS Backgrounds §5.5 overlapping-curves correction ONCE here (a
            // single uniform scale over the box), so the SAME normalized radii feed
            // every consumer — background fill, subtree clip, border ring, box-shadow
            // and outline. Radii that already fit are returned unchanged
            // (byte-identical); a node without an arranged box keeps its raw radii.
            let radii = {
                let raw = corner_radii_of(computed, node);
                boxes
                    .get(&node)
                    .map_or(raw, |layout_box| raw.normalized(layout_box.bounds))
            };
            let has_radius = corner_radii_present(radii);
            // A node with `overflow` other than `visible` clips its subtree to its
            // box (`hidden` / `clip` / `scroll` / `auto`; scroll/auto clip like
            // hidden — no scrollbars yet). `visible` / absent adds no clip.
            let clips_overflow = overflow_clips(computed, node);
            // The clip shape (border box) bracketing this node's subtree, recorded
            // for a second pass once every descendant's exit index is known. A node
            // that rounds its corners OR clips overflow emits exactly ONE clip: the
            // ROUNDED border box when it has a radius (which also clips overflow),
            // else a plain rect clip to the border box.
            if (has_radius || clips_overflow) && let Some(layout_box) = boxes.get(&node) {
                let clip = if has_radius {
                    Clip::rounded_rect(layout_box.bounds, radii)
                } else {
                    Clip::rect(layout_box.bounds)
                };
                clip_brackets.push((node, clip));
            }

            // A scroll container (present in `scroll`): record its descendant
            // scroll transform (elided at `scroll_top == 0` — the identity) and its
            // scrollbar. The `overflow` clip above already brackets the same
            // subtree, so the scroll transform nests inside it (CSS Overflow 3
            // §scrollport). Absent from the map ⇒ nothing recorded ⇒ byte-identical.
            if let (Some(paint), Some(layout_box)) = (scroll.get(&node), boxes.get(&node)) {
                if paint.scroll_top != 0.0 {
                    scroll_brackets.push((node, paint.scroll_top));
                }
                scrollbar_units.push((
                    node,
                    layout_box.bounds,
                    paint.scroll_top,
                    paint.content_height,
                ));
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

            // A CSS `filter` (blur and/or color ops) brackets this node's subtree
            // with a filter group layer clipped to its border box. It needs the
            // arranged box (the effect region), so — unlike opacity — it is only
            // recorded for a node with a box. The color-op chain folds into one
            // 4×5 matrix; blur keeps its radius. No blur AND no color op (or no
            // box) records nothing (byte-for-byte identical).
            if let Some(layout_box) = boxes.get(&node) {
                let radius = filter_blur_of(computed, node);
                let color_matrix = filter_color_matrix_of(computed, node);
                if radius.is_some() || color_matrix.is_some() {
                    blur_brackets.push((
                        node,
                        layout_box.bounds,
                        radius.unwrap_or(0.0),
                        color_matrix,
                    ));
                }
            }

            // A CSS `outline` (focus ring): when the node has a non-`none`
            // `outline-style` and a positive `outline-width`, record the stroked
            // ring it paints OUTSIDE its border box (inflated by
            // `outline-offset + width/2`). It needs the arranged box; no outline
            // (or no box) records nothing (byte-for-byte identical to before).
            if let Some(layout_box) = boxes.get(&node)
                && let Some(command) = outline_paint(computed, node, layout_box.bounds, radii, has_radius)
            {
                outline_brackets.push((node, command));
            }

            // This node's own paint accumulates into a private builder so it can be
            // reordered as one unit; only leaf commands are emitted, so it stays
            // balanced.
            let mut node_paint = DisplayListBuilder::new();

            // CSS `backdrop-filter: blur(r)`: the FIRST command of the node's paint
            // unit, emitted BEFORE its own shadow / background / border / content. It
            // frosts (gaussian-blurs) the backdrop — everything painted earlier
            // (prior siblings, ancestor content) — within the node's border box,
            // clipped to its rounded corners, so the node's (usually translucent)
            // background then draws OVER the frosted backdrop → acrylic. It needs the
            // arranged box (the frost region); no `backdrop-filter` (or no box) emits
            // nothing (byte-for-byte identical). Only the GPU (Vello) backend frosts;
            // the CPU rasterizer honestly skips it (documented).
            if let Some(radius) = backdrop_blur_of(computed, node)
                && let Some(layout_box) = boxes.get(&node)
            {
                node_paint.backdrop_filter(layout_box.bounds, radii, radius);
            }

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
            let filter_ops = filter_ops_of(computed, node);
            if let (Some(filter_ops), Some(layout_box)) = (filter_ops, boxes.get(&node)) {
                let current = current_color_of(computed, node);
                for shadow in filter_ops.iter().rev().filter_map(FilterOp::as_drop_shadow) {
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
                // border/content — matching CSS background paint order. The image's
                // tile is placed (`background-position`), sized (`background-size`),
                // and tiled (`background-repeat`) within the node's border box (the
                // paint region), resolved by `background_image_geometry`. A URL not
                // present in `images` (unloaded or failed) emits nothing, so the
                // color background shows through — the regression-safe path
                // (byte-identical when `images` is empty). This is the SINGLE-layer
                // path; the multi-layer list is handled just below.
                if let Some(image) = background_image_url_of(computed, node)
                    .and_then(|url| images.get(url))
                {
                    let (tile, repeat_x, repeat_y) =
                        background_image_geometry(computed, node, image, layout_box.bounds);
                    node_paint.draw_image(
                        image.clone(),
                        tile,
                        layout_box.bounds,
                        repeat_x,
                        repeat_y,
                    );
                }

                // Multiple `background-image` layers (a comma list). CSS lists them
                // top→bottom (first = top) and paints back-to-front, so we iterate
                // in REVERSE: the last-listed (bottom) layer paints first, the
                // first-listed (top) last. Every layer paints OVER the
                // `background-color` fill emitted above and UNDER the border/content.
                // The single-layer case never reaches here (it stays a flat value
                // above), so one-layer backgrounds are byte-identical. Per-layer
                // `background-size`/`-position`/`-repeat` currently reuse the node's
                // single resolved value for every layer (those longhands are not yet
                // comma lists).
                if let Some(layers) = background_layers_of(computed, node) {
                    for layer in layers.iter().rev() {
                        paint_background_layer(
                            &mut node_paint,
                            layer,
                            computed,
                            node,
                            layout_box.bounds,
                            radii,
                            has_radius,
                            images,
                        );
                    }
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

            // Border edges paint over the background, under the text. There are two
            // shapes:
            //
            // - A **uniform** border (all four widths equal & positive, all four
            //   colors equal, every side `solid`) on a node with a positive corner
            //   radius paints as ONE even-odd rounded-rect RING: the border-box
            //   rounded rect with the padding-box rounded rect (radii reduced by the
            //   border width) punched out. This rounds the border's INNER corners as
            //   well as its outer ones, matching CSS — the mitered-quad path below
            //   keeps its corners square-cornered (but 45°-mitered). The ring is
            //   self-contained, so it does not rely on the surrounding rounded
            //   `PushClip` for its shape.
            // - Otherwise (no radius, or a non-uniform border) each visible side
            //   paints one **mitered trapezoid quad**: its outer edge spans the full
            //   border-box side and its inner edge is inset by the two adjacent
            //   border widths, so the corners split at 45° between their two sides
            //   (the correct CSS border-corner model, and what makes the CSS
            //   border-triangle trick render — a collapsed side becomes a triangle).
            //   A rounded non-uniform border is clipped to the rounded border box
            //   (the surrounding `PushClip`) and its inner corners stay approximate
            //   (documented). A transparent or zero-width side, or an unauthored
            //   border, emits nothing.
            if let Some(layout_box) = boxes.get(&node) {
                if let Some((width, color)) =
                    uniform_border(computed, node).filter(|_| has_radius)
                {
                    let ring = uniform_border_ring(layout_box.bounds, radii, width);
                    node_paint.fill(ring, Brush::solid(scene_color_from(color)));
                } else if let Some((width, color)) = uniform_dashed_border(computed, node) {
                    // A uniform DASHED border is a single dashed stroke run along
                    // the border box inset by width/2, so the centered stroke band
                    // exactly fills the CSS border area (rounded when a radius is
                    // present). Solid borders never reach here.
                    let path = dashed_border_path(layout_box.bounds, radii, width, has_radius);
                    let stroke = Stroke::new(Brush::solid(scene_color_from(color)), width)
                        .with_dash(dashed_border_pattern(width), 0.0);
                    node_paint.stroke(path, stroke);
                } else {
                    for (path, color) in border_fills(computed, node, layout_box.bounds) {
                        node_paint.fill(path, Brush::solid(scene_color_from(color)));
                    }
                }
            }

            // An element's own image content (an `<img>` / `Image` primitive's
            // `src`) whose image has been resolved paints as the element's CONTENT:
            // OVER its background / border (emitted after them), UNDER its children
            // and text (they are separate paint units at a later tree index). The
            // image is sized/placed by `object-fit` (default `fill` = stretch to the
            // box) about the box center, and its `region` is the box so `cover`
            // overflow is clipped. The paint uses the element's BORDER box (the
            // content-box insets are not resolved in this pure coordinator — a
            // documented approximation). A `src` absent from `images` (unloaded or
            // failed) or no `src` at all emits nothing (byte-identical when `images`
            // is empty). A `background-image` and an element `src` both paint: the
            // background behind, the element image above.
            if let Some(layout_box) = boxes.get(&node)
                && let Some(image) = element_image_of(tree, node).and_then(|src| images.get(src))
            {
                let tile = object_fit_tile(computed, node, image, layout_box.bounds);
                node_paint.draw_image(image.clone(), tile, layout_box.bounds, false, false);
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
                    // The displayed text, computed through the SAME `display_text`
                    // seam the layout (measure/arrange) pass uses, so this frame's
                    // box and the glyphs painted here shape the identical string
                    // (CSS Text 3 §4.1.1). While actively editing this node's text,
                    // the live session buffer (with any IME preedit spliced in for
                    // DISPLAY, spec §5.12) is shaped RAW — `text-transform` (§2.1)
                    // and `white-space` collapsing (§4.1.1) are skipped so the
                    // caret/selection can index the shaped string by live buffer
                    // byte offsets; otherwise the retained content flows through
                    // casing then whitespace processing. The field is wrapped by its
                    // OWN `white-space` in both states, so a single-line `input`
                    // (`nowrap`, per the UA sheet) stays one line while typing — it
                    // does NOT reflow like a `<textarea>` — while a `pre-wrap` field
                    // keeps its literal newlines and wraps at the box width.
                    let white_space = white_space_for(computed.get(&node));
                    let displayed = display_text(
                        retained_content,
                        active,
                        text_transform_for(computed.get(&node)),
                        white_space,
                    );
                    let content: &str = displayed.as_ref();
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
                    // Root `font-size` resolves any `rem` in the text metrics (CSS
                    // Values 4 §5.1.1); `em` / `%` resolve against this node's
                    // `font_size`. Same bases the layout phase measured with.
                    let root_font = root_font_size(computed, tree.root());
                    // The node's authored `font-family` drives the fallback chain,
                    // and its `font-weight`/`font-style`/`direction` drive the
                    // shaping attributes and base direction — the SAME inputs the
                    // layout phase measured this box with.
                    let fallback = families_for(computed.get(&node));
                    let attributes = attributes_for(computed.get(&node));
                    let direction = direction_for(computed.get(&node));
                    let alignment = alignment_for(computed.get(&node));
                    let overflow = overflow_for_text(tree, computed, node);
                    let origin = layout_box.origin();
                    // The editing field's horizontal scroll offset (single-line
                    // text-control scroll model, CSS Overflow 3 §3.1): the run,
                    // caret, selection highlight and preedit are all translated
                    // left by `scroll_left` so the caret stays inside the field's
                    // content box, and the field's `overflow: hidden` clip (an
                    // ancestor bracket) hides the scrolled-out head/tail. It is
                    // `0.0` on every non-editing text node and on a fitting or
                    // wrapping field, so the paint is then byte-identical to the
                    // unscrolled path. The runtime derives it with the SAME advance
                    // model (`caret_advance_x`) used below, so paint and hit-test
                    // agree.
                    let scroll_left = active.map_or(0.0, |view| view.scroll_left);
                    // The wrapped multi-line paragraph the glyphs are drawn from
                    // (spec §5.10). The caret + selection resolve their (line, x, y)
                    // against THIS same layout — a byte's line by its wrapped range,
                    // its pen x by the source-order advance model relative to that
                    // line's start — so a caret lands on the line its glyphs paint
                    // on. A single-line (`nowrap`) field wraps to one line, so the
                    // caret resolves to line 0 at the identical pen x the flat
                    // `caret_offset_x` returned (byte-identical single-line paint).
                    let layout = self.shape_lines(
                        content,
                        &fallback,
                        &attributes,
                        direction,
                        alignment,
                        overflow,
                        white_space,
                        font_size,
                        layout_box.size().width,
                        line_height_for(computed.get(&node), font_size, root_font),
                        letter_spacing_for(computed.get(&node), font_size, root_font),
                        word_spacing_for(computed.get(&node), font_size, root_font),
                        tab_size_for(computed.get(&node), font_size, root_font),
                    );
                    // Selection highlight, painted BEHIND the glyphs (emitted before
                    // the runs so the text shows over it): the documented
                    // semi-transparent selection color. A span over the wrapped
                    // `layout` paints ONE partial rect per covered line (spec §5.11):
                    // the leading line from x(start) to its trailing edge, whole
                    // interior lines full-width, the trailing line up to x(end). The
                    // per-line geometry is computed by the domain
                    // `SelectionGeometry::from_paragraph` (the source of truth) in
                    // paragraph-local space using the SAME per-line spaced/tabbed
                    // advance + alignment `x_offset` model the caret uses; the
                    // coordinator only translates each rect by the box `origin` and
                    // the field's `−scroll_left`. A span confined to ONE line yields
                    // exactly the single rect the old first-line-only path produced
                    // (byte-identical). A collapsed selection paints nothing.
                    if let Some(selection) = display_selection
                        && !selection.is_empty()
                    {
                        let geometry = SelectionGeometry::from_paragraph(
                            &layout,
                            selection.start(),
                            selection.end(),
                        );
                        for line_rect in geometry.rects() {
                            node_paint.fill_rect(
                                Rect::new(
                                    Point::new(
                                        origin.x + line_rect.origin.x - scroll_left,
                                        origin.y + line_rect.origin.y,
                                    ),
                                    line_rect.size,
                                ),
                                Brush::solid(SELECTION_COLOR),
                            );
                        }
                    }
                    for line in layout.lines() {
                        let baseline = line.baseline();
                        let line_top = origin.y + line.top();
                        let line_left = origin.x + line.x_offset() - scroll_left;
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
                    // SAME line-aware, spaced advance-walk the caret/selection use
                    // (`caret_line_geometry`), over the spliced display `content`:
                    // from x(at) to x(at + preedit len), including the preedit line's
                    // alignment `x_offset` so a right/centre-aligned field underlines
                    // the composing glyphs. Only for the editing field with a live,
                    // non-empty preedit.
                    if let Some(preedit) = active.and_then(|view| view.preedit)
                        && !preedit.text.is_empty()
                    {
                        let at = preedit.at.min(content.len());
                        let line_height = self.paragraph.line_metrics(font_size).line_height();
                        let (top0, _h0, xoff0, x0) =
                            self.caret_line_geometry(&layout, font_size, at);
                        let (_, _, _, x1) = self
                            .caret_line_geometry(&layout, font_size, at + preedit.text.len());
                        node_paint.fill_rect(
                            Rect::new(
                                Point::new(
                                    origin.x + xoff0 + x0 - scroll_left,
                                    origin.y + top0 + line_height - PREEDIT_UNDERLINE_THICKNESS,
                                ),
                                Size::new((x1 - x0).max(0.0), PREEDIT_UNDERLINE_THICKNESS),
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
                        // Line-aware caret geometry (spec §5.10/§5.11): the focus
                        // byte resolves to its wrapped line's top + height and its
                        // pen x within that line, so a caret on line 2 paints at
                        // y ≈ line-height and an x measured within line 2 — not the
                        // summed width of every line. At a soft-wrap boundary the
                        // focus byte is shared by two lines; the selection's
                        // `CaretAffinity` disambiguates — Upstream binds the caret to
                        // the trailing edge of the earlier line, Downstream (the
                        // default) to the leading edge of the later line. Single-line
                        // (and every default-affinity caret) → line 0, top 0, and the
                        // identical pen x (byte-identical).
                        let (top, height, x_offset, caret_x) = self.caret_line_geometry_affinity(
                            &layout,
                            content,
                            font_size,
                            selection.focus(),
                            selection.affinity(),
                        );
                        node_paint.fill_rect(
                            Rect::new(
                                Point::new(origin.x + x_offset + caret_x - scroll_left, origin.y + top),
                                Size::new(CARET_WIDTH, height),
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
                let layer = layermap.get(&node).copied().unwrap_or(PaintLayer::NonPositioned);
                units.push(PaintUnit {
                    z,
                    layer,
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
            || !blur_brackets.is_empty()
            || !outline_brackets.is_empty()
            || !scroll_brackets.is_empty()
            || !scrollbar_units.is_empty()
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
            // Scroll transform brackets are emitted BEFORE the clip brackets so that
            // when a scroll container carries both (it always does — `overflow:
            // scroll/auto` clips too), the tie at the shared `(z, exit, MAX − depth)`
            // pop key resolves — via the stable sort — with the scroll transform's
            // `PopTransform` BEFORE the clip's `PopClip` (the scroll transform closes
            // *inside* the clip). The `PushTransform` sorts AFTER the node's own
            // paint (`SCROLL_TRANSFORM_PUSH_ORDER > PAINT_ORDER`) and after the clip
            // push, so only the descendants (a strictly greater `tree_index`) are
            // scrolled — the container's own background/border stay fixed in the
            // scrollport (CSS Overflow 3 §scrollport).
            for (node, scroll_top) in scroll_brackets {
                let z = zmap.get(&node).copied().unwrap_or(0);
                let layer = layermap.get(&node).copied().unwrap_or(PaintLayer::NonPositioned);
                let enter = enter_index.get(&node).copied().unwrap_or(0);
                let exit = exit_index.get(&node).copied().unwrap_or(enter);
                let depth = depth_of.get(&node).copied().unwrap_or(0);
                let affine = AffineTransform::translate(0.0, -scroll_top);
                units.push(PaintUnit {
                    z,
                    layer,
                    tree_index: enter,
                    order: PaintUnit::SCROLL_TRANSFORM_PUSH_ORDER,
                    commands: vec![PaintCommand::PushTransform(affine)],
                });
                units.push(PaintUnit {
                    z,
                    layer,
                    tree_index: exit,
                    order: PaintUnit::transform_pop_order(depth),
                    commands: vec![PaintCommand::PopTransform],
                });
            }
            for (node, clip) in clip_brackets {
                let z = zmap.get(&node).copied().unwrap_or(0);
                let layer = layermap.get(&node).copied().unwrap_or(PaintLayer::NonPositioned);
                let enter = enter_index.get(&node).copied().unwrap_or(0);
                let exit = exit_index.get(&node).copied().unwrap_or(enter);
                let depth = depth_of.get(&node).copied().unwrap_or(0);
                units.push(PaintUnit {
                    z,
                    layer,
                    tree_index: enter,
                    order: PaintUnit::CLIP_PUSH_ORDER,
                    commands: vec![PaintCommand::PushClip(clip)],
                });
                units.push(PaintUnit {
                    z,
                    layer,
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
                let layer = layermap.get(&node).copied().unwrap_or(PaintLayer::NonPositioned);
                let enter = enter_index.get(&node).copied().unwrap_or(0);
                let exit = exit_index.get(&node).copied().unwrap_or(enter);
                let depth = depth_of.get(&node).copied().unwrap_or(0);
                units.push(PaintUnit {
                    z,
                    layer,
                    tree_index: enter,
                    order: PaintUnit::LAYER_PUSH_ORDER,
                    commands: vec![PaintCommand::PushLayer(
                        SceneLayer::new().with_opacity(alpha),
                    )],
                });
                units.push(PaintUnit {
                    z,
                    layer,
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
                let layer = layermap.get(&node).copied().unwrap_or(PaintLayer::NonPositioned);
                let enter = enter_index.get(&node).copied().unwrap_or(0);
                let exit = exit_index.get(&node).copied().unwrap_or(enter);
                let depth = depth_of.get(&node).copied().unwrap_or(0);
                units.push(PaintUnit {
                    z,
                    layer,
                    tree_index: enter,
                    order: PaintUnit::TRANSFORM_PUSH_ORDER,
                    commands: vec![PaintCommand::PushTransform(affine)],
                });
                units.push(PaintUnit {
                    z,
                    layer,
                    tree_index: exit,
                    order: PaintUnit::transform_pop_order(depth),
                    commands: vec![PaintCommand::PopTransform],
                });
            }
            // Blur group layers are the OUTERMOST subtree wrapper: their push sorts
            // before every other bracket (BLUR_PUSH_ORDER < TRANSFORM_PUSH_ORDER) and
            // — emitted LAST here — their pop ties are broken to close after the
            // transform/opacity/clip pops, so the blur wraps the fully-composited
            // element. The clip is the node's border box, the blur region the
            // renderer's owned gaussian pass operates over.
            for (node, bounds, radius, color_matrix) in blur_brackets {
                let z = zmap.get(&node).copied().unwrap_or(0);
                let paint_layer = layermap.get(&node).copied().unwrap_or(PaintLayer::NonPositioned);
                let enter = enter_index.get(&node).copied().unwrap_or(0);
                let exit = exit_index.get(&node).copied().unwrap_or(enter);
                let depth = depth_of.get(&node).copied().unwrap_or(0);
                let mut layer = SceneLayer::new().with_clip(Clip::rect(bounds));
                // Blur and color-matrix can coexist on ONE filter layer; a zero
                // radius carries no blur effect (the color-only path). The renderer
                // applies the color matrix, then the gaussian, then composites.
                if radius > 0.0 {
                    layer.add_effect(LayerEffect::blur(radius));
                }
                if let Some(matrix) = color_matrix {
                    layer.add_effect(LayerEffect::color_matrix(matrix));
                }
                units.push(PaintUnit {
                    z,
                    layer: paint_layer,
                    tree_index: enter,
                    order: PaintUnit::BLUR_PUSH_ORDER,
                    commands: vec![PaintCommand::PushLayer(layer)],
                });
                units.push(PaintUnit {
                    z,
                    layer: paint_layer,
                    tree_index: exit,
                    order: PaintUnit::blur_pop_order(depth),
                    commands: vec![PaintCommand::PopLayer],
                });
            }
            // The outline (focus ring) paints as its OWN unit at the subtree's exit
            // index with `OUTLINE_ORDER` — sorting after every `Pop*` bracket at the
            // same index — so it lands OUTSIDE the node's rounded clip and ON TOP of
            // the node's whole subtree. Emitted LAST so a tie with a depth-0
            // `PopClip` (both at `i64::MAX`) breaks in the outline's favour (the ring
            // paints after the clip closes, unclipped).
            for (node, command) in outline_brackets {
                let z = zmap.get(&node).copied().unwrap_or(0);
                let layer = layermap.get(&node).copied().unwrap_or(PaintLayer::NonPositioned);
                let enter = enter_index.get(&node).copied().unwrap_or(0);
                let exit = exit_index.get(&node).copied().unwrap_or(enter);
                units.push(PaintUnit {
                    z,
                    layer,
                    tree_index: exit,
                    order: PaintUnit::OUTLINE_ORDER,
                    commands: vec![command],
                });
            }
            // A scroll container's vertical scrollbar (track + thumb), painted as its
            // OWN unit at the subtree exit index with `SCROLLBAR_ORDER` (the ceiling)
            // and emitted LAST, so it sorts after the scroll transform's
            // `PopTransform` — it does NOT scroll — and after the overflow clip's
            // `PopClip`, landing on top of the container's whole scrolled subtree
            // (CSS Overflow 3 §scrollbars). Its rects lie within the scrollport, so
            // being outside the clip is visually identical to being inside it.
            for (node, scrollport, scroll_top, content_height) in scrollbar_units {
                let commands = scrollbar_commands(scrollport, scroll_top, content_height);
                if commands.is_empty() {
                    continue;
                }
                let z = zmap.get(&node).copied().unwrap_or(0);
                let layer = layermap.get(&node).copied().unwrap_or(PaintLayer::NonPositioned);
                let exit = exit_index
                    .get(&node)
                    .copied()
                    .or_else(|| enter_index.get(&node).copied())
                    .unwrap_or(0);
                units.push(PaintUnit {
                    z,
                    layer,
                    tree_index: exit,
                    order: PaintUnit::SCROLLBAR_ORDER,
                    commands,
                });
            }
        }

        // Stable sort by `(z, layer, tree_index, order)`, all ascending (CSS 2.1
        // §9.9 / Appendix E painting order): higher z paints last (on top); within
        // one z bucket the §9.9 `layer` rank lifts a positioned `z:auto`/`0` box
        // (`PositionedZeroOrAuto`) above every non-positioned unit (`NonPositioned`)
        // regardless of tree order — the fix for bug B2; equal-(z, layer) units keep
        // pre-order (document) order; and `order` phases a subtree's `PushClip`
        // before the node's paint and its `PopClip` after the whole subtree. When
        // nothing is positioned every `layer` is `NonPositioned` and, with no radius,
        // every `order` is `PAINT_ORDER`, so the key reduces to `(z, tree_index)`,
        // reproducing the pre-fix pre-order sequence byte-for-byte.
        units.sort_by_key(|unit| (unit.z, unit.layer, unit.tree_index, unit.order));

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

/// Resolves the used `text-overflow` for an anonymous text `node` (CSS UI 3
/// §5.2).
///
/// `text-overflow` applies to the block container and does **not** inherit, so
/// the anonymous text node's own computed value is always the initial `clip`.
/// The governing value is the block container's, read from the text node's parent
/// (falling back to the node itself). This mirrors the layout coordinator so the
/// compose pass truncates identically to the measure pass. (Nested inline boxes
/// are not walked — a documented approximation of the flat text model.)
fn overflow_for_text(tree: &RetainedTree, computed: &ComputedStyles, node: NodeId) -> TextOverflow {
    let container = tree
        .get(node)
        .and_then(|n| n.logical_parent().parent())
        .unwrap_or(node);
    overflow_for(computed.get(&container))
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

/// Reads a node's computed multi-layer `background-image` list (two or more
/// comma-separated layers), in CSS author order (first = top), or [`None`] when
/// the node has a single flat `background-image` (a lone gradient/url) or none.
///
/// The single-layer case is deliberately *not* reported here — it stays on the
/// flat gradient/url paint path so a one-layer background is byte-identical.
fn background_layers_of(computed: &ComputedStyles, node: NodeId) -> Option<&[BackgroundLayer]> {
    computed
        .get(&node)
        .and_then(|style| style.get(PropertyId::BackgroundImage))
        .and_then(ComputedValue::as_background_layers)
}

/// Fills a node's border box with `brush`, choosing a rounded-rect outline when
/// the node has a positive corner radius and a plain rectangle otherwise —
/// the shared fill used by both the single background path and each background
/// layer.
fn fill_background_box(
    paint: &mut DisplayListBuilder,
    bounds: Rect,
    radii: CornerRadii,
    has_radius: bool,
    brush: Brush,
) {
    if has_radius {
        paint.fill(Path::rounded_rect(bounds, radii), brush);
    } else {
        paint.fill_rect(bounds, brush);
    }
}

/// Paints one `background-image` [`BackgroundLayer`] over a node's border box
/// `bounds`: a gradient fills the box (rounded when `has_radius`), a `url(...)`
/// layer draws its resolved image with the node's `background-size`/`-position`/
/// `-repeat` geometry (spec §5.2).
///
/// A stop-less gradient (never produced by the CSS lowering) and a `url(...)`
/// whose image is absent from `images` (unloaded / failed) both paint nothing,
/// letting the layers below show through.
#[allow(clippy::too_many_arguments)]
fn paint_background_layer(
    paint: &mut DisplayListBuilder,
    layer: &BackgroundLayer,
    computed: &ComputedStyles,
    node: NodeId,
    bounds: Rect,
    radii: CornerRadii,
    has_radius: bool,
    images: &HashMap<Box<str>, SceneImage>,
) {
    match layer {
        BackgroundLayer::Linear(gradient) if !gradient.stops().is_empty() => {
            let brush = Brush::Linear(linear_gradient_brush(gradient, bounds));
            fill_background_box(paint, bounds, radii, has_radius, brush);
        }
        BackgroundLayer::Radial(gradient) if !gradient.stops().is_empty() => {
            let brush = Brush::Radial(radial_gradient_brush(gradient, bounds));
            fill_background_box(paint, bounds, radii, has_radius, brush);
        }
        BackgroundLayer::Conic(gradient) if !gradient.stops().is_empty() => {
            let brush = Brush::Conic(conic_gradient_brush(gradient, bounds));
            fill_background_box(paint, bounds, radii, has_radius, brush);
        }
        BackgroundLayer::Url(url) => {
            if let Some(image) = images.get(&**url) {
                let (tile, repeat_x, repeat_y) =
                    background_image_geometry(computed, node, image, bounds);
                paint.draw_image(image.clone(), tile, bounds, repeat_x, repeat_y);
            }
        }
        // A stop-less gradient paints nothing.
        BackgroundLayer::Linear(_) | BackgroundLayer::Radial(_) | BackgroundLayer::Conic(_) => {}
    }
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

/// Reads a node's computed `background-size`, defaulting to the CSS initial
/// `auto` when absent (an unauthored / unresolved property).
fn background_size_of(computed: &ComputedStyles, node: NodeId) -> BackgroundSize {
    computed
        .get(&node)
        .and_then(|style| style.get(PropertyId::BackgroundSize))
        .and_then(ComputedValue::as_background_size)
        .unwrap_or(BackgroundSize::Auto)
}

/// Reads a node's computed `background-position`, defaulting to the CSS initial
/// `0% 0%` (top-left) when absent.
fn background_position_of(computed: &ComputedStyles, node: NodeId) -> BackgroundPosition {
    computed
        .get(&node)
        .and_then(|style| style.get(PropertyId::BackgroundPosition))
        .and_then(ComputedValue::as_background_position)
        .unwrap_or_default()
}

/// Reads a node's computed `background-repeat`, defaulting to the CSS initial
/// `repeat` (tile both axes) when absent.
fn background_repeat_of(computed: &ComputedStyles, node: NodeId) -> BackgroundRepeat {
    computed
        .get(&node)
        .and_then(|style| style.get(PropertyId::BackgroundRepeat))
        .and_then(ComputedValue::as_background_repeat)
        .unwrap_or(BackgroundRepeat::Repeat)
}

/// Resolves the CSS background placement geometry for a node's resolved image
/// into `(tile, repeat_x, repeat_y)` — the fields the
/// [`DrawImage`](obscur_scene::command::PaintCommand::DrawImage) command carries
/// (spec §5.2, CSS Backgrounds §3).
///
/// The `tile` is one copy of the image: its size comes from `background-size`
/// against the image's intrinsic pixel size and the paint `region` (the border
/// box), and its origin comes from `background-position` (the CSS position
/// formula). `repeat_x` / `repeat_y` are the `background-repeat` axis flags.
fn background_image_geometry(
    computed: &ComputedStyles,
    node: NodeId,
    image: &SceneImage,
    region: Rect,
) -> (Rect, bool, bool) {
    #[allow(clippy::cast_precision_loss)]
    let (iw, ih) = (image.width() as f32, image.height() as f32);
    let bw = region.size.width;
    let bh = region.size.height;
    let size = background_size_of(computed, node);
    let position = background_position_of(computed, node);
    let repeat = background_repeat_of(computed, node);
    let (tw, th) = resolve_background_tile_size(size, iw, ih, bw, bh);
    let ox = region.origin.x + resolve_background_position(position.x(), bw, tw);
    let oy = region.origin.y + resolve_background_position(position.y(), bh, th);
    (
        Rect::new(Point::new(ox, oy), Size::new(tw, th)),
        repeat.repeat_x(),
        repeat.repeat_y(),
    )
}

/// Reads a retained element node's own image `src` (an `<img>` / `Image`
/// primitive's source), if any — the key the runtime resolves against its image
/// cache.
///
/// Returns [`None`] for a node with no retained record or no `image_source`
/// attribute (every non-image element and every text node).
fn element_image_of(tree: &RetainedTree, node: NodeId) -> Option<&str> {
    tree.get(node).and_then(|n| n.attributes().image_source())
}

/// Reads a node's computed `object-fit`, defaulting to the CSS initial `fill`
/// (stretch to the box) when absent.
fn object_fit_of(computed: &ComputedStyles, node: NodeId) -> ObjectFit {
    computed
        .get(&node)
        .and_then(|style| style.get(PropertyId::ObjectFit))
        .and_then(ComputedValue::as_object_fit)
        .unwrap_or(ObjectFit::Fill)
}

/// Resolves the placed tile rect for an element's image `content` within its box
/// `region`, per `object-fit`, centered (the `object-position: 50% 50%` default).
///
/// The tile size comes from [`resolve_object_fit_size`] against the image's
/// intrinsic pixel size and the box size; the tile is then centered within the
/// region so `contain` letterboxes and `cover` overflows symmetrically (the
/// `region` passed to `DrawImage` clips the overflow).
#[allow(clippy::cast_precision_loss)]
fn object_fit_tile(
    computed: &ComputedStyles,
    node: NodeId,
    image: &SceneImage,
    region: Rect,
) -> Rect {
    let fit = object_fit_of(computed, node);
    let (iw, ih) = (image.width() as f32, image.height() as f32);
    let bw = region.size.width;
    let bh = region.size.height;
    let (tw, th) = resolve_object_fit_size(fit, iw, ih, bw, bh);
    // Center the tile (object-position default 50% 50%): (box - tile) / 2 per axis.
    let ox = region.origin.x + (bw - tw) / 2.0;
    let oy = region.origin.y + (bh - th) / 2.0;
    Rect::new(Point::new(ox, oy), Size::new(tw, th))
}

/// Resolves the drawn image tile width/height (logical px) from `object-fit`, the
/// image's intrinsic size `(iw, ih)`, and the box size `(bw, bh)` — the replaced-
/// element analog of [`resolve_background_tile_size`].
///
/// * `fill` → the box size (stretch, aspect ignored — the CSS initial);
/// * `contain` → the largest aspect-preserving size that fits (min scale);
/// * `cover` → the smallest aspect-preserving size that covers (max scale);
/// * `none` → the intrinsic size (unscaled);
/// * `scale-down` → `min(none, contain)` (intrinsic, shrunk only if it overflows).
///
/// A degenerate intrinsic size (`iw`/`ih` ≤ 0) cannot preserve an aspect ratio, so
/// it falls back to a full-box stretch (matching the background path).
fn resolve_object_fit_size(fit: ObjectFit, iw: f32, ih: f32, bw: f32, bh: f32) -> (f32, f32) {
    if iw <= 0.0 || ih <= 0.0 {
        return (bw, bh);
    }
    match fit {
        ObjectFit::Fill => (bw, bh),
        ObjectFit::Contain => {
            let scale = (bw / iw).min(bh / ih);
            (iw * scale, ih * scale)
        }
        ObjectFit::Cover => {
            let scale = (bw / iw).max(bh / ih);
            (iw * scale, ih * scale)
        }
        ObjectFit::None => (iw, ih),
        ObjectFit::ScaleDown => {
            // min(none, contain): the contain scale, but never larger than 1.
            let scale = (bw / iw).min(bh / ih).min(1.0);
            (iw * scale, ih * scale)
        }
    }
}

/// Resolves the drawn tile width/height (logical px) from `background-size`, the
/// image's intrinsic size `(iw, ih)`, and the box size `(bw, bh)`.
///
/// * `auto` → intrinsic pixel size; `cover` → the smallest size that fully covers
///   the box (max scale, aspect preserved); `contain` → the largest that fits
///   (min scale, aspect preserved); explicit → each axis resolved against the box,
///   a single `auto` axis derived from the other to preserve the intrinsic aspect
///   ratio (both `auto` = intrinsic). A degenerate intrinsic size falls back to a
///   full-box stretch.
fn resolve_background_tile_size(
    size: BackgroundSize,
    iw: f32,
    ih: f32,
    bw: f32,
    bh: f32,
) -> (f32, f32) {
    // A degenerate intrinsic size cannot preserve an aspect ratio; stretch to fill.
    if iw <= 0.0 || ih <= 0.0 {
        return (bw, bh);
    }
    match size {
        BackgroundSize::Auto => (iw, ih),
        BackgroundSize::Cover => {
            let scale = (bw / iw).max(bh / ih);
            (iw * scale, ih * scale)
        }
        BackgroundSize::Contain => {
            let scale = (bw / iw).min(bh / ih);
            (iw * scale, ih * scale)
        }
        BackgroundSize::Explicit { width, height } => {
            let w = resolve_background_size_length(width, bw);
            let h = resolve_background_size_length(height, bh);
            match (w, h) {
                (Some(w), Some(h)) => (w, h),
                // A single `auto` axis preserves the intrinsic aspect ratio.
                (Some(w), None) => (w, w * ih / iw),
                (None, Some(h)) => (h * iw / ih, h),
                (None, None) => (iw, ih),
            }
        }
    }
}

/// Resolves one explicit `background-size` component against a box extent: an
/// absolute `px` directly, a `Percent` as a fraction of the box extent. `auto`
/// (and any context-dependent unit, a documented approximation) yields [`None`]
/// so the axis is derived from the other.
fn resolve_background_size_length(length: Length, basis: f32) -> Option<f32> {
    match length {
        Length::Px(px) => Some(px.0),
        Length::Percent(pct) => Some(pct / 100.0 * basis),
        _ => None,
    }
}

/// Resolves one `background-position` component to the tile's origin offset from
/// the box edge, following the CSS position formula: an absolute `px` is that
/// offset directly; a `Percent` aligns the image's `pct` point to the box's `pct`
/// point, i.e. `(box_extent − tile_extent) × pct`. Any other (context-dependent)
/// length falls back to `0` (the top/left edge, a documented approximation).
fn resolve_background_position(length: Length, box_extent: f32, tile_extent: f32) -> f32 {
    match length {
        Length::Px(px) => px.0,
        Length::Percent(pct) => (box_extent - tile_extent) * pct / 100.0,
        _ => 0.0,
    }
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

/// Reads a node's computed ordered `filter` op list, if present.
///
/// Returns [`None`] when the node has no computed style, no `filter` property, or
/// a non-filter value — the common (unauthored) case, which emits nothing.
/// `filter: none` (and a filter with only out-of-scope functions) is an **empty**
/// list. Each entry is a [`FilterOp`]: a drop-shadow (painted behind the node like
/// a `box-shadow` drop shadow) or a `blur(<radius>)` content blur.
fn filter_ops_of(computed: &ComputedStyles, node: NodeId) -> Option<&[FilterOp]> {
    computed
        .get(&node)
        .and_then(|style| style.get(PropertyId::Filter))
        .and_then(ComputedValue::as_filter_ops)
}

/// The effective `filter` blur radius (logical px) bracketing a node's subtree, or
/// [`None`] when no positive-radius `blur()` op is authored (the byte-identical
/// no-blur path).
///
/// A `filter` may list several `blur()` ops; CSS applies them sequentially, but
/// combining separable gaussians is out of scope here, so the **maximum** authored
/// radius is used (multiple blurs on one element are uncommon; documented).
fn filter_blur_of(computed: &ComputedStyles, node: NodeId) -> Option<f32> {
    let radius = filter_ops_of(computed, node)?
        .iter()
        .filter_map(FilterOp::as_blur)
        .fold(0.0_f32, f32::max);
    (radius > 0.0).then_some(radius)
}

/// The node's `filter` color-op chain folded into ONE 4×5 affine color matrix, or
/// [`None`] when it authors no color op (the byte-identical no-color path — no
/// `ColorMatrix` effect is emitted).
///
/// See [`color_matrix_of_ops`] for the composition. `blur()` / `drop-shadow()` are
/// spatial, not color, and are ignored here (they keep their own passes).
fn filter_color_matrix_of(computed: &ComputedStyles, node: NodeId) -> Option<[f32; 20]> {
    color_matrix_of_ops(filter_ops_of(computed, node)?)
}

/// Folds a `filter` op list's **color** ops (CSS Filter Effects §) into a single
/// 4×5 affine color matrix, applied in author order, or [`None`] when the list has
/// no color op (so the coordinator emits **no** `ColorMatrix` effect — byte
/// identical to no filter).
///
/// Every color function is an affine transform `rgba' = M·rgba + bias`, so a chain
/// composes by matrix multiply: applying `(Mₙ, bₙ)` after the running `(M, b)`
/// gives `M' = Mₙ·M`, `b' = Mₙ·b + bₙ`. Starting from the identity and folding
/// left-to-right yields one matrix equivalent to the whole chain. The result is
/// flattened row-major: `arr[r*4+c]` is the 4×4 linear entry, `arr[16+r]` the bias.
fn color_matrix_of_ops(ops: &[FilterOp]) -> Option<[f32; 20]> {
    if !ops.iter().any(FilterOp::is_color_op) {
        return None;
    }
    // Running affine (linear 4×4 `m`, bias `b`), identity to start.
    let mut m: [[f32; 4]; 4] = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];
    let mut b: [f32; 4] = [0.0; 4];
    for op in ops {
        let Some((om, ob)) = color_op_matrix(op) else {
            continue; // spatial op (blur / drop-shadow) — not a color transform
        };
        // Compose op AFTER the running affine: m ← om·m, b ← om·b + ob.
        m = mat_mul(&om, &m);
        b = add4(mat_vec(&om, b), ob);
    }
    let mut out = [0.0_f32; 20];
    for (r, row) in m.iter().enumerate() {
        out[r * 4..r * 4 + 4].copy_from_slice(row);
    }
    out[16..20].copy_from_slice(&b);
    Some(out)
}

/// The 4×4 linear part + 4-entry bias of a single color-op [`FilterOp`], or
/// [`None`] for a spatial op (`blur` / `drop-shadow`). Matrices are taken verbatim
/// from CSS Filter Effects § (<https://drafts.fxtf.org/filter-effects/>).
fn color_op_matrix(op: &FilterOp) -> Option<([[f32; 4]; 4], [f32; 4])> {
    // A diagonal RGB scale (alpha untouched) with an RGB bias.
    let rgb_diag = |k: f32, bias: f32| {
        (
            [
                [k, 0.0, 0.0, 0.0],
                [0.0, k, 0.0, 0.0],
                [0.0, 0.0, k, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            [bias, bias, bias, 0.0],
        )
    };
    // A 3×3 RGB matrix (alpha untouched, zero bias).
    let rgb_3x3 = |r: [f32; 3], g: [f32; 3], b: [f32; 3]| {
        (
            [
                [r[0], r[1], r[2], 0.0],
                [g[0], g[1], g[2], 0.0],
                [b[0], b[1], b[2], 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            [0.0; 4],
        )
    };
    Some(match *op {
        // RGB *= b.
        FilterOp::Brightness(b) => rgb_diag(b, 0.0),
        // RGB' = (RGB − 0.5)·c + 0.5  ⇒  diagonal c, bias 0.5·(1 − c).
        FilterOp::Contrast(c) => rgb_diag(c, 0.5 * (1.0 - c)),
        // A *= o (alpha-only diagonal).
        FilterOp::Opacity(o) => (
            [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, o],
            ],
            [0.0; 4],
        ),
        // channel' = (1 − 2i)·channel + i, per RGB.
        FilterOp::Invert(i) => rgb_diag(1.0 - 2.0 * i, i),
        // Saturate: the spec luminance saturation matrix (0.213/0.715/0.072).
        FilterOp::Saturate(s) => rgb_3x3(
            [0.213 + 0.787 * s, 0.715 - 0.715 * s, 0.072 - 0.072 * s],
            [0.213 - 0.213 * s, 0.715 + 0.285 * s, 0.072 - 0.072 * s],
            [0.213 - 0.213 * s, 0.715 - 0.715 * s, 0.072 + 0.928 * s],
        ),
        // Grayscale: interpolate toward the luma matrix (0.2126/0.7152/0.0722)
        // with t = 1 − amount; t == 0 (grayscale(1)) is the pure luma matrix.
        FilterOp::Grayscale(a) => {
            let t = 1.0 - a;
            rgb_3x3(
                [0.2126 + 0.7874 * t, 0.7152 - 0.7152 * t, 0.0722 - 0.0722 * t],
                [0.2126 - 0.2126 * t, 0.7152 + 0.2848 * t, 0.0722 - 0.0722 * t],
                [0.2126 - 0.2126 * t, 0.7152 - 0.7152 * t, 0.0722 + 0.9278 * t],
            )
        }
        // Sepia: the spec sepia matrix interpolated with t = 1 − amount.
        FilterOp::Sepia(a) => {
            let t = 1.0 - a;
            rgb_3x3(
                [0.393 + 0.607 * t, 0.769 - 0.769 * t, 0.189 - 0.189 * t],
                [0.349 - 0.349 * t, 0.686 + 0.314 * t, 0.168 - 0.168 * t],
                [0.272 - 0.272 * t, 0.534 - 0.534 * t, 0.131 + 0.869 * t],
            )
        }
        // Hue-rotate: the spec hue-rotation matrix built from cos θ / sin θ and the
        // luminance constants (θ in radians).
        FilterOp::HueRotate(theta) => {
            let (s, c) = theta.sin_cos();
            rgb_3x3(
                [
                    0.213 + c * 0.787 - s * 0.213,
                    0.715 - c * 0.715 - s * 0.715,
                    0.072 - c * 0.072 + s * 0.928,
                ],
                [
                    0.213 - c * 0.213 + s * 0.143,
                    0.715 + c * 0.285 + s * 0.140,
                    0.072 - c * 0.072 - s * 0.283,
                ],
                [
                    0.213 - c * 0.213 - s * 0.787,
                    0.715 - c * 0.715 + s * 0.715,
                    0.072 + c * 0.928 + s * 0.072,
                ],
            )
        }
        // Spatial ops are not color transforms.
        FilterOp::Blur(_) | FilterOp::DropShadow(_) => return None,
    })
}

/// Row-major 4×4 · 4×4 product (`a · b`).
fn mat_mul(a: &[[f32; 4]; 4], b: &[[f32; 4]; 4]) -> [[f32; 4]; 4] {
    let mut out = [[0.0_f32; 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            let mut sum = 0.0;
            for k in 0..4 {
                sum += a[i][k] * b[k][j];
            }
            out[i][j] = sum;
        }
    }
    out
}

/// Row-major 4×4 · vec4 product (`m · v`).
fn mat_vec(m: &[[f32; 4]; 4], v: [f32; 4]) -> [f32; 4] {
    let mut out = [0.0_f32; 4];
    for i in 0..4 {
        out[i] = m[i][0] * v[0] + m[i][1] * v[1] + m[i][2] * v[2] + m[i][3] * v[3];
    }
    out
}

/// Component-wise vec4 sum.
fn add4(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2], a[3] + b[3]]
}

/// Reads a node's computed `backdrop-filter` blur radius (logical px), or
/// [`None`] when no positive-radius `blur()` is authored (the byte-identical
/// no-effect path).
///
/// `backdrop-filter` reuses the [`PropertyValue::Filter`](obscur_style::property::PropertyValue::Filter)
/// op list (only `blur()` ops survive its lowering). A value may list several
/// `blur()` ops; the **maximum** authored radius is used (multiple backdrop blurs
/// on one element are uncommon; documented). The coordinator emits this radius as a
/// [`BackdropFilter`](obscur_scene::command::PaintCommand::BackdropFilter) command
/// frosting the backdrop under the node's border box.
fn backdrop_blur_of(computed: &ComputedStyles, node: NodeId) -> Option<f32> {
    let radius = computed
        .get(&node)
        .and_then(|style| style.get(PropertyId::BackdropFilter))
        .and_then(ComputedValue::as_filter_ops)?
        .iter()
        .filter_map(FilterOp::as_blur)
        .fold(0.0_f32, f32::max);
    (radius > 0.0).then_some(radius)
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
/// The direction defines the gradient *line*: a line through the box center, with
/// its `0%` / `100%` endpoints at the projections onto that line of the box extent
/// it covers. The endpoints are `center ∓ dir · half_len` where `dir` is the unit
/// gradient-line vector (screen space, y down) and, per the note under
/// <https://drafts.csswg.org/css-images-3/#linear-gradient-syntax>, the gradient
/// line length is `|w·dir_x| + |h·dir_y|` (so `half_len` is half of that). Two
/// direction forms feed `dir`:
///
/// * an explicit `<angle>` (CSS `0` = to top, increasing clockwise) → `dir =
///   (sin θ, −cos θ)`, giving the classic `|w·sin θ| + |h·cos θ|` line length;
/// * a `to <corner>` → the **aspect-ratio-aware** direction (spec §3.1): the line
///   points into the corner's quadrant and is perpendicular to the diagonal
///   joining the two neighbouring corners, i.e. `dir = normalize(±h, ∓w)`. This is
///   *not* a fixed 45° except on a square box.
///
/// Each style stop's authored (resolved) `0.0..=1.0` position becomes the scene
/// stop's offset, and its [`StyleColor`] becomes the scene paint [`Color`]; an
/// unpositioned stop (never produced by the CSS lowering, which resolves all
/// positions) falls back to its index-proportional offset.
///
/// A non-repeating gradient keeps the default
/// [`ExtendMode::Pad`](obscur_scene::paint::ExtendMode::Pad) and its axis spans
/// the whole box (the `0%`/`100%` endpoints at `center ∓ dir · half_len`). A
/// [`repeating`](LinearGradient::repeating) gradient instead sizes its axis to
/// **one period** (the axis runs from the box's `0%` anchor a distance of one
/// period along `dir`, with the stops already normalized to `[0, 1]` over that
/// period) and uses [`ExtendMode::Repeat`](obscur_scene::paint::ExtendMode::Repeat)
/// so the renderer tiles the pattern across the box. The period is either an
/// absolute px length or a fraction of the full gradient line
/// (`2 · half_len`). On Vello this tiles as a true repeating ramp; on the CPU
/// path a repeated gradient flattens to its first stop (the documented
/// GPU-class-effect policy), so a stripe reads as a flat first-stop fill there.
fn linear_gradient_brush(gradient: &LinearGradient, bounds: Rect) -> SceneLinearGradient {
    let w = bounds.size.width;
    let h = bounds.size.height;
    let cx = bounds.origin.x + w / 2.0;
    let cy = bounds.origin.y + h / 2.0;
    // Unit direction of increasing gradient offset (0% → 100%), in screen space
    // (y down). A `to <corner>` uses the aspect-ratio-aware direction (spec §3.1);
    // otherwise the angle's (sin θ, −cos θ).
    let (dir_x, dir_y) = gradient.corner().map_or_else(
        || {
            let theta = gradient.angle_deg().to_radians();
            (theta.sin(), -theta.cos())
        },
        |corner| corner_direction(corner, w, h),
    );
    // Gradient-line length = |w·dir_x| + |h·dir_y| (spec §3.1 projection note),
    // valid for any unit `dir` (reduces to |w·sin θ| + |h·cos θ| for an angle).
    let half_len = (w * dir_x).abs().mul_add(0.5, (h * dir_y).abs() * 0.5);
    let start = Point::new(cx - dir_x * half_len, cy - dir_y * half_len);
    let stops = gradient_stops(gradient.stops());
    // A repeating gradient's axis spans exactly one period from the box's 0%
    // anchor, and the pattern tiles with `ExtendMode::Repeat`.
    if let Some(period) = gradient.period() {
        let line_len = 2.0 * half_len;
        let period_px = match period {
            Length::Px(px) => px.0,
            Length::Percent(pct) => pct / 100.0 * line_len,
            _ => line_len,
        };
        let end = Point::new(start.x + dir_x * period_px, start.y + dir_y * period_px);
        return SceneLinearGradient::new(start, end, stops).with_extend(ExtendMode::Repeat);
    }
    let end = Point::new(cx + dir_x * half_len, cy + dir_y * half_len);
    SceneLinearGradient::new(start, end, stops)
}

/// The unit direction of a `to <corner>` gradient line over a `w × h` box, in
/// screen space (y down) (spec §3.1).
///
/// The gradient line points into the corner's quadrant and is perpendicular to the
/// diagonal joining the two neighbouring corners; that diagonal is `(w, h)`, so the
/// perpendicular pointing right is `(h, −w)` and each corner picks the signs that
/// aim into its quadrant (`+x` = right, `−y` = up). On a degenerate (zero-area)
/// box the direction falls back to straight down.
fn corner_direction(corner: Corner, w: f32, h: f32) -> (f32, f32) {
    let (sx, sy) = match corner {
        Corner::TopRight => (h, -w),
        Corner::BottomRight => (h, w),
        Corner::BottomLeft => (-h, w),
        Corner::TopLeft => (-h, -w),
    };
    let len = sx.hypot(sy);
    if len == 0.0 {
        (0.0, 1.0)
    } else {
        (sx / len, sy / len)
    }
}

/// Builds a scene [`SceneRadialGradient`] fill brush for a style
/// [`RadialGradient`] over the box `bounds` (spec §5.2, sizing §3.2).
///
/// The box-relative center (`center_x` / `center_y`) resolves to an absolute point
/// against the box; the ending [`shape`](RadialShape) + [`size`](RadialSize)
/// resolve to a **per-axis** radius pair `(rx, ry)` via [`resolve_radii`]. A
/// `circle` yields `rx == ry`; an `ellipse` yields independent radii (e.g.
/// `farthest-corner` on a non-square box gives `rx ≠ ry`), carried faithfully by
/// the scene [`SceneRadialGradient`] so Vello stretches the concentric circle into
/// the ellipse. Each style stop's resolved `0.0..=1.0` position becomes the scene
/// stop's offset; the brush keeps the default
/// [`ExtendMode::Pad`](obscur_scene::paint::ExtendMode::Pad).
fn radial_gradient_brush(gradient: &RadialGradient, bounds: Rect) -> SceneRadialGradient {
    let cx = resolve_center(gradient.center_x(), bounds.origin.x, bounds.size.width);
    let cy = resolve_center(gradient.center_y(), bounds.origin.y, bounds.size.height);
    let (rx, ry) = resolve_radii(gradient.shape(), gradient.size(), cx, cy, bounds);
    SceneRadialGradient::new_elliptical(
        Point::new(cx, cy),
        rx,
        ry,
        gradient_stops(gradient.stops()),
    )
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

/// Resolves a [`RadialGradient`]'s ending [`shape`](RadialShape) +
/// [`size`](RadialSize) to a per-axis radius pair `(rx, ry)`, given the gradient's
/// absolute center `(cx, cy)` and the box `bounds` (CSS Images §3.2).
///
/// Let the center's distances to the four sides give `dx_near`/`dx_far` (nearest /
/// farthest vertical side) and `dy_near`/`dy_far` (nearest / farthest horizontal
/// side). Then, per the spec's sizing rules:
///
/// * **circle** collapses to one radius `r` (`rx == ry`): `closest-side` =
///   `min(dx_near, dy_near)`, `farthest-side` = `max(dx_far, dy_far)`,
///   `closest-corner` = `hypot(dx_near, dy_near)`, `farthest-corner` =
///   `hypot(dx_far, dy_far)`;
/// * **ellipse** keeps independent axes: `closest-side` = `(dx_near, dy_near)`,
///   `farthest-side` = `(dx_far, dy_far)`. For the *corner* extents the ellipse
///   keeps the corresponding side ellipse's aspect ratio while passing through the
///   corner, which (because that corner sits at exactly the side distances on both
///   axes) scales each side radius by `√2`: `closest-corner` =
///   `(dx_near, dy_near)·√2`, `farthest-corner` = `(dx_far, dy_far)·√2`.
///
/// An explicit length is a concrete radius: `px` directly, a percentage relative to
/// the corresponding box dimension (spec: radii percentages resolve against the
/// gradient box's width for `rx` and height for `ry`); for a circle the single
/// length applies to both axes (a percentage resolves against the box width). Any
/// other length falls back to `farthest-corner`.
fn resolve_radii(
    shape: RadialShape,
    size: RadialSize,
    cx: f32,
    cy: f32,
    bounds: Rect,
) -> (f32, f32) {
    const SQRT2: f32 = std::f32::consts::SQRT_2;
    let (left, top, right, bottom) =
        (bounds.origin.x, bounds.origin.y, bounds.right(), bounds.bottom());
    let dx_near = (cx - left).abs().min((right - cx).abs());
    let dx_far = (cx - left).abs().max((right - cx).abs());
    let dy_near = (cy - top).abs().min((bottom - cy).abs());
    let dy_far = (cy - top).abs().max((bottom - cy).abs());
    let (w, h) = (bounds.size.width, bounds.size.height);
    match shape {
        RadialShape::Circle => {
            let r = match size {
                RadialSize::ClosestSide => dx_near.min(dy_near),
                RadialSize::FarthestSide => dx_far.max(dy_far),
                RadialSize::ClosestCorner => dx_near.hypot(dy_near),
                RadialSize::FarthestCorner => dx_far.hypot(dy_far),
                RadialSize::Length(Length::Px(px)) => px.0,
                RadialSize::Length(Length::Percent(pct)) => pct / 100.0 * w,
                RadialSize::Length(_) => dx_far.hypot(dy_far),
            };
            (r, r)
        }
        RadialShape::Ellipse => match size {
            RadialSize::ClosestSide => (dx_near, dy_near),
            RadialSize::FarthestSide => (dx_far, dy_far),
            RadialSize::ClosestCorner => (dx_near * SQRT2, dy_near * SQRT2),
            RadialSize::FarthestCorner => (dx_far * SQRT2, dy_far * SQRT2),
            RadialSize::Length(len) => {
                (resolve_axis_radius(len, w), resolve_axis_radius(len, h))
            }
        },
    }
}

/// Resolves an explicit radius [`Length`] against a box axis of length `extent`:
/// an absolute `px` directly, a percentage as that fraction of `extent`; any other
/// (context-dependent) length falls back to the full `extent`.
fn resolve_axis_radius(len: Length, extent: f32) -> f32 {
    match len {
        Length::Px(px) => px.0,
        Length::Percent(pct) => pct / 100.0 * extent,
        _ => extent,
    }
}

/// Computes the mitered trapezoid **quads** painting a node's border, in top/
/// right/bottom/left paint order, each paired with its side color (spec CSS
/// Backgrounds and Borders §4).
///
/// A side contributes a quad only when its computed `border-*-style` is `solid`,
/// its `border-*-width` is a positive pixel length, **and** its resolved color is
/// not fully transparent; every other side (unauthored, `none`, zero-width, or a
/// transparent color) paints nothing, so a node with no authored border yields an
/// empty list (byte-identical to before borders were painted). Skipping
/// transparent sides is what makes the classic CSS border-triangle trick work.
///
/// Each side's quad is a **mitered trapezoid**: its outer edge spans the full
/// border-box side, and its inner edge is inset by the two adjacent border widths,
/// so the diagonals from each outer corner to the matching inner corner are the
/// 45°-ish miters that split every corner between its two sides. Adjacent sides
/// therefore share a corner diagonal exactly (no overlap, no gap), which is the
/// correct CSS border-corner model. When a box collapses (a `0×0` content box, so
/// only the border widths remain) a side whose outer edge shrinks to a point
/// degenerates into a **triangle** — e.g. a lone `border-bottom` yields an
/// up-pointing triangle — exactly the arrow/marker glyph the border trick draws.
/// For a normal, single-color uniform border the four trapezoids tile the same
/// band the old four rects did (visually identical; corners now mitered rather
/// than square-joined). A side's color is its `border-*-color`, falling back to
/// the node's computed `color` (CSS `currentColor`) when the color is unset.
fn border_fills(computed: &ComputedStyles, node: NodeId, bounds: Rect) -> Vec<(Path, StyleColor)> {
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
    // Outer corners (the border box) and inner corners (each inset by its two
    // adjacent border widths). A side's miter runs from an outer corner to the
    // matching inner corner.
    let outer_tl = Point::new(x, y);
    let outer_tr = Point::new(x + w, y);
    let outer_br = Point::new(x + w, y + h);
    let outer_bl = Point::new(x, y + h);
    let inner_tl = Point::new(x + left, y + top);
    let inner_tr = Point::new(x + w - right, y + top);
    let inner_br = Point::new(x + w - right, y + h - bottom);
    let inner_bl = Point::new(x + left, y + h - bottom);
    // A closed quad path from four corners, in order.
    let quad = |a: Point, b: Point, c: Point, d: Point| -> Path {
        let mut path = Path::new();
        path.move_to(a).line_to(b).line_to(c).line_to(d).close();
        path
    };
    let mut out = Vec::new();
    // Each side: outer full edge, then the two inner corners (inset by adjacent
    // widths). Emitted only when solid, positive-width, and non-transparent.
    let mut push_side = |width: f32,
                         style_id: PropertyId,
                         color_id: PropertyId,
                         corners: [Point; 4]| {
        if width > 0.0 && border_is_solid(computed, node, style_id) {
            let color = border_color_of(computed, node, color_id, current);
            // A fully transparent side paints nothing (the border-triangle trick).
            if color.a > 0.0 {
                out.push((quad(corners[0], corners[1], corners[2], corners[3]), color));
            }
        }
    };
    push_side(
        top,
        PropertyId::BorderTopStyle,
        PropertyId::BorderTopColor,
        [outer_tl, outer_tr, inner_tr, inner_tl],
    );
    push_side(
        right,
        PropertyId::BorderRightStyle,
        PropertyId::BorderRightColor,
        [outer_tr, outer_br, inner_br, inner_tr],
    );
    push_side(
        bottom,
        PropertyId::BorderBottomStyle,
        PropertyId::BorderBottomColor,
        [outer_br, outer_bl, inner_bl, inner_br],
    );
    push_side(
        left,
        PropertyId::BorderLeftStyle,
        PropertyId::BorderLeftColor,
        [outer_bl, outer_tl, inner_tl, inner_bl],
    );
    out
}

/// The single width and color of a node's border when it is **uniform**, else
/// `None`.
///
/// A border is uniform when all four sides share one positive pixel width, one
/// color, and the `solid` style. Only such a border can be painted as a single
/// even-odd rounded ring (a concentric inner rounded rect), so a non-uniform
/// border (differing widths / colors / styles, or any non-`solid` side) returns
/// `None` and keeps the per-side four-rect approximation.
fn uniform_border(computed: &ComputedStyles, node: NodeId) -> Option<(f32, StyleColor)> {
    let top = border_width_of(computed, node, PropertyId::BorderTopWidth);
    let right = border_width_of(computed, node, PropertyId::BorderRightWidth);
    let bottom = border_width_of(computed, node, PropertyId::BorderBottomWidth);
    let left = border_width_of(computed, node, PropertyId::BorderLeftWidth);
    // All four widths equal and positive.
    if !(top > 0.0 && top == right && top == bottom && top == left) {
        return None;
    }
    // Every side `solid`.
    if !(border_is_solid(computed, node, PropertyId::BorderTopStyle)
        && border_is_solid(computed, node, PropertyId::BorderRightStyle)
        && border_is_solid(computed, node, PropertyId::BorderBottomStyle)
        && border_is_solid(computed, node, PropertyId::BorderLeftStyle))
    {
        return None;
    }
    // All four colors equal (resolving `currentColor` the same way for each side).
    let current = current_color_of(computed, node);
    let ct = border_color_of(computed, node, PropertyId::BorderTopColor, current);
    let cr = border_color_of(computed, node, PropertyId::BorderRightColor, current);
    let cb = border_color_of(computed, node, PropertyId::BorderBottomColor, current);
    let cl = border_color_of(computed, node, PropertyId::BorderLeftColor, current);
    if ct == cr && ct == cb && ct == cl {
        Some((top, ct))
    } else {
        None
    }
}

/// Builds the even-odd rounded-rect ring painting a uniform border of `width`
/// around `bounds` (the border box) with authored corner `radii`.
///
/// The ring's outer outline is the border-box rounded rect; its inner outline is
/// the padding box — `bounds` inset by `width` on every side (clamped to a
/// non-negative size) — with each corner radius reduced to `max(0, r − width)`,
/// the concentric inner radius. Filled even-odd, the inner rounded rect becomes a
/// hole so the border's inner corners are rounded (spec CSS Backgrounds §4).
fn uniform_border_ring(bounds: Rect, radii: CornerRadii, width: f32) -> Path {
    let inner = Rect::new(
        Point::new(bounds.origin.x + width, bounds.origin.y + width),
        Size::new(
            (bounds.size.width - 2.0 * width).max(0.0),
            (bounds.size.height - 2.0 * width).max(0.0),
        ),
    );
    let inner_radii = CornerRadii {
        top_left: (radii.top_left - width).max(0.0),
        top_right: (radii.top_right - width).max(0.0),
        bottom_right: (radii.bottom_right - width).max(0.0),
        bottom_left: (radii.bottom_left - width).max(0.0),
    };
    Path::rounded_rect_ring(bounds, radii, inner, inner_radii)
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

/// Whether a node's computed `overflow` clips its subtree to its box.
///
/// Reads [`PropertyId::Overflow`] — a [`PropertyValue::Keyword`] backed by one of
/// the reserved [`layout_keyword`] `OVERFLOW_*` handles. Every value except
/// `visible` clips (`hidden` / `clip` / `scroll` / `auto`; the last two clip like
/// `hidden`, as no scrollbars are synthesised yet). An absent property is the CSS
/// initial `visible` — no clip, byte-for-byte identical to before overflow existed.
fn overflow_clips(computed: &ComputedStyles, node: NodeId) -> bool {
    matches!(
        computed
            .get(&node)
            .and_then(|style| style.get(PropertyId::Overflow))
            .map(ComputedValue::value),
        Some(&PropertyValue::Keyword(sym)) if sym != layout_keyword::OVERFLOW_VISIBLE
    )
}

/// Builds the paint commands for a scroll container's vertical scrollbar — a
/// track filling the right gutter of the `scrollport` plus a thumb whose length
/// and position reflect `scroll_top` and `content_height` (CSS Overflow 3
/// §scrollbars).
///
/// The thumb length is `scrollport_h · scrollport_h / content_height` (the
/// visible fraction of the content), clamped to at least [`SCROLLBAR_MIN_THUMB`],
/// and its top is `scroll_top / (content_height − scrollport_h)` of the free
/// track. Returns an empty list when the geometry is degenerate (a non-positive
/// scrollport, or content that does not exceed the scrollport), so a
/// non-overflowing container paints no scrollbar. The thumb is **not**
/// interactive-draggable in this pass — the wheel drives `scroll_top`; drag is a
/// documented follow-up.
#[must_use]
fn scrollbar_commands(scrollport: Rect, scroll_top: f32, content_height: f32) -> Vec<PaintCommand> {
    let track_h = scrollport.size.height;
    let track_w = SCROLLBAR_WIDTH.min(scrollport.size.width);
    let scroll_range = content_height - track_h;
    if track_h <= 0.0 || track_w <= 0.0 || scroll_range <= 0.0 {
        return Vec::new();
    }
    let track_x = scrollport.right() - track_w;
    let track_y = scrollport.origin.y;
    let track = Rect::new(Point::new(track_x, track_y), Size::new(track_w, track_h));

    // Thumb length = the visible fraction of the content, floored to a grabbable
    // minimum but never taller than the track.
    let thumb_h = (track_h * track_h / content_height).clamp(SCROLLBAR_MIN_THUMB.min(track_h), track_h);
    let free = (track_h - thumb_h).max(0.0);
    let thumb_y = track_y + free * (scroll_top / scroll_range).clamp(0.0, 1.0);
    let thumb = Rect::new(Point::new(track_x, thumb_y), Size::new(track_w, thumb_h));

    let mut builder = DisplayListBuilder::new();
    builder.fill_rect(track, Brush::solid(SCROLLBAR_TRACK_COLOR));
    builder.fill_rect(thumb, Brush::solid(SCROLLBAR_THUMB_COLOR));
    builder.build().commands().to_vec()
}

/// Whether a node's computed `border-*-style` (`id`) is the reserved `solid`
/// handle. An absent style (the deferred `none` initial) or any other value is
/// treated as not solid, so no edge is painted.
fn border_is_solid(computed: &ComputedStyles, node: NodeId, id: PropertyId) -> bool {
    border_style_is(computed, node, id, layout_keyword::BORDER_STYLE_SOLID)
}

/// Whether a node's computed `border-*-style` (`id`) is the reserved `dashed`
/// handle (`dotted` folds here too). Any other value — including `solid`, `none`,
/// or an absent style — is not dashed.
fn border_is_dashed(computed: &ComputedStyles, node: NodeId, id: PropertyId) -> bool {
    border_style_is(computed, node, id, layout_keyword::BORDER_STYLE_DASHED)
}

/// Whether a node's computed `border-*-style` (`id`) equals the reserved
/// `expected` keyword handle.
fn border_style_is(
    computed: &ComputedStyles,
    node: NodeId,
    id: PropertyId,
    expected: obscur_identity::InternedSymbol,
) -> bool {
    matches!(
        computed
            .get(&node)
            .and_then(|style| style.get(id))
            .map(ComputedValue::value),
        Some(&PropertyValue::Keyword(sym)) if sym == expected
    )
}

/// The single width and color of a node's border when it is **uniform** and
/// every side is **dashed** (or `dotted`, which folds to dashed), else `None`.
///
/// Mirrors [`uniform_border`] but gates on the `dashed` style: all four sides
/// share one positive pixel width, one color, and the `dashed` style. Only such a
/// border paints as a single dashed stroke; a non-uniform dashed border (mixed
/// widths / colors, or a mix with solid / no sides) returns `None` and falls back
/// to [`border_fills`] (which paints only the `solid` sides — the documented
/// non-uniform fallback, so mixed dashed sides simply do not paint).
fn uniform_dashed_border(computed: &ComputedStyles, node: NodeId) -> Option<(f32, StyleColor)> {
    let top = border_width_of(computed, node, PropertyId::BorderTopWidth);
    let right = border_width_of(computed, node, PropertyId::BorderRightWidth);
    let bottom = border_width_of(computed, node, PropertyId::BorderBottomWidth);
    let left = border_width_of(computed, node, PropertyId::BorderLeftWidth);
    if !(top > 0.0 && top == right && top == bottom && top == left) {
        return None;
    }
    if !(border_is_dashed(computed, node, PropertyId::BorderTopStyle)
        && border_is_dashed(computed, node, PropertyId::BorderRightStyle)
        && border_is_dashed(computed, node, PropertyId::BorderBottomStyle)
        && border_is_dashed(computed, node, PropertyId::BorderLeftStyle))
    {
        return None;
    }
    let current = current_color_of(computed, node);
    let ct = border_color_of(computed, node, PropertyId::BorderTopColor, current);
    let cr = border_color_of(computed, node, PropertyId::BorderRightColor, current);
    let cb = border_color_of(computed, node, PropertyId::BorderBottomColor, current);
    let cl = border_color_of(computed, node, PropertyId::BorderLeftColor, current);
    if ct == cr && ct == cb && ct == cl {
        Some((top, ct))
    } else {
        None
    }
}

/// The dash pattern (in logical pixels) for a dashed border of stroke `width`:
/// `[3·width on, 3·width off]`. CSS leaves the exact dash length implementation-
/// defined; a multiple of the border width gives dashes that scale with the
/// border and read as a normal dashed rule.
fn dashed_border_pattern(width: f32) -> [f32; 2] {
    let dash = (3.0 * width).max(1.0);
    [dash, dash]
}

/// Builds the path a uniform dashed border strokes along: the border box inset by
/// `width / 2` on every side, so a stroke of `width` centered on it fills the CSS
/// border band exactly (CSS borders sit inside the border box). Rounded when the
/// element has a corner radius (each radius reduced by `width / 2`, the centerline
/// radius), else a plain rectangle.
fn dashed_border_path(bounds: Rect, radii: CornerRadii, width: f32, has_radius: bool) -> Path {
    let half = width / 2.0;
    let center = Rect::new(
        Point::new(bounds.origin.x + half, bounds.origin.y + half),
        Size::new(
            (bounds.size.width - width).max(0.0),
            (bounds.size.height - width).max(0.0),
        ),
    );
    if has_radius {
        let center_radii = CornerRadii {
            top_left: (radii.top_left - half).max(0.0),
            top_right: (radii.top_right - half).max(0.0),
            bottom_right: (radii.bottom_right - half).max(0.0),
            bottom_left: (radii.bottom_left - half).max(0.0),
        };
        Path::rounded_rect(center, center_radii)
    } else {
        Path::rect(center)
    }
}

/// Builds the [`PaintCommand`] stroking a node's CSS `outline` (a focus ring),
/// or [`None`] when no outline should paint.
///
/// An outline paints only when the node's `outline-style` is not `none`
/// (`solid` / `dashed`, `dotted` folding to dashed) AND its `outline-width` is a
/// positive pixel length. Unlike a border, the outline sits OUTSIDE the border
/// box: the stroke path is `bounds` inflated by `outline-offset + width/2` on
/// every side, so the outline's inner edge is `offset` px outside the border box
/// and the centered stroke is `width` thick. When the node has a border radius the
/// path is a rounded rect whose corner radii are `border_radius + offset + width/2`
/// (clamped non-negative), concentric with the rounded border box; otherwise it is
/// a plain rectangle. A `dashed` outline reuses the border dash pattern
/// ([`dashed_border_pattern`]); the color falls back to the computed `color`
/// (`currentColor`) when `outline-color` is unset. The outline reserves no layout
/// space and is emitted so it paints on top of — and outside — the node.
fn outline_paint(
    computed: &ComputedStyles,
    node: NodeId,
    bounds: Rect,
    radii: CornerRadii,
    has_radius: bool,
) -> Option<PaintCommand> {
    let width = border_width_of(computed, node, PropertyId::OutlineWidth);
    if width <= 0.0 {
        return None;
    }
    let dashed = border_is_dashed(computed, node, PropertyId::OutlineStyle);
    let solid = border_is_solid(computed, node, PropertyId::OutlineStyle);
    // Only a `solid` or `dashed` (incl. folded `dotted`) outline paints; `none` or
    // an absent style paints nothing.
    if !dashed && !solid {
        return None;
    }
    let offset = border_width_of(computed, node, PropertyId::OutlineOffset);
    // The stroke centerline runs `offset + width/2` outside the border box, so the
    // outline's inner edge sits `offset` px out and the band is `width` thick.
    let d = offset + width / 2.0;
    let rect = Rect::new(
        Point::new(bounds.origin.x - d, bounds.origin.y - d),
        Size::new(
            (bounds.size.width + 2.0 * d).max(0.0),
            (bounds.size.height + 2.0 * d).max(0.0),
        ),
    );
    let path = if has_radius {
        // Concentric outer rounding: each corner radius grows by `d`.
        let outline_radii = CornerRadii {
            top_left: (radii.top_left + d).max(0.0),
            top_right: (radii.top_right + d).max(0.0),
            bottom_right: (radii.bottom_right + d).max(0.0),
            bottom_left: (radii.bottom_left + d).max(0.0),
        };
        Path::rounded_rect(rect, outline_radii)
    } else {
        Path::rect(rect)
    };
    let current = current_color_of(computed, node);
    let color = border_color_of(computed, node, PropertyId::OutlineColor, current);
    let mut stroke = Stroke::new(Brush::solid(scene_color_from(color)), width);
    if dashed {
        // Reuse the border dash convention — no duplicate dash logic.
        stroke = stroke.with_dash(dashed_border_pattern(width), 0.0);
    }
    Some(PaintCommand::DrawPath {
        path,
        paint: Paint::stroke(stroke),
    })
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
///
/// The `.placeholder` overlay is **excluded**: it is the CSS `::placeholder`
/// pseudo-element (a non-editable hint shown while `:placeholder-shown` holds),
/// not the field's editable text, so text inside a `.placeholder` subtree is never
/// treated as the editing field's text — the session never overwrites it with the
/// typed value (no `placeholder + typed`) and paints no caret into it.
pub(super) fn is_editing_text_node(tree: &RetainedTree, node: NodeId, field: NodeId) -> bool {
    let mut current = node;
    // At most `len` hops reach the root from any node in a well-formed tree; the
    // inclusive bound tolerates a self-parent without looping.
    for _ in 0..=tree.len() {
        if let Some(n) = tree.get(current)
            && n.attributes().has_class("placeholder")
        {
            return false;
        }
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
    use obscur_scene::paint::Paint;
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
        // A plain linear gradient keeps the default Pad extend.
        assert_eq!(g.extend(), ExtendMode::Pad);
        assert!(list.is_balanced());
    }

    #[test]
    fn repeating_linear_gradient_background_tiles_one_period() {
        use obscur_style::property::GradientStop as StyleStop;

        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);

        // `repeating-linear-gradient(90deg, #000 0 4px, #fff 4px 8px)`: an 8px
        // period of two hard bands, to the right, over a 100×50 box.
        let gradient = LinearGradient::new_repeating(
            90.0,
            vec![
                StyleStop::new(StyleColor::BLACK, 0.0),
                StyleStop::new(StyleColor::BLACK, 0.5),
                StyleStop::new(StyleColor::rgba(1.0, 1.0, 1.0, 1.0), 0.5),
                StyleStop::new(StyleColor::rgba(1.0, 1.0, 1.0, 1.0), 1.0),
            ],
            Length::px(8.0),
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
        // The axis spans exactly one 8px period from the box's 0% anchor, and the
        // pattern tiles via Repeat.
        assert_eq!(g.extend(), ExtendMode::Repeat);
        let near = |a: Point, b: Point| (a.x - b.x).abs() < 1e-3 && (a.y - b.y).abs() < 1e-3;
        assert!(near(g.start(), Point::new(0.0, 25.0)), "start {:?}", g.start());
        assert!(near(g.end(), Point::new(8.0, 25.0)), "end {:?}", g.end());
        assert_eq!(g.stops().len(), 4);
        assert_eq!(g.stops()[1].offset(), 0.5);
        assert!(list.is_balanced());
    }

    /// A solid 2×2 opaque RGBA image, for the `background-image: url(...)` tests.
    fn canned_image() -> SceneImage {
        use obscur_scene::color::AlphaMode;
        let pixels: std::sync::Arc<[u8]> = std::sync::Arc::from(vec![0u8; 2 * 2 * 4]);
        SceneImage::new(2, 2, pixels, AlphaMode::Straight).expect("2x2 rgba is valid")
    }

    /// Collects the `DrawImage` `(tile, region, repeat_x, repeat_y)` tuples of a
    /// display list, in paint order.
    fn image_draws(list: &DisplayList) -> Vec<(Rect, Rect, bool, bool)> {
        list.commands()
            .iter()
            .filter_map(|c| match c {
                PaintCommand::DrawImage {
                    tile,
                    region,
                    repeat_x,
                    repeat_y,
                    ..
                } => Some((*tile, *region, *repeat_x, *repeat_y)),
                _ => None,
            })
            .collect()
    }

    /// A `ComputedStyle` carrying a `background-image: url(...)` plus explicit
    /// `background-size` / `-position` / `-repeat` computed values.
    fn with_background_layer(
        url: &str,
        size: BackgroundSize,
        position: BackgroundPosition,
        repeat: BackgroundRepeat,
    ) -> ComputedStyle {
        let mut style =
            with_background_image(PropertyValue::BackgroundImageUrl(url.into()));
        style.set(
            PropertyId::BackgroundSize,
            ComputedValue::from_concrete(PropertyValue::BackgroundSize(size)).expect("concrete"),
        );
        style.set(
            PropertyId::BackgroundPosition,
            ComputedValue::from_concrete(PropertyValue::BackgroundPosition(position))
                .expect("concrete"),
        );
        style.set(
            PropertyId::BackgroundRepeat,
            ComputedValue::from_concrete(PropertyValue::BackgroundRepeat(repeat)).expect("concrete"),
        );
        style
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
        assert!(image_draws(&bare).is_empty(), "no image resolved → no DrawImage");
        assert!(bare.is_balanced());

        // With the URL resolved: one DrawImage. With NO authored size/position/
        // repeat, the CSS initials apply (auto size = the intrinsic 2×2 tile at the
        // box's top-left, repeat both axes) — the CSS-correct default, NOT a
        // full-box stretch. The region is the node's border box.
        let mut images = HashMap::new();
        images.insert(Box::<str>::from("x.png"), canned_image());
        let list =
            SceneCoordinator::new().compose_with_images(&tree, &computed, &boxes, None, &images);
        let draws = image_draws(&list);
        assert_eq!(draws.len(), 1, "one resolved image paints one DrawImage");
        let (tile, region, rx, ry) = draws[0];
        assert_eq!(region, node_box.bounds, "region is the node's border box");
        // Intrinsic 2×2 tile at the box origin (top-left).
        assert_eq!(tile.origin, node_box.bounds.origin);
        assert!((tile.size.width - 2.0).abs() < 1e-3);
        assert!((tile.size.height - 2.0).abs() < 1e-3);
        assert!(rx && ry, "CSS initial background-repeat is repeat on both axes");
        assert!(list.is_balanced());
    }

    #[test]
    fn background_no_repeat_contain_fits_one_tile() {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);

        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(
            a,
            with_background_layer(
                "x.png",
                BackgroundSize::Contain,
                BackgroundPosition::default(),
                BackgroundRepeat::NoRepeat,
            ),
        );
        let mut boxes = LayoutBoxes::new();
        boxes.insert(a, box_at(0.0, 0.0, 10.0, 10.0));

        let mut images = HashMap::new();
        images.insert(Box::<str>::from("x.png"), canned_image());
        let list =
            SceneCoordinator::new().compose_with_images(&tree, &computed, &boxes, None, &images);
        let draws = image_draws(&list);
        assert_eq!(draws.len(), 1);
        let (tile, _region, rx, ry) = draws[0];
        // `contain` of a 2×2 image into a 10×10 box scales to 10×10 (aspect kept).
        assert!((tile.size.width - 10.0).abs() < 1e-3, "contain width, got {tile:?}");
        assert!((tile.size.height - 10.0).abs() < 1e-3, "contain height");
        assert_eq!(tile.origin, Point::new(0.0, 0.0), "top-left position");
        assert!(!rx && !ry, "no-repeat clears both axis flags");
    }

    #[test]
    fn background_cover_scales_to_cover_the_box() {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);

        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(
            a,
            with_background_layer(
                "x.png",
                BackgroundSize::Cover,
                BackgroundPosition::default(),
                BackgroundRepeat::NoRepeat,
            ),
        );
        let mut boxes = LayoutBoxes::new();
        boxes.insert(a, box_at(0.0, 0.0, 10.0, 10.0));

        let mut images = HashMap::new();
        images.insert(Box::<str>::from("x.png"), canned_image());
        let list =
            SceneCoordinator::new().compose_with_images(&tree, &computed, &boxes, None, &images);
        let (tile, _region, _rx, _ry) = image_draws(&list)[0];
        // `cover` scales to at least the box on both axes (≥ 10×10). The region
        // clips any overflow at paint time.
        assert!(tile.size.width >= 10.0 - 1e-3, "cover width ≥ box, got {tile:?}");
        assert!(tile.size.height >= 10.0 - 1e-3, "cover height ≥ box");
    }

    /// Sets an element node's own image `src` on its retained attributes, the way
    /// the reconciler would from an `Image` primitive's `SRC` prop.
    fn set_image_src(tree: &mut RetainedTree, node: NodeId, src: &str) {
        use obscur_view::view::{StyleAttributes, ViewProperty, ViewValue, SRC};
        let attrs =
            StyleAttributes::extract(&[ViewProperty::new(SRC, ViewValue::Text(src.into()))]);
        if let Some(n) = tree.get_mut(node) {
            n.set_attributes(attrs);
        }
    }

    /// A `ComputedStyle` carrying a single `object-fit`.
    fn with_object_fit(fit: ObjectFit) -> ComputedStyle {
        let mut style = ComputedStyle::new();
        style.set(
            PropertyId::ObjectFit,
            ComputedValue::from_concrete(PropertyValue::ObjectFit(fit)).expect("concrete"),
        );
        style
    }

    #[test]
    fn element_image_src_emits_draw_image_as_content() {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let img = add_child(&mut tree, root, 10);
        set_image_src(&mut tree, img, "logo");

        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        // Default object-fit (fill): no ObjectFit set → stretch to the box.
        computed.insert(img, ComputedStyle::new());

        let mut boxes = LayoutBoxes::new();
        let node_box = box_at(0.0, 0.0, 10.0, 10.0);
        boxes.insert(img, node_box);

        // Absent from the map: no DrawImage, no panic (honest skip).
        let empty = HashMap::new();
        let bare =
            SceneCoordinator::new().compose_with_images(&tree, &computed, &boxes, None, &empty);
        assert!(image_draws(&bare).is_empty(), "no resolved image → no DrawImage");
        assert!(bare.is_balanced());

        // Resolved: one DrawImage over the box; default `fill` stretches to the box.
        let mut images = HashMap::new();
        images.insert(Box::<str>::from("logo"), canned_image());
        let list =
            SceneCoordinator::new().compose_with_images(&tree, &computed, &boxes, None, &images);
        let draws = image_draws(&list);
        assert_eq!(draws.len(), 1, "the element image paints one DrawImage");
        let (tile, region, rx, ry) = draws[0];
        assert_eq!(region, node_box.bounds, "region is the element box");
        assert_eq!(tile, node_box.bounds, "object-fit fill stretches to the box");
        assert!(!rx && !ry, "an element image never tiles");
        assert!(list.is_balanced());
    }

    #[test]
    fn object_fit_none_centers_the_intrinsic_tile() {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let img = add_child(&mut tree, root, 10);
        set_image_src(&mut tree, img, "logo");

        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(img, with_object_fit(ObjectFit::None));

        let mut boxes = LayoutBoxes::new();
        boxes.insert(img, box_at(0.0, 0.0, 10.0, 10.0));

        let mut images = HashMap::new();
        images.insert(Box::<str>::from("logo"), canned_image());
        let list =
            SceneCoordinator::new().compose_with_images(&tree, &computed, &boxes, None, &images);
        let (tile, region, _rx, _ry) = image_draws(&list)[0];
        // `none`: the intrinsic 2×2 tile, centered in the 10×10 box at (4, 4).
        assert!((tile.size.width - 2.0).abs() < 1e-3 && (tile.size.height - 2.0).abs() < 1e-3);
        assert!((tile.origin.x - 4.0).abs() < 1e-3 && (tile.origin.y - 4.0).abs() < 1e-3);
        assert_eq!(region.size, Size::new(10.0, 10.0), "region clips overflow to the box");
    }

    #[test]
    fn object_fit_contain_and_cover_fit_the_box() {
        // A non-square box makes `contain` (letterbox) and `cover` (clip) differ.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let img = add_child(&mut tree, root, 10);
        set_image_src(&mut tree, img, "logo");

        let node_box = box_at(0.0, 0.0, 20.0, 10.0);

        // contain: min-scale fit of a 2×2 image into 20×10 → scale 5 → 10×10,
        // centered horizontally at x=(20−10)/2=5, y=0.
        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(img, with_object_fit(ObjectFit::Contain));
        let mut boxes = LayoutBoxes::new();
        boxes.insert(img, node_box);
        let mut images = HashMap::new();
        images.insert(Box::<str>::from("logo"), canned_image());
        let list =
            SceneCoordinator::new().compose_with_images(&tree, &computed, &boxes, None, &images);
        let (tile, _region, _rx, _ry) = image_draws(&list)[0];
        assert!((tile.size.width - 10.0).abs() < 1e-3 && (tile.size.height - 10.0).abs() < 1e-3);
        assert!((tile.origin.x - 5.0).abs() < 1e-3 && (tile.origin.y - 0.0).abs() < 1e-3);

        // cover: max-scale of 2×2 into 20×10 → scale 10 → 20×20, centered vertically
        // at y=(10−20)/2=−5 so it fully covers; the region clips the overflow.
        computed.insert(img, with_object_fit(ObjectFit::Cover));
        let list =
            SceneCoordinator::new().compose_with_images(&tree, &computed, &boxes, None, &images);
        let (tile, region, _rx, _ry) = image_draws(&list)[0];
        assert!(tile.size.width >= 20.0 - 1e-3 && tile.size.height >= 10.0 - 1e-3);
        assert_eq!(region.size, Size::new(20.0, 10.0), "cover overflow clips to the box");
    }

    #[test]
    fn background_position_right_bottom_places_tile_at_far_corner() {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);

        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(
            a,
            with_background_layer(
                "x.png",
                BackgroundSize::Auto,
                // `right bottom` == 100% / 100%.
                BackgroundPosition::new(Length::Percent(100.0), Length::Percent(100.0)),
                BackgroundRepeat::NoRepeat,
            ),
        );
        let mut boxes = LayoutBoxes::new();
        boxes.insert(a, box_at(0.0, 0.0, 10.0, 10.0));

        let mut images = HashMap::new();
        images.insert(Box::<str>::from("x.png"), canned_image());
        let list =
            SceneCoordinator::new().compose_with_images(&tree, &computed, &boxes, None, &images);
        let (tile, _region, _rx, _ry) = image_draws(&list)[0];
        // Intrinsic 2×2 tile positioned bottom-right: origin = box − tile = (8, 8).
        assert!((tile.origin.x - 8.0).abs() < 1e-3, "x at box right − tile, got {tile:?}");
        assert!((tile.origin.y - 8.0).abs() < 1e-3, "y at box bottom − tile");
        assert!((tile.size.width - 2.0).abs() < 1e-3);
        assert!((tile.size.height - 2.0).abs() < 1e-3);
    }

    #[test]
    fn background_default_repeat_sets_both_axis_flags() {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);

        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(
            a,
            with_background_layer(
                "x.png",
                BackgroundSize::Auto,
                BackgroundPosition::default(),
                BackgroundRepeat::Repeat,
            ),
        );
        let mut boxes = LayoutBoxes::new();
        boxes.insert(a, box_at(0.0, 0.0, 10.0, 10.0));

        let mut images = HashMap::new();
        images.insert(Box::<str>::from("x.png"), canned_image());
        let list =
            SceneCoordinator::new().compose_with_images(&tree, &computed, &boxes, None, &images);
        let (_tile, _region, rx, ry) = image_draws(&list)[0];
        assert!(rx && ry, "repeat sets both axis flags");
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

    #[test]
    fn multiple_background_layers_paint_back_to_front_over_color() {
        use obscur_style::property::GradientStop as StyleStop;

        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);

        // Two layers, in CSS author order: a linear gradient on TOP, a url image on
        // the BOTTOM — over a background-color.
        let gradient = LinearGradient::new(
            90.0,
            vec![
                StyleStop::new(StyleColor::rgba(1.0, 0.0, 0.0, 1.0), 0.0),
                StyleStop::new(StyleColor::rgba(0.0, 0.0, 1.0, 1.0), 1.0),
            ],
        );
        let layers: Box<[BackgroundLayer]> = Box::new([
            BackgroundLayer::Linear(gradient),
            BackgroundLayer::Url("x.png".into()),
        ]);
        let mut style = with_background_image(PropertyValue::BackgroundLayers(layers));
        style.set(
            PropertyId::BackgroundColor,
            ComputedValue::color(StyleColor::rgba(0.0, 1.0, 0.0, 1.0)),
        );
        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(a, style);

        let mut boxes = LayoutBoxes::new();
        boxes.insert(a, box_at(0.0, 0.0, 100.0, 50.0));

        let mut images = HashMap::new();
        images.insert(Box::<str>::from("x.png"), canned_image());
        let list =
            SceneCoordinator::new().compose_with_images(&tree, &computed, &boxes, None, &images);
        assert!(list.is_balanced());

        // Paint order, bottom-to-top: the background-COLOR fill, then the BOTTOM
        // layer (the url image, painted first), then the TOP layer (the gradient
        // fill, painted last so it sits on top). BOTH a gradient fill and a
        // DrawImage are emitted.
        let kinds: Vec<&str> = list
            .commands()
            .iter()
            .filter_map(|c| match c {
                PaintCommand::DrawPath { .. } => Some("fill"),
                PaintCommand::DrawImage { .. } => Some("image"),
                _ => None,
            })
            .collect();
        assert_eq!(
            kinds,
            vec!["fill", "image", "fill"],
            "color, then bottom url layer, then top gradient layer"
        );
        // The two fills are the solid background-color (bottom) and the gradient
        // (top); the image draw is the middle (bottom) layer.
        let brushes = fill_brushes(&list);
        assert!(matches!(brushes[0], Brush::Solid(_)), "bg-color fills first");
        assert!(
            matches!(brushes[1], Brush::Linear(_)),
            "the top gradient layer fills last"
        );
        assert_eq!(image_draws(&list).len(), 1, "the bottom url layer draws one image");
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

    #[test]
    fn linear_corner_direction_is_aspect_ratio_aware() {
        use obscur_style::property::GradientStop as StyleStop;

        // `linear-gradient(to top right, black, white)` on a WIDE 200×50 box.
        let stops = vec![
            StyleStop::new(StyleColor::BLACK, 0.0),
            StyleStop::new(StyleColor::rgba(1.0, 1.0, 1.0, 1.0), 1.0),
        ];
        let gradient = LinearGradient::new_corner(Corner::TopRight, 45.0, stops);
        let bounds = Rect::new(Point::new(0.0, 0.0), Size::new(200.0, 50.0));
        let g = linear_gradient_brush(&gradient, bounds);

        // Direction points toward the actual top-right corner: normalize(h, −w) =
        // normalize(50, −200), NOT a 45° diagonal (spec §3.1).
        let dir = Point::new(g.end().x - g.start().x, g.end().y - g.start().y);
        let len = dir.x.hypot(dir.y);
        let (ux, uy) = (dir.x / len, dir.y / len);
        let expected = 50.0_f32.hypot(200.0);
        assert!((ux - 50.0 / expected).abs() < 1e-3, "dir.x {ux}");
        assert!((uy - (-200.0 / expected)).abs() < 1e-3, "dir.y {uy}");
        // Emphatically not 45° (sin45 ≈ 0.707).
        assert!((ux - std::f32::consts::FRAC_1_SQRT_2).abs() > 0.4, "not 45°");
        // Endpoints are symmetric about the box center (100, 25).
        let midx = (g.start().x + g.end().x) / 2.0;
        let midy = (g.start().y + g.end().y) / 2.0;
        assert!((midx - 100.0).abs() < 1e-3 && (midy - 25.0).abs() < 1e-3);
    }

    #[test]
    fn linear_angle_gradient_line_length_matches_formula() {
        use obscur_style::property::GradientStop as StyleStop;

        // `linear-gradient(45deg, …)` line length = |W·sin45| + |H·cos45|.
        let stops = vec![
            StyleStop::new(StyleColor::BLACK, 0.0),
            StyleStop::new(StyleColor::rgba(1.0, 1.0, 1.0, 1.0), 1.0),
        ];
        let gradient = LinearGradient::new(45.0, stops);
        let bounds = Rect::new(Point::new(0.0, 0.0), Size::new(200.0, 50.0));
        let g = linear_gradient_brush(&gradient, bounds);
        let line_len = (g.end().x - g.start().x).hypot(g.end().y - g.start().y);
        let s = std::f32::consts::FRAC_1_SQRT_2;
        let expected = (200.0 * s).abs() + (50.0 * s).abs();
        assert!((line_len - expected).abs() < 1e-3, "line {line_len} vs {expected}");
    }

    #[test]
    fn resolve_radii_circle_extents() {
        // Centered in a 100×50 box: center (50, 25).
        let bounds = Rect::new(Point::new(0.0, 0.0), Size::new(100.0, 50.0));
        // closest-side circle = min distance to a side = 25 (half height).
        let (rx, ry) = resolve_radii(RadialShape::Circle, RadialSize::ClosestSide, 50.0, 25.0, bounds);
        assert!((rx - 25.0).abs() < 1e-3 && (ry - 25.0).abs() < 1e-3, "closest-side {rx},{ry}");
        // farthest-corner circle = hypot(50, 25).
        let (rx, ry) =
            resolve_radii(RadialShape::Circle, RadialSize::FarthestCorner, 50.0, 25.0, bounds);
        let want = 50.0_f32.hypot(25.0);
        assert!((rx - want).abs() < 1e-3 && (rx - ry).abs() < 1e-6, "farthest-corner {rx},{ry}");
    }

    #[test]
    fn resolve_radii_ellipse_nonsquare_has_rx_ne_ry() {
        // Centered in a WIDE 480×120 box: center (240, 60).
        let bounds = Rect::new(Point::new(0.0, 0.0), Size::new(480.0, 120.0));
        // farthest-corner ellipse keeps the far-side ratio, scaled by √2:
        // (dx_far, dy_far)·√2 = (240√2, 60√2).
        let (rx, ry) =
            resolve_radii(RadialShape::Ellipse, RadialSize::FarthestCorner, 240.0, 60.0, bounds);
        let s = std::f32::consts::SQRT_2;
        assert!((rx - 240.0 * s).abs() < 1e-2, "rx {rx}");
        assert!((ry - 60.0 * s).abs() < 1e-2, "ry {ry}");
        assert!(rx > ry, "a non-square ellipse is stretched: rx {rx} > ry {ry}");
        // closest-side ellipse = the per-axis near distances (240, 60), still rx≠ry.
        let (rx, ry) =
            resolve_radii(RadialShape::Ellipse, RadialSize::ClosestSide, 240.0, 60.0, bounds);
        assert!((rx - 240.0).abs() < 1e-3 && (ry - 60.0).abs() < 1e-3, "closest-side {rx},{ry}");
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
        let ops: Vec<FilterOp> = shadows.into_iter().map(FilterOp::DropShadow).collect();
        style.set(
            PropertyId::Filter,
            ComputedValue::from_concrete(PropertyValue::Filter(ops.into_boxed_slice()))
                .expect("a filter op list is a concrete computed value"),
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
                vec![FilterOp::DropShadow(filter_shadow)].into_boxed_slice(),
            ))
            .expect("a filter op list is a concrete computed value"),
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

    #[test]
    fn inset_box_shadow_emits_draw_shadow_over_background() {
        // A node with `box-shadow: 0 0 0 3px black inset` over its background:
        // the inset shadow emits `DrawShadow { inset: true }` and paints AFTER
        // (over) the background fill, but its geometry stays the border box (the
        // renderer confines the inner frame to it).
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);

        let shadow = BoxShadow::new(
            Length::px(0.0),
            Length::px(0.0),
            Length::px(0.0),
            Length::px(3.0),
            Some(StyleColor::BLACK),
            true,
        );
        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(
            a,
            with_background_and_shadow(StyleColor::rgba(0.0, 1.0, 0.0, 1.0), vec![shadow]),
        );

        let mut boxes = LayoutBoxes::new();
        let bounds = box_at(0.0, 0.0, 20.0, 10.0);
        boxes.insert(a, bounds);

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);

        // Exactly one DrawShadow, flagged inset, carrying the authored params, and
        // its rect is the border box (the inset frame is bounded to it).
        let shadows = shadow_commands(&list);
        assert_eq!(shadows.len(), 1);
        let PaintCommand::DrawShadow {
            rect,
            spread,
            inset,
            ..
        } = shadows[0]
        else {
            panic!("expected a DrawShadow");
        };
        assert!(*inset, "an `inset` shadow lowers to DrawShadow{{inset:true}}");
        assert_eq!(*spread, 3.0);
        assert_eq!(*rect, Rect::new(Point::new(0.0, 0.0), Size::new(20.0, 10.0)));

        // The inset shadow paints AFTER (over) the background fill.
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
        assert!(
            fill_pos < shadow_pos,
            "inset shadow paints over the background"
        );
        assert!(list.is_balanced());
    }

    /// A `ComputedStyle` carrying a `background-color` and a `filter: blur(r)`.
    fn with_background_and_blur(bg: StyleColor, radius: f32) -> ComputedStyle {
        let mut style = ComputedStyle::new();
        style.set(PropertyId::BackgroundColor, ComputedValue::color(bg));
        style.set(
            PropertyId::Filter,
            ComputedValue::from_concrete(PropertyValue::Filter(
                vec![FilterOp::Blur(radius)].into_boxed_slice(),
            ))
            .expect("a filter op list is a concrete computed value"),
        );
        style
    }

    #[test]
    fn filter_blur_brackets_subtree_with_a_blur_layer() {
        // A node with `filter: blur(6px)` over a background brackets its subtree
        // paint with a `PushLayer(blur)` … `PopLayer`, the layer clipped to its
        // border box and carrying the blur effect.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);

        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(
            a,
            with_background_and_blur(StyleColor::rgba(0.0, 1.0, 0.0, 1.0), 6.0),
        );

        let mut boxes = LayoutBoxes::new();
        let bounds = box_at(0.0, 0.0, 20.0, 10.0);
        boxes.insert(a, bounds);

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        assert!(list.is_balanced());

        // Exactly one blur PushLayer, clipped to the border box, carrying blur(6).
        let blur_layers: Vec<&SceneLayer> = list
            .commands()
            .iter()
            .filter_map(|c| match c {
                PaintCommand::PushLayer(layer)
                    if layer.effects().iter().any(|e| e.is_blur()) =>
                {
                    Some(layer)
                }
                _ => None,
            })
            .collect();
        assert_eq!(blur_layers.len(), 1, "one blur group layer");
        let layer = blur_layers[0];
        assert_eq!(
            layer.effects(),
            &[LayerEffect::blur(6.0)],
            "the layer carries the authored blur radius"
        );
        assert_eq!(
            layer.clip().and_then(Clip::bounds),
            Some(bounds.bounds),
            "the blur layer is clipped to the element's border box"
        );

        // The blur PushLayer opens BEFORE the background fill, and the matching
        // PopLayer closes AFTER it (the layer wraps the subtree paint).
        let push = list
            .commands()
            .iter()
            .position(|c| matches!(c, PaintCommand::PushLayer(l) if l.effects().iter().any(|e| e.is_blur())))
            .expect("blur push present");
        let fill = list
            .commands()
            .iter()
            .position(|c| matches!(c, PaintCommand::DrawPath { .. }))
            .expect("background present");
        let pop = list
            .commands()
            .iter()
            .rposition(|c| matches!(c, PaintCommand::PopLayer))
            .expect("pop present");
        assert!(push < fill && fill < pop, "blur layer wraps the subtree paint");
    }

    #[test]
    fn no_blur_filter_emits_no_layer() {
        // Regression: a background with only a `drop-shadow` filter (no blur) emits
        // no blur PushLayer — byte-identical to the pre-blur path.
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
        assert!(
            !list
                .commands()
                .iter()
                .any(|c| matches!(c, PaintCommand::PushLayer(l) if l.effects().iter().any(|e| e.is_blur()))),
            "a drop-shadow-only filter emits no blur layer"
        );
    }

    #[test]
    fn color_matrix_none_when_no_color_op() {
        // No color op (empty, or blur / drop-shadow only) ⇒ no ColorMatrix (the
        // byte-identical no-color path).
        assert_eq!(color_matrix_of_ops(&[]), None);
        assert_eq!(color_matrix_of_ops(&[FilterOp::Blur(4.0)]), None);
        let shadow = BoxShadow::new(
            Length::px(1.0),
            Length::px(1.0),
            Length::px(0.0),
            Length::px(0.0),
            None,
            false,
        );
        assert_eq!(color_matrix_of_ops(&[FilterOp::DropShadow(shadow)]), None);
    }

    #[test]
    fn color_matrix_brightness_is_a_diagonal() {
        // brightness(2) ⇒ diagonal 2 on RGB, alpha 1, zero bias.
        let m = color_matrix_of_ops(&[FilterOp::Brightness(2.0)]).expect("color op");
        assert_eq!(m[0], 2.0); // r←r
        assert_eq!(m[5], 2.0); // g←g
        assert_eq!(m[10], 2.0); // b←b
        assert_eq!(m[15], 1.0); // a←a
        assert_eq!(&m[16..20], &[0.0, 0.0, 0.0, 0.0]); // no bias
        // Off-diagonal RGB entries are zero.
        assert_eq!(m[1], 0.0);
        assert_eq!(m[4], 0.0);
    }

    #[test]
    fn color_matrix_grayscale_one_is_the_luma_matrix() {
        // grayscale(1) folds to the spec luma matrix: every RGB row is the same
        // luminance triple (0.2126, 0.7152, 0.0722).
        let m = color_matrix_of_ops(&[FilterOp::Grayscale(1.0)]).expect("color op");
        for row in 0..3 {
            assert!((m[row * 4] - 0.2126).abs() < 1e-6, "row {row} r coeff");
            assert!((m[row * 4 + 1] - 0.7152).abs() < 1e-6, "row {row} g coeff");
            assert!((m[row * 4 + 2] - 0.0722).abs() < 1e-6, "row {row} b coeff");
        }
        assert_eq!(m[15], 1.0);
    }

    #[test]
    fn color_matrix_hue_rotate_90deg_builds_the_spec_matrix() {
        // hue-rotate(90°): cos = 0, sin = 1 ⇒ row0 = (0, 0, 1) exactly (the spec
        // constants collapse: 0.072 − 0 + 0.928 = 1).
        let m = color_matrix_of_ops(&[FilterOp::HueRotate(core::f32::consts::FRAC_PI_2)])
            .expect("color op");
        assert!(m[0].abs() < 1e-6, "r0c0");
        assert!(m[1].abs() < 1e-6, "r0c1");
        assert!((m[2] - 1.0).abs() < 1e-6, "r0c2 == 1");
    }

    #[test]
    fn color_matrix_chain_composes_in_author_order() {
        // grayscale(1) THEN brightness(2): grayscale collapses to luma, brightness
        // scales the result ×2, so row0 r-coeff = 2 × 0.2126 = 0.4252.
        let m = color_matrix_of_ops(&[FilterOp::Grayscale(1.0), FilterOp::Brightness(2.0)])
            .expect("color op");
        assert!((m[0] - 0.4252).abs() < 1e-5, "composed r0c0");
        assert!((m[1] - 1.4304).abs() < 1e-5, "composed r0c1 = 2×0.7152");
        // Swapping the order changes the result (brightness first, then luma): the
        // luma row absorbs the ×2 into its own weights, so r0c0 = 0.2126 (not 0.4252).
        let swapped = color_matrix_of_ops(&[FilterOp::Brightness(2.0), FilterOp::Grayscale(1.0)])
            .expect("color op");
        assert!((swapped[0] - 0.4252).abs() < 1e-5, "brightness∘luma still ×2 in r0c0");
        // The two orders differ overall (bias / alpha aside, the linear parts diverge
        // once a biased op like contrast is involved) — assert order matters via a
        // contrast+brightness pair.
        let a = color_matrix_of_ops(&[FilterOp::Contrast(2.0), FilterOp::Brightness(0.5)])
            .expect("color op");
        let b = color_matrix_of_ops(&[FilterOp::Brightness(0.5), FilterOp::Contrast(2.0)])
            .expect("color op");
        assert_ne!(a[16], b[16], "order changes the folded bias");
    }

    /// A `ComputedStyle` carrying a `background-color` and a `filter` op list.
    fn with_background_and_filter_ops(bg: StyleColor, ops: Vec<FilterOp>) -> ComputedStyle {
        let mut style = ComputedStyle::new();
        style.set(PropertyId::BackgroundColor, ComputedValue::color(bg));
        style.set(
            PropertyId::Filter,
            ComputedValue::from_concrete(PropertyValue::Filter(ops.into_boxed_slice()))
                .expect("a filter op list is a concrete computed value"),
        );
        style
    }

    #[test]
    fn filter_color_op_brackets_subtree_with_a_color_matrix_layer() {
        // A node with `filter: grayscale(1)` (a color op, no blur) brackets its
        // subtree with a PushLayer carrying a ColorMatrix effect and NO blur.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);

        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(
            a,
            with_background_and_filter_ops(
                StyleColor::rgba(0.0, 1.0, 0.0, 1.0),
                vec![FilterOp::Grayscale(1.0)],
            ),
        );

        let mut boxes = LayoutBoxes::new();
        let bounds = box_at(0.0, 0.0, 20.0, 10.0);
        boxes.insert(a, bounds);

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        assert!(list.is_balanced());

        let filter_layers: Vec<&SceneLayer> = list
            .commands()
            .iter()
            .filter_map(|c| match c {
                PaintCommand::PushLayer(layer)
                    if layer.effects().iter().any(|e| e.is_color_matrix()) =>
                {
                    Some(layer)
                }
                _ => None,
            })
            .collect();
        assert_eq!(filter_layers.len(), 1, "one color-matrix group layer");
        let layer = filter_layers[0];
        assert!(
            !layer.effects().iter().any(|e| e.is_blur()),
            "a color-only filter carries no blur effect"
        );
        let matrix = layer
            .effects()
            .iter()
            .find_map(|e| e.as_color_matrix())
            .expect("a color matrix effect");
        // grayscale(1) is the luma matrix.
        assert!((matrix[0] - 0.2126).abs() < 1e-6);
        assert_eq!(
            layer.clip().and_then(Clip::bounds),
            Some(bounds.bounds),
            "the filter layer is clipped to the element's border box"
        );
    }

    #[test]
    fn filter_blur_and_color_op_coexist_on_one_layer() {
        // `filter: grayscale(1) blur(4px)` puts BOTH a blur and a ColorMatrix effect
        // on the single filter layer.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);

        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(
            a,
            with_background_and_filter_ops(
                StyleColor::rgba(0.0, 1.0, 0.0, 1.0),
                vec![FilterOp::Grayscale(1.0), FilterOp::Blur(4.0)],
            ),
        );

        let mut boxes = LayoutBoxes::new();
        boxes.insert(a, box_at(0.0, 0.0, 20.0, 10.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        let layer = list
            .commands()
            .iter()
            .find_map(|c| match c {
                PaintCommand::PushLayer(layer)
                    if layer.effects().iter().any(|e| e.is_color_matrix()) =>
                {
                    Some(layer)
                }
                _ => None,
            })
            .expect("one filter layer");
        assert!(layer.effects().iter().any(|e| e.is_blur()), "blur present");
        assert!(
            layer.effects().iter().any(|e| e.is_color_matrix()),
            "color matrix present"
        );
    }

    #[test]
    fn no_color_op_emits_no_color_matrix_layer() {
        // Regression: a plain background (no filter) emits no ColorMatrix effect —
        // byte-identical to before color filters existed.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);

        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        let mut style = ComputedStyle::new();
        style.set(
            PropertyId::BackgroundColor,
            ComputedValue::color(StyleColor::rgba(0.0, 1.0, 0.0, 1.0)),
        );
        computed.insert(a, style);

        let mut boxes = LayoutBoxes::new();
        boxes.insert(a, box_at(0.0, 0.0, 20.0, 10.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        assert!(
            !list
                .commands()
                .iter()
                .any(|c| matches!(c, PaintCommand::PushLayer(l) if l.effects().iter().any(|e| e.is_color_matrix()))),
            "no filter emits no color-matrix layer"
        );
    }

    /// A `ComputedStyle` carrying a `background-color` and a
    /// `backdrop-filter: blur(r)` (reusing the `Filter` op list).
    fn with_background_and_backdrop(bg: StyleColor, radius: f32) -> ComputedStyle {
        let mut style = ComputedStyle::new();
        style.set(PropertyId::BackgroundColor, ComputedValue::color(bg));
        style.set(
            PropertyId::BackdropFilter,
            ComputedValue::from_concrete(PropertyValue::Filter(
                vec![FilterOp::Blur(radius)].into_boxed_slice(),
            ))
            .expect("a filter op list is a concrete computed value"),
        );
        style
    }

    #[test]
    fn backdrop_filter_emits_command_before_background() {
        // A node with `backdrop-filter: blur(12px)` over a background emits a
        // BackdropFilter command with its border box + radius, positioned BEFORE
        // (behind) its own background fill so the bg draws over the frosted backdrop.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);

        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(
            a,
            with_background_and_backdrop(StyleColor::rgba(1.0, 1.0, 1.0, 0.3), 12.0),
        );

        let mut boxes = LayoutBoxes::new();
        let bounds = box_at(2.0, 3.0, 20.0, 10.0);
        boxes.insert(a, bounds);

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        assert!(list.is_balanced());

        // Exactly one BackdropFilter, carrying the border box + blur radius.
        let backdrops: Vec<_> = list
            .commands()
            .iter()
            .filter_map(|c| match c {
                PaintCommand::BackdropFilter {
                    bounds,
                    blur,
                    ..
                } => Some((*bounds, *blur)),
                _ => None,
            })
            .collect();
        assert_eq!(backdrops.len(), 1, "one backdrop-filter command");
        assert_eq!(backdrops[0].0, bounds.bounds, "frost region is the border box");
        assert_eq!(backdrops[0].1, 12.0, "carries the authored blur radius");

        // The BackdropFilter is emitted BEFORE the node's own background fill.
        let backdrop_pos = list
            .commands()
            .iter()
            .position(|c| matches!(c, PaintCommand::BackdropFilter { .. }))
            .expect("backdrop present");
        let fill_pos = list
            .commands()
            .iter()
            .position(|c| matches!(c, PaintCommand::DrawPath { .. }))
            .expect("background present");
        assert!(
            backdrop_pos < fill_pos,
            "backdrop-filter frosts the backdrop before the element's own bg draws over it"
        );
    }

    #[test]
    fn no_backdrop_filter_emits_no_command() {
        // Regression: a node with a background but no `backdrop-filter` emits no
        // BackdropFilter command — byte-identical to before backdrop-filter existed.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);

        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        let mut style = ComputedStyle::new();
        style.set(
            PropertyId::BackgroundColor,
            ComputedValue::color(StyleColor::rgba(0.0, 0.0, 1.0, 1.0)),
        );
        computed.insert(a, style);

        let mut boxes = LayoutBoxes::new();
        boxes.insert(a, box_at(0.0, 0.0, 20.0, 10.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        assert!(
            !list
                .commands()
                .iter()
                .any(|c| matches!(c, PaintCommand::BackdropFilter { .. })),
            "no backdrop-filter emits no command"
        );
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
    fn line_height_two_widens_wrapped_line_spacing() {
        // `line-height: 2` makes each wrapped line box 2·16 = 32 tall, so the three
        // wrapped lines stack by 32 (not the 16px font-metric height) and the extra
        // leading (16px) is split half above / half below — the glyph baseline
        // shifts down to 12.8 + (32-16)/2 = 20.8.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let text = add_text_child(&mut tree, root, "aaaa bbbb cccc");

        let mut style = ComputedStyle::new();
        style.set(PropertyId::LineHeight, ComputedValue::number(2.0));
        let mut computed = ComputedStyles::new();
        computed.insert(text, style);

        let mut boxes = LayoutBoxes::new();
        // Box as layout sizes it under line-height:2 — 50px wide, 3·32 = 96 tall.
        boxes.insert(text, box_at(10.0, 20.0, 50.0, 96.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        let runs = text_runs(&list);
        assert_eq!(runs.len(), 3, "one DrawText per wrapped line");
        // Lines spaced by the resolved 32px line height, not the 16px metric.
        assert_eq!(runs[0].0, Point::new(10.0, 20.0));
        assert_eq!(runs[1].0, Point::new(10.0, 52.0));
        assert_eq!(runs[2].0, Point::new(10.0, 84.0));
        // Half-leading centres each line's glyphs on the shifted baseline.
        for (_, run, _) in &runs {
            for glyph in run.glyphs() {
                assert_eq!(glyph.y(), 20.8);
            }
        }
        assert!(list.is_balanced());
    }

    #[test]
    fn explicit_newline_forces_a_second_line_under_pre_line() {
        // Under `white-space: pre-line` a `\n` is a preserved mandatory break (CSS
        // Text 3 §3): even in a box wide enough that nothing wraps, "ab\ncd" splits
        // into two lines, so the scene emits two runs stacked by the 16px line
        // height. (Under the initial `normal` the break would collapse to a space —
        // see `explicit_newline_collapses_to_a_space_under_normal`.)
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let text = add_text_child(&mut tree, root, "ab\ncd");

        let mut computed = ComputedStyles::new();
        let mut style = ComputedStyle::new();
        style.set(
            PropertyId::WhiteSpace,
            ComputedValue::keyword(obscur_style::property::layout_keyword::WHITE_SPACE_PRE_LINE),
        );
        computed.insert(text, style);
        let mut boxes = LayoutBoxes::new();
        boxes.insert(text, box_at(0.0, 0.0, 1000.0, 32.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        let runs = text_runs(&list);
        assert_eq!(runs.len(), 2, "\\n forces a second line under pre-line");
        assert_eq!(runs[0].0.y, 0.0);
        assert_eq!(runs[1].0.y, 16.0);
        assert!(list.is_balanced());
    }

    #[test]
    fn explicit_newline_collapses_to_a_space_under_normal() {
        // The spec-correct initial behaviour (CSS Text 3 §4.1.1): under
        // `white-space: normal` a segment break collapses to a space, so "ab\ncd"
        // in a wide box is a SINGLE line — the pre-`white-space` code path forced a
        // hard break here, which was `pre-line`-ish and wrong for the initial value.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let text = add_text_child(&mut tree, root, "ab\ncd");

        let mut computed = ComputedStyles::new();
        computed.insert(text, ComputedStyle::new());
        let mut boxes = LayoutBoxes::new();
        boxes.insert(text, box_at(0.0, 0.0, 1000.0, 32.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        let runs = text_runs(&list);
        assert_eq!(runs.len(), 1, "\\n collapses to a space under normal");
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

    /// Collects the `DrawPath` fills of a display list as `(polygon vertices,
    /// solid color)` pairs, in paint order. The vertices are the `MoveTo`/`LineTo`
    /// anchor points of the path (a `Close` adds no point), so a mitered border
    /// quad yields its four corners and a degenerate side yields the triangle's
    /// four listed points (two of which coincide at the collapsed apex).
    fn path_polygons(list: &DisplayList) -> Vec<(Vec<Point>, Color)> {
        use obscur_scene::path::PathSegment;
        list.commands()
            .iter()
            .filter_map(|c| match c {
                PaintCommand::DrawPath { path, paint } => {
                    let pts = path
                        .segments()
                        .iter()
                        .filter_map(|s| match s {
                            PathSegment::MoveTo(p) | PathSegment::LineTo(p) => Some(*p),
                            _ => None,
                        })
                        .collect();
                    Some((pts, paint.brush().as_solid()?.color()))
                }
                _ => None,
            })
            .collect()
    }

    /// The set of DISTINCT vertices of a polygon, sorted for stable comparison.
    /// Collapsed corners (equal points) fold together, so a degenerate border
    /// quad reduces to its three triangle vertices.
    fn distinct_vertices(pts: &[Point]) -> Vec<(f32, f32)> {
        let mut v: Vec<(f32, f32)> = pts.iter().map(|p| (p.x, p.y)).collect();
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        v.dedup();
        v
    }

    #[test]
    fn border_paints_four_mitered_side_quads_covering_the_edges() {
        // A 100×50 box at (10,20) with `border: 2px solid red` paints four mitered
        // trapezoid quads in top/right/bottom/left order, each red, tiling the ring
        // with 45°-mitered corners (outer edge full, inner edge inset by 2px on
        // each adjacent side).
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);

        let red = StyleColor::rgba(1.0, 0.0, 0.0, 1.0);
        let mut computed = ComputedStyles::new();
        computed.insert(root, with_border(2.0, red));

        let mut boxes = LayoutBoxes::new();
        boxes.insert(root, box_at(10.0, 20.0, 100.0, 50.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        let polys = path_polygons(&list);
        assert_eq!(polys.len(), 4, "four side quads, no background");
        // Every side is red.
        for (_, color) in &polys {
            assert_eq!(*color, Color::rgba(1.0, 0.0, 0.0, 1.0));
        }
        // Outer corners (10,20)-(110,70); inner corners inset by 2px: (12,22)-(108,68).
        // top: outer TL,TR then inner TR,TL.
        assert_eq!(
            polys[0].0,
            vec![p(10.0, 20.0), p(110.0, 20.0), p(108.0, 22.0), p(12.0, 22.0)],
        );
        // right: outer TR,BR then inner BR,TR.
        assert_eq!(
            polys[1].0,
            vec![p(110.0, 20.0), p(110.0, 70.0), p(108.0, 68.0), p(108.0, 22.0)],
        );
        // bottom: outer BR,BL then inner BL,BR.
        assert_eq!(
            polys[2].0,
            vec![p(110.0, 70.0), p(10.0, 70.0), p(12.0, 68.0), p(108.0, 68.0)],
        );
        // left: outer BL,TL then inner TL,BL.
        assert_eq!(
            polys[3].0,
            vec![p(10.0, 70.0), p(10.0, 20.0), p(12.0, 22.0), p(12.0, 68.0)],
        );
        // The four quads cover the whole 2px border band: their bounding boxes
        // union to the full outer ring (top/bottom span the full width, left/right
        // the full height — corners now shared by the miters, not overlapped).
        let fills = path_fills(&list);
        assert_eq!(fills[0].0, Rect::new(Point::new(10.0, 20.0), Size::new(100.0, 2.0)));
        assert_eq!(fills[1].0, Rect::new(Point::new(108.0, 20.0), Size::new(2.0, 50.0)));
        assert_eq!(fills[2].0, Rect::new(Point::new(10.0, 68.0), Size::new(100.0, 2.0)));
        assert_eq!(fills[3].0, Rect::new(Point::new(10.0, 20.0), Size::new(2.0, 50.0)));
        assert!(list.is_balanced());
    }

    /// Shorthand for a `Point` in the border-geometry tests.
    fn p(x: f32, y: f32) -> Point {
        Point::new(x, y)
    }

    /// Builds a `ComputedStyle` with per-side SOLID borders: `widths` and `colors`
    /// are in top/right/bottom/left order. Every side is `solid`; a zero-width or
    /// transparent side paints nothing regardless.
    fn triangle_border(widths: [f32; 4], colors: [StyleColor; 4]) -> ComputedStyle {
        let mut style = ComputedStyle::new();
        let w_ids = [
            PropertyId::BorderTopWidth,
            PropertyId::BorderRightWidth,
            PropertyId::BorderBottomWidth,
            PropertyId::BorderLeftWidth,
        ];
        let s_ids = [
            PropertyId::BorderTopStyle,
            PropertyId::BorderRightStyle,
            PropertyId::BorderBottomStyle,
            PropertyId::BorderLeftStyle,
        ];
        let c_ids = [
            PropertyId::BorderTopColor,
            PropertyId::BorderRightColor,
            PropertyId::BorderBottomColor,
            PropertyId::BorderLeftColor,
        ];
        for (id, w) in w_ids.into_iter().zip(widths) {
            style.set(id, ComputedValue::length(Length::px(w)));
        }
        for id in s_ids {
            style.set(id, ComputedValue::keyword(layout_keyword::BORDER_STYLE_SOLID));
        }
        for (id, c) in c_ids.into_iter().zip(colors) {
            style.set(id, ComputedValue::color(c));
        }
        style
    }

    #[test]
    fn border_triangle_trick_renders_the_four_arrow_triangles() {
        // The classic CSS border-triangle trick: a 0×0-content box whose border
        // box is exactly its border widths, with three sides transparent and one
        // side colored, collapses the colored side's trapezoid into a TRIANGLE —
        // the arrow/marker glyph used for select ▼, disclosure ▸▾, steppers ▲▼.
        // The border box is (left + right) × (top + bottom).
        let colored = StyleColor::rgba(0.2, 0.2, 0.2, 1.0); // #333
        let clear = StyleColor::rgba(0.0, 0.0, 0.0, 0.0); // transparent

        // (widths [t,r,b,l], colored side index, box w, box h, expected 3 vertices).
        let cases: [([f32; 4], usize, f32, f32, [(f32, f32); 3]); 4] = [
            // ▽ down — border-top colored, box 10×6: base at top, apex at bottom.
            ([6.0, 5.0, 0.0, 5.0], 0, 10.0, 6.0, [(0.0, 0.0), (10.0, 0.0), (5.0, 6.0)]),
            // △ up — border-bottom colored, box 10×6: base at bottom, apex at top.
            ([0.0, 5.0, 6.0, 5.0], 2, 10.0, 6.0, [(0.0, 6.0), (10.0, 6.0), (5.0, 0.0)]),
            // ▷ right — border-left colored, box 6×10: base at left, apex at right.
            ([5.0, 0.0, 5.0, 6.0], 3, 6.0, 10.0, [(0.0, 0.0), (0.0, 10.0), (6.0, 5.0)]),
            // ◁ left — border-right colored, box 6×10: base at right, apex at left.
            ([5.0, 6.0, 5.0, 0.0], 1, 6.0, 10.0, [(6.0, 0.0), (6.0, 10.0), (0.0, 5.0)]),
        ];

        for (widths, colored_side, bw, bh, expected) in cases {
            let mut colors = [clear; 4];
            colors[colored_side] = colored;

            let mut tree = RetainedTree::new();
            let root = tree.alloc(NodeKind::Root);
            tree.set_root(root);
            let mut computed = ComputedStyles::new();
            computed.insert(root, triangle_border(widths, colors));
            let mut boxes = LayoutBoxes::new();
            boxes.insert(root, box_at(0.0, 0.0, bw, bh));

            let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
            let polys = path_polygons(&list);
            // Only the one colored side is emitted; the three transparent sides
            // (and any zero-width side) paint nothing.
            assert_eq!(polys.len(), 1, "one triangle for widths {widths:?}");
            assert_eq!(polys[0].1, scene_color_from(colored), "the triangle is #333");
            // Exactly three distinct vertices: the collapsed side's two outer (or
            // two inner) corners coincide at the apex.
            let mut want: Vec<(f32, f32)> = expected.to_vec();
            want.sort_by(|a, b| a.partial_cmp(b).unwrap());
            assert_eq!(
                distinct_vertices(&polys[0].0),
                want,
                "expected arrow triangle for widths {widths:?}"
            );
            assert!(list.is_balanced());
        }
    }

    #[test]
    fn transparent_border_side_is_not_emitted() {
        // A per-side border where one side is a transparent color paints only the
        // opaque sides — the transparent side emits no quad.
        let opaque = StyleColor::rgba(0.0, 0.0, 1.0, 1.0);
        let clear = StyleColor::rgba(0.0, 0.0, 0.0, 0.0);

        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        // Top transparent; right/bottom/left opaque blue.
        let mut computed = ComputedStyles::new();
        computed.insert(
            root,
            triangle_border([2.0, 2.0, 2.0, 2.0], [clear, opaque, opaque, opaque]),
        );
        let mut boxes = LayoutBoxes::new();
        boxes.insert(root, box_at(0.0, 0.0, 40.0, 30.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        let polys = path_polygons(&list);
        assert_eq!(polys.len(), 3, "transparent top side skipped, three opaque sides");
        for (_, color) in &polys {
            assert_eq!(*color, scene_color_from(opaque));
        }
    }

    /// Builds a `ComputedStyle` with a uniform DASHED border of `width` px and
    /// the given per-side `color`.
    fn with_dashed_border(width: f32, color: StyleColor) -> ComputedStyle {
        let mut style = with_border(width, color);
        for id in [
            PropertyId::BorderTopStyle,
            PropertyId::BorderRightStyle,
            PropertyId::BorderBottomStyle,
            PropertyId::BorderLeftStyle,
        ] {
            style.set(id, ComputedValue::keyword(layout_keyword::BORDER_STYLE_DASHED));
        }
        style
    }

    /// Collects every `DrawPath` whose paint is a `Stroke`, as `(path, stroke)`.
    fn stroke_paths(list: &DisplayList) -> Vec<(Path, Stroke)> {
        list.commands()
            .iter()
            .filter_map(|c| match c {
                PaintCommand::DrawPath {
                    path,
                    paint: Paint::Stroke(stroke),
                } => Some((path.clone(), stroke.clone())),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn uniform_dashed_border_emits_a_dashed_stroke_not_fill_rects() {
        // A 100×50 box with `border: 2px dashed red` paints ONE dashed stroke
        // along the border-box centerline, not the four solid fill-rects.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);

        let red = StyleColor::rgba(1.0, 0.0, 0.0, 1.0);
        let mut computed = ComputedStyles::new();
        computed.insert(root, with_dashed_border(2.0, red));

        let mut boxes = LayoutBoxes::new();
        boxes.insert(root, box_at(10.0, 20.0, 100.0, 50.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        // No solid fill DrawPaths for the border — only the dashed stroke.
        let fill_count = list
            .commands()
            .iter()
            .filter(|c| matches!(c, PaintCommand::DrawPath { paint: Paint::Fill(_), .. }))
            .count();
        assert_eq!(fill_count, 0, "dashed border does not fill-rect");
        let strokes = stroke_paths(&list);
        assert_eq!(strokes.len(), 1, "one dashed border stroke");
        let (path, stroke) = &strokes[0];
        assert_eq!(stroke.width(), 2.0);
        assert!(stroke.is_dashed(), "carries a non-empty dash pattern");
        assert_eq!(
            stroke.brush().as_solid().map(|s| s.color()),
            Some(scene_color_from(red))
        );
        // The stroked path is the border box inset by width/2 (= 1px) on each side.
        assert_eq!(
            path.bounds(),
            Some(Rect::new(Point::new(11.0, 21.0), Size::new(98.0, 48.0)))
        );
        assert!(list.is_balanced());
    }

    #[test]
    fn solid_border_still_emits_fill_rects_not_a_dashed_stroke() {
        // Regression guard: a uniform SOLID border keeps the four side fills (now
        // mitered quads) and emits no dashed stroke.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);

        let mut computed = ComputedStyles::new();
        computed.insert(root, with_border(2.0, StyleColor::rgba(1.0, 0.0, 0.0, 1.0)));

        let mut boxes = LayoutBoxes::new();
        boxes.insert(root, box_at(10.0, 20.0, 100.0, 50.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        assert_eq!(path_fills(&list).len(), 4, "four solid side rects");
        assert!(stroke_paths(&list).is_empty(), "no dashed stroke for a solid border");
    }

    /// Builds a `ComputedStyle` with an `outline` of `width` px, `offset` px, the
    /// given `style` keyword handle, and `color`.
    fn with_outline(
        width: f32,
        offset: f32,
        style: obscur_identity::InternedSymbol,
        color: StyleColor,
    ) -> ComputedStyle {
        let mut s = ComputedStyle::new();
        s.set(PropertyId::OutlineWidth, ComputedValue::length(Length::px(width)));
        s.set(PropertyId::OutlineOffset, ComputedValue::length(Length::px(offset)));
        s.set(PropertyId::OutlineStyle, ComputedValue::keyword(style));
        s.set(PropertyId::OutlineColor, ComputedValue::color(color));
        s
    }

    #[test]
    fn dashed_outline_strokes_a_ring_outside_the_border_box() {
        // A 100×50 box at (10,20) with `outline: 2px dashed; outline-offset: 3px`
        // strokes ONE dashed ring, centered `offset + width/2` = 4px outside the
        // border box, so its bounds exceed the border box by that inflation.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);

        let teal = StyleColor::rgba(0.17, 0.16, 0.15, 1.0);
        let mut computed = ComputedStyles::new();
        computed.insert(
            root,
            with_outline(2.0, 3.0, layout_keyword::BORDER_STYLE_DASHED, teal),
        );

        let mut boxes = LayoutBoxes::new();
        boxes.insert(root, box_at(10.0, 20.0, 100.0, 50.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        let strokes = stroke_paths(&list);
        assert_eq!(strokes.len(), 1, "one outline ring stroke");
        let (path, stroke) = &strokes[0];
        assert_eq!(stroke.width(), 2.0);
        assert!(stroke.is_dashed(), "dashed outline carries a dash pattern");
        assert_eq!(
            stroke.brush().as_solid().map(|s| s.color()),
            Some(scene_color_from(teal))
        );
        // Centerline at border box inflated by offset(3) + width/2(1) = 4 on each
        // side: origin (6,16), size (108,58). Its inner edge (7,17) is offset(3) px
        // OUTSIDE the border box (10,20) — a focus ring with a visible gap.
        let bounds = path.bounds().expect("outline path has bounds");
        assert_eq!(bounds, Rect::new(Point::new(6.0, 16.0), Size::new(108.0, 58.0)));
        assert!(bounds.origin.x < 10.0 && bounds.origin.y < 20.0, "outside the border box");
        assert!(list.is_balanced());
    }

    #[test]
    fn solid_outline_strokes_a_ring_and_no_outline_emits_none() {
        // A solid outline strokes one solid (non-dashed) ring; with the style set
        // to `none` (and otherwise identical props) nothing is emitted.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let mut boxes = LayoutBoxes::new();
        boxes.insert(root, box_at(0.0, 0.0, 40.0, 40.0));
        let color = StyleColor::rgba(0.1, 0.2, 0.9, 1.0);

        // Solid outline → one non-dashed stroke.
        let mut solid = ComputedStyles::new();
        solid.insert(
            root,
            with_outline(2.0, 0.0, layout_keyword::BORDER_STYLE_SOLID, color),
        );
        let list = SceneCoordinator::new().compose(&tree, &solid, &boxes, None);
        let strokes = stroke_paths(&list);
        assert_eq!(strokes.len(), 1, "one solid outline ring");
        assert!(!strokes[0].1.is_dashed(), "solid outline is not dashed");

        // `outline-style: none` → no outline paint at all.
        let mut none = ComputedStyles::new();
        none.insert(
            root,
            with_outline(2.0, 0.0, layout_keyword::BORDER_STYLE_NONE, color),
        );
        let list = SceneCoordinator::new().compose(&tree, &none, &boxes, None);
        assert!(stroke_paths(&list).is_empty(), "outline-style:none emits nothing");
        assert!(list.commands().is_empty(), "no outline, no other paint → empty list");
    }

    #[test]
    fn no_outline_is_byte_identical() {
        // Regression guard: a plain background node with no outline paints exactly
        // as before (one fill, no stroke, no extra brackets).
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let mut computed = ComputedStyles::new();
        computed.insert(root, with_background(StyleColor::rgba(1.0, 0.0, 0.0, 1.0)));
        let mut boxes = LayoutBoxes::new();
        boxes.insert(root, box_at(0.0, 0.0, 30.0, 30.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        assert_eq!(fill_count(&list), 1, "only the background");
        assert!(stroke_paths(&list).is_empty(), "no outline stroke");
        assert_eq!(list.commands().len(), 1, "exactly one command (byte-identical)");
    }

    #[test]
    fn rounded_node_gets_a_rounded_outline_path() {
        // A node with a border radius gets a ROUNDED outline path (its bounding box
        // is still the inflated border box), whose corner radii grow by the
        // inflation — concentric with the rounded border box.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);

        let color = StyleColor::rgba(0.17, 0.16, 0.15, 1.0);
        let mut style = with_outline(2.0, 3.0, layout_keyword::BORDER_STYLE_SOLID, color);
        for id in [
            PropertyId::BorderTopLeftRadius,
            PropertyId::BorderTopRightRadius,
            PropertyId::BorderBottomRightRadius,
            PropertyId::BorderBottomLeftRadius,
        ] {
            style.set(id, ComputedValue::length(Length::px(8.0)));
        }
        let mut computed = ComputedStyles::new();
        computed.insert(root, style);
        let mut boxes = LayoutBoxes::new();
        boxes.insert(root, box_at(10.0, 20.0, 100.0, 50.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        let strokes = stroke_paths(&list);
        assert_eq!(strokes.len(), 1, "one rounded outline stroke");
        let (path, _) = &strokes[0];
        // The rounded path is not a plain rect (it has cubic corner segments), and
        // its bounds are the inflated border box (inflated by 4 on each side).
        assert!(
            path.segments()
                .iter()
                .any(|s| matches!(s, obscur_scene::path::PathSegment::CubicTo { .. })),
            "rounded outline uses a rounded-rect (cubic) path"
        );
        assert_eq!(
            path.bounds(),
            Some(Rect::new(Point::new(6.0, 16.0), Size::new(108.0, 58.0)))
        );
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

    /// A `ComputedStyle` with a uniform solid border and a uniform `border-radius`.
    fn with_border_and_radius(width: f32, color: StyleColor, radius: f32) -> ComputedStyle {
        let mut style = with_border(width, color);
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

    #[test]
    fn uniform_rounded_border_emits_one_even_odd_ring_with_rounded_inner() {
        // A 100×50 box at (10,20) with `5px solid` + `border-radius: 14px` paints
        // ONE even-odd ring — NOT four abutting rects — so the border's INNER
        // corners round (inner radius = 14 − 5 = 9) instead of staying square.
        use obscur_scene::path::{FillRule, Path, PathSegment};

        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);

        let red = StyleColor::rgba(1.0, 0.0, 0.0, 1.0);
        let mut computed = ComputedStyles::new();
        computed.insert(root, with_border_and_radius(5.0, red, 14.0));

        let mut boxes = LayoutBoxes::new();
        boxes.insert(root, box_at(10.0, 20.0, 100.0, 50.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        // Exactly one border fill (no background) — the ring, not four rects.
        let paths: Vec<&Path> = list
            .commands()
            .iter()
            .filter_map(|c| match c {
                PaintCommand::DrawPath { path, .. } => Some(path),
                _ => None,
            })
            .collect();
        assert_eq!(paths.len(), 1, "one ring fill, not four side rects");
        let ring = paths[0];
        assert_eq!(ring.fill_rule(), FillRule::EvenOdd, "ring is even-odd");
        // Two closed rounded-rect subpaths (outer + inner) ⇒ two `MoveTo`s.
        let moves = ring
            .segments()
            .iter()
            .filter(|s| matches!(s, PathSegment::MoveTo(_)))
            .count();
        assert_eq!(moves, 2, "outer + inner subpaths");
        // The outer subpath is the border-box rounded rect; the inner subpath is
        // the border box inset by 5px on every side with radius 14 − 5 = 9.
        let outer = Path::rounded_rect(
            Rect::new(Point::new(10.0, 20.0), Size::new(100.0, 50.0)),
            CornerRadii::uniform(14.0),
        );
        let inner = Path::rounded_rect(
            Rect::new(Point::new(15.0, 25.0), Size::new(90.0, 40.0)),
            CornerRadii::uniform(9.0),
        );
        assert_eq!(&ring.segments()[..outer.len()], outer.segments());
        assert_eq!(&ring.segments()[outer.len()..], inner.segments());
        assert!(list.is_balanced());
    }

    #[test]
    fn non_uniform_rounded_border_keeps_four_rects() {
        // A rounded border with a DIFFERENTLY colored side is non-uniform, so it
        // stays the four-rect approximation (no even-odd ring).
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);

        let red = StyleColor::rgba(1.0, 0.0, 0.0, 1.0);
        let blue = StyleColor::rgba(0.0, 0.0, 1.0, 1.0);
        let mut style = with_border_and_radius(5.0, red, 14.0);
        style.set(PropertyId::BorderTopColor, ComputedValue::color(blue));
        let mut computed = ComputedStyles::new();
        computed.insert(root, style);

        let mut boxes = LayoutBoxes::new();
        boxes.insert(root, box_at(10.0, 20.0, 100.0, 50.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        assert_eq!(fill_count(&list), 4, "non-uniform ⇒ four side rects");
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

    /// A `ComputedStyle` carrying a `background-color` and an `overflow` keyword.
    fn with_background_and_overflow(
        color: StyleColor,
        overflow: obscur_identity::InternedSymbol,
    ) -> ComputedStyle {
        let mut style = with_background(color);
        style.set(PropertyId::Overflow, ComputedValue::keyword(overflow));
        style
    }

    /// A `root (overflow:auto, 100×60)` with a child overflowing to 120 tall, and
    /// the matching `(tree, computed, boxes)` — the fixture the scroll tests share.
    fn scroll_fixture() -> (RetainedTree, ComputedStyles, LayoutBoxes, NodeId, NodeId) {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let child = add_child(&mut tree, root, 10);
        let mut computed = ComputedStyles::new();
        computed.insert(
            root,
            with_background_and_overflow(
                StyleColor::rgba(1.0, 1.0, 1.0, 1.0),
                layout_keyword::OVERFLOW_AUTO,
            ),
        );
        computed.insert(child, with_background(StyleColor::rgba(0.0, 1.0, 0.0, 1.0)));
        let mut boxes = LayoutBoxes::new();
        boxes.insert(root, box_at(0.0, 0.0, 100.0, 60.0));
        boxes.insert(child, box_at(0.0, 0.0, 100.0, 120.0)); // content overflows (120 > 60)
        (tree, computed, boxes, root, child)
    }

    #[test]
    fn empty_scroll_map_is_byte_identical_to_compose() {
        // Byte-identical guard: composing with an EMPTY scroll map reproduces
        // `compose` command-for-command (no scroll transform, no scrollbar).
        let (tree, computed, boxes, _root, _child) = scroll_fixture();
        let coord = SceneCoordinator::new();
        let plain = coord.compose(&tree, &computed, &boxes, None);
        let empty = coord.compose_with_images_scrolled(
            &tree,
            &computed,
            &boxes,
            None,
            &HashMap::new(),
            &HashMap::new(),
        );
        assert_eq!(plain.commands(), empty.commands(), "empty scroll map ⇒ identical list");
    }

    #[test]
    fn scroll_container_brackets_content_in_a_translate_inside_the_clip() {
        // With a scroll entry the coordinator wraps the DESCENDANTS in a
        // translate(0,-80) nested INSIDE the overflow clip, and paints a scrollbar
        // (track + thumb) AFTER the clip closes (on top, unscrolled).
        let (tree, computed, boxes, root, _child) = scroll_fixture();
        let mut scroll = HashMap::new();
        scroll.insert(root, ScrollPaint { scroll_top: 80.0, content_height: 120.0 });
        let list = SceneCoordinator::new().compose_with_images_scrolled(
            &tree,
            &computed,
            &boxes,
            None,
            &HashMap::new(),
            &scroll,
        );
        let cmds = list.commands();
        assert!(list.is_balanced(), "scroll + clip brackets stay balanced");

        let clip_open = cmds.iter().position(|c| matches!(c, PaintCommand::PushClip(_))).unwrap();
        let scroll_push = cmds
            .iter()
            .position(|c| matches!(c, PaintCommand::PushTransform(t) if (t.coeffs[5] + 80.0).abs() < 0.01))
            .expect("a translate(0,-80) scroll transform");
        let scroll_pop = cmds
            .iter()
            .rposition(|c| matches!(c, PaintCommand::PopTransform))
            .unwrap();
        let clip_close = cmds.iter().rposition(|c| matches!(c, PaintCommand::PopClip)).unwrap();
        // Clip opens outside the scroll transform; the scroll transform closes
        // inside the clip (scroll_pop before clip_close).
        assert!(clip_open < scroll_push, "scroll transform nests inside the clip");
        assert!(scroll_pop < clip_close, "scroll transform closes before the clip");

        // The scrollbar: two fills AFTER the clip closes (on top, not scrolled).
        let fills_after_clip = cmds[clip_close + 1..]
            .iter()
            .filter(|c| matches!(c, PaintCommand::DrawPath { .. }))
            .count();
        assert_eq!(fills_after_clip, 2, "a scrollbar track + thumb paint on top");
    }

    #[test]
    fn overflow_hidden_brackets_subtree_with_rect_clip() {
        // root (bg + overflow:hidden) -> child (bg) offset BEYOND the root box.
        // The subtree is bracketed by a plain-rect clip to the root's border box,
        // clipping the overflowing child.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let child = add_child(&mut tree, root, 10);

        let mut computed = ComputedStyles::new();
        computed.insert(
            root,
            with_background_and_overflow(
                StyleColor::rgba(1.0, 0.0, 0.0, 1.0),
                layout_keyword::OVERFLOW_HIDDEN,
            ),
        );
        computed.insert(child, with_background(StyleColor::rgba(0.0, 1.0, 0.0, 1.0)));

        let mut boxes = LayoutBoxes::new();
        boxes.insert(root, box_at(0.0, 0.0, 100.0, 60.0));
        // The child extends past the root's right/bottom edges.
        boxes.insert(child, box_at(80.0, 40.0, 60.0, 60.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        let cmds = list.commands();
        // PushClip first, PopClip last — the clip brackets the whole subtree.
        match &cmds[0] {
            PaintCommand::PushClip(Clip::Shape(shape)) => {
                assert!(
                    matches!(shape, obscur_scene::clip::ClipShape::Rect(_)),
                    "overflow with no radius ⇒ a plain rect clip, got {shape:?}"
                );
                assert_eq!(
                    shape.bounds(),
                    Some(Rect::new(Point::new(0.0, 0.0), Size::new(100.0, 60.0))),
                    "clip is the root's border box"
                );
            }
            other => panic!("expected a PushClip rect, got {other:?}"),
        }
        assert!(matches!(cmds.last(), Some(PaintCommand::PopClip)), "clip closes last");
        assert!(list.is_balanced(), "the clip bracket is balanced");
    }

    #[test]
    fn overflow_visible_adds_no_clip() {
        // Regression guard: `overflow: visible` (the initial) adds NO clip — the
        // overflowing child is not clipped, byte-identical to before overflow.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let child = add_child(&mut tree, root, 10);

        let mut computed = ComputedStyles::new();
        computed.insert(
            root,
            with_background_and_overflow(
                StyleColor::rgba(1.0, 0.0, 0.0, 1.0),
                layout_keyword::OVERFLOW_VISIBLE,
            ),
        );
        computed.insert(child, with_background(StyleColor::rgba(0.0, 1.0, 0.0, 1.0)));

        let mut boxes = LayoutBoxes::new();
        boxes.insert(root, box_at(0.0, 0.0, 100.0, 60.0));
        boxes.insert(child, box_at(80.0, 40.0, 60.0, 60.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        assert!(
            !list.commands().iter().any(|c| matches!(
                c,
                PaintCommand::PushClip(_) | PaintCommand::PopClip
            )),
            "overflow: visible adds no clip"
        );
    }

    #[test]
    fn overflow_with_radius_emits_single_rounded_clip() {
        // A node with BOTH a border-radius and overflow:hidden emits exactly ONE
        // clip — the rounded border box — not two.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let child = add_child(&mut tree, root, 10);

        let mut root_style = with_background_and_radius(StyleColor::rgba(1.0, 0.0, 0.0, 1.0), 8.0);
        root_style.set(
            PropertyId::Overflow,
            ComputedValue::keyword(layout_keyword::OVERFLOW_HIDDEN),
        );
        let mut computed = ComputedStyles::new();
        computed.insert(root, root_style);
        computed.insert(child, with_background(StyleColor::rgba(0.0, 1.0, 0.0, 1.0)));

        let mut boxes = LayoutBoxes::new();
        boxes.insert(root, box_at(0.0, 0.0, 100.0, 60.0));
        boxes.insert(child, box_at(80.0, 40.0, 60.0, 60.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        let cmds = list.commands();
        let pushes = cmds
            .iter()
            .filter(|c| matches!(c, PaintCommand::PushClip(_)))
            .count();
        assert_eq!(pushes, 1, "radius + overflow ⇒ a single clip, not two");
        match &cmds[0] {
            PaintCommand::PushClip(Clip::Shape(shape)) => assert!(
                matches!(shape, obscur_scene::clip::ClipShape::RoundedRect { .. }),
                "the single clip is the ROUNDED border box, got {shape:?}"
            ),
            other => panic!("expected a rounded PushClip, got {other:?}"),
        }
        assert!(list.is_balanced(), "the clip bracket is balanced");
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

    /// A text node carrying `content` under the root, box at the origin sized
    /// `w`×`h` (a narrow box wraps the text), red text — the multi-line editing
    /// fixture for wrapped selection/caret geometry.
    fn wrapped_editing_fixture(
        content: &str,
        w: f32,
        h: f32,
    ) -> (RetainedTree, ComputedStyles, LayoutBoxes, NodeId) {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let text = add_text_child(&mut tree, root, content);
        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(text, with_color(StyleColor::rgba(1.0, 0.0, 0.0, 1.0)));
        let mut boxes = LayoutBoxes::new();
        boxes.insert(text, box_at(0.0, 0.0, w, h));
        (tree, computed, boxes, text)
    }

    /// The origin of the single collapsed-caret fill (the thin `CARET_WIDTH` bar).
    fn caret_bar_origin(list: &DisplayList) -> Point {
        caret_fills(list)
            .into_iter()
            .find(|(r, _)| (r.size.width - CARET_WIDTH).abs() < 1e-4)
            .expect("a caret bar")
            .0
            .origin
    }

    #[test]
    fn single_line_selection_still_paints_exactly_one_rect() {
        // (b) A span confined to ONE line (a wide, non-wrapping box) paints a single
        // rect from x(start) to x(end) — byte-identical to the old first-line path.
        let adv = 16.0 * MockTextShaper::ADVANCE_RATIO;
        let (tree, computed, boxes, text) = editing_fixture("abcd");
        let view = EditingView::new(text, "abcd", TextSelection::new(1, 3));
        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, Some(view));

        let fills = caret_fills(&list);
        assert_eq!(fills.len(), 1, "one line → exactly one highlight rect");
        let (rect, brush) = &fills[0];
        assert_eq!(*brush, Brush::solid(SELECTION_COLOR));
        assert!((rect.origin.x - adv).abs() < 1e-4, "starts at x(1) = one advance");
        assert!((rect.size.width - adv * 2.0).abs() < 1e-4, "spans bytes 1..3 = two advances");
        assert_eq!(rect.size.height, 16.0, "spans the line height");
        assert_eq!(rect.origin.y, 0.0, "on the first line");
    }

    #[test]
    fn selection_spanning_three_wrapped_lines_paints_three_per_line_rects() {
        // (a) A 50px box wraps "aaaa bbbb cccc" to three lines; a selection [2,12)
        // covers all three, so the highlight is THREE rects — the start line's TAIL,
        // the middle line FULL, the end line's HEAD — each the domain
        // `SelectionGeometry` rect translated by the box origin (0,0, no scroll).
        let coordinator = SceneCoordinator::new();
        let (tree, computed, boxes, text) =
            wrapped_editing_fixture("aaaa bbbb cccc", 50.0, 48.0);
        let view = EditingView::new(text, "aaaa bbbb cccc", TextSelection::new(2, 12));
        let list = coordinator.compose(&tree, &computed, &boxes, Some(view));

        let highlights: Vec<Rect> = caret_fills(&list)
            .into_iter()
            .filter(|(_, brush)| *brush == Brush::solid(SELECTION_COLOR))
            .map(|(r, _)| r)
            .collect();

        // The painted rects equal the domain geometry over the same wrapped layout.
        let layout =
            coordinator.shape_text_lines(&tree, &computed, text, "aaaa bbbb cccc", 50.0);
        let expected = SelectionGeometry::from_paragraph(&layout, 2, 12);
        assert_eq!(expected.len(), 3, "the selection covers three wrapped lines");
        assert_eq!(highlights.len(), 3, "one highlight rect per covered line");
        for (got, want) in highlights.iter().zip(expected.rects()) {
            assert_eq!(got, want, "painted rect == domain geometry (origin 0, no scroll)");
        }

        // Per-line extents: start TAIL begins mid-line-0 (after 2 glyphs), the
        // interior line is FULL from x 0, the end HEAD begins at x 0 and spans 2
        // glyphs. Lines stack by the 16px line height.
        let adv = 16.0 * MockTextShaper::ADVANCE_RATIO;
        assert!((highlights[0].origin.x - adv * 2.0).abs() < 1e-3, "start tail after 2 glyphs");
        assert_eq!(highlights[0].origin.y, 0.0, "start line");
        assert_eq!(highlights[1].origin.x, 0.0, "interior full width from x 0");
        assert_eq!(highlights[1].origin.y, 16.0, "middle line");
        assert_eq!(highlights[2].origin.x, 0.0, "end head from x 0");
        assert_eq!(highlights[2].origin.y, 32.0, "end line");
        assert!((highlights[2].size.width - adv * 2.0).abs() < 1e-3, "end head spans 2 glyphs");
        assert!(list.is_balanced(), "the display list stays balanced");
    }

    #[test]
    fn caret_at_soft_wrap_boundary_honors_affinity() {
        // (c) "aaaa bbbb cccc" wraps at 50px; byte 5 (the 'b' starting line 1) is a
        // SOFT-wrap boundary shared by line 0's end and line 1's start. Downstream
        // (the default) places the caret at the LEADING edge of the next line
        // (line 1, x 0); Upstream at the TRAILING edge of the previous line (line 0,
        // its used pen width).
        let coordinator = SceneCoordinator::new();
        let (tree, computed, boxes, text) =
            wrapped_editing_fixture("aaaa bbbb cccc", 50.0, 48.0);
        let layout =
            coordinator.shape_text_lines(&tree, &computed, text, "aaaa bbbb cccc", 50.0);

        // Downstream (default affinity) → start of line 1.
        let down = EditingView::new(text, "aaaa bbbb cccc", TextSelection::caret(5));
        let d = caret_bar_origin(&coordinator.compose(&tree, &computed, &boxes, Some(down)));
        assert_eq!(d.y, 16.0, "downstream → line 1 top (byte-identical to before)");
        assert!((d.x - 0.0).abs() < 1e-4, "downstream → line 1 leading edge (x 0)");

        // Upstream affinity → end of line 0.
        let up = EditingView::new(
            text,
            "aaaa bbbb cccc",
            TextSelection::caret(5).with_affinity(CaretAffinity::Upstream),
        );
        let u = caret_bar_origin(&coordinator.compose(&tree, &computed, &boxes, Some(up)));
        assert_eq!(u.y, 0.0, "upstream → line 0 top");
        let line0_trailing = SceneCoordinator::line_caret_x(&layout.lines()[0], 5);
        assert!(line0_trailing > 0.0, "line 0 has a non-zero trailing edge");
        assert!(
            (u.x - line0_trailing).abs() < 1e-3,
            "upstream → line 0 trailing pen x ({line0_trailing}), got {}",
            u.x
        );
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

    // ---- Caret geometry under letter-spacing + text-align (this step) --------

    /// A single text node carrying `content` at the origin in a `box_w`×16 box,
    /// red text, with `mutate` applied to its computed style so a test can author
    /// `letter-spacing` / `text-align`. Mirrors [`editing_fixture`] but exercises
    /// the spacing/alignment the caret + click geometry must honor.
    fn styled_editing_fixture(
        content: &str,
        box_w: f32,
        mutate: impl FnOnce(&mut ComputedStyle),
    ) -> (RetainedTree, ComputedStyles, LayoutBoxes, NodeId) {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let text = add_text_child(&mut tree, root, content);
        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        let mut style = with_color(StyleColor::rgba(1.0, 0.0, 0.0, 1.0));
        mutate(&mut style);
        computed.insert(text, style);
        let mut boxes = LayoutBoxes::new();
        boxes.insert(text, box_at(0.0, 0.0, box_w, 16.0));
        (tree, computed, boxes, text)
    }

    /// The painted x of the single collapsed-caret fill in `list`.
    fn caret_paint_x(list: &DisplayList) -> f32 {
        let fills = caret_fills(list);
        assert_eq!(fills.len(), 1, "exactly one caret fill");
        fills[0].0.origin.x
    }

    /// The sum of every drawn glyph advance across the display list's text runs —
    /// the painted line's used content width (single line, left-aligned).
    fn drawn_run_width(list: &DisplayList) -> f32 {
        list.commands()
            .iter()
            .filter_map(|c| match c {
                PaintCommand::DrawText { run, .. } => {
                    Some(run.glyphs().iter().map(PositionedGlyph::advance).sum::<f32>())
                }
                _ => None,
            })
            .sum()
    }

    #[test]
    fn caret_x_honors_letter_spacing_and_click_is_the_exact_inverse() {
        // A `letter-spacing: 4px` field (CSS Text 3 §8.1): the caret at the end
        // sits on the painted run's right edge — the SPACED glyph advances, not the
        // un-spaced sum — and a click there resolves back to the last byte (an exact
        // inverse). §8.1 adds 4px after each of the 4 clusters, so the painted run
        // width is 4·9.6 + 4·4 = 54.4 (NOT the spacing-blind 4·9.6 = 38.4). The
        // caret is derived from the SAME composed line, so it equals the run right
        // edge by construction.
        let advance = 16.0 * MockTextShaper::ADVANCE_RATIO; // 9.6
        let ls = 4.0_f32;
        let coordinator = SceneCoordinator::new();
        let author = |s: &mut ComputedStyle| {
            s.set(PropertyId::LetterSpacing, ComputedValue::length(Length::px(ls)));
        };
        let (tree, computed, boxes, text) = styled_editing_fixture("abcd", 200.0, author);
        let view = EditingView::new(text, "abcd", TextSelection::caret(4));
        let list = coordinator.compose(&tree, &computed, &boxes, Some(view));

        let caret_x = caret_paint_x(&list);
        let run_right = drawn_run_width(&list);
        let expected = advance * 4.0 + ls * 4.0; // 54.4
        assert!(
            (caret_x - expected).abs() < 0.5,
            "caret x at end is the spaced run width ~{expected}, got {caret_x}",
        );
        assert!(
            (caret_x - run_right).abs() < 0.5,
            "caret sits on the painted run right edge {run_right}, got {caret_x}",
        );
        assert!(
            (caret_x - advance * 4.0).abs() > 1.0,
            "caret x is NOT the spacing-blind sum {}",
            advance * 4.0,
        );
        // The click at the caret x is the exact inverse: it resolves to text.len().
        let byte =
            coordinator.caret_offset_at_point(&tree, &computed, text, "abcd", 200.0, caret_x, 8.0);
        assert_eq!(byte, 4, "click at the end caret x returns text.len()");
        // A caret at a middle boundary and its click are also exact inverses.
        let (tree2, computed2, boxes2, text2) = styled_editing_fixture("abcd", 200.0, author);
        let mid = coordinator
            .compose(
                &tree2,
                &computed2,
                &boxes2,
                Some(EditingView::new(text2, "abcd", TextSelection::caret(2))),
            );
        let mid_x = caret_paint_x(&mid);
        assert!(
            (mid_x - (advance + ls) * 2.0).abs() < 0.5,
            "caret at byte 2 is two spaced advances ~{}, got {mid_x}",
            (advance + ls) * 2.0,
        );
        assert_eq!(
            coordinator.caret_offset_at_point(&tree2, &computed2, text2, "abcd", 200.0, mid_x, 8.0),
            2,
            "click at the byte-2 caret x returns 2",
        );
    }

    #[test]
    fn caret_x_honors_text_align_right_and_click_is_the_exact_inverse() {
        // A `text-align: right` field (CSS 2.1 §16.2): the line is shifted right by
        // `x_offset = box − line_width`, so the end caret sits at the box's right
        // edge (the right-aligned last glyph) and a click there maps back to the
        // last byte. No spacing → line width 4·9.6 = 38.4, box 200 → x_offset 161.6,
        // end caret x = 161.6 + 38.4 = 200.
        let advance = 16.0 * MockTextShaper::ADVANCE_RATIO; // 9.6
        let box_w = 200.0_f32;
        let coordinator = SceneCoordinator::new();
        let author = |s: &mut ComputedStyle| {
            s.set(
                PropertyId::TextAlign,
                ComputedValue::keyword(layout_keyword::TEXT_ALIGN_RIGHT),
            );
        };
        let (tree, computed, boxes, text) = styled_editing_fixture("abcd", box_w, author);
        let view = EditingView::new(text, "abcd", TextSelection::caret(4));
        let list = coordinator.compose(&tree, &computed, &boxes, Some(view));

        let caret_x = caret_paint_x(&list);
        assert!(
            (caret_x - box_w).abs() < 0.5,
            "right-aligned end caret sits at the box right edge {box_w}, got {caret_x}",
        );
        assert_eq!(
            coordinator.caret_offset_at_point(&tree, &computed, text, "abcd", box_w, caret_x, 8.0),
            4,
            "click at the right-aligned end caret x returns text.len()",
        );
        // A middle boundary is still an exact inverse: caret at 1 → x_offset + 1·adv.
        let (t1, c1, b1, n1) = styled_editing_fixture("abcd", box_w, author);
        let mid = coordinator.compose(
            &t1,
            &c1,
            &b1,
            Some(EditingView::new(n1, "abcd", TextSelection::caret(1))),
        );
        let mid_x = caret_paint_x(&mid);
        let x_offset = box_w - advance * 4.0; // 161.6
        assert!(
            (mid_x - (x_offset + advance)).abs() < 0.5,
            "right-aligned caret at byte 1 is x_offset + 1 advance ~{}, got {mid_x}",
            x_offset + advance,
        );
        assert_eq!(
            coordinator.caret_offset_at_point(&t1, &c1, n1, "abcd", box_w, mid_x, 8.0),
            1,
            "click at the right-aligned byte-1 caret x returns 1",
        );
    }

    #[test]
    fn caret_x_honors_text_align_center_and_click_is_the_exact_inverse() {
        // A `text-align: center` field (CSS 2.1 §16.2): `x_offset = (box −
        // line_width)/2`. No spacing → line width 38.4, box 200 → x_offset 80.8, end
        // caret x = 80.8 + 38.4 = 119.2; a click there maps back to text.len().
        let advance = 16.0 * MockTextShaper::ADVANCE_RATIO;
        let box_w = 200.0_f32;
        let coordinator = SceneCoordinator::new();
        let author = |s: &mut ComputedStyle| {
            s.set(
                PropertyId::TextAlign,
                ComputedValue::keyword(layout_keyword::TEXT_ALIGN_CENTER),
            );
        };
        let (tree, computed, boxes, text) = styled_editing_fixture("abcd", box_w, author);
        let view = EditingView::new(text, "abcd", TextSelection::caret(4));
        let list = coordinator.compose(&tree, &computed, &boxes, Some(view));

        let caret_x = caret_paint_x(&list);
        let x_offset = (box_w - advance * 4.0) / 2.0; // 80.8
        let expected = x_offset + advance * 4.0; // 119.2
        assert!(
            (caret_x - expected).abs() < 0.5,
            "centered end caret is x_offset + run width ~{expected}, got {caret_x}",
        );
        assert_eq!(
            coordinator.caret_offset_at_point(&tree, &computed, text, "abcd", box_w, caret_x, 8.0),
            4,
            "click at the centered end caret x returns text.len()",
        );
    }

    #[test]
    fn caret_x_is_byte_identical_for_a_plain_left_aligned_field() {
        // The default field — no letter/word-spacing, no tab, `text-align: start`
        // (spacing 0, x_offset 0) — is byte-identical: the caret x at each boundary
        // is exactly `index · advance`, the same flat un-spaced sum as before, and
        // the click is its exact inverse.
        let advance = 16.0 * MockTextShaper::ADVANCE_RATIO;
        let coordinator = SceneCoordinator::new();
        for index in 0..=4usize {
            let (tree, computed, boxes, text) = styled_editing_fixture("abcd", 200.0, |_| {});
            let view = EditingView::new(text, "abcd", TextSelection::caret(index));
            let list = coordinator.compose(&tree, &computed, &boxes, Some(view));
            let caret_x = caret_paint_x(&list);
            #[allow(clippy::cast_precision_loss)]
            let expected = advance * index as f32;
            assert!(
                (caret_x - expected).abs() < 1e-4,
                "plain field caret at {index} is index·advance {expected}, got {caret_x}",
            );
            assert_eq!(
                coordinator.caret_offset_at_point(
                    &tree, &computed, text, "abcd", 200.0, caret_x, 8.0,
                ),
                index,
                "plain field click at the byte-{index} caret x returns {index}",
            );
        }
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

    /// Sets the reserved `class` list on a retained element, the way the
    /// reconciler would from a `CLASS` prop.
    fn set_class(tree: &mut RetainedTree, node: NodeId, class: &str) {
        use obscur_view::view::{StyleAttributes, ViewProperty, ViewValue, CLASS};
        let attrs =
            StyleAttributes::extract(&[ViewProperty::new(CLASS, ViewValue::Text(class.into()))]);
        if let Some(n) = tree.get_mut(node) {
            n.set_attributes(attrs);
        }
    }

    #[test]
    fn placeholder_overlay_text_is_not_the_fields_editable_text() {
        // The `.placeholder` overlay (CSS `::placeholder`) is a non-editable hint,
        // not the field's editable text: `is_editing_text_node` must return false
        // for text inside a `.placeholder` subtree even though it descends from the
        // editing field, so the compose override never overwrites it with the typed
        // value nor paints a caret into it. A bare (non-placeholder) value text of
        // the SAME field still resolves as editable.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let field = add_child(&mut tree, root, 501);
        // The `.placeholder` overlay element + its (skipped) hint text.
        let overlay = add_child(&mut tree, field, 502);
        set_class(&mut tree, overlay, "placeholder");
        let hint = add_text_child(&mut tree, overlay, "Type here…");
        // A separate, ordinary value text of the same field.
        let value = add_text_child(&mut tree, field, "x");

        assert!(
            !is_editing_text_node(&tree, hint, field),
            "placeholder hint text must NOT be treated as the field's editable text"
        );
        assert!(
            is_editing_text_node(&tree, value, field),
            "the field's ordinary value text is still editable"
        );
        assert!(is_editing_text_node(&tree, field, field), "the field is itself editable");
    }
}
