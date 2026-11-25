//! Lower COLRv1 paint graphs into clipped contours and isolated layers.

use obscur_scene::paint::ExtendMode;
use obscur_scene::{color::Color, path::Path};
use skrifa::color::{Brush, ColorGlyph, ColorPainter, CompositeMode, Extend, Transform};
use skrifa::instance::LocationRef;
use skrifa::raw::{TableProvider, types::BoundingBox};
use skrifa::{FontRef, GlyphId};
use vello::kurbo::{Affine, BezPath, Rect, Shape};

use crate::conversion::FAUX_ITALIC_SKEW;

use super::color_composition;
use super::color_glyph::Palette;
use super::color_paint_command::ColorPaintCommand;
use super::font_paint::FontPaint;
use super::geometry::{from_outline, unscaled_outline};

pub(super) fn paint(
    face: &FontRef<'_>,
    glyph: &ColorGlyph<'_>,
    size: f32,
    oblique: bool,
) -> Result<Vec<ColorPaintCommand>, &'static str> {
    let upem = face
        .head()
        .map_err(|_| "invalid font header")?
        .units_per_em();
    if upem == 0 {
        return Err("invalid COLRv1 scale");
    }
    let scale = f64::from(size) / f64::from(upem);
    let shear = if oblique { FAUX_ITALIC_SKEW } else { 0.0 };
    let mut painter = Painter {
        face,
        palette: Palette::new(face)?,
        transforms: vec![Affine::new([scale, 0.0, scale * shear, -scale, 0.0, 0.0])],
        clips: Vec::new(),
        commands: Vec::new(),
        error: None,
    };
    glyph
        .paint(LocationRef::default(), &mut painter)
        .map_err(|_| "invalid COLRv1 paint graph")?;
    painter.error.map_or(Ok(painter.commands), Err)
}

struct Painter<'a> {
    face: &'a FontRef<'a>,
    palette: Palette<'a>,
    transforms: Vec<Affine>,
    clips: Vec<Path>,
    commands: Vec<ColorPaintCommand>,
    error: Option<&'static str>,
}

impl Painter<'_> {
    fn transform(&self) -> Affine {
        *self.transforms.last().expect("glyph transform")
    }

    fn path(&self, mut outline: BezPath) -> Path {
        outline.apply_affine(self.transform());
        from_outline(&outline)
    }

    fn outline(&mut self, glyph: GlyphId) -> Path {
        match unscaled_outline(self.face, glyph) {
            Ok(outline) => self.path(outline),
            Err(error) => {
                self.error.get_or_insert(error);
                Path::new()
            }
        }
    }

    fn push(&mut self, path: Path) {
        self.clips.push(path.clone());
        self.commands.push(ColorPaintCommand::Push(path));
    }

    fn fill_path(&mut self, path: Path, brush: Brush<'_>, brush_transform: Affine) {
        if self.error.is_some() {
            return;
        }
        let command = match brush {
            Brush::Solid {
                palette_index,
                alpha,
            } => self
                .palette
                .color(palette_index)
                .map(|color| ColorPaintCommand::Fill(path, color, alpha.clamp(0.0, 1.0))),
            Brush::LinearGradient {
                p0,
                p1,
                color_stops,
                extend,
            } => {
                let dx = p1.x - p0.x;
                let dy = p1.y - p0.y;
                let length = dx * dx + dy * dy;
                let slope = if color_stops.len() == 1 {
                    [0.0; 2]
                } else {
                    [dx / length, dy / length]
                };
                self.gradient(
                    0.0,
                    [p0.x, p0.y, slope[0], slope[1]],
                    [0.0; 4],
                    color_stops,
                    extend,
                    brush_transform,
                )
                .map(|paint| ColorPaintCommand::Gradient(path, paint))
            }
            Brush::RadialGradient {
                c0,
                r0,
                c1,
                r1,
                color_stops,
                extend,
            } => self
                .gradient(
                    3.0,
                    [c0.x, c0.y, c1.x - c0.x, c1.y - c0.y],
                    [r0, r1 - r0, 0.0, 0.0],
                    color_stops,
                    extend,
                    brush_transform,
                )
                .map(|paint| ColorPaintCommand::Gradient(path, paint)),
            Brush::SweepGradient {
                c0,
                start_angle,
                end_angle,
                color_stops,
                extend,
            } => self
                .gradient(
                    4.0,
                    [c0.x, c0.y, start_angle / 360.0, end_angle / 360.0],
                    [0.0; 4],
                    color_stops,
                    extend,
                    brush_transform,
                )
                .map(|paint| ColorPaintCommand::Gradient(path, paint)),
        };
        match command {
            Ok(command) => self.commands.push(command),
            Err(error) => self.error = Some(error),
        }
    }

    #[allow(clippy::too_many_arguments, clippy::cast_possible_truncation)]
    fn gradient(
        &self,
        kind: f32,
        position: [f32; 4],
        parameters: [f32; 4],
        stops: &[skrifa::color::ColorStop],
        extend: Extend,
        brush_transform: Affine,
    ) -> Result<FontPaint, &'static str> {
        let [a, b, c, d, x, y] = (self.transform() * brush_transform)
            .inverse()
            .as_coeffs()
            .map(|value| value as f32);
        let stops = stops
            .iter()
            .map(|stop| {
                Ok((
                    stop.offset,
                    self.palette.color(stop.palette_index)?,
                    stop.alpha.clamp(0.0, 1.0),
                ))
            })
            .collect::<Result<_, &'static str>>()?;
        let extend = match extend {
            Extend::Pad => ExtendMode::Pad,
            Extend::Repeat => ExtendMode::Repeat,
            Extend::Reflect => ExtendMode::Reflect,
            _ => return Err("invalid COLRv1 gradient extend mode"),
        };
        Ok(FontPaint {
            kind,
            extend,
            geometry: [position, [a, c, x, 0.0], [b, d, y, 0.0], parameters],
            stops,
        })
    }
}

impl ColorPainter for Painter<'_> {
    fn push_transform(&mut self, t: Transform) {
        self.transforms.push(self.transform() * affine(t));
    }

    fn pop_transform(&mut self) {
        self.transforms.pop();
    }

    fn push_clip_glyph(&mut self, glyph: GlyphId) {
        let path = self.outline(glyph);
        self.push(path);
    }

    fn push_clip_box(&mut self, bounds: BoundingBox<f32>) {
        let rect = Rect::new(
            bounds.x_min.into(),
            bounds.y_min.into(),
            bounds.x_max.into(),
            bounds.y_max.into(),
        );
        self.push(self.path(rect.to_path(0.1)));
    }

    fn pop_clip(&mut self) {
        self.clips.pop();
        self.commands.push(ColorPaintCommand::Pop);
    }

    fn fill(&mut self, brush: Brush<'_>) {
        if let Some(path) = self.clips.last().cloned() {
            self.fill_path(path, brush, Affine::IDENTITY);
        } else {
            self.error.get_or_insert("unbounded COLRv1 paint");
        }
    }

    fn fill_glyph(&mut self, glyph: GlyphId, transform: Option<Transform>, brush: Brush<'_>) {
        let path = self.outline(glyph);
        self.fill_path(path, brush, transform.map_or(Affine::IDENTITY, affine));
    }

    fn push_layer(&mut self, mode: CompositeMode) {
        self.commands
            .push(ColorPaintCommand::BeginGroup(color_composition::blend(
                mode,
            )));
    }

    fn pop_layer(&mut self) {
        self.commands.push(ColorPaintCommand::EndGroup);
    }
}

fn affine(t: Transform) -> Affine {
    Affine::new([t.xx, t.yx, t.xy, t.yy, t.dx, t.dy].map(f64::from))
}
