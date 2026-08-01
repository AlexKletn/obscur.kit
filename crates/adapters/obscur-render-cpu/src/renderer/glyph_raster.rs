#![doc = include_str!("glyph_raster.doc/_module.md")]

use std::collections::HashMap;
use std::sync::Arc;

use obscur_geometry::Point;
use obscur_scene::paint::Brush;
use obscur_scene::text::GlyphRun;
use skrifa::instance::{LocationRef, Size as SkrifaSize};
use skrifa::outline::DrawSettings;
use skrifa::{FontRef, GlyphId, MetadataProvider};

use crate::renderer::brush_fill::brush_color;
use crate::renderer::glyph_pen::GlyphPathPen;
use crate::renderer::skia_lowering::solid_paint;

#[doc = include_str!("glyph_raster.doc/draw_text_run.md")]
pub(crate) fn draw_text_run(
    pixmap: &mut tiny_skia::Pixmap,
    origin: Point,
    run: &GlyphRun,
    brush: &Brush,
    clip: Option<&tiny_skia::Mask>,
    transform: tiny_skia::Transform,
    fonts: &HashMap<u32, Arc<[u8]>>,
) {
    let Some(color) = brush_color(brush) else {
        return;
    };
    let Some(sk_paint) = solid_paint(color) else {
        return;
    };
    let font_size = run.font_size();
    if font_size <= 0.0 {
        return;
    }

    // Real outlines when the run's font is registered and its bytes are a valid
    // sfnt; box-coverage fallback otherwise. Synthetic-style hints (spec §5.18)
    // travel with the run: an oblique run is sheared, a bold run thickened.
    let skew = if run.synth_oblique() {
        FAUX_ITALIC_SKEW
    } else {
        0.0
    };
    if let Some(bytes) = fonts.get(&run.font())
        && let Ok(font_ref) = FontRef::new(bytes)
    {
        draw_glyph_outlines(
            pixmap,
            origin,
            run,
            &font_ref,
            &sk_paint,
            clip,
            transform,
            skew,
            run.synth_bold(),
        );
        return;
    }
    draw_glyph_boxes(pixmap, origin, run, &sk_paint, clip, transform, skew);
}

#[doc = include_str!("glyph_raster.doc/FAUX_ITALIC_SKEW.md")]
const FAUX_ITALIC_SKEW: f32 = 0.25;

#[doc = include_str!("glyph_raster.doc/FAUX_BOLD_STROKE_FRACTION.md")]
const FAUX_BOLD_STROKE_FRACTION: f32 = 0.03;

#[doc = include_str!("glyph_raster.doc/draw_glyph_outlines.md")]
#[allow(clippy::too_many_arguments)]
fn draw_glyph_outlines(
    pixmap: &mut tiny_skia::Pixmap,
    origin: Point,
    run: &GlyphRun,
    font: &FontRef<'_>,
    sk_paint: &tiny_skia::Paint<'_>,
    clip: Option<&tiny_skia::Mask>,
    transform: tiny_skia::Transform,
    skew: f32,
    bold: bool,
) {
    let outlines = font.outline_glyphs();
    let size = SkrifaSize::new(run.font_size());
    // Faux-bold: stroke the filled outline with a hairline proportional to the
    // font size so the shape gains weight without re-shaping.
    let bold_stroke = bold.then(|| tiny_skia::Stroke {
        width: run.font_size() * FAUX_BOLD_STROKE_FRACTION,
        ..Default::default()
    });
    for glyph in run.glyphs() {
        let Some(outline) = outlines.get(GlyphId::new(glyph.glyph_id())) else {
            continue;
        };
        let pen_x = origin.x + glyph.x();
        let baseline_y = origin.y + glyph.y();
        let mut pen = GlyphPathPen::with_skew(pen_x, baseline_y, skew);
        let settings = DrawSettings::unhinted(size, LocationRef::default());
        if outline.draw(settings, &mut pen).is_err() {
            continue;
        }
        let Some(sk_path) = pen.into_path() else {
            continue;
        };
        pixmap.fill_path(
            &sk_path,
            sk_paint,
            tiny_skia::FillRule::Winding,
            transform,
            clip,
        );
        if let Some(stroke) = &bold_stroke {
            pixmap.stroke_path(&sk_path, sk_paint, stroke, transform, clip);
        }
    }
}

#[doc = include_str!("glyph_raster.doc/draw_glyph_boxes.md")]
fn draw_glyph_boxes(
    pixmap: &mut tiny_skia::Pixmap,
    origin: Point,
    run: &GlyphRun,
    sk_paint: &tiny_skia::Paint<'_>,
    clip: Option<&tiny_skia::Mask>,
    transform: tiny_skia::Transform,
    skew: f32,
) {
    let font_size = run.font_size();
    for glyph in run.glyphs() {
        // Skip whitespace: the mock shaper's glyph id is the Unicode scalar.
        if char::from_u32(glyph.glyph_id()).is_none_or(char::is_whitespace) {
            continue;
        }
        let width = glyph.advance();
        if width <= 0.0 {
            continue;
        }
        let x = origin.x + glyph.x();
        let baseline = origin.y + glyph.y();
        let top = baseline - font_size;
        let sk_path = if skew == 0.0 {
            let Some(rect) = tiny_skia::Rect::from_xywh(x, top, width, font_size) else {
                continue;
            };
            tiny_skia::PathBuilder::from_rect(rect)
        } else {
            // Parallelogram: baseline edge unshifted, top edge slid right by
            // `skew * font_size`.
            let shift = skew * font_size;
            let mut builder = tiny_skia::PathBuilder::new();
            builder.move_to(x, baseline);
            builder.line_to(x + width, baseline);
            builder.line_to(x + width + shift, top);
            builder.line_to(x + shift, top);
            builder.close();
            let Some(path) = builder.finish() else {
                continue;
            };
            path
        };
        pixmap.fill_path(
            &sk_path,
            sk_paint,
            tiny_skia::FillRule::Winding,
            transform,
            clip,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_scene::color::Color;
    use obscur_scene::text::PositionedGlyph;

    /// A 64x64 transparent pixmap to rasterize a run into.
    fn pixmap() -> tiny_skia::Pixmap {
        tiny_skia::Pixmap::new(64, 64).expect("64x64 pixmap allocates")
    }

    /// One glyph at the pen origin, baseline at `y = 40`, font size 32.
    fn run_of(scalar: u32) -> GlyphRun {
        GlyphRun::new(
            vec![PositionedGlyph::new(scalar, 0, 0.0, 40.0, 20.0)],
            32.0,
            "x",
        )
    }

    fn white() -> Brush {
        Brush::solid(Color::WHITE)
    }

    #[test]
    fn an_unregistered_font_falls_back_to_an_advance_width_coverage_cell() {
        let mut target = pixmap();
        draw_text_run(
            &mut target,
            Point::new(0.0, 0.0),
            &run_of(u32::from('A')),
            &white(),
            None,
            tiny_skia::Transform::identity(),
            &HashMap::new(),
        );
        // The cell spans the advance (20) by the font size (32) above the baseline.
        assert!(
            target.pixel(10, 30).expect("pixel").alpha() > 200,
            "the fallback cell is painted"
        );
        assert_eq!(
            target.pixel(50, 30).expect("pixel").alpha(),
            0,
            "nothing is painted past the advance"
        );
    }

    #[test]
    fn a_whitespace_glyph_paints_nothing_in_the_fallback() {
        let mut target = pixmap();
        draw_text_run(
            &mut target,
            Point::new(0.0, 0.0),
            &run_of(u32::from(' ')),
            &white(),
            None,
            tiny_skia::Transform::identity(),
            &HashMap::new(),
        );
        assert!(
            target.pixels().iter().all(|p| p.alpha() == 0),
            "a space draws no coverage cell"
        );
    }

    #[test]
    fn a_non_positive_font_size_draws_nothing() {
        let mut target = pixmap();
        let run = GlyphRun::new(
            vec![PositionedGlyph::new(u32::from('A'), 0, 0.0, 40.0, 20.0)],
            0.0,
            "x",
        );
        draw_text_run(
            &mut target,
            Point::new(0.0, 0.0),
            &run,
            &white(),
            None,
            tiny_skia::Transform::identity(),
            &HashMap::new(),
        );
        assert!(target.pixels().iter().all(|p| p.alpha() == 0));
    }

    #[test]
    fn a_stop_less_gradient_brush_skips_the_whole_run() {
        use obscur_scene::paint::RadialGradient;

        let mut target = pixmap();
        draw_text_run(
            &mut target,
            Point::new(0.0, 0.0),
            &run_of(u32::from('A')),
            &Brush::Radial(RadialGradient::new(Point::ORIGIN, 1.0, Vec::new())),
            None,
            tiny_skia::Transform::identity(),
            &HashMap::new(),
        );
        assert!(
            target.pixels().iter().all(|p| p.alpha() == 0),
            "no resolvable colour means no mark"
        );
    }
}
