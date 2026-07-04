//! Converts an obscur-scene [`DisplayList`] into a [`vello::Scene`].
//!
//! This is the top-level pure conversion of the adapter (spec §2.7): it walks the
//! display list's commands **in paint order** and replays them onto a
//! `vello::Scene`, delegating geometry/paint/clip translation to the other
//! converters in this module. Paint order, clipping, and layer nesting are
//! preserved exactly; only rasterization is Vello's (spec §5.2 determinism
//! boundary).

use obscur_scene::clip::Clip;
use obscur_scene::command::{DisplayList, PaintCommand};
use obscur_scene::layer::{CompositionMode, SceneLayer};
use obscur_scene::paint::{BlendMode, Paint};
use vello::kurbo::{Affine, Rect as KurboRect};
use vello::peniko;
use vello::Scene;

use super::brush_converter::BrushConverter;
use super::clip_converter::ClipConverter;
use super::color_converter::ColorConverter;
use super::path_converter::PathConverter;

/// Half-extent (in logical pixels) of the fallback clip used for a layer with no
/// geometric clip shape. Large enough to bound any realistic scene while
/// remaining finite (so it never produces non-finite path coordinates).
const UNBOUNDED_HALF_EXTENT: f64 = 1.0e9;

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
#[derive(Debug, Default, Clone, Copy)]
pub struct SceneConverter {
    path: PathConverter,
    brush: BrushConverter,
    clip: ClipConverter,
}

impl SceneConverter {
    /// Creates a scene converter with freshly constructed leaf converters.
    #[must_use]
    pub fn new() -> Self {
        let color = ColorConverter::new();
        let path = PathConverter::new();
        Self {
            path,
            brush: BrushConverter::new(color),
            clip: ClipConverter::new(path),
        }
    }

    /// Converts an obscur-scene [`DisplayList`] into a [`vello::Scene`].
    ///
    /// Commands are replayed in order. Pops are balanced against a depth counter,
    /// so a malformed (unbalanced) list is handled without panicking: a stray pop
    /// is ignored, and any layers left open are closed at the end.
    #[must_use]
    pub fn convert(&self, list: &DisplayList) -> Scene {
        let mut scene = Scene::new();
        let mut depth: u32 = 0;

        for command in list.commands() {
            match command {
                PaintCommand::DrawPath { path, paint } => self.draw_path(&mut scene, path, paint),
                PaintCommand::PushClip(clip) => {
                    self.push_clip(&mut scene, clip);
                    depth += 1;
                }
                PaintCommand::PushLayer(layer) => {
                    self.push_layer(&mut scene, layer);
                    depth += 1;
                }
                PaintCommand::PopClip | PaintCommand::PopLayer => {
                    if depth > 0 {
                        scene.pop_layer();
                        depth -= 1;
                    }
                }
                // Text-run glyph rasterization is added in a later step; ignore
                // for now so the display list still converts.
                PaintCommand::DrawText { .. } => {}
            }
        }

        // Close any layers a dangling-push (unbalanced) list left open.
        for _ in 0..depth {
            scene.pop_layer();
        }

        scene
    }

    /// Fills or strokes a single path.
    fn draw_path(self, scene: &mut Scene, path: &obscur_scene::path::Path, paint: &Paint) {
        let bez = self.path.convert(path);
        let brush = self.brush.convert(paint.brush());
        match paint {
            Paint::Fill(_) => {
                let fill = self.path.fill_rule(path.fill_rule());
                scene.fill(fill, Affine::IDENTITY, &brush, None, &bez);
            }
            Paint::Stroke(stroke) => {
                let style = self.brush.stroke(stroke);
                scene.stroke(&style, Affine::IDENTITY, &brush, None, &bez);
            }
        }
    }

    /// Pushes a clip as a neutral-blend layer bounded by the clip's shape.
    fn push_clip(self, scene: &mut Scene, clip: &Clip) {
        let shape = self.clip.shape(clip);
        scene.push_layer(
            peniko::Fill::NonZero,
            peniko::BlendMode::default(),
            1.0,
            Affine::IDENTITY,
            &shape,
        );
    }

    /// Pushes a compositing layer with its opacity, blend, and clip shape.
    fn push_layer(self, scene: &mut Scene, layer: &SceneLayer) {
        let blend = peniko::BlendMode::new(mix(layer.blend_mode()), compose(layer.composition()));
        let alpha = layer.opacity();
        if let Some(clip @ Clip::Shape(_)) = layer.clip() {
            let shape = self.clip.shape(clip);
            scene.push_layer(peniko::Fill::NonZero, blend, alpha, Affine::IDENTITY, &shape);
        } else {
            let unbounded = KurboRect::new(
                -UNBOUNDED_HALF_EXTENT,
                -UNBOUNDED_HALF_EXTENT,
                UNBOUNDED_HALF_EXTENT,
                UNBOUNDED_HALF_EXTENT,
            );
            scene.push_layer(
                peniko::Fill::NonZero,
                blend,
                alpha,
                Affine::IDENTITY,
                &unbounded,
            );
        }
    }
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
