#![doc = include_str!("scene_converter.doc/_module.md")]

use std::collections::HashMap;

use obscur_scene::clip::Clip;
use obscur_scene::command::{DisplayList, PaintCommand};
use obscur_scene::layer::{CompositionMode, LayerEffect, SceneLayer};
use obscur_scene::paint::{BlendMode, Brush, Paint};
use obscur_scene::text::GlyphRun;
use vello::kurbo::{Affine, Diagonal2};
use vello::peniko::{self, FontData};
use vello::{FontEmbolden, Scene};

use super::blur_layer::BlurLayer;
use super::brush_converter::BrushConverter;
use super::clip_converter::ClipConverter;
use super::color_converter::ColorConverter;
use super::image_painter::draw_image;
use super::path_converter::PathConverter;
use super::scene_segment::SceneSegment;
use super::shadow_painter::draw_shadow;

#[doc = include_str!("scene_converter.doc/FAUX_ITALIC_SKEW.md")]
const FAUX_ITALIC_SKEW: f64 = 0.25;

#[doc = include_str!("scene_converter.doc/FAUX_BOLD_EMBOLDEN_FRACTION.md")]
const FAUX_BOLD_EMBOLDEN_FRACTION: f64 = 0.02;

#[doc = include_str!("scene_converter.doc/SceneConverter.md")]
#[derive(Debug, Default, Clone)]
pub struct SceneConverter {
    path: PathConverter,
    brush: BrushConverter,
    clip: ClipConverter,
    #[doc = include_str!("scene_converter.doc/fonts.md")]
    fonts: HashMap<u32, FontData>,
}

impl SceneConverter {
    #[doc = include_str!("scene_converter.doc/new.md")]
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

    #[doc = include_str!("scene_converter.doc/register_font.md")]
    pub fn register_font(&mut self, id: u32, font: FontData) {
        self.fonts.insert(id, font);
    }

    #[doc = include_str!("scene_converter.doc/set_font.md")]
    pub fn set_font(&mut self, font: FontData) {
        self.register_font(0, font);
    }

    #[doc = include_str!("scene_converter.doc/font_for.md")]
    fn font_for(&self, id: u32) -> Option<&FontData> {
        self.fonts.get(&id).or_else(|| self.fonts.get(&0))
    }

    #[doc = include_str!("scene_converter.doc/can_draw_text.md")]
    pub(crate) fn can_draw_text(&self, id: u32) -> bool {
        self.font_for(id).is_some()
    }

    #[doc = include_str!("scene_converter.doc/convert.md")]
    #[must_use]
    pub fn convert(&self, list: &DisplayList) -> Scene {
        let mut scene = Scene::new();
        self.emit(&mut scene, list.commands(), Affine::IDENTITY, None);
        scene
    }

    #[doc = include_str!("scene_converter.doc/convert_with_blurs.md")]
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

    #[doc = include_str!("scene_converter.doc/convert_segments.md")]
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

    #[doc = include_str!("scene_converter.doc/content_segment.md")]
    fn content_segment(&self, commands: &[PaintCommand]) -> SceneSegment {
        let mut scene = Scene::new();
        let mut blurs: Vec<BlurLayer> = Vec::new();
        self.emit(&mut scene, commands, Affine::IDENTITY, Some(&mut blurs));
        SceneSegment::Content { scene, blurs }
    }

    #[doc = include_str!("scene_converter.doc/emit.md")]
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

    #[doc = include_str!("scene_converter.doc/draw_text.md")]
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

    #[doc = include_str!("scene_converter.doc/draw_path.md")]
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

    #[doc = include_str!("scene_converter.doc/push_clip.md")]
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

    #[doc = include_str!("scene_converter.doc/push_layer.md")]
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

#[doc = include_str!("scene_converter.doc/layer_blur_radius.md")]
fn layer_blur_radius(layer: &SceneLayer) -> Option<f32> {
    layer.effects().iter().find_map(|effect| match effect {
        LayerEffect::Blur { radius } => Some(*radius),
        _ => None,
    })
}

#[doc = include_str!("scene_converter.doc/layer_color_matrix.md")]
fn layer_color_matrix(layer: &SceneLayer) -> Option<[f32; 20]> {
    layer
        .effects()
        .iter()
        .find_map(|effect| effect.as_color_matrix())
}

#[doc = include_str!("scene_converter.doc/matching_layer_pop.md")]
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

#[doc = include_str!("scene_converter.doc/transform_rect.md")]
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

#[doc = include_str!("scene_converter.doc/affine_scale.md")]
#[allow(clippy::cast_possible_truncation)]
fn affine_scale(a: Affine) -> f32 {
    let m = a.as_coeffs();
    (m[0] * m[3] - m[1] * m[2]).abs().sqrt() as f32
}

#[doc = include_str!("scene_converter.doc/to_kurbo_affine.md")]
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

#[doc = include_str!("scene_converter.doc/mix.md")]
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

#[doc = include_str!("scene_converter.doc/compose.md")]
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
