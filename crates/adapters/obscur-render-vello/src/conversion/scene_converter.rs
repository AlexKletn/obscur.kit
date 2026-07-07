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
use obscur_scene::command::{DisplayList, PaintCommand};
use obscur_scene::color::AlphaMode;
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

/// Half-extent (in logical pixels) of the fallback clip used for a layer with no
/// geometric clip shape. Large enough to bound any realistic scene while
/// remaining finite (so it never produces non-finite path coordinates).
const UNBOUNDED_HALF_EXTENT: f64 = 1.0e9;

/// The synthetic-oblique shear factor: the standard faux-italic skew of `0.25`
/// (`tan ≈ 14°`) applied as a per-glyph shear so above-baseline points slide
/// right and the glyph leans forward (spec §5.18). Matches the CPU backend's
/// factor so both fake the same slant.
const FAUX_ITALIC_SKEW: f64 = 0.25;

/// The synthetic-embolden amount as a fraction of the run's font size, fed to
/// Vello's [`FontEmbolden`] to thicken outlines and fake a heavier weight
/// (spec §5.18).
const FAUX_BOLD_EMBOLDEN_FRACTION: f64 = 0.02;

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
                // A blur layer (with a clip bounds) is extracted into its own
                // sub-scene and omitted here, so it is not also drawn un-blurred.
                PaintCommand::PushLayer(layer)
                    if extract.is_some()
                        && layer_blur_radius(layer).is_some()
                        && layer.clip().and_then(Clip::bounds).is_some() =>
                {
                    let radius = layer_blur_radius(layer).unwrap_or(0.0);
                    // Both are `Some` per the match guard.
                    let bounds = layer.clip().and_then(Clip::bounds).unwrap_or_default();
                    let end = matching_layer_pop(commands, index);
                    let inner = &commands[index + 1..end];
                    // Build the bracketed subtree as its own scene. Sub-scenes never
                    // extract nested blurs (drawn as ordinary layers — documented).
                    let mut sub = Scene::new();
                    self.emit(&mut sub, inner, current, None);
                    if let Some(blurs) = extract.as_deref_mut() {
                        blurs.push(BlurLayer::new(sub, bounds, radius));
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
        match paint {
            Paint::Fill(_) => {
                let fill = self.path.fill_rule(path.fill_rule());
                scene.fill(fill, transform, &brush, None, &bez);
            }
            Paint::Stroke(stroke) => {
                let style = self.brush.stroke(stroke);
                scene.stroke(&style, transform, &brush, None, &bez);
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
        if let Some(clip @ Clip::Shape(_)) = layer.clip() {
            let shape = self.clip.shape(clip);
            scene.push_layer(peniko::Fill::NonZero, blend, alpha, transform, &shape);
        } else {
            let unbounded = KurboRect::new(
                -UNBOUNDED_HALF_EXTENT,
                -UNBOUNDED_HALF_EXTENT,
                UNBOUNDED_HALF_EXTENT,
                UNBOUNDED_HALF_EXTENT,
            );
            scene.push_layer(peniko::Fill::NonZero, blend, alpha, transform, &unbounded);
        }
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
    // Drop shadows inflate by `spread`; inset shadows deflate by it.
    let inflate = f64::from(if inset { -spread } else { spread });
    let (ox, oy) = (f64::from(offset.0), f64::from(offset.1));
    let x0 = f64::from(rect.origin.x) + ox - inflate;
    let y0 = f64::from(rect.origin.y) + oy - inflate;
    let x1 = f64::from(rect.origin.x + rect.size.width) + ox + inflate;
    let y1 = f64::from(rect.origin.y + rect.size.height) + oy + inflate;
    if x1 <= x0 || y1 <= y0 {
        return;
    }
    let kurbo_rect = KurboRect::new(x0, y0, x1, y1);
    // A single corner radius: the max of the four (documented approximation).
    let radius = f64::from(
        radii
            .top_left
            .max(radii.top_right)
            .max(radii.bottom_right)
            .max(radii.bottom_left)
            .max(0.0),
    );
    // Map the CSS blur radius to a gaussian standard deviation.
    let std_dev = f64::from(blur.max(0.0)) / 2.0;
    let peniko_color = ColorConverter::new().convert(color);
    scene.draw_blurred_rounded_rect(transform, kurbo_rect, peniko_color, radius, std_dev);
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
        scene.push_layer(peniko::Fill::NonZero, peniko::BlendMode::default(), 1.0, transform, &clip);
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
        let image = SceneImage::new(0, 0, Arc::from(Vec::new()), AlphaMode::Straight)
            .expect("0x0 image");
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
    fn inset_shadow_converts_without_panicking() {
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
        // Must not panic (inset is approximated).
        let _scene = SceneConverter::new().convert(&list);
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
        let run = GlyphRun::new(
            vec![PositionedGlyph::new(1, 0, 0.0, 16.0, 9.6)],
            16.0,
            "x",
        )
        .with_font(2);
        let list = DisplayList::from_commands(vec![PaintCommand::DrawText {
            origin: Point::new(0.0, 0.0),
            run,
            brush: Brush::solid(Color::WHITE),
        }]);
        let scene = converter.convert(&list);
        assert!(scene.encoding().is_empty());
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
        assert!(main.encoding().is_empty(), "blurred content omitted from main");
        assert_eq!(blurs.len(), 1);
        assert_eq!(blurs[0].bounds, bounds);
        assert_eq!(blurs[0].radius, 6.0);
        // The sub-scene actually holds the bracketed draw.
        assert!(!blurs[0].scene.encoding().is_empty(), "sub-scene has the content");

        // Back-compat: `convert` draws the same content inline (un-blurred), so it
        // is non-empty and carries no separate blur layer.
        let inline = SceneConverter::new().convert(&list);
        assert!(!inline.encoding().is_empty());
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
