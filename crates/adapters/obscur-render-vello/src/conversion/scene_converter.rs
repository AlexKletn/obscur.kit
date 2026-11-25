#![doc = include_str!("../../docs/api/conversion/scene_converter.doc/_module.md")]

use obscur_identity::FastMap;
use obscur_scene::clip::Clip;
use obscur_scene::command::{DisplayList, PaintCommand};
use obscur_scene::layer::{CompositionMode, LayerEffect, SceneLayer};
use obscur_scene::paint::{BlendMode, Brush, Paint};
use obscur_scene::text::{GlyphRun, SYNTHETIC_BOLD_OUTSET_FRACTION};
use vello::kurbo::{Affine, Diagonal2};
use vello::peniko::{self, FontData};
use vello::{FontEmbolden, Scene};

use super::blur_layer::BlurLayer;
use super::brush_converter::BrushConverter;
use super::clip_converter::ClipConverter;
use super::color_converter::ColorConverter;
use super::image_blobs::ImageBlobs;
use super::image_painter::draw_image;
use super::path_converter::PathConverter;
use super::scene_scope::SceneScope;
use super::scene_segment::SceneSegment;
use super::shadow_painter::draw_shadow;

type FilterSink<'a> = dyn FnMut(&mut Scene, BlurLayer) + 'a;

/// The synthetic-oblique shear factor: the standard faux-italic skew of `0.25`
/// (`tan ≈ 14°`) applied as a per-glyph shear so above-baseline points slide
/// right and the glyph leans forward (spec §5.18). Matches the CPU backend's
/// factor so both fake the same slant.
pub(crate) const FAUX_ITALIC_SKEW: f64 = 0.25;

#[doc = include_str!("../../docs/api/conversion/scene_converter.doc/positioned_glyphs.md")]
fn positioned_glyphs(
    run: &GlyphRun,
    origin: obscur_geometry::Point,
) -> impl ExactSizeIterator<Item = vello::Glyph> + '_ {
    run.glyphs().iter().map(move |glyph| vello::Glyph {
        id: glyph.glyph_id(),
        x: origin.x + glyph.x(),
        y: origin.y + glyph.y(),
    })
}

#[doc = include_str!("../../docs/api/conversion/scene_converter.doc/same_bytes.md")]
fn same_bytes(held: &[u8], offered: &[u8]) -> bool {
    (std::ptr::eq(held.as_ptr(), offered.as_ptr()) && held.len() == offered.len())
        || held == offered
}

#[doc = include_str!("../../docs/api/conversion/scene_converter.doc/SceneConverter.md")]
#[derive(Debug, Default, Clone)]
pub struct SceneConverter {
    path: PathConverter,
    brush: BrushConverter,
    clip: ClipConverter,
    #[doc = include_str!("../../docs/api/conversion/scene_converter.doc/fonts.md")]
    fonts: FastMap<u32, FontData>,
    /// The `ImageData` handed to vello for each image, kept between frames so its
    /// atlas keeps its upload.
    image_blobs: ImageBlobs,
}

impl SceneConverter {
    #[doc = include_str!("../../docs/api/conversion/scene_converter.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        let color = ColorConverter::new();
        let path = PathConverter::new();
        Self {
            path,
            brush: BrushConverter::new(color),
            clip: ClipConverter::new(path),
            fonts: FastMap::default(),
            image_blobs: ImageBlobs::default(),
        }
    }

    #[doc = include_str!("../../docs/api/conversion/scene_converter.doc/register_font.md")]
    pub fn register_font(&mut self, id: u32, font: FontData) {
        // A `Blob` takes its identity from a global counter, so the same bytes
        // wrapped twice are two different fonts as far as Vello is concerned —
        // and that identity is the key of Vello's glyph-outline cache. Replacing
        // a face with itself would therefore throw away every outline cached for
        // it, invisibly: nothing breaks, nothing moves on screen, every glyph is
        // simply extracted and encoded again until the orphaned entries age out.
        // So a face is replaced only when it is a different face.
        if let Some(held) = self.fonts.get(&id)
            && held.index == font.index
            && same_bytes(held.data.as_ref(), font.data.as_ref())
        {
            return;
        }
        self.fonts.insert(id, font);
    }

    #[doc = include_str!("../../docs/api/conversion/scene_converter.doc/set_font.md")]
    pub fn set_font(&mut self, font: FontData) {
        self.register_font(0, font);
    }

    /// Selects the [`FontData`] a run tagged with handle `id` draws against:
    /// the exact match if present, otherwise the primary (id `0`) as a fallback,
    /// otherwise `None` (the run is skipped rather than panicking).
    pub(crate) fn font_for(&self, id: u32) -> Option<&FontData> {
        self.fonts.get(&id).or_else(|| self.fonts.get(&0))
    }

    #[doc = include_str!("../../docs/api/conversion/scene_converter.doc/can_draw_text.md")]
    pub(crate) fn can_draw_text(&self, id: u32) -> bool {
        self.font_for(id).is_some()
    }

    #[doc = include_str!("../../docs/api/conversion/scene_converter.doc/face_bytes.md")]
    #[cfg(test)]
    pub(crate) fn face_bytes(&self, id: u32) -> Option<&[u8]> {
        self.fonts.get(&id).map(|font| font.data.as_ref())
    }

    #[doc = include_str!("../../docs/api/conversion/scene_converter.doc/convert.md")]
    #[must_use]
    pub fn convert(&self, list: &DisplayList) -> Scene {
        self.image_blobs.begin_frame();
        let mut scene = Scene::new();
        self.emit(&mut scene, list.commands(), Affine::IDENTITY, None);
        scene
    }

    #[doc = include_str!("../../docs/api/conversion/scene_converter.doc/convert_with_blurs.md")]
    #[must_use]
    pub fn convert_with_blurs(&self, list: &DisplayList) -> (Scene, Vec<BlurLayer>) {
        let mut scene = Scene::new();
        let blurs = self.convert_with_blurs_into(&mut scene, list);
        (scene, blurs)
    }

    #[doc = include_str!("../../docs/api/conversion/scene_converter.doc/convert_with_blurs_into.md")]
    pub fn convert_with_blurs_into(&self, scene: &mut Scene, list: &DisplayList) -> Vec<BlurLayer> {
        // `reset` keeps the six encoding streams and empties them; a fresh
        // `Scene` throws the buffers away and grows them back from zero on every
        // frame, which is the same allocation pattern each time and never the
        // right one.
        scene.reset();
        self.image_blobs.begin_frame();
        let mut blurs: Vec<BlurLayer> = Vec::new();
        self.emit(
            scene,
            list.commands(),
            Affine::IDENTITY,
            Some(&mut |_, layer| blurs.push(layer)),
        );
        blurs
    }

    #[doc = include_str!("../../docs/api/conversion/scene_converter.doc/convert_segments.md")]
    #[must_use]
    pub fn convert_segments(&self, list: &DisplayList) -> Vec<SceneSegment> {
        self.image_blobs.begin_frame();
        let mut segments = Vec::new();
        let mut run = Vec::new();
        let mut scopes: Vec<&PaintCommand> = Vec::new();
        for command in list.commands() {
            match command {
                PaintCommand::BackdropFilter {
                    bounds,
                    radii,
                    blur,
                } if scopes.iter().all(|scope| match scope {
                    PaintCommand::PushLayer(layer) => Self::can_split_backdrop_layer(layer),
                    _ => true,
                }) =>
                {
                    let transform =
                        scopes
                            .iter()
                            .fold(Affine::IDENTITY, |current, scope| match scope {
                                PaintCommand::PushTransform(transform) => {
                                    current * to_kurbo_affine(transform)
                                }
                                PaintCommand::PushLayer(layer) => {
                                    current * to_kurbo_affine(&layer.transform())
                                }
                                _ => current,
                            });
                    close_run(&mut run, &scopes);
                    self.append_content_segments(&run, &mut segments);
                    let mut mask_commands: Vec<_> =
                        scopes.iter().map(|&scope| scope.clone()).collect();
                    mask_commands.push(PaintCommand::DrawPath {
                        path: obscur_scene::path::Path::rounded_rect(*bounds, *radii),
                        paint: Paint::solid_fill(obscur_scene::color::Color::WHITE),
                    });
                    close_run(&mut mask_commands, &scopes);
                    let mut coverage = Scene::new();
                    self.emit(&mut coverage, &mask_commands, Affine::IDENTITY, None);
                    let scale = affine_scale(transform);
                    segments.push(SceneSegment::Backdrop {
                        bounds: transform_rect(transform, *bounds),
                        radii: radii.map(|corner| {
                            obscur_geometry::CornerRadius::new(corner.x * scale, corner.y * scale)
                        }),
                        blur: *blur * scale,
                        coverage,
                    });
                    run.clear();
                    run.extend(scopes.iter().map(|&scope| scope.clone()));
                    continue;
                }
                PaintCommand::PushTransform(_)
                | PaintCommand::PushClip(_)
                | PaintCommand::PushLayer(_) => scopes.push(command),
                PaintCommand::PopTransform | PaintCommand::PopClip | PaintCommand::PopLayer => {
                    if scopes
                        .last()
                        .is_some_and(|scope| command.closes_scope(scope))
                    {
                        scopes.pop();
                    } else {
                        continue;
                    }
                }
                _ => {}
            }
            run.push(command.clone());
        }
        self.append_content_segments(&run, &mut segments);
        segments
    }

    pub(crate) fn can_extract_filter(layer: &SceneLayer) -> bool {
        layer.clip().and_then(Clip::bounds).is_some()
            && layer.blend_mode().is_normal()
            && layer.composition().is_src_over()
            && layer.opacity() == 1.0
    }

    pub(crate) fn can_split_backdrop_layer(layer: &SceneLayer) -> bool {
        !layer.is_isolated()
            && layer.opacity() == 1.0
            && layer.blend_mode().is_normal()
            && layer.composition().is_src_over()
            && layer.effects().is_empty()
            && layer
                .clip()
                .is_none_or(|clip| matches!(clip, Clip::Shape(_)))
    }

    fn append_content_segments(&self, commands: &[PaintCommand], segments: &mut Vec<SceneSegment>) {
        let mut scene = Scene::new();
        self.emit_ordered(&mut scene, commands, segments);
        segments.push(SceneSegment::Content {
            scene,
            blurs: Vec::new(),
        });
    }

    pub(crate) fn convert_ordered_into(
        &self,
        scene: &mut Scene,
        list: &DisplayList,
    ) -> Vec<SceneSegment> {
        scene.reset();
        self.image_blobs.begin_frame();
        let mut segments = Vec::new();
        self.emit_ordered(scene, list.commands(), &mut segments);
        segments
    }

    fn emit_ordered(
        &self,
        scene: &mut Scene,
        commands: &[PaintCommand],
        segments: &mut Vec<SceneSegment>,
    ) {
        self.emit(
            scene,
            commands,
            Affine::IDENTITY,
            Some(&mut |scene, layer| {
                segments.push(SceneSegment::Content {
                    scene: std::mem::take(scene),
                    blurs: vec![layer],
                });
            }),
        );
    }

    #[doc = include_str!("../../docs/api/conversion/scene_converter.doc/emit.md")]
    fn emit(
        &self,
        scene: &mut Scene,
        commands: &[PaintCommand],
        base: Affine,
        mut extract: Option<&mut FilterSink<'_>>,
    ) {
        let mut layers: Vec<SceneScope<'_>> = Vec::new();
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
                        && layers.is_empty()
                        && Self::can_extract_filter(layer)
                        && (layer_blur_radius(layer).is_some()
                            || layer_color_matrix(layer).is_some()) =>
                {
                    let current = current * to_kurbo_affine(&layer.transform());
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
                    if let Some(clip) = layer.clip() {
                        self.push_clip(&mut sub, clip, current);
                    }
                    self.emit(&mut sub, inner, current, None);
                    if let Some(clip) = layer.clip() {
                        self.pop_layer(&mut sub, Some((clip, current)));
                    }
                    if let Some(extract) = extract.as_deref_mut() {
                        extract(scene, BlurLayer::new(sub, bounds, radius, color_matrix));
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
                    let depth = layers
                        .iter()
                        .rev()
                        .find_map(|scope| scope.layer_transform_depth);
                    if depth.is_none_or(|depth| transforms.len() > depth + 1) {
                        transforms.pop();
                    }
                }
                PaintCommand::PushClip(clip) => {
                    self.push_clip(scene, clip, current);
                    layers.push(SceneScope {
                        clip: Some((clip, current)),
                        layer_transform_depth: None,
                    });
                }
                PaintCommand::PushLayer(layer) => {
                    let depth = transforms.len();
                    let current = current * to_kurbo_affine(&layer.transform());
                    transforms.push(current);
                    self.push_layer(scene, layer, current);
                    layers.push(SceneScope {
                        clip: layer.clip().map(|clip| (clip, current)),
                        layer_transform_depth: Some(depth),
                    });
                }
                PaintCommand::PopClip | PaintCommand::PopLayer => {
                    let closes_layer = matches!(commands[index], PaintCommand::PopLayer);
                    if layers
                        .last()
                        .is_some_and(|scope| scope.layer_transform_depth.is_some() == closes_layer)
                    {
                        if let Some(scope) = layers.pop() {
                            if let Some(depth) = scope.layer_transform_depth {
                                transforms.truncate(depth);
                            }
                            self.pop_layer(scene, scope.clip);
                        }
                    }
                }
                PaintCommand::DrawText { origin, run, brush } => {
                    // One command, one glyph run. Adjacent same-state commands
                    // are deliberately *not* gathered into one: a Vello glyph
                    // run is one path and one draw object, so merging would
                    // replace N source-over composites with a single non-zero
                    // fill of their union, and those two agree only where no two
                    // glyphs reach the same pixel. Measured deltas and the test
                    // that holds this line are in `tests/text_run_pixels.rs`.
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
                    draw_image(
                        scene,
                        image,
                        tile,
                        region,
                        *repeat_x,
                        *repeat_y,
                        current,
                        &self.image_blobs,
                    );
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

        while let Some(scope) = layers.pop() {
            self.pop_layer(scene, scope.clip);
        }
    }

    #[doc = include_str!("../../docs/api/conversion/scene_converter.doc/draw_text.md")]
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

        // Before anything is built. An empty run must leave the scene untouched —
        // `draw_glyphs` is never entered — and empty runs really do arrive: a
        // paragraph emits one for an empty line. Emptiness is a property of the
        // run, so it is asked of the run: this used to map the glyphs into a
        // `Vec` and ask *that*, one heap allocation per text command per frame
        // to answer a question the source had already answered.
        if run.is_empty() {
            return;
        }
        let glyphs = positioned_glyphs(run, origin);

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
            let amount = f64::from(run.font_size()) * f64::from(SYNTHETIC_BOLD_OUTSET_FRACTION);
            draw = draw.font_embolden(FontEmbolden::new(Diagonal2::new(amount, amount)));
        }
        draw.draw(peniko::Fill::NonZero, glyphs);
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

    fn layer_clip_shape(&self, clip: &Clip) -> vello::kurbo::BezPath {
        match clip {
            Clip::ImageMask { bounds, .. } => self.clip.shape(&Clip::rect(*bounds)),
            _ => self.clip.shape(clip),
        }
    }

    fn pop_layer(&self, scene: &mut Scene, clip: Option<(&Clip, Affine)>) {
        if let Some((
            Clip::ImageMask {
                kind,
                image,
                bounds,
            },
            transform,
        )) = clip
        {
            let shape = self.clip.shape(&Clip::rect(*bounds));
            scene.push_layer(
                peniko::Fill::NonZero,
                peniko::BlendMode::new(peniko::Mix::Normal, peniko::Compose::DestIn),
                1.0,
                transform,
                &shape,
            );
            let luminance = kind
                .kind()
                .is_luminance()
                .then(|| self.image_blobs.luminance_mask(image))
                .flatten();
            let source = luminance.as_ref().unwrap_or(image);
            draw_image(
                scene,
                source,
                bounds,
                bounds,
                false,
                false,
                transform,
                &self.image_blobs,
            );
            scene.pop_layer();
        }
        scene.pop_layer();
    }

    pub(super) fn clip_fill_rule(&self, clip: &Clip) -> peniko::Fill {
        match clip {
            Clip::Shape(obscur_scene::clip::ClipShape::Path(path)) => {
                self.path.fill_rule(path.fill_rule())
            }
            _ => peniko::Fill::NonZero,
        }
    }

    /// Pushes a clip as a neutral-blend layer bounded by the clip's shape, mapped
    /// by the active `transform`.
    fn push_clip(&self, scene: &mut Scene, clip: &Clip, transform: Affine) {
        let shape = self.layer_clip_shape(clip);
        let fill = self.clip_fill_rule(clip);
        scene.push_layer(fill, peniko::BlendMode::default(), 1.0, transform, &shape);
    }

    /// Pushes a compositing layer with its opacity, blend, and clip shape, the
    /// shape mapped by the active `transform`.
    fn push_layer(&self, scene: &mut Scene, layer: &SceneLayer, transform: Affine) {
        let blend = peniko::BlendMode::new(mix(layer.blend_mode()), compose(layer.composition()));
        let alpha = layer.opacity();
        // An image mask clips to its bounds here and applies its sampled
        // coverage when the group closes.
        match layer.clip() {
            Some(clip) => {
                let shape = self.layer_clip_shape(clip);
                scene.push_layer(self.clip_fill_rule(clip), blend, alpha, transform, &shape);
            }
            None => scene.push_layer(
                peniko::Fill::NonZero,
                blend,
                alpha,
                transform,
                ClipConverter::unbounded_ref(),
            ),
        }
    }
}

fn close_run(run: &mut Vec<PaintCommand>, scopes: &[&PaintCommand]) {
    run.extend(scopes.iter().rev().filter_map(|scope| scope.scope_close()));
}

/// The first finite positive [`Blur`](LayerEffect::Blur) radius on a layer.
/// A zero radius leaves the layer in the ordinary drawing pass.
pub(crate) fn layer_blur_radius(layer: &SceneLayer) -> Option<f32> {
    layer.effects().iter().find_map(|effect| match effect {
        LayerEffect::Blur { radius } if radius.is_finite() && *radius > 0.0 => Some(*radius),
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

/// The index of the `PopLayer` matching the `PushLayer` at `start`,
/// respecting nested clips and layers. Returns
/// `commands.len()` for an unbalanced list (the rest is treated as the subtree).
fn matching_layer_pop(commands: &[PaintCommand], start: usize) -> usize {
    let mut scopes = Vec::new();
    for (offset, command) in commands[start..].iter().enumerate() {
        match command {
            PaintCommand::PushClip(_) | PaintCommand::PushLayer(_) => scopes.push(command),
            PaintCommand::PopClip | PaintCommand::PopLayer => {
                if scopes
                    .last()
                    .is_some_and(|scope| command.closes_scope(scope))
                {
                    scopes.pop();
                    if scopes.is_empty() {
                        return start + offset;
                    }
                }
            }
            _ => {}
        }
    }
    commands.len()
}

#[doc = include_str!("../../docs/api/conversion/scene_converter.doc/transform_rect.md")]
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
pub(super) fn to_kurbo_affine(t: &obscur_geometry::AffineTransform) -> Affine {
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

/// Maps an obscur-scene [`BlendMode`] to a peniko [`Mix`](peniko::Mix) blend.
pub(crate) fn mix(mode: BlendMode) -> peniko::Mix {
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
        BlendMode::Hue => peniko::Mix::Hue,
        BlendMode::Saturation => peniko::Mix::Saturation,
        BlendMode::Color => peniko::Mix::Color,
        BlendMode::Luminosity => peniko::Mix::Luminosity,
    }
}

/// Maps an obscur-scene [`CompositionMode`] to a peniko
/// [`Compose`](peniko::Compose) operator.
pub(crate) fn compose(mode: CompositionMode) -> peniko::Compose {
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
    // The font-registry tests build a `FontData` from raw bytes; production code
    // here takes fonts already wrapped, so the `Blob` constructor is a test import.
    use vello::peniko::Blob;

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
    fn a_spread_shadows_corner_radius_is_eased_in_the_encoding() {
        use obscur_geometry::CornerRadii;
        // Spec §6.1.1 on the GPU path, read back from the encoding rather than
        // trusted. A 100x100 box, 50px radius, 100px spread (r = .5): the shadow's
        // corner radius must be the eased
        //   50 + 100 x (1 + (.5 - 1)^3) = 137.5,
        // not the raw 50 + 100 = 150.
        //
        // `Scene::encoding()` is public, and `vello_encoding 0.9`'s
        // `DrawBlurRoundedRect` is `{ color: DrawColor(u32), width: f32,
        // height: f32, radius: f32, std_dev: f32 }` pushed to `draw_data` as five
        // u32 words by `Encoding::encode_blurred_rounded_rect` — so the radius this
        // backend actually hands Vello is word 3.
        let list = DisplayList::from_commands(vec![PaintCommand::DrawShadow {
            rect: Rect::new(Point::new(100.0, 100.0), Size::new(100.0, 100.0)),
            radii: CornerRadii::uniform(50.0),
            color: Color::BLACK,
            blur: 0.0,
            spread: 100.0,
            offset: (0.0, 0.0),
            inset: false,
        }]);
        let scene = SceneConverter::new().convert(&list);
        let draw_data = &scene.encoding().draw_data;
        assert_eq!(
            draw_data.len(),
            5,
            "one blurred rounded rect: [rgba, width, height, radius, std_dev]"
        );
        let radius = f32::from_bits(draw_data[3]);
        assert!(
            (radius - 137.5).abs() < 1e-3,
            "§6.1.1: Vello must receive the eased 137.5, not the raw 150 — got {radius}"
        );
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
    fn an_inset_shadows_negative_spread_eases_the_hole_in_the_encoding() {
        use obscur_geometry::CornerRadii;
        // §6.1.1's second trigger on the GPU path: an inset shadow contracts by
        // `spread`, so a NEGATIVE spread grows its hole and must ease.
        //
        // 600x600 box, 50px radius, spread -100, offset (300,300). The punched hole
        // is (200,200)-(1000,1000) — 800x800 — with radius 137.5 eased vs 150 raw.
        //
        // The inset path encodes several draw objects (clip, fill, the blur punch),
        // so the blur record is located by its own width/height signature rather
        // than a hardcoded tag offset: `DrawBlurRoundedRect` is
        // `[rgba, width, height, radius, std_dev]`, so the radius is the word right
        // after the 800/800 pair.
        let list = DisplayList::from_commands(vec![PaintCommand::DrawShadow {
            rect: Rect::new(Point::new(0.0, 0.0), Size::new(600.0, 600.0)),
            radii: CornerRadii::uniform(50.0),
            color: Color::BLACK,
            blur: 0.0,
            spread: -100.0,
            offset: (300.0, 300.0),
            inset: true,
        }]);
        let scene = SceneConverter::new().convert(&list);
        let words: Vec<f32> = scene
            .encoding()
            .draw_data
            .iter()
            .map(|w| f32::from_bits(*w))
            .collect();
        let at = words
            .windows(3)
            .position(|w| (w[0] - 800.0).abs() < 1e-3 && (w[1] - 800.0).abs() < 1e-3 && w[2] > 1.0)
            .expect("the 800x800 hole's blur record must be encoded");
        let radius = words[at + 2];
        assert!(
            (radius - 137.5).abs() < 1e-3,
            "§6.1.1: an inset shadow's negative spread must ease the hole to 137.5, \
             not grow it raw to 150 — got {radius}"
        );
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
    fn wrong_pop_does_not_end_a_filtered_subtree() {
        let commands = [
            PaintCommand::PushLayer(SceneLayer::new()),
            PaintCommand::PopClip,
            PaintCommand::PopLayer,
        ];
        assert_eq!(matching_layer_pop(&commands, 0), 2);
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

    #[doc = include_str!("../../docs/api/conversion/scene_converter.doc/dropping_a_face_would_draw_the_wrong_one_not_nothing.md")]
    #[test]
    fn dropping_a_face_would_draw_the_wrong_one_not_nothing() {
        use vello::peniko::Blob;
        let primary = FontData::new(Blob::from(vec![7u8, 7, 7]), 0);
        let discovered = FontData::new(Blob::from(vec![8u8, 8, 8]), 0);
        let mut converter = SceneConverter::new();
        converter.register_font(0, primary.clone());
        converter.register_font(9, discovered.clone());
        assert_eq!(converter.font_for(9), Some(&discovered));

        // What an eviction would do, done by hand.
        converter.fonts.remove(&9);

        assert_eq!(
            converter.font_for(9),
            Some(&primary),
            "a handle whose face was dropped does NOT resolve to nothing — it \
             resolves to the primary, whose glyph ids mean something else \
             entirely. This is why this table has no ceiling and no eviction: \
             the only re-supply path (`InstalledFonts::take_discovered`) drains, \
             so a dropped face never comes back, and what it leaves behind is \
             wrong glyphs rather than absent ones. See `fonts.md`."
        );
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
    // ---- one command, one glyph run --------------------------------------
    //
    // These need a font that really parses: `vello::DrawGlyphs::draw` unwraps
    // `FontRef::from_index`, so a `Blob` of three bytes panics there. The
    // repository ships one, and it is the same file the toolkit's own bootstrap
    // installs as the sans-serif generic.
    const A_REAL_FACE: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../assets/fonts/NotoSans-Regular.ttf"
    ));

    fn a_converter_with_a_real_face() -> SceneConverter {
        let mut converter = SceneConverter::new();
        converter.register_font(0, FontData::new(Blob::from(A_REAL_FACE.to_vec()), 0));
        converter
    }

    fn a_run(glyph: u32, x: f32) -> obscur_scene::text::GlyphRun {
        use obscur_scene::text::{GlyphRun, PositionedGlyph};
        GlyphRun::new(
            vec![PositionedGlyph::new(glyph, 0, x, 0.0, 10.0)],
            16.0,
            "x",
        )
    }

    fn a_text_command(run: obscur_scene::text::GlyphRun, origin: f32) -> PaintCommand {
        PaintCommand::DrawText {
            origin: Point::new(origin, 40.0),
            run,
            brush: Brush::solid(Color::WHITE),
        }
    }

    /// The glyphs a scene ended up encoding, in order, as plain numbers.
    fn encoded_glyphs(scene: &Scene) -> Vec<(u32, u32, u32)> {
        scene
            .encoding()
            .resources
            .glyphs
            .iter()
            .map(|glyph| (glyph.id, glyph.x.to_bits(), glyph.y.to_bits()))
            .collect()
    }

    fn encoded_runs(scene: &Scene) -> usize {
        scene.encoding().resources.glyph_runs.len()
    }

    #[test]
    fn every_text_command_gets_a_glyph_run_of_its_own() {
        // The guard on the line `draw_text` holds. Two adjacent commands in
        // identical state — the keyboard's own shape, a key's primary and
        // shifted legends — are the tempting case to fold into one
        // `draw_glyphs`, and folding them changes the drawing: a Vello glyph
        // run is ONE path and ONE draw object (`Resolver::resolve` does
        // `layout.n_paths += 1` per run and terminates it with a single
        // `PathTag::PATH`), so a fold turns N source-over composites into one
        // non-zero fill of the union. `tests/text_run_pixels.rs` measures what
        // that costs in pixels; this is the cheap sentry that fails everywhere,
        // GPU or not, the moment someone folds them again.
        let converter = a_converter_with_a_real_face();
        let list = DisplayList::from_commands(vec![
            a_text_command(a_run(20, 0.0), 10.0),
            a_text_command(a_run(21, 0.0), 30.0),
        ]);
        let scene = converter.convert(&list);
        assert_eq!(encoded_runs(&scene), 2, "one glyph run per text command");
        assert_eq!(encoded_glyphs(&scene).len(), 2);
    }

    #[test]
    fn two_commands_and_one_command_holding_both_glyphs_are_not_the_same_drawing() {
        // The claim the merge was accepted on, tested instead of asserted. The
        // two are equal in the glyphs they carry and unequal in what draws
        // them: two paths and two draw objects against one. Everything the
        // GPU does differently between them follows from that count, so it is
        // the one fact worth pinning down without a device.
        use obscur_scene::text::{GlyphRun, PositionedGlyph};
        let converter = a_converter_with_a_real_face();
        let pair = DisplayList::from_commands(vec![
            a_text_command(a_run(20, 1.5), 10.0),
            a_text_command(a_run(21, 2.5), 30.0),
        ]);
        let single = DisplayList::from_commands(vec![a_text_command(
            GlyphRun::new(
                vec![
                    PositionedGlyph::new(20, 0, 1.5, 0.0, 10.0),
                    PositionedGlyph::new(21, 0, 22.5, 0.0, 10.0),
                ],
                16.0,
                "xy",
            ),
            10.0,
        )]);
        let (apart, together) = (converter.convert(&pair), converter.convert(&single));
        assert_eq!(
            encoded_glyphs(&apart),
            encoded_glyphs(&together),
            "the same glyphs at the same absolute places, either way"
        );
        assert_eq!(encoded_runs(&apart), 2);
        assert_eq!(encoded_runs(&together), 1);
    }

    #[test]
    fn a_command_puts_every_glyph_where_it_asked_for_it() {
        // Glyph positions handed to Vello are absolute, resolved against the
        // command's own origin. Get this wrong and the second legend slides
        // onto the first.
        let converter = a_converter_with_a_real_face();
        let apart = DisplayList::from_commands(vec![
            a_text_command(a_run(20, 1.5), 10.0),
            a_text_command(a_run(21, 2.5), 30.0),
        ]);
        let scene = converter.convert(&apart);
        assert_eq!(
            encoded_glyphs(&scene),
            vec![
                (20, 11.5_f32.to_bits(), 40.0_f32.to_bits()),
                (21, 32.5_f32.to_bits(), 40.0_f32.to_bits()),
            ]
        );
    }

    #[test]
    fn an_empty_run_beside_a_drawn_one_neither_hides_it_nor_adds_a_glyph() {
        // Empty runs really arrive — a paragraph emits one per empty line —
        // and they must contribute nothing and prevent nothing.
        use obscur_scene::text::GlyphRun;
        let converter = a_converter_with_a_real_face();
        let list = DisplayList::from_commands(vec![
            a_text_command(GlyphRun::new(Vec::new(), 16.0, ""), 10.0),
            a_text_command(a_run(20, 0.0), 30.0),
        ]);
        let scene = converter.convert(&list);
        assert_eq!(encoded_runs(&scene), 1, "the empty one enters no pipeline");
        assert_eq!(encoded_glyphs(&scene).len(), 1);
    }

    #[test]
    fn nothing_but_empty_runs_never_enters_the_glyph_pipeline() {
        use obscur_scene::text::GlyphRun;
        let converter = a_converter_with_a_real_face();
        let list = DisplayList::from_commands(vec![
            a_text_command(GlyphRun::new(Vec::new(), 16.0, ""), 10.0),
            a_text_command(GlyphRun::new(Vec::new(), 16.0, ""), 30.0),
        ]);
        let scene = converter.convert(&list);
        assert!(scene.encoding().is_empty());
    }

    #[test]
    fn what_vello_is_handed_counts_itself_before_it_is_walked() {
        use obscur_scene::text::{GlyphRun, PositionedGlyph};
        // `DrawGlyphs::draw` feeds this iterator straight into `Vec::extend`,
        // which reserves the `size_hint` lower bound in one go — so the count
        // the iterator states BEFORE anything walks it is the whole point, and
        // it must be the run's own glyph count.
        //
        // Asked of `positioned_glyphs`, the function `draw_text` calls, rather
        // than of a copy of it built in the test: the copy is what the previous
        // version of this test compared against itself, and it would have gone
        // on passing while `draw_text` handed Vello something else entirely.
        let run = GlyphRun::new(
            vec![
                PositionedGlyph::new(20, 0, 0.0, 0.0, 10.0),
                PositionedGlyph::new(21, 1, 10.0, 0.0, 10.0),
                PositionedGlyph::new(22, 2, 20.0, 0.0, 10.0),
            ],
            16.0,
            "xyz",
        );
        let glyphs = super::positioned_glyphs(&run, Point::new(10.0, 40.0));
        assert_eq!(
            glyphs.len(),
            run.glyphs().len(),
            "stated up front, before a single glyph is produced"
        );
        // And what it then produces has to agree with what it promised, in
        // order and offset by the command's origin.
        assert_eq!(
            glyphs
                .map(|glyph| (glyph.id, glyph.x, glyph.y))
                .collect::<Vec<_>>(),
            vec![(20, 10.0, 40.0), (21, 20.0, 40.0), (22, 30.0, 40.0)]
        );
    }

    // ---- the font registry's own identity -------------------------------

    #[test]
    fn registering_the_same_bytes_again_keeps_the_font_vellos_glyph_cache_is_keyed_by() {
        // `Blob` ids come from a global counter, so the same bytes wrapped twice
        // are two fonts, and the id is what `GlyphCache::session` keys outlines
        // by. Re-registering must therefore be a no-op, or the cache is thrown
        // away with nothing on screen to say so.
        let mut converter = SceneConverter::new();
        converter.register_font(3, FontData::new(Blob::from(vec![1u8, 2, 3]), 0));
        let first = converter.font_for(3).unwrap().data.id();
        converter.register_font(3, FontData::new(Blob::from(vec![1u8, 2, 3]), 0));
        assert_eq!(
            converter.font_for(3).unwrap().data.id(),
            first,
            "the same face re-registered must stay the same face"
        );
    }

    #[test]
    fn registering_a_different_face_under_a_live_handle_replaces_it() {
        // The other half: the guard must not become "never replace". A handle
        // pointed at new bytes draws with the new bytes.
        let mut converter = SceneConverter::new();
        converter.register_font(3, FontData::new(Blob::from(vec![1u8, 2, 3]), 0));
        let first = converter.font_for(3).unwrap().data.id();
        converter.register_font(3, FontData::new(Blob::from(vec![9u8, 9, 9]), 0));
        assert_ne!(converter.font_for(3).unwrap().data.id(), first);
        assert_eq!(converter.font_for(3).unwrap().data.as_ref(), &[9u8, 9, 9]);
    }

    #[test]
    fn the_same_bytes_at_a_different_face_index_are_a_different_face() {
        // A `.ttc` holds several faces behind one byte range; the index is half
        // the identity, and a guard that compared only bytes would pin the
        // wrong one.
        let mut converter = SceneConverter::new();
        converter.register_font(3, FontData::new(Blob::from(vec![1u8, 2, 3]), 0));
        converter.register_font(3, FontData::new(Blob::from(vec![1u8, 2, 3]), 1));
        assert_eq!(converter.font_for(3).unwrap().index, 1);
    }

    #[test]
    fn two_blobs_over_the_same_bytes_really_do_get_different_ids() {
        // The premise the guard exists for. If Vello's blob ids ever became
        // content-derived, the guard would be pointless and this test says so
        // before anyone reads the guard and is puzzled by it.
        let bytes = vec![4u8, 5, 6];
        assert_ne!(
            Blob::from(bytes.clone()).id(),
            Blob::from(bytes).id(),
            "blob identity is a counter, not the content"
        );
    }
}
