//! Converts an obscur-scene [`DisplayList`] into a [`vello::Scene`].
//!
//! This is the top-level pure conversion of the adapter (spec §2.7): it walks the
//! display list's commands **in paint order** and replays them onto a
//! `vello::Scene`, delegating geometry/paint/clip translation to the other
//! converters in this module. Paint order, clipping, and layer nesting are
//! preserved exactly; only rasterization is Vello's (spec §5.2 determinism
//! boundary).

use std::collections::HashMap;

use obscur_scene::clip::Clip;
use obscur_scene::color::AlphaMode;
use obscur_scene::command::{DisplayList, PaintCommand};
use obscur_scene::layer::{CompositionMode, LayerEffect, SceneLayer};
use obscur_scene::paint::{BlendMode, Brush, Paint, SceneImage};
use obscur_scene::text::GlyphRun;
use vello::kurbo::{Affine, Diagonal2, Rect as KurboRect};
use vello::peniko::{self, Blob, FontData, ImageAlphaType, ImageData, ImageFormat};
use vello::{FontEmbolden, Scene};

use super::blur_layer::BlurLayer;
use super::brush_converter::BrushConverter;
use super::clip_converter::ClipConverter;
use super::color_converter::ColorConverter;
use super::path_converter::PathConverter;

/// The synthetic-oblique shear factor: the standard faux-italic skew of `0.25`
/// (`tan ≈ 14°`) applied as a per-glyph shear so above-baseline points slide
/// right and the glyph leans forward (spec §5.18). Matches the CPU backend's
/// factor so both fake the same slant.
const FAUX_ITALIC_SKEW: f64 = 0.25;

/// The synthetic-embolden amount as a fraction of the run's font size, fed to
/// Vello's [`FontEmbolden`] to thicken outlines and fake a heavier weight
/// (spec §5.18).
const FAUX_BOLD_EMBOLDEN_FRACTION: f64 = 0.02;

/// One segment of a display list split at [`BackdropFilter`] boundaries by
/// [`SceneConverter::convert_segments`] (spec §5.2, CSS `backdrop-filter`).
///
/// A `backdrop-filter` frosts the frame accumulated so far, so the renderer must
/// composite content and frost the backdrop **in order**. The converter emits an
/// alternating sequence: content to render, a backdrop to frost, more content, …
///
/// [`BackdropFilter`]: PaintCommand::BackdropFilter
// A `Content` segment owns a `vello::Scene` (inherently large, like `BlurLayer`),
// while `Backdrop` is a few floats; boxing the scene would only add an allocation
// to the common variant. The size difference is expected.
#[allow(clippy::large_enum_variant)]
pub enum SceneSegment {
    /// A run of drawable content (between two backdrop boundaries) as a Vello
    /// [`Scene`] plus any `filter: blur` layers extracted from it (composited on
    /// top exactly as the non-segmented path does).
    Content {
        /// The converted sub-scene for this run, in absolute coordinates.
        scene: Scene,
        /// The `filter: blur` layers extracted from this run.
        blurs: Vec<BlurLayer>,
    },
    /// A `backdrop-filter` boundary: frost (gaussian-blur) the accumulated frame
    /// within `bounds`, clipped to the rounded `radii`, by `blur` logical pixels.
    Backdrop {
        /// The panel's border box (the frost region), absolute logical pixels.
        bounds: obscur_geometry::Rect,
        /// The panel's per-corner radii (the frost is clipped to this shape).
        radii: obscur_geometry::CornerRadii,
        /// The CSS blur radius in logical pixels (`> 0`).
        blur: f32,
    },
}

impl std::fmt::Debug for SceneSegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Content { blurs, .. } => f
                .debug_struct("SceneSegment::Content")
                .field("blurs", &blurs.len())
                .field("scene", &"<vello::Scene>")
                .finish(),
            Self::Backdrop {
                bounds,
                radii,
                blur,
            } => f
                .debug_struct("SceneSegment::Backdrop")
                .field("bounds", bounds)
                .field("radii", radii)
                .field("blur", blur)
                .finish(),
        }
    }
}

/// A converter from an obscur-scene [`DisplayList`] to a [`vello::Scene`].
///
/// It owns the leaf converters ([`PathConverter`], [`BrushConverter`],
/// [`ClipConverter`]) and drives them while replaying the command stream.
///
/// # Command mapping
///
/// - [`PaintCommand::DrawPath`] → [`Scene::fill`] (for a
///   [`Paint::Fill`], using the path's own fill rule) or [`Scene::stroke`] (for a
///   [`Paint::Stroke`]), each with the identity transform.
/// - [`PaintCommand::DrawImage`] → [`Scene::draw_image`]: the decoded RGBA8 pixels
///   become a [`peniko::ImageData`] (zero-copy from the image's `Arc<[u8]>`, tagged
///   with its own alpha encoding) drawn at its intrinsic size, with a transform
///   that scales each copy to its tile rect. The tile is placed / sized
///   (`background-position` / `-size`), clipped to the paint region (border box)
///   with a pushed clip layer, and tiled along the `repeat_x` / `repeat_y` axes
///   (`background-repeat`) with a bounded per-copy loop.
/// - [`PaintCommand::PushClip`] → [`Scene::push_layer`] with a neutral blend and
///   the clip's shape, which Vello treats as a clip region.
/// - [`PaintCommand::PushLayer`] → [`Scene::push_layer`] with the layer's blend,
///   composition, and opacity. The layer's clip shape is used when present;
///   otherwise a large finite rectangle stands in for an unbounded layer.
/// - [`PaintCommand::PopClip`] / [`PaintCommand::PopLayer`] → [`Scene::pop_layer`],
///   guarded by a depth counter so an unbalanced list never triggers a stray pop.
///
/// # Layer approximations (spec §5.2)
///
/// A [`SceneLayer`] carries a transform, opacity, blend mode, Porter-Duff
/// operator, and effects. This converter maps opacity, blend mode, and operator
/// (see [`BlendMode`] / [`CompositionMode`] mapping) and clips to the layer's
/// shape. The layer transform and additional effects (e.g. blur) are **not**
/// applied here; they are the renderer sub-step's concern. Paint order and
/// nesting — the properties the determinism boundary requires — are preserved.
///
/// # Text
///
/// [`PaintCommand::DrawText`] carries a pre-shaped
/// [`GlyphRun`](obscur_scene::text::GlyphRun): text is shaped **once**, in the
/// `obscur-text` domain, so this adapter never re-shapes — it consumes the run's
/// glyph positions verbatim (the same run the software backend consumes, spec
/// §5.2). Rasterizing to *real* outlines still needs host fonts, supplied via
/// [`Self::register_font`] / [`Self::set_font`] (fonts are a host resource,
/// mirroring host-provides-device); with no font registered the converter
/// degrades gracefully and skips text runs. Each glyph's
/// [`glyph_id`](obscur_scene::text::PositionedGlyph::glyph_id) is already a
/// **font glyph index** (the domain shaped against the run's resolved font), so it
/// is used **directly** — no Unicode charmap remap — and drawn at the
/// domain-supplied position; Vello's glyph pipeline rasterizes it (spec §5.2).
///
/// ## Multi-font runs
///
/// A [`GlyphRun`] carries a [`font`](GlyphRun::font) handle (`0` = primary): the
/// runtime emits one `DrawText` per font-segment, each tagged with its resolved
/// face's id. The converter keeps a registry keyed by that `u32` handle
/// ([`Self::register_font`]) and draws each run against the matching font, so a
/// mixed-script line (e.g. Latin + Cyrillic resolved to two different faces)
/// rasterizes each segment from its covering font. An unknown handle falls back
/// to id `0`; with neither present the run is skipped (no panic).
#[derive(Debug, Default, Clone)]
pub struct SceneConverter {
    path: PathConverter,
    brush: BrushConverter,
    clip: ClipConverter,
    /// Host-provided fonts keyed by the scene [`GlyphRun`] font handle (`0` =
    /// primary). Empty until [`Self::register_font`] / [`Self::set_font`] is
    /// called; text is skipped while a run's font (and the id-`0` fallback) are
    /// both absent.
    fonts: HashMap<u32, FontData>,
}

impl SceneConverter {
    /// Creates a scene converter with freshly constructed leaf converters.
    ///
    /// No fonts are registered; call [`Self::register_font`] (or the
    /// back-compat [`Self::set_font`]) before text runs will render.
    #[must_use]
    pub fn new() -> Self {
        let color = ColorConverter::new();
        let path = PathConverter::new();
        Self {
            path,
            brush: BrushConverter::new(color),
            clip: ClipConverter::new(path),
            fonts: HashMap::new(),
        }
    }

    /// Registers (or replaces) the host-provided [`FontData`] for the run font
    /// handle `id` (`0` = primary; higher ids are fallback faces).
    ///
    /// Fonts are a host resource; the adapter never fabricates one. The `id` must
    /// match the face id the runtime's font resolver assigned (so scene run
    /// handles line up with these fonts). Until a covering font is registered,
    /// the run's [`PaintCommand::DrawText`] is skipped.
    pub fn register_font(&mut self, id: u32, font: FontData) {
        self.fonts.insert(id, font);
    }

    /// Attaches (or replaces) the primary host font (font handle `0`).
    ///
    /// Back-compat shim over [`Self::register_font`]: single-font callers keep
    /// working, and every untagged run (`font == 0`) renders against this face.
    pub fn set_font(&mut self, font: FontData) {
        self.register_font(0, font);
    }

    /// Selects the [`FontData`] a run tagged with handle `id` draws against:
    /// the exact match if present, otherwise the primary (id `0`) as a fallback,
    /// otherwise `None` (the run is skipped rather than panicking).
    fn font_for(&self, id: u32) -> Option<&FontData> {
        self.fonts.get(&id).or_else(|| self.fonts.get(&0))
    }

    /// Whether a run tagged with handle `id` will draw at all here, or be skipped
    /// for want of a font (spec §5.2).
    ///
    /// [`font_for`](Self::font_for)'s own rule, exposed so the renderer can tell a
    /// host *which* runs its scene will lose. It has to be that same rule: a scan
    /// that disagrees with the draw path reports a skip that did not happen, or
    /// stays quiet about one that did.
    pub(crate) fn can_draw_text(&self, id: u32) -> bool {
        self.font_for(id).is_some()
    }

    /// Converts an obscur-scene [`DisplayList`] into a single [`vello::Scene`],
    /// drawing any `filter: blur` subtree **inline and un-blurred**.
    ///
    /// This is the back-compat entry point (used by callers and tests that do not
    /// run the owned gaussian pass). For the real blur, the renderer uses
    /// [`Self::convert_with_blurs`], which extracts blur subtrees into separate
    /// sub-scenes. With no blur layer the two produce the identical scene.
    #[must_use]
    pub fn convert(&self, list: &DisplayList) -> Scene {
        let mut scene = Scene::new();
        self.emit(&mut scene, list.commands(), Affine::IDENTITY, None);
        scene
    }

    /// Converts an obscur-scene [`DisplayList`] into a main [`vello::Scene`] plus a
    /// list of [`BlurLayer`]s to blur and composite separately (spec §5.2).
    ///
    /// A `filter: blur(r)` element's subtree is bracketed by a `PushLayer(Blur)` …
    /// `PopLayer` (by the scene coordinator) and is therefore **contiguous** in
    /// paint order. This walk detects that bracket, collects the bracketed commands
    /// into their own sub-scene (recorded as a [`BlurLayer`] with the layer's clip
    /// bounds and radius), and **omits** them from the main scene so they are not
    /// also drawn un-blurred. Everything else is emitted exactly as [`Self::convert`]
    /// would, so a list with no blur layer returns `(convert(list), vec![])`,
    /// byte-for-byte identical to the previous single render.
    ///
    /// # Nesting
    ///
    /// Only the top level extracts blurs. A blur layer nested inside another is
    /// drawn as an ordinary layer in the outer sub-scene (its content is blurred
    /// once, by the outer pass, rather than separately) — a documented minimal
    /// nesting limitation, not a panic.
    #[must_use]
    pub fn convert_with_blurs(&self, list: &DisplayList) -> (Scene, Vec<BlurLayer>) {
        let mut scene = Scene::new();
        let mut blurs: Vec<BlurLayer> = Vec::new();
        self.emit(
            &mut scene,
            list.commands(),
            Affine::IDENTITY,
            Some(&mut blurs),
        );
        (scene, blurs)
    }

    /// Splits the display list at each
    /// [`BackdropFilter`](PaintCommand::BackdropFilter) into an ordered list of
    /// [`SceneSegment`]s the renderer composites in sequence (spec §5.2, CSS
    /// `backdrop-filter`).
    ///
    /// `backdrop-filter` frosts the frame **as accumulated so far**, so it is
    /// order-dependent and cannot live inside one Vello scene. This walk breaks the
    /// command stream at every backdrop boundary: the commands between boundaries
    /// become a [`SceneSegment::Content`] (converted exactly as
    /// [`Self::convert_with_blurs`], so each still extracts its own `filter: blur`
    /// layers), and each boundary becomes a [`SceneSegment::Backdrop`] carrying the
    /// frost region + radii + radius. The renderer renders and composites each
    /// content segment onto an accumulating frame, then frosts that frame in place
    /// at each backdrop boundary.
    ///
    /// # Limitation
    ///
    /// Each content run is converted as an **independent** scene, so a `PushClip` /
    /// `PushTransform` left open across a backdrop boundary is closed at the run's
    /// end and does not carry into the next run (its pops there are ignored). A
    /// backdrop-filter nested under such an ancestor is therefore approximate; the
    /// common top-level acrylic panel is exact. A list with no backdrop-filter
    /// returns a single `Content` segment equal to `convert_with_blurs`.
    #[must_use]
    pub fn convert_segments(&self, list: &DisplayList) -> Vec<SceneSegment> {
        let commands = list.commands();
        let mut segments: Vec<SceneSegment> = Vec::new();
        let mut start = 0usize;
        for (index, command) in commands.iter().enumerate() {
            if let PaintCommand::BackdropFilter {
                bounds,
                radii,
                blur,
            } = command
            {
                segments.push(self.content_segment(&commands[start..index]));
                segments.push(SceneSegment::Backdrop {
                    bounds: *bounds,
                    radii: *radii,
                    blur: *blur,
                });
                start = index + 1;
            }
        }
        segments.push(self.content_segment(&commands[start..]));
        segments
    }

    /// Converts one content run (the commands between two backdrop boundaries) into
    /// a [`SceneSegment::Content`], extracting its `filter: blur` layers just like
    /// [`Self::convert_with_blurs`].
    fn content_segment(&self, commands: &[PaintCommand]) -> SceneSegment {
        let mut scene = Scene::new();
        let mut blurs: Vec<BlurLayer> = Vec::new();
        self.emit(&mut scene, commands, Affine::IDENTITY, Some(&mut blurs));
        SceneSegment::Content { scene, blurs }
    }

    /// Replays `commands` onto `scene` under a `base` transform, in paint order.
    ///
    /// When `extract` is `Some`, a `PushLayer` carrying a
    /// [`Blur`](obscur_scene::layer::LayerEffect::Blur) effect (and a clip whose
    /// bounds give the blur region) is peeled off: its bracketed commands become a
    /// separate sub-scene pushed into `extract`, and are **not** drawn here. When
    /// `extract` is `None` (a sub-scene render), such a layer is drawn as an
    /// ordinary layer (its content painted un-blurred). Pops are balanced against a
    /// depth counter, so a malformed list never panics: a stray pop is ignored and
    /// open layers are closed at the end.
    fn emit(
        &self,
        scene: &mut Scene,
        commands: &[PaintCommand],
        base: Affine,
        mut extract: Option<&mut Vec<BlurLayer>>,
    ) {
        let mut depth: u32 = 0;
        // The active CSS 2D `transform` stack: each `PushTransform` composes its
        // affine onto the running transform, and every draw between it and the
        // matching `PopTransform` is mapped by the top of the stack. Empty (== the
        // `base` transform) when no transform is active.
        let mut transforms: Vec<Affine> = Vec::new();
        let mut index = 0;
        while index < commands.len() {
            let current = transforms.last().copied().unwrap_or(base);
            match &commands[index] {
                // A filter layer (a blur and/or a color matrix, with a clip bounds)
                // is extracted into its own sub-scene and omitted here, so it is not
                // also drawn unfiltered.
                PaintCommand::PushLayer(layer)
                    if extract.is_some()
                        && (layer_blur_radius(layer).is_some()
                            || layer_color_matrix(layer).is_some())
                        && layer.clip().and_then(Clip::bounds).is_some() =>
                {
                    let color_matrix = layer_color_matrix(layer);
                    // The bounds are `Some` per the match guard. They come in the
                    // layer's LOCAL space, but the sub-scene below is emitted under
                    // `current` (which folds the root device-scale + any scroll
                    // translate + ancestor transforms), so its content is drawn in
                    // device space. The offscreen the renderer sizes/places from
                    // `bounds` must be in that SAME device space, and the gaussian
                    // radius must scale with it — otherwise at `device_scale > 1`
                    // (HiDPI) the device-space content falls outside a logical-sized
                    // offscreen and the filtered subtree drops out entirely. Map the
                    // bounds through `current` and scale the radius by its linear
                    // factor. At `current == identity` (scale 1, no scroll) this is a
                    // no-op, so the scale-1 path stays byte-identical.
                    let local = layer.clip().and_then(Clip::bounds).unwrap_or_default();
                    let bounds = transform_rect(current, local);
                    let radius = layer_blur_radius(layer).unwrap_or(0.0) * affine_scale(current);
                    let end = matching_layer_pop(commands, index);
                    let inner = &commands[index + 1..end];
                    // Build the bracketed subtree as its own scene. Sub-scenes never
                    // extract nested filters (drawn as ordinary layers — documented).
                    let mut sub = Scene::new();
                    self.emit(&mut sub, inner, current, None);
                    if let Some(blurs) = extract.as_deref_mut() {
                        blurs.push(BlurLayer::new(sub, bounds, radius, color_matrix));
                    }
                    // Skip past the matching PopLayer (both consumed, still balanced).
                    index = end + 1;
                    continue;
                }
                PaintCommand::DrawPath { path, paint } => {
                    self.draw_path(scene, path, paint, current);
                }
                PaintCommand::PushTransform(transform) => {
                    transforms.push(current * to_kurbo_affine(transform));
                }
                PaintCommand::PopTransform => {
                    transforms.pop();
                }
                PaintCommand::PushClip(clip) => {
                    self.push_clip(scene, clip, current);
                    depth += 1;
                }
                PaintCommand::PushLayer(layer) => {
                    self.push_layer(scene, layer, current);
                    depth += 1;
                }
                PaintCommand::PopClip | PaintCommand::PopLayer => {
                    if depth > 0 {
                        scene.pop_layer();
                        depth -= 1;
                    }
                }
                PaintCommand::DrawText { origin, run, brush } => {
                    self.draw_text(scene, *origin, run, brush, current);
                }
                PaintCommand::DrawShadow {
                    rect,
                    radii,
                    color,
                    blur,
                    spread,
                    offset,
                    inset,
                } => {
                    draw_shadow(
                        scene, rect, radii, *color, *blur, *spread, *offset, *inset, current,
                    );
                }
                PaintCommand::DrawImage {
                    image,
                    tile,
                    region,
                    repeat_x,
                    repeat_y,
                } => {
                    draw_image(scene, image, tile, region, *repeat_x, *repeat_y, current);
                }
                // `backdrop-filter` blurs the accumulated frame BEHIND the element,
                // so it cannot be expressed inside a single Vello scene: it is a
                // segmentation boundary consumed by [`Self::convert_segments`] and
                // the renderer's segmented compositing. In the inline
                // (`convert` / `convert_with_blurs`) paths it is a no-op — those
                // back-compat entry points show the backdrop un-frosted (honest;
                // the frost needs the renderer's own wgpu passes).
                PaintCommand::BackdropFilter { .. } => {}
            }
            index += 1;
        }

        // Close any layers a dangling-push (unbalanced) list left open.
        for _ in 0..depth {
            scene.pop_layer();
        }
    }

    /// Rasterizes a single **pre-shaped** text run at `origin`.
    ///
    /// The run was shaped once in the `obscur-text` domain against this same host
    /// font, so this method neither re-shapes nor remaps: each glyph's
    /// [`glyph_id`](obscur_scene::text::PositionedGlyph::glyph_id) is already a
    /// **font glyph index** and is used **directly**, placed at its domain-supplied
    /// run-relative position offset by `origin`. The assembled glyphs are drawn with
    /// Vello's glyph pipeline, brushed with the command's own [`Brush`].
    ///
    /// Degrades gracefully: with no host font set the run is skipped rather than
    /// erroring.
    fn draw_text(
        &self,
        scene: &mut Scene,
        origin: obscur_geometry::Point,
        run: &GlyphRun,
        brush: &Brush,
        transform: Affine,
    ) {
        // Pick the run's font by its handle (`0` = primary); fall back to id 0,
        // else skip. No matching font → no text (documented graceful
        // degradation).
        let Some(font) = self.font_for(run.font()) else {
            return;
        };

        let glyphs: Vec<vello::Glyph> = run
            .glyphs()
            .iter()
            .map(|glyph| {
                // The domain glyph id IS this font's glyph index (shaped against
                // the same host font); use it directly. Positions come from the
                // domain, never re-shaped.
                vello::Glyph {
                    id: glyph.glyph_id(),
                    x: origin.x + glyph.x(),
                    y: origin.y + glyph.y(),
                }
            })
            .collect();
        if glyphs.is_empty() {
            return;
        }

        // Reuse the same Brush→peniko path fills use, so a `Brush::solid(color)`
        // becomes the glyph brush (gradients carry over too).
        let peniko_brush = self.brush.convert(brush);
        let mut draw = scene
            .draw_glyphs(font)
            .font_size(run.font_size())
            .transform(transform)
            .brush(&peniko_brush);
        // Synthetic styling (spec §5.18): when the run resolved to an upright /
        // lighter face because no true italic / bold cut was registered, fake the
        // style. Vello's glyph pipeline exposes both directly — a per-glyph shear
        // for oblique and synthetic embolden for bold.
        if run.synth_oblique() {
            draw = draw.glyph_transform(Some(Affine::skew(FAUX_ITALIC_SKEW, 0.0)));
        }
        if run.synth_bold() {
            let amount = f64::from(run.font_size()) * FAUX_BOLD_EMBOLDEN_FRACTION;
            draw = draw.font_embolden(FontEmbolden::new(Diagonal2::new(amount, amount)));
        }
        draw.draw(peniko::Fill::NonZero, glyphs.into_iter());
    }

    /// Fills or strokes a single path under the active `transform`.
    fn draw_path(
        &self,
        scene: &mut Scene,
        path: &obscur_scene::path::Path,
        paint: &Paint,
        transform: Affine,
    ) {
        let bez = self.path.convert(path);
        let brush = self.brush.convert(paint.brush());
        // An elliptical radial gradient carries a brush transform that stretches the
        // concentric circle into the authored ellipse (spec §3.2).
        let brush_transform = BrushConverter::brush_transform(paint.brush());
        match paint {
            Paint::Fill(_) => {
                let fill = self.path.fill_rule(path.fill_rule());
                scene.fill(fill, transform, &brush, brush_transform, &bez);
            }
            Paint::Stroke(stroke) => {
                let style = self.brush.stroke(stroke);
                scene.stroke(&style, transform, &brush, brush_transform, &bez);
            }
        }
    }

    /// Pushes a clip as a neutral-blend layer bounded by the clip's shape, mapped
    /// by the active `transform`.
    fn push_clip(&self, scene: &mut Scene, clip: &Clip, transform: Affine) {
        let shape = self.clip.shape(clip);
        scene.push_layer(
            peniko::Fill::NonZero,
            peniko::BlendMode::default(),
            1.0,
            transform,
            &shape,
        );
    }

    /// Pushes a compositing layer with its opacity, blend, and clip shape, the
    /// shape mapped by the active `transform`.
    fn push_layer(&self, scene: &mut Scene, layer: &SceneLayer, transform: Affine) {
        let blend = peniko::BlendMode::new(mix(layer.blend_mode()), compose(layer.composition()));
        let alpha = layer.opacity();
        // A mask clip resolves to the same unbounded outline as no clip at all, so
        // both cases go through `ClipConverter` rather than a second, divergent
        // mask rule here (spec §5.2: clipping must match across backends).
        let shape = layer
            .clip()
            .map_or_else(ClipConverter::unbounded, |clip| self.clip.shape(clip));
        scene.push_layer(peniko::Fill::NonZero, blend, alpha, transform, &shape);
    }
}

/// The first [`Blur`](LayerEffect::Blur) radius on a layer, if any. A layer may
/// carry several effects; the coordinator emits exactly one blur, so the first is
/// used.
fn layer_blur_radius(layer: &SceneLayer) -> Option<f32> {
    layer.effects().iter().find_map(|effect| match effect {
        LayerEffect::Blur { radius } => Some(*radius),
        _ => None,
    })
}

/// The first [`ColorMatrix`](LayerEffect::ColorMatrix) folded matrix on a layer, if
/// any — the color-op `filter` chain the coordinator composed into one 4×5 matrix.
fn layer_color_matrix(layer: &SceneLayer) -> Option<[f32; 20]> {
    layer
        .effects()
        .iter()
        .find_map(|effect| effect.as_color_matrix())
}

/// The index of the `PopLayer`/`PopClip` matching the `PushLayer` at `start`,
/// counting nested `Push*`/`Pop*` of both clip and layer kinds. Returns
/// `commands.len()` for an unbalanced list (the rest is treated as the subtree).
fn matching_layer_pop(commands: &[PaintCommand], start: usize) -> usize {
    let mut depth: i32 = 0;
    for (offset, command) in commands[start..].iter().enumerate() {
        match command {
            PaintCommand::PushClip(_) | PaintCommand::PushLayer(_) => depth += 1,
            PaintCommand::PopClip | PaintCommand::PopLayer => {
                depth -= 1;
                if depth == 0 {
                    return start + offset;
                }
            }
            _ => {}
        }
    }
    commands.len()
}

/// Maps an axis-aligned `rect` (a filter layer's local clip bounds) through the
/// accumulated transform `a` and returns the bounding box of the result, in `a`'s
/// target (device) space. For the usual scale + translate `a` (root device-scale,
/// scroll offset, ancestor transforms) the box is exact; for a rotation it is the
/// enclosing AABB, which still contains the whole filtered subtree. At
/// `a == identity` it returns `rect` unchanged, keeping the scale-1 path
/// byte-identical.
#[allow(clippy::cast_possible_truncation)]
fn transform_rect(a: Affine, rect: obscur_geometry::Rect) -> obscur_geometry::Rect {
    let m = a.as_coeffs();
    let map = |x: f64, y: f64| (m[0] * x + m[2] * y + m[4], m[1] * x + m[3] * y + m[5]);
    let x0 = f64::from(rect.origin.x);
    let y0 = f64::from(rect.origin.y);
    let x1 = f64::from(rect.right());
    let y1 = f64::from(rect.bottom());
    let corners = [map(x0, y0), map(x1, y0), map(x0, y1), map(x1, y1)];
    let (mut min_x, mut min_y) = corners[0];
    let (mut max_x, mut max_y) = corners[0];
    for &(px, py) in &corners[1..] {
        min_x = min_x.min(px);
        min_y = min_y.min(py);
        max_x = max_x.max(px);
        max_y = max_y.max(py);
    }
    obscur_geometry::Rect::new(
        obscur_geometry::Point::new(min_x as f32, min_y as f32),
        obscur_geometry::Size::new((max_x - min_x) as f32, (max_y - min_y) as f32),
    )
}

/// The linear (uniform) scale factor of `a`, `sqrt(|det|)` — used to scale a
/// gaussian blur radius from logical to device pixels so the blur strength tracks
/// the element's on-screen size. `1.0` for an identity/translate-only transform.
#[allow(clippy::cast_possible_truncation)]
fn affine_scale(a: Affine) -> f32 {
    let m = a.as_coeffs();
    (m[0] * m[3] - m[1] * m[2]).abs().sqrt() as f32
}

/// Converts an obscur-geometry [`AffineTransform`](obscur_geometry::AffineTransform)
/// into a kurbo [`Affine`]. Both store the six coefficients `[a, b, c, d, e, f]`
/// in the identical layout, so the conversion is a straight widening to `f64`.
fn to_kurbo_affine(t: &obscur_geometry::AffineTransform) -> Affine {
    let m = t.coeffs;
    Affine::new([
        f64::from(m[0]),
        f64::from(m[1]),
        f64::from(m[2]),
        f64::from(m[3]),
        f64::from(m[4]),
        f64::from(m[5]),
    ])
}

/// Rasterizes a CSS `box-shadow` as a gaussian-blurred rounded rectangle
/// (spec §5.2).
///
/// The box's border box is translated by `offset` and inflated (a drop shadow) or
/// deflated (an inset shadow) by `spread`; Vello's
/// [`Scene::draw_blurred_rounded_rect`] then blurs it, mapping the CSS blur
/// *radius* to a gaussian standard deviation of `blur / 2`. Vello takes a single
/// corner radius, so the four `radii` collapse to their maximum (a documented
/// approximation for non-uniform corners). An **inset** shadow is approximated by
/// the same blurred rounded rect at the deflated geometry — Vello does not clip it
/// to the box interior here, so inset fidelity is approximate (documented). A
/// degenerate (non-positive) rect is skipped.
#[allow(clippy::too_many_arguments)]
fn draw_shadow(
    scene: &mut Scene,
    rect: &obscur_geometry::Rect,
    radii: &obscur_geometry::CornerRadii,
    color: obscur_scene::color::Color,
    blur: f32,
    spread: f32,
    offset: (f32, f32),
    inset: bool,
    transform: Affine,
) {
    // An inset shadow is an *inner* shadow confined to the box, not an offset
    // rounded rect — it takes its own path so it is clipped to the box interior
    // and painted as a soft frame rather than a filled blob.
    if inset {
        draw_inset_shadow(scene, rect, radii, color, blur, spread, offset, transform);
        return;
    }
    // Drop shadows inflate by `spread`.
    let inflate = f64::from(spread);
    let (ox, oy) = (f64::from(offset.0), f64::from(offset.1));
    let x0 = f64::from(rect.origin.x) + ox - inflate;
    let y0 = f64::from(rect.origin.y) + oy - inflate;
    let x1 = f64::from(rect.origin.x + rect.size.width) + ox + inflate;
    let y1 = f64::from(rect.origin.y + rect.size.height) + oy + inflate;
    if x1 <= x0 || y1 <= y0 {
        return;
    }
    let kurbo_rect = KurboRect::new(x0, y0, x1, y1);
    // Spec §7.1.1: an outset shadow's corner radii are the box radii grown by
    // `+spread` (a sharp corner stays sharp) so the rounded corners stay
    // concentric with the box. Collapse the grown radii to a single corner
    // radius — the max of the four (documented approximation for non-uniform
    // corners).
    let grown = radii.offset(spread);
    let radius = f64::from(
        grown
            .top_left
            .max(grown.top_right)
            .max(grown.bottom_right)
            .max(grown.bottom_left)
            .max(0.0),
    );
    // Map the CSS blur radius to a gaussian standard deviation.
    let std_dev = f64::from(blur.max(0.0)) / 2.0;
    let peniko_color = ColorConverter::new().convert(color);
    scene.draw_blurred_rounded_rect(transform, kurbo_rect, peniko_color, radius, std_dev);
}

/// Rasterizes an **inset** `box-shadow` as an inner shadow confined to the box.
///
/// Unlike a drop shadow (a blurred rounded rect behind the box), an inset shadow
/// darkens the *inside* edges of the box. It is built in three steps, all clipped
/// to the box's rounded border box so nothing can bleed outside it:
///
/// 1. Push a clip layer to the box's rounded rect (the confinement + isolation).
/// 2. Fill that whole shape with the shadow color.
/// 3. Punch a hole in the fill with a blurred rounded rect — the box deflated by
///    `spread` and shifted by `offset` — composited with [`Compose::DestOut`], so
///    only the soft frame between the box edge and the hole survives. The blur
///    feathers the hole's edge, giving the standard inner-shadow gradient; with
///    `blur == 0` and a positive `spread` the frame is a crisp inner ring.
///
/// The hole is punched with an opaque color because `DestOut` uses only the source
/// alpha; the visible shadow keeps the authored color's own alpha from step 2.
#[allow(clippy::too_many_arguments)]
fn draw_inset_shadow(
    scene: &mut Scene,
    rect: &obscur_geometry::Rect,
    radii: &obscur_geometry::CornerRadii,
    color: obscur_scene::color::Color,
    blur: f32,
    spread: f32,
    offset: (f32, f32),
    transform: Affine,
) {
    let bx0 = f64::from(rect.origin.x);
    let by0 = f64::from(rect.origin.y);
    let bx1 = f64::from(rect.origin.x + rect.size.width);
    let by1 = f64::from(rect.origin.y + rect.size.height);
    if bx1 <= bx0 || by1 <= by0 {
        return;
    }
    // A single corner radius: the max of the four (documented approximation),
    // matching the drop-shadow path.
    let radius = f64::from(
        radii
            .top_left
            .max(radii.top_right)
            .max(radii.bottom_right)
            .max(radii.bottom_left)
            .max(0.0),
    );
    let box_shape = KurboRect::new(bx0, by0, bx1, by1).to_rounded_rect(radius);
    let peniko_color = ColorConverter::new().convert(color);

    // (1) Clip + isolate to the box: the inner shadow can never escape it.
    scene.push_layer(
        peniko::Fill::NonZero,
        peniko::BlendMode::default(),
        1.0,
        transform,
        &box_shape,
    );
    // (2) Paint the shadow color across the whole box interior.
    scene.fill(
        peniko::Fill::NonZero,
        transform,
        peniko_color,
        None,
        &box_shape,
    );

    // (3) Punch out the lit interior (box deflated by `spread`, shifted by
    // `offset`), blurred, leaving only the inner frame.
    let sp = f64::from(spread);
    let ox = f64::from(offset.0);
    let oy = f64::from(offset.1);
    let ix0 = bx0 + ox + sp;
    let iy0 = by0 + oy + sp;
    let ix1 = bx1 + ox - sp;
    let iy1 = by1 + oy - sp;
    if ix1 > ix0 && iy1 > iy0 {
        let inner = KurboRect::new(ix0, iy0, ix1, iy1);
        let inner_radius = (radius - sp).max(0.0);
        let std_dev = f64::from(blur.max(0.0)) / 2.0;
        // Opaque punch: `DestOut` clears the destination by the source alpha.
        let punch = peniko::Color::new([
            peniko_color.components[0],
            peniko_color.components[1],
            peniko_color.components[2],
            1.0,
        ]);
        let dest_out = peniko::BlendMode::new(peniko::Mix::Normal, peniko::Compose::DestOut);
        scene.push_layer(peniko::Fill::NonZero, dest_out, 1.0, transform, &box_shape);
        scene.draw_blurred_rounded_rect(transform, inner, punch, inner_radius, std_dev);
        scene.pop_layer();
    }
    // Close the box clip layer. A collapsed inner rect (huge `spread`) leaves the
    // whole box filled with the shadow color — the correct limit.
    scene.pop_layer();
}

/// A `Sized` wrapper over a shared `Arc<[u8]>` pixel buffer so it can back a
/// peniko [`Blob`] without copying: `Arc<[u8]>` is an unsized fat pointer that
/// cannot coerce to `Arc<dyn AsRef<[u8]>>`, but `Arc<SharedPixels>` can.
struct SharedPixels(std::sync::Arc<[u8]>);

impl AsRef<[u8]> for SharedPixels {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// The largest number of image copies drawn per axis for a tiled
/// (`background-repeat`) background — a bounded-loop guard matching the CPU
/// backend, so both agree. The `region` clip trims any overdraw regardless.
const MAX_TILES_PER_AXIS: i32 = 8192;

/// The smallest tile extent (logical px) that still tiles on an axis; at or below
/// it the axis is treated as non-repeating so a degenerate size cannot spin the
/// tile loop.
const MIN_TILE_EXTENT: f32 = 0.5;

/// Draws a decoded [`SceneImage`] as one copy scaled to `tile`, clipped to
/// `region`, and optionally tiled along the `repeat_x` / `repeat_y` axes to fill
/// `region` (spec §5.16, CSS Backgrounds §3).
///
/// The RGBA8 pixels are wrapped into a [`peniko::ImageData`] — zero-copy, since
/// the [`SceneImage`]'s `Arc<[u8]>` becomes the image [`Blob`] directly — tagged
/// with the image's own alpha encoding ([`AlphaMode::Straight`] →
/// [`ImageAlphaType::Alpha`], [`AlphaMode::Premultiplied`] →
/// [`ImageAlphaType::AlphaPremultiplied`]), so Vello premultiplies honestly rather
/// than this adapter double-premultiplying. [`Scene::draw_image`] draws the image
/// at its intrinsic `width × height`; a per-copy transform scales that to `tile`'s
/// size and translates it to each copy's origin, composed under the active
/// `transform`.
///
/// peniko 0.6's `ImageData` exposes `x_extend` / `y_extend`, but Vello 0.9's
/// `Scene::draw_image` draws a single image instance (extend only pads sampling at
/// the image's own edges, it does not tile across a larger area), so tiling is done
/// with the SAME bounded per-copy loop as the CPU backend, and both are confined to
/// `region` by a pushed clip layer. A single tile fully inside `region` with no
/// tiling takes neither the loop nor the clip — visually identical to the previous
/// stretch draw. A zero-sized image, tile, or region is skipped.
#[allow(clippy::cast_precision_loss, clippy::too_many_arguments)]
fn draw_image(
    scene: &mut Scene,
    image: &SceneImage,
    tile: &obscur_geometry::Rect,
    region: &obscur_geometry::Rect,
    repeat_x: bool,
    repeat_y: bool,
    transform: Affine,
) {
    let (iw, ih) = (image.width(), image.height());
    if iw == 0
        || ih == 0
        || tile.size.width <= 0.0
        || tile.size.height <= 0.0
        || region.size.width <= 0.0
        || region.size.height <= 0.0
    {
        return;
    }
    // A tile too small to tile safely repeats as a single copy on that axis.
    let repeat_x = repeat_x && tile.size.width > MIN_TILE_EXTENT;
    let repeat_y = repeat_y && tile.size.height > MIN_TILE_EXTENT;

    let alpha_type = match image.alpha_mode() {
        AlphaMode::Straight => ImageAlphaType::Alpha,
        AlphaMode::Premultiplied => ImageAlphaType::AlphaPremultiplied,
    };
    let data = ImageData {
        // Zero-copy: wrap the image's shared `Arc<[u8]>` in a `Sized` newtype so it
        // coerces to the `Arc<dyn AsRef<[u8]>>` a `Blob` holds — only a refcount
        // bump, never a pixel copy.
        data: Blob::new(std::sync::Arc::new(SharedPixels(image.pixels_arc()))),
        format: ImageFormat::Rgba8,
        alpha_type,
        width: iw,
        height: ih,
    };

    // Confine drawing to `region` unless a single tile already sits inside it.
    let contained = !repeat_x
        && !repeat_y
        && tile.origin.x >= region.origin.x - 1e-3
        && tile.origin.y >= region.origin.y - 1e-3
        && tile.origin.x + tile.size.width <= region.origin.x + region.size.width + 1e-3
        && tile.origin.y + tile.size.height <= region.origin.y + region.size.height + 1e-3;
    let clipped = !contained;
    if clipped {
        let clip = KurboRect::new(
            f64::from(region.origin.x),
            f64::from(region.origin.y),
            f64::from(region.origin.x + region.size.width),
            f64::from(region.origin.y + region.size.height),
        );
        scene.push_layer(
            peniko::Fill::NonZero,
            peniko::BlendMode::default(),
            1.0,
            transform,
            &clip,
        );
    }

    let (i0, i1) = tile_index_range(
        repeat_x,
        region.origin.x,
        region.origin.x + region.size.width,
        tile.origin.x,
        tile.size.width,
    );
    let (j0, j1) = tile_index_range(
        repeat_y,
        region.origin.y,
        region.origin.y + region.size.height,
        tile.origin.y,
        tile.size.height,
    );
    // Intrinsic → tile: scale the natural `iw × ih` to the tile size (constant
    // across copies), then translate to each copy's origin.
    let scale = Affine::scale_non_uniform(
        f64::from(tile.size.width) / f64::from(iw),
        f64::from(tile.size.height) / f64::from(ih),
    );
    for j in j0..=j1 {
        for i in i0..=i1 {
            let ox = f64::from(tile.origin.x) + f64::from(i) * f64::from(tile.size.width);
            let oy = f64::from(tile.origin.y) + f64::from(j) * f64::from(tile.size.height);
            let place = Affine::translate((ox, oy));
            scene.draw_image(&data, transform * place * scale);
        }
    }

    if clipped {
        scene.pop_layer();
    }
}

/// Computes the inclusive tile index range `[start, end]` covering the span
/// `[r0, r1]` with copies at `t0 + k * ts` (`ts > 0`), matching the CPU backend. A
/// non-repeating axis is the single index `0`; the range is padded by one on each
/// side and clamped to [`MAX_TILES_PER_AXIS`] copies (the `region` clip trims any
/// overdraw).
#[allow(clippy::cast_possible_truncation)]
fn tile_index_range(repeat: bool, r0: f32, r1: f32, t0: f32, ts: f32) -> (i32, i32) {
    if !repeat || ts <= 0.0 {
        return (0, 0);
    }
    let start_f = ((r0 - t0) / ts).floor();
    let end_f = ((r1 - t0) / ts).ceil();
    if !start_f.is_finite() || !end_f.is_finite() {
        return (0, 0);
    }
    let start = start_f as i32 - 1;
    let mut end = end_f as i32 + 1;
    if end - start + 1 > MAX_TILES_PER_AXIS {
        end = start + MAX_TILES_PER_AXIS - 1;
    }
    (start, end)
}

/// Maps an obscur-scene [`BlendMode`] to a peniko [`Mix`](peniko::Mix) blend.
fn mix(mode: BlendMode) -> peniko::Mix {
    match mode {
        BlendMode::Normal => peniko::Mix::Normal,
        BlendMode::Multiply => peniko::Mix::Multiply,
        BlendMode::Screen => peniko::Mix::Screen,
        BlendMode::Overlay => peniko::Mix::Overlay,
        BlendMode::Darken => peniko::Mix::Darken,
        BlendMode::Lighten => peniko::Mix::Lighten,
        BlendMode::ColorDodge => peniko::Mix::ColorDodge,
        BlendMode::ColorBurn => peniko::Mix::ColorBurn,
        BlendMode::HardLight => peniko::Mix::HardLight,
        BlendMode::SoftLight => peniko::Mix::SoftLight,
        BlendMode::Difference => peniko::Mix::Difference,
        BlendMode::Exclusion => peniko::Mix::Exclusion,
    }
}

/// Maps an obscur-scene [`CompositionMode`] to a peniko
/// [`Compose`](peniko::Compose) operator.
fn compose(mode: CompositionMode) -> peniko::Compose {
    match mode {
        CompositionMode::SrcOver => peniko::Compose::SrcOver,
        CompositionMode::Src => peniko::Compose::Copy,
        CompositionMode::DstOver => peniko::Compose::DestOver,
        CompositionMode::Clear => peniko::Compose::Clear,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_geometry::{Point, Rect, Size};
    use obscur_scene::color::Color;
    use obscur_scene::paint::{Brush, Stroke};
    use obscur_scene::path::Path;

    fn rect_path() -> Path {
        Path::rect(Rect::new(Point::new(0.0, 0.0), Size::new(10.0, 10.0)))
    }

    #[test]
    fn empty_list_produces_empty_scene() {
        let scene = SceneConverter::new().convert(&DisplayList::new());
        assert!(scene.encoding().is_empty());
    }

    #[test]
    fn single_filled_rect_produces_non_empty_scene() {
        let list = DisplayList::from_commands(vec![PaintCommand::DrawPath {
            path: rect_path(),
            paint: Paint::solid_fill(Color::rgba(1.0, 0.0, 0.0, 1.0)),
        }]);
        let scene = SceneConverter::new().convert(&list);
        assert!(!scene.encoding().is_empty());
    }

    #[test]
    fn stroked_rect_produces_non_empty_scene() {
        let list = DisplayList::from_commands(vec![PaintCommand::DrawPath {
            path: rect_path(),
            paint: Paint::stroke(Stroke::new(Brush::solid(Color::BLACK), 2.0)),
        }]);
        let scene = SceneConverter::new().convert(&list);
        assert!(!scene.encoding().is_empty());
    }

    #[test]
    fn balanced_clip_list_converts_without_panicking() {
        let list = DisplayList::from_commands(vec![
            PaintCommand::PushClip(Clip::rect(Rect::new(
                Point::new(0.0, 0.0),
                Size::new(5.0, 5.0),
            ))),
            PaintCommand::DrawPath {
                path: rect_path(),
                paint: Paint::solid_fill(Color::WHITE),
            },
            PaintCommand::PopClip,
        ]);
        assert!(list.is_balanced());
        let scene = SceneConverter::new().convert(&list);
        assert!(!scene.encoding().is_empty());
    }

    #[test]
    fn nested_layer_and_clip_convert() {
        let layer = SceneLayer::new()
            .with_opacity(0.5)
            .with_blend_mode(BlendMode::Multiply);
        let list = DisplayList::from_commands(vec![
            PaintCommand::PushLayer(layer),
            PaintCommand::PushClip(Clip::rect(Rect::new(
                Point::new(0.0, 0.0),
                Size::new(5.0, 5.0),
            ))),
            PaintCommand::DrawPath {
                path: rect_path(),
                paint: Paint::solid_fill(Color::WHITE),
            },
            PaintCommand::PopClip,
            PaintCommand::PopLayer,
        ]);
        // Must not panic; balanced push/pop.
        let _scene = SceneConverter::new().convert(&list);
    }

    #[test]
    fn draw_image_produces_non_empty_scene() {
        use obscur_scene::color::AlphaMode;
        use obscur_scene::paint::SceneImage;
        use std::sync::Arc;

        // A 2×2 opaque image stretched to an 8×8 tile == region rect (no repeat).
        let image = SceneImage::new(2, 2, Arc::from(vec![255u8; 16]), AlphaMode::Straight)
            .expect("2x2 image");
        let rect = Rect::new(Point::new(1.0, 1.0), Size::new(8.0, 8.0));
        let list = DisplayList::from_commands(vec![PaintCommand::DrawImage {
            image,
            tile: rect,
            region: rect,
            repeat_x: false,
            repeat_y: false,
        }]);
        let scene = SceneConverter::new().convert(&list);
        assert!(!scene.encoding().is_empty());
    }

    #[test]
    fn repeated_image_tiles_and_stays_balanced() {
        use obscur_scene::color::AlphaMode;
        use obscur_scene::paint::SceneImage;
        use std::sync::Arc;

        // A 2×2 image, a 4×4 tile, repeated across a 16×16 region: the tile loop
        // draws several copies inside a pushed region clip layer, and the scene is
        // non-empty. (The layer is popped, so encoding stays well-formed.)
        let image = SceneImage::new(2, 2, Arc::from(vec![255u8; 16]), AlphaMode::Straight)
            .expect("2x2 image");
        let list = DisplayList::from_commands(vec![PaintCommand::DrawImage {
            image,
            tile: Rect::new(Point::new(0.0, 0.0), Size::new(4.0, 4.0)),
            region: Rect::new(Point::new(0.0, 0.0), Size::new(16.0, 16.0)),
            repeat_x: true,
            repeat_y: true,
        }]);
        let scene = SceneConverter::new().convert(&list);
        assert!(!scene.encoding().is_empty());
    }

    #[test]
    fn zero_sized_image_encodes_nothing() {
        use obscur_scene::color::AlphaMode;
        use obscur_scene::paint::SceneImage;
        use std::sync::Arc;

        // A 0×0 image has no pixels to sample: it is skipped, not encoded.
        let image =
            SceneImage::new(0, 0, Arc::from(Vec::new()), AlphaMode::Straight).expect("0x0 image");
        let rect = Rect::new(Point::new(0.0, 0.0), Size::new(8.0, 8.0));
        let list = DisplayList::from_commands(vec![PaintCommand::DrawImage {
            image,
            tile: rect,
            region: rect,
            repeat_x: false,
            repeat_y: false,
        }]);
        let scene = SceneConverter::new().convert(&list);
        assert!(scene.encoding().is_empty());
    }

    #[test]
    fn draw_shadow_produces_non_empty_scene() {
        use obscur_geometry::CornerRadii;
        let list = DisplayList::from_commands(vec![PaintCommand::DrawShadow {
            rect: Rect::new(Point::new(0.0, 0.0), Size::new(20.0, 10.0)),
            radii: CornerRadii::uniform(4.0),
            color: Color::BLACK,
            blur: 8.0,
            spread: 0.0,
            offset: (4.0, 4.0),
            inset: false,
        }]);
        let scene = SceneConverter::new().convert(&list);
        assert!(!scene.encoding().is_empty());
    }

    #[test]
    fn inset_shadow_produces_non_empty_balanced_scene() {
        use obscur_geometry::CornerRadii;
        let list = DisplayList::from_commands(vec![PaintCommand::DrawShadow {
            rect: Rect::new(Point::new(0.0, 0.0), Size::new(20.0, 20.0)),
            radii: CornerRadii::default(),
            color: Color::rgba(0.0, 0.0, 0.0, 0.5),
            blur: 2.0,
            spread: 2.0,
            offset: (0.0, 0.0),
            inset: true,
        }]);
        // The inner-frame path (box clip + fill + DestOut punch) encodes content
        // and every pushed layer is popped (must not panic or dangle).
        let scene = SceneConverter::new().convert(&list);
        assert!(!scene.encoding().is_empty());
    }

    #[test]
    fn inset_shadow_with_collapsed_hole_still_converts() {
        use obscur_geometry::CornerRadii;
        // A spread larger than half the box collapses the inner hole: the whole box
        // fills with the shadow color (no punch layer) and must not panic.
        let list = DisplayList::from_commands(vec![PaintCommand::DrawShadow {
            rect: Rect::new(Point::new(0.0, 0.0), Size::new(10.0, 10.0)),
            radii: CornerRadii::default(),
            color: Color::BLACK,
            blur: 0.0,
            spread: 40.0,
            offset: (0.0, 0.0),
            inset: true,
        }]);
        let scene = SceneConverter::new().convert(&list);
        assert!(!scene.encoding().is_empty());
    }

    #[test]
    fn unbalanced_stray_pop_does_not_panic() {
        let list = DisplayList::from_commands(vec![
            PaintCommand::PopClip,
            PaintCommand::DrawPath {
                path: rect_path(),
                paint: Paint::solid_fill(Color::WHITE),
            },
        ]);
        assert!(!list.is_balanced());
        let scene = SceneConverter::new().convert(&list);
        assert!(!scene.encoding().is_empty());
    }

    #[test]
    fn draw_text_without_font_is_skipped() {
        use obscur_scene::text::{GlyphRun, PositionedGlyph};
        // With no host font set, a pre-shaped run degrades gracefully: it neither
        // panics nor encodes anything.
        let run = GlyphRun::new(
            vec![PositionedGlyph::new(u32::from('C'), 0, 0.0, 16.0, 9.6)],
            16.0,
            "Count: 0",
        );
        let list = DisplayList::from_commands(vec![PaintCommand::DrawText {
            origin: Point::new(0.0, 0.0),
            run,
            brush: Brush::solid(Color::WHITE),
        }]);
        let scene = SceneConverter::new().convert(&list);
        assert!(scene.encoding().is_empty());
    }

    #[test]
    fn set_font_and_register_font_populate_the_registry() {
        use vello::peniko::Blob;
        let mut converter = SceneConverter::new();
        // `set_font` is the back-compat shim: it registers the primary (id 0).
        converter.set_font(FontData::new(Blob::from(vec![1u8, 2, 3]), 0));
        // `register_font` adds a distinct fallback face under id 1.
        converter.register_font(1, FontData::new(Blob::from(vec![4u8, 5, 6]), 0));
        assert_eq!(converter.fonts.len(), 2);
        assert!(converter.fonts.contains_key(&0));
        assert!(converter.fonts.contains_key(&1));
    }

    #[test]
    fn run_font_handle_selects_the_matching_registered_font() {
        use vello::peniko::Blob;
        let primary = FontData::new(Blob::from(vec![1u8, 1, 1]), 0);
        let fallback = FontData::new(Blob::from(vec![2u8, 2, 2]), 0);
        let mut converter = SceneConverter::new();
        converter.register_font(0, primary.clone());
        converter.register_font(1, fallback.clone());
        // A run tagged `font = 1` draws against font id 1, not the primary.
        assert_eq!(converter.font_for(1), Some(&fallback));
        // The primary (untagged, `font = 0`) run still draws against id 0.
        assert_eq!(converter.font_for(0), Some(&primary));
    }

    #[test]
    fn unknown_font_handle_falls_back_to_primary() {
        use vello::peniko::Blob;
        let primary = FontData::new(Blob::from(vec![7u8, 7, 7]), 0);
        let mut converter = SceneConverter::new();
        converter.register_font(0, primary.clone());
        // No font id 5 is registered → fall back to the primary (id 0).
        assert_eq!(converter.font_for(5), Some(&primary));
    }

    #[test]
    fn unknown_font_handle_without_primary_is_skipped_without_panic() {
        use obscur_scene::text::{GlyphRun, PositionedGlyph};
        use vello::peniko::Blob;
        // Only a fallback face (id 1) is registered — no primary (id 0). A run
        // tagged with an unknown handle (2) resolves to neither, so it is
        // skipped: `font_for` is `None` and conversion encodes nothing.
        let mut converter = SceneConverter::new();
        converter.register_font(1, FontData::new(Blob::from(vec![9u8]), 0));
        assert!(converter.font_for(2).is_none());
        let run =
            GlyphRun::new(vec![PositionedGlyph::new(1, 0, 0.0, 16.0, 9.6)], 16.0, "x").with_font(2);
        let list = DisplayList::from_commands(vec![PaintCommand::DrawText {
            origin: Point::new(0.0, 0.0),
            run,
            brush: Brush::solid(Color::WHITE),
        }]);
        let scene = converter.convert(&list);
        assert!(scene.encoding().is_empty());
    }

    #[test]
    fn can_draw_text_answers_exactly_what_font_for_resolves() {
        // The §5.2 fallback scan asks this before reporting a run skipped, so it has
        // to agree with `font_for` on every case — a stricter answer invents a
        // fallback that never happens, a looser one stays quiet while the run
        // vanishes.
        let mut converter = SceneConverter::new();
        // Nothing registered: no run draws.
        assert!(!converter.can_draw_text(0));
        assert!(!converter.can_draw_text(3));

        // A non-primary face covers its own handle only.
        converter.register_font(1, FontData::new(Blob::from(vec![9u8]), 0));
        assert!(converter.can_draw_text(1));
        assert!(!converter.can_draw_text(3));

        // A primary (id 0) covers every handle, which is why an uncovered run is a
        // fallback here only when the primary is missing too.
        converter.register_font(0, FontData::new(Blob::from(vec![7u8]), 0));
        assert!(converter.can_draw_text(3));
        assert_eq!(converter.can_draw_text(3), converter.font_for(3).is_some());
    }

    #[test]
    fn blur_layer_is_extracted_into_a_sub_scene() {
        use obscur_scene::layer::LayerEffect;

        // A blur layer bracketing a filled rect: `convert_with_blurs` peels it into
        // a BlurLayer (sub-scene + bounds + radius) and OMITS it from the main scene.
        let bounds = Rect::new(Point::new(2.0, 3.0), Size::new(20.0, 10.0));
        let mut layer = SceneLayer::new().with_clip(Clip::rect(bounds));
        layer.add_effect(LayerEffect::blur(6.0));
        let list = DisplayList::from_commands(vec![
            PaintCommand::PushLayer(layer),
            PaintCommand::DrawPath {
                path: rect_path(),
                paint: Paint::solid_fill(Color::rgba(1.0, 0.0, 0.0, 1.0)),
            },
            PaintCommand::PopLayer,
        ]);

        let (main, blurs) = SceneConverter::new().convert_with_blurs(&list);
        // The blurred content moved to a sub-scene, so the main scene is empty.
        assert!(
            main.encoding().is_empty(),
            "blurred content omitted from main"
        );
        assert_eq!(blurs.len(), 1);
        assert_eq!(blurs[0].bounds, bounds);
        assert_eq!(blurs[0].radius, 6.0);
        // The sub-scene actually holds the bracketed draw.
        assert!(
            !blurs[0].scene.encoding().is_empty(),
            "sub-scene has the content"
        );

        // Back-compat: `convert` draws the same content inline (un-blurred), so it
        // is non-empty and carries no separate blur layer.
        let inline = SceneConverter::new().convert(&list);
        assert!(!inline.encoding().is_empty());
    }

    #[test]
    fn blur_bounds_and_radius_follow_the_root_device_scale() {
        use obscur_scene::layer::LayerEffect;

        // A blur layer nested under a root `scale(2)` transform (as the runtime
        // brackets the whole scene at `device_scale == 2` on HiDPI). The sub-scene
        // is emitted in device space, so its bounds and radius must ALSO be in
        // device space — otherwise the device-space content falls outside a
        // logical-sized offscreen and the filtered subtree drops out (the empty
        // "linearRGB blur" card bug).
        let bounds = Rect::new(Point::new(2.0, 3.0), Size::new(20.0, 10.0));
        let mut layer = SceneLayer::new().with_clip(Clip::rect(bounds));
        layer.add_effect(LayerEffect::blur(6.0));
        let list = DisplayList::from_commands(vec![
            PaintCommand::PushTransform(obscur_geometry::AffineTransform::scale(2.0, 2.0)),
            PaintCommand::PushLayer(layer),
            PaintCommand::DrawPath {
                path: rect_path(),
                paint: Paint::solid_fill(Color::rgba(1.0, 0.0, 0.0, 1.0)),
            },
            PaintCommand::PopLayer,
            PaintCommand::PopTransform,
        ]);

        let (_main, blurs) = SceneConverter::new().convert_with_blurs(&list);
        assert_eq!(blurs.len(), 1);
        // Bounds scaled to device space: origin (2,3)→(4,6), size (20,10)→(40,20).
        assert_eq!(
            blurs[0].bounds,
            Rect::new(Point::new(4.0, 6.0), Size::new(40.0, 20.0))
        );
        // Radius scaled by the linear factor 2: 6 → 12.
        assert!((blurs[0].radius - 12.0).abs() < 1e-4);
    }

    #[test]
    fn color_matrix_layer_is_extracted_with_its_matrix() {
        use obscur_scene::layer::LayerEffect;

        // A color-ONLY filter layer (a ColorMatrix effect, no blur) is peeled into a
        // BlurLayer carrying `radius == 0` and the folded matrix — driving the
        // renderer's owned color-matrix pass.
        let bounds = Rect::new(Point::new(1.0, 1.0), Size::new(16.0, 8.0));
        let mut matrix = [0.0_f32; 20];
        matrix[0] = 0.5; // an arbitrary non-identity matrix
        matrix[5] = 0.5;
        matrix[10] = 0.5;
        matrix[15] = 1.0;
        let mut layer = SceneLayer::new().with_clip(Clip::rect(bounds));
        layer.add_effect(LayerEffect::color_matrix(matrix));
        let list = DisplayList::from_commands(vec![
            PaintCommand::PushLayer(layer),
            PaintCommand::DrawPath {
                path: rect_path(),
                paint: Paint::solid_fill(Color::rgba(1.0, 0.0, 0.0, 1.0)),
            },
            PaintCommand::PopLayer,
        ]);

        let (main, filters) = SceneConverter::new().convert_with_blurs(&list);
        assert!(
            main.encoding().is_empty(),
            "filtered content omitted from main"
        );
        assert_eq!(filters.len(), 1);
        assert_eq!(filters[0].bounds, bounds);
        assert_eq!(filters[0].radius, 0.0, "color-only filter has no blur");
        assert_eq!(
            filters[0].color_matrix,
            Some(matrix),
            "matrix carried through"
        );
        assert!(
            !filters[0].scene.encoding().is_empty(),
            "sub-scene has the content"
        );
    }

    #[test]
    fn backdrop_filter_splits_the_display_list_into_segments() {
        use obscur_geometry::CornerRadii;
        // A backdrop (a filled rect) + a BackdropFilter boundary + a panel fill:
        // `convert_segments` yields Content(backdrop) → Backdrop → Content(panel).
        let bounds = Rect::new(Point::new(4.0, 4.0), Size::new(20.0, 12.0));
        let list = DisplayList::from_commands(vec![
            PaintCommand::DrawPath {
                path: rect_path(),
                paint: Paint::solid_fill(Color::rgba(0.0, 0.0, 1.0, 1.0)),
            },
            PaintCommand::BackdropFilter {
                bounds,
                radii: CornerRadii::uniform(6.0),
                blur: 10.0,
            },
            PaintCommand::DrawPath {
                path: rect_path(),
                paint: Paint::solid_fill(Color::rgba(1.0, 1.0, 1.0, 0.3)),
            },
        ]);

        let segments = SceneConverter::new().convert_segments(&list);
        assert_eq!(segments.len(), 3, "content / backdrop / content");
        match &segments[0] {
            SceneSegment::Content { scene, blurs } => {
                assert!(!scene.encoding().is_empty(), "backdrop content present");
                assert!(blurs.is_empty());
            }
            other => panic!("expected leading content, got {other:?}"),
        }
        match &segments[1] {
            SceneSegment::Backdrop {
                bounds: b, blur, ..
            } => {
                assert_eq!(*b, bounds);
                assert_eq!(*blur, 10.0);
            }
            other => panic!("expected a backdrop boundary, got {other:?}"),
        }
        match &segments[2] {
            SceneSegment::Content { scene, .. } => {
                assert!(!scene.encoding().is_empty(), "panel content present");
            }
            other => panic!("expected trailing content, got {other:?}"),
        }
    }

    #[test]
    fn no_backdrop_filter_yields_a_single_content_segment() {
        // With no BackdropFilter the whole list is one Content segment (the
        // byte-identical non-segmented path).
        let list = DisplayList::from_commands(vec![PaintCommand::DrawPath {
            path: rect_path(),
            paint: Paint::solid_fill(Color::rgba(0.0, 0.0, 1.0, 1.0)),
        }]);
        let segments = SceneConverter::new().convert_segments(&list);
        assert_eq!(segments.len(), 1);
        assert!(matches!(segments[0], SceneSegment::Content { .. }));
    }

    #[test]
    fn no_blur_layer_yields_no_blur_layers() {
        // A plain filled rect: `convert_with_blurs` returns the same scene `convert`
        // does and an empty blur list (the byte-identical no-blur path).
        let list = DisplayList::from_commands(vec![PaintCommand::DrawPath {
            path: rect_path(),
            paint: Paint::solid_fill(Color::rgba(0.0, 0.0, 1.0, 1.0)),
        }]);
        let (main, blurs) = SceneConverter::new().convert_with_blurs(&list);
        assert!(blurs.is_empty());
        assert!(!main.encoding().is_empty());
    }

    #[test]
    fn unbalanced_missing_pop_does_not_panic() {
        let list = DisplayList::from_commands(vec![
            PaintCommand::PushClip(Clip::rect(Rect::new(
                Point::new(0.0, 0.0),
                Size::new(5.0, 5.0),
            ))),
            PaintCommand::DrawPath {
                path: rect_path(),
                paint: Paint::solid_fill(Color::WHITE),
            },
        ]);
        assert!(!list.is_balanced());
        // Dangling push is closed at the end; no panic.
        let _scene = SceneConverter::new().convert(&list);
    }
}
