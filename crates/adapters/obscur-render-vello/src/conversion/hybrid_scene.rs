//! Conversion from the kit's display list to Sparse Strips drawing commands.

use obscur_scene::command::{DisplayList, PaintCommand};
use obscur_scene::paint::{Brush, Paint};
use obscur_scene::text::{GlyphRun, SYNTHETIC_BOLD_OUTSET_FRACTION};
use vello::kurbo::{Affine, Diagonal2, Rect};
use vello::peniko::{self, Fill};
use vello_hybrid::{Resources, Scene};

use super::scene_converter::{FAUX_ITALIC_SKEW, compose, mix, to_kurbo_affine};
use super::shadow_painter::blurred_shadow_radius;
use super::{BrushConverter, ClipConverter, ColorConverter, PathConverter, SceneConverter};

pub(crate) fn emit(
    scene: &mut Scene,
    resources: &mut Resources,
    list: &DisplayList,
    converter: &SceneConverter,
) {
    let paths = PathConverter::new();
    let clips = ClipConverter::new(paths);
    let brushes = BrushConverter::new(ColorConverter::new());
    let mut transforms = Vec::new();
    let mut layers = 0;
    for command in list.commands() {
        let transform = transforms.last().copied().unwrap_or(Affine::IDENTITY);
        scene.set_transform(transform);
        match command {
            PaintCommand::PushTransform(next) => transforms.push(transform * to_kurbo_affine(next)),
            PaintCommand::PopTransform => {
                transforms.pop();
            }
            PaintCommand::DrawPath { path, paint } => {
                let shape = paths.convert(path);
                set_brush(scene, &brushes, paint.brush());
                match paint {
                    Paint::Fill(_) => {
                        scene.set_fill_rule(paths.fill_rule(path.fill_rule()));
                        scene.fill_path(&shape);
                    }
                    Paint::Stroke(stroke) => {
                        scene.set_stroke(brushes.stroke(stroke));
                        scene.stroke_path(&shape);
                    }
                }
            }
            PaintCommand::PushClip(clip) => {
                scene.set_fill_rule(converter.clip_fill_rule(clip));
                scene.push_clip_layer(&clips.shape(clip));
                layers += 1;
            }
            PaintCommand::PushLayer(layer) => {
                let transform = transform * to_kurbo_affine(&layer.transform());
                transforms.push(transform);
                scene.set_transform(transform);
                let blend =
                    peniko::BlendMode::new(mix(layer.blend_mode()), compose(layer.composition()));
                let clip = layer.clip().map(|clip| clips.shape(clip));
                scene.set_fill_rule(
                    layer
                        .clip()
                        .map_or(Fill::NonZero, |clip| converter.clip_fill_rule(clip)),
                );
                scene.push_layer(
                    clip.as_ref(),
                    Some(blend),
                    Some(layer.opacity()),
                    None,
                    None,
                );
                layers += 1;
            }
            PaintCommand::PopClip | PaintCommand::PopLayer if layers > 0 => {
                scene.pop_layer();
                if matches!(command, PaintCommand::PopLayer) {
                    transforms.pop();
                }
                layers -= 1;
            }
            PaintCommand::DrawText { origin, run, brush } => {
                set_brush(scene, &brushes, brush);
                draw_text(scene, resources, converter, *origin, run);
            }
            PaintCommand::DrawShadow {
                rect,
                radii,
                color,
                blur,
                spread,
                offset,
                inset: false,
            } => {
                let bounds = Rect::new(
                    f64::from(rect.origin.x + offset.0 - spread),
                    f64::from(rect.origin.y + offset.1 - spread),
                    f64::from(rect.origin.x + rect.size.width + offset.0 + spread),
                    f64::from(rect.origin.y + rect.size.height + offset.1 + spread),
                );
                if bounds.width() > 0.0 && bounds.height() > 0.0 {
                    scene.set_paint(ColorConverter::new().convert(*color));
                    scene.reset_paint_transform();
                    let radius = blurred_shadow_radius(radii.spread_offset(*spread));
                    if *blur > 0.0 {
                        #[allow(clippy::cast_possible_truncation)]
                        scene.fill_blurred_rounded_rect(&bounds, radius as f32, blur / 2.0);
                    } else {
                        scene.set_fill_rule(Fill::NonZero);
                        scene.fill_path(&vello::kurbo::Shape::to_path(
                            &bounds.to_rounded_rect(radius),
                            0.1,
                        ));
                    }
                }
            }
            PaintCommand::DrawImage { .. }
            | PaintCommand::BackdropFilter { .. }
            | PaintCommand::DrawShadow { .. }
            | PaintCommand::PopClip
            | PaintCommand::PopLayer => {}
        }
    }
    for _ in 0..layers {
        scene.pop_layer();
    }
}

fn set_brush(scene: &mut Scene, brushes: &BrushConverter, brush: &Brush) {
    match brushes.convert(brush) {
        peniko::Brush::Solid(color) => scene.set_paint(color),
        peniko::Brush::Gradient(gradient) => scene.set_paint(gradient),
        peniko::Brush::Image(_) => unreachable!("kit path brushes are colors or gradients"),
    }
    scene.set_paint_transform(BrushConverter::brush_transform(brush).unwrap_or(Affine::IDENTITY));
}

fn draw_text(
    scene: &mut Scene,
    resources: &mut Resources,
    converter: &SceneConverter,
    origin: obscur_geometry::Point,
    run: &GlyphRun,
) {
    let Some(font) = converter.font_for(run.font()).filter(|_| !run.is_empty()) else {
        return;
    };
    let mut glyph_run = scene.glyph_run(resources, font).font_size(run.font_size());
    if run.synth_oblique() {
        glyph_run = glyph_run.glyph_transform(Affine::skew(FAUX_ITALIC_SKEW, 0.0));
    }
    if run.synth_bold() {
        let amount = f64::from(run.font_size()) * f64::from(SYNTHETIC_BOLD_OUTSET_FRACTION);
        glyph_run =
            glyph_run.font_embolden(glifo::FontEmbolden::new(Diagonal2::new(amount, amount)));
    }
    glyph_run.fill_glyphs(run.glyphs().iter().map(|glyph| glifo::Glyph {
        id: glyph.glyph_id(),
        x: origin.x + glyph.x(),
        y: origin.y + glyph.y(),
    }));
}
