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
use obscur_scene::layer::{CompositionMode, SceneLayer};
use obscur_scene::paint::{BlendMode, Brush, Paint};
use obscur_scene::text::GlyphRun;
use vello::kurbo::{Affine, Diagonal2, Rect as KurboRect};
use vello::peniko::{self, FontData};
use vello::{FontEmbolden, Scene};

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
                PaintCommand::DrawText { origin, run, brush } => {
                    self.draw_text(&mut scene, *origin, run, brush);
                }
            }
        }

        // Close any layers a dangling-push (unbalanced) list left open.
        for _ in 0..depth {
            scene.pop_layer();
        }

        scene
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

    /// Fills or strokes a single path.
    fn draw_path(&self, scene: &mut Scene, path: &obscur_scene::path::Path, paint: &Paint) {
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
    fn push_clip(&self, scene: &mut Scene, clip: &Clip) {
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
    fn push_layer(&self, scene: &mut Scene, layer: &SceneLayer) {
        let blend = peniko::BlendMode::new(mix(layer.blend_mode()), compose(layer.composition()));
        let alpha = layer.opacity();
        if let Some(clip @ Clip::Shape(_)) = layer.clip() {
            let shape = self.clip.shape(clip);
            scene.push_layer(
                peniko::Fill::NonZero,
                blend,
                alpha,
                Affine::IDENTITY,
                &shape,
            );
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
