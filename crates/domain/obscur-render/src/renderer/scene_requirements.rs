//! Collects the effects and fonts required by a display list.

use obscur_capability::SupportLevel;
use obscur_scene::clip::Clip;
use obscur_scene::command::{DisplayList, PaintCommand};
use obscur_scene::layer::{LayerEffect, SceneLayer};
use obscur_scene::paint::{BlendMode, Brush, Paint};

use super::{Renderer, SceneEffect};

#[doc = include_str!("../../docs/api/renderer/scene_requirements.doc/scene_effects.md")]
#[must_use]
pub fn scene_effects(list: &DisplayList) -> Vec<SceneEffect> {
    let mut found: Vec<SceneEffect> = Vec::new();
    let mut push = |effect: SceneEffect| {
        if !found.contains(&effect) {
            found.push(effect);
        }
    };
    for command in list.commands() {
        match command {
            PaintCommand::DrawPath { paint, .. } => {
                let brush = match paint {
                    Paint::Fill(brush) => brush,
                    Paint::Stroke(stroke) => stroke.brush(),
                };
                if !matches!(brush, Brush::Solid(_)) {
                    push(SceneEffect::Gradient);
                }
            }
            PaintCommand::DrawText { .. } => push(SceneEffect::Text),
            PaintCommand::DrawShadow { blur, .. } => {
                push(SceneEffect::Shadow);
                // A zero radius is a hard-edged shadow by definition, so it demands
                // no blur; any positive radius does.
                if *blur > 0.0 {
                    push(SceneEffect::Blur);
                }
            }
            PaintCommand::DrawImage { .. } => push(SceneEffect::Image),
            PaintCommand::BackdropFilter { blur, .. } => {
                // `blur: 0` is documented as no effect at all.
                if *blur > 0.0 {
                    push(SceneEffect::BackdropFilter);
                }
            }
            PaintCommand::PushClip(clip) => push(clip_effect(clip)),
            PaintCommand::PushLayer(layer) => layer_effects(layer, &mut push),
            PaintCommand::PushTransform(_)
            | PaintCommand::PopTransform
            | PaintCommand::PopClip
            | PaintCommand::PopLayer => {}
        }
    }
    found
}

#[doc = include_str!("../../docs/api/renderer/scene_requirements.doc/unsupported_effects.md")]
#[must_use]
pub fn unsupported_effects(renderer: &dyn Renderer, list: &DisplayList) -> Vec<SceneEffect> {
    scene_effects(list)
        .into_iter()
        .filter(|effect| renderer.supports_effect(*effect) == SupportLevel::Unavailable)
        .collect()
}

#[doc = include_str!("../../docs/api/renderer/scene_requirements.doc/text_run_font_handles.md")]
#[must_use]
pub fn text_run_font_handles(list: &DisplayList) -> Vec<u32> {
    let mut found: Vec<u32> = Vec::new();
    for command in list.commands() {
        if let PaintCommand::DrawText { run, .. } = command
            && !run.glyphs().is_empty()
            && !found.contains(&run.font())
        {
            found.push(run.font());
        }
    }
    found
}

/// The effect a clip demands: a shape clip confines by geometry, a mask clip by
/// coverage read from separate source content.
const fn clip_effect(clip: &Clip) -> SceneEffect {
    match clip {
        Clip::Shape(_) => SceneEffect::Clip,
        Clip::Mask(_) => SceneEffect::Mask,
        Clip::ImageMask { .. } => SceneEffect::ImageMask,
    }
}

/// The effects a `PushLayer` demands, pushed through `push`.
fn layer_effects(layer: &SceneLayer, push: &mut impl FnMut(SceneEffect)) {
    if layer.blend_mode() != BlendMode::Normal {
        push(SceneEffect::BlendMode);
    }
    if let Some(clip) = layer.clip() {
        push(clip_effect(clip));
    }
    for effect in layer.effects() {
        match effect {
            LayerEffect::Blur { radius } if *radius > 0.0 => push(SceneEffect::Blur),
            LayerEffect::ColorMatrix(_) => push(SceneEffect::ColorMatrix),
            LayerEffect::Blend(mode) if *mode != BlendMode::Normal => push(SceneEffect::BlendMode),
            // A zero-radius blur and a `Normal` blend ask for nothing. Group opacity
            // reaches a backend as the layer's `opacity` FIELD (which both in-tree
            // backends honor), never as this effect; a `LayerEffect::Opacity` would
            // be dropped by both, and it has no variant in the vocabulary, so it is
            // not diagnosed here.
            LayerEffect::Blur { .. } | LayerEffect::Blend(_) | LayerEffect::Opacity(_) => {}
        }
    }
    // The layer's own `transform` and Porter-Duff `composition` are compositing
    // parameters, not effects: they are outside this vocabulary (see `SceneEffect`).
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::{HeadlessRenderer, RendererId};
    use obscur_geometry::{CornerRadii, Point, Rect, Size};
    use obscur_scene::clip::Mask;
    use obscur_scene::color::{AlphaMode, Color};
    use obscur_scene::paint::{GradientStop, LinearGradient, SceneImage, SolidBrush};
    use obscur_scene::path::Path;

    fn a_rect() -> Rect {
        Rect::new(Point::new(0.0, 0.0), Size::new(10.0, 10.0))
    }

    fn gradient_brush() -> Brush {
        Brush::Linear(LinearGradient::new(
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            vec![
                GradientStop::new(0.0, Color::BLACK),
                GradientStop::new(1.0, Color::WHITE),
            ],
        ))
    }

    fn shadow(blur: f32) -> PaintCommand {
        PaintCommand::DrawShadow {
            rect: a_rect(),
            radii: CornerRadii::uniform(0.0),
            color: Color::BLACK,
            blur,
            spread: 0.0,
            offset: (1.0, 1.0),
            inset: false,
        }
    }

    #[test]
    fn solid_fill_demands_no_effect() {
        let list = DisplayList::from_commands(vec![PaintCommand::DrawPath {
            path: Path::new(),
            paint: Paint::Fill(Brush::Solid(SolidBrush::new(Color::BLACK))),
        }]);
        assert_eq!(scene_effects(&list), Vec::new());
    }

    #[test]
    fn gradient_fill_and_stroke_both_demand_gradient() {
        let list = DisplayList::from_commands(vec![
            PaintCommand::DrawPath {
                path: Path::new(),
                paint: Paint::Fill(gradient_brush()),
            },
            PaintCommand::DrawPath {
                path: Path::new(),
                paint: Paint::Stroke(obscur_scene::paint::Stroke::new(gradient_brush(), 1.0)),
            },
        ]);
        // Deduplicated: two gradient draws are one demand.
        assert_eq!(scene_effects(&list), vec![SceneEffect::Gradient]);
    }

    #[test]
    fn shadow_demands_blur_only_when_its_radius_is_positive() {
        let hard = DisplayList::from_commands(vec![shadow(0.0)]);
        assert_eq!(scene_effects(&hard), vec![SceneEffect::Shadow]);
        let soft = DisplayList::from_commands(vec![shadow(4.0)]);
        assert_eq!(
            scene_effects(&soft),
            vec![SceneEffect::Shadow, SceneEffect::Blur]
        );
    }

    #[test]
    fn clip_and_mask_are_distinct_demands() {
        let image = SceneImage::new(1, 1, vec![255; 4], AlphaMode::Straight).unwrap();
        let list = DisplayList::from_commands(vec![
            PaintCommand::PushClip(Clip::rect(a_rect())),
            PaintCommand::PopClip,
            PaintCommand::PushClip(Clip::mask(Mask::alpha())),
            PaintCommand::PopClip,
            PaintCommand::PushClip(Clip::image_mask(Mask::alpha(), image, a_rect())),
            PaintCommand::PopClip,
        ]);
        assert_eq!(
            scene_effects(&list),
            vec![SceneEffect::Clip, SceneEffect::Mask, SceneEffect::ImageMask]
        );
    }

    #[test]
    fn layer_demands_its_blend_clip_and_effects() {
        let mut layer = SceneLayer::new()
            .with_blend_mode(BlendMode::Multiply)
            .with_clip(Clip::rect(a_rect()));
        layer.add_effect(LayerEffect::blur(3.0));
        layer.add_effect(LayerEffect::color_matrix([0.0; 20]));
        let list = DisplayList::from_commands(vec![
            PaintCommand::PushLayer(layer),
            PaintCommand::PopLayer,
        ]);
        assert_eq!(
            scene_effects(&list),
            vec![
                SceneEffect::BlendMode,
                SceneEffect::Clip,
                SceneEffect::Blur,
                SceneEffect::ColorMatrix,
            ]
        );
    }

    #[test]
    fn group_opacity_is_not_in_the_vocabulary() {
        // The layer's opacity FIELD is honored by both backends and demands nothing;
        // a `LayerEffect::Opacity` is outside the vocabulary and is not diagnosed.
        let mut layer = SceneLayer::new().with_opacity(0.5);
        layer.add_effect(LayerEffect::opacity(0.5));
        let list = DisplayList::from_commands(vec![
            PaintCommand::PushLayer(layer),
            PaintCommand::PopLayer,
        ]);
        assert_eq!(scene_effects(&list), Vec::new());
    }

    #[test]
    fn unsupported_effects_filters_by_the_backend_table() {
        // `HeadlessRenderer` rasterizes nothing, so every demand is unavailable.
        let renderer = HeadlessRenderer::new(RendererId::from_raw(1));
        let list = DisplayList::from_commands(vec![shadow(2.0)]);
        assert_eq!(
            unsupported_effects(&renderer, &list),
            vec![SceneEffect::Shadow, SceneEffect::Blur]
        );
        // An empty scene demands nothing, so nothing is unsupported.
        assert_eq!(
            unsupported_effects(&renderer, &DisplayList::new()),
            Vec::new()
        );
    }

    #[test]
    fn text_run_font_handles_are_deduplicated_and_skip_empty_runs() {
        use obscur_scene::text::{GlyphRun, PositionedGlyph};
        let glyph = || PositionedGlyph::new(1, 0, 0.0, 12.0, 8.0);
        let run = |handle: u32| GlyphRun::new(vec![glyph()], 12.0, "a").with_font(handle);
        let text = |run: GlyphRun| PaintCommand::DrawText {
            origin: Point::new(0.0, 0.0),
            run,
            brush: Brush::Solid(SolidBrush::new(Color::BLACK)),
        };
        let list = DisplayList::from_commands(vec![
            text(run(3)),
            text(run(1)),
            // A repeat of a handle already seen is one demand, not two.
            text(run(3)),
            // An empty run draws nothing with or without its font: not a demand.
            text(GlyphRun::new(Vec::new(), 12.0, "").with_font(9)),
        ]);
        assert_eq!(text_run_font_handles(&list), vec![3, 1]);
        // A scene with no text asks for no font at all.
        assert_eq!(text_run_font_handles(&DisplayList::new()), Vec::new());
    }
}
