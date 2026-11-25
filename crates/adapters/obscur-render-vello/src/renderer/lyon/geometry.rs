//! Tessellates geometry and reuses it independently of paint and transforms.

use lyon_tessellation::math::{Point as LyonPoint, point};
use lyon_tessellation::{
    BuffersBuilder, FillOptions, FillTessellator, FillVertex, StrokeOptions, StrokeTessellator,
    StrokeVertex, VertexBuffers,
};
use obscur_geometry::Point;
use obscur_scene::path::{FillRule, Path, PathSegment};
use obscur_scene::text::SYNTHETIC_BOLD_OUTSET_FRACTION;
use skrifa::color::ColorGlyphFormat;
use skrifa::instance::{LocationRef, Size};
use skrifa::outline::DrawSettings;
use skrifa::raw::TableProvider;
use skrifa::{FontRef, GlyphId, MetadataProvider};

use super::bitmap_glyph::Bitmap;
use super::cached_shape::CachedShape;
pub(super) use super::color_draw::ColorDraw;
use super::color_glyph;
use super::color_paint_command::ColorPaintCommand;
use super::color_painter;
pub(super) use super::geometry_result::Geometry;
use super::glyph_pen::GlyphPen;
pub(super) use super::shape::Shape;
pub(super) use super::stroke_geometry::StrokeGeometry;

use crate::conversion::{FAUX_ITALIC_SKEW, PathConverter, SceneConverter};
use vello::kurbo;

pub(super) type Contour = VertexBuffers<LyonPoint, u32>;

#[derive(Default)]
pub(super) struct GeometryCache {
    fill: FillTessellator,
    stroke: StrokeTessellator,
    entries: Vec<(CachedShape, Geometry)>,
    cursor: usize,
}

impl GeometryCache {
    pub(super) fn begin(&mut self) {
        self.cursor = 0;
    }

    pub(super) fn finish(&mut self) {
        self.entries.truncate(self.cursor);
    }

    pub(super) fn get(
        &mut self,
        shape: Shape<'_>,
        fonts: &SceneConverter,
    ) -> Result<&Geometry, &'static str> {
        let index = self.cursor;
        if !self
            .entries
            .get(index)
            .is_some_and(|(cached, _)| cached.borrow() == shape)
        {
            let buffers = match &shape {
                Shape::Path(path, stroke) => Geometry::Mesh(self.tessellate(path, *stroke)?),
                Shape::Glyph {
                    font,
                    glyph,
                    size,
                    oblique,
                    bold,
                } => {
                    let font = fonts.font_for(*font).ok_or("unregistered font")?;
                    let face = FontRef::new(font.data.as_ref()).map_err(|_| "invalid font data")?;
                    let glyph = GlyphId::new(*glyph);
                    self.glyph(&face, glyph, *size, *oblique, *bold)?
                }
            };
            let entry = (shape.into(), buffers);
            if index == self.entries.len() {
                self.entries.push(entry);
            } else {
                self.entries[index] = entry;
            }
        }
        self.cursor += 1;
        Ok(&self.entries[index].1)
    }

    fn glyph(
        &mut self,
        face: &FontRef<'_>,
        glyph: GlyphId,
        size: f32,
        oblique: bool,
        bold: bool,
    ) -> Result<Geometry, &'static str> {
        if let Some(colr) = face.color_glyphs().get(glyph) {
            if matches!(colr.format(), ColorGlyphFormat::ColrV1) {
                let commands = color_painter::paint(face, &colr, size, oblique)?;
                let mut layers = Vec::with_capacity(commands.len());
                for command in commands {
                    layers.push(match command {
                        ColorPaintCommand::Fill(path, color, alpha) => {
                            ColorDraw::Fill(color, alpha, self.tessellate(&path, None)?)
                        }
                        ColorPaintCommand::Gradient(path, paint) => {
                            ColorDraw::Gradient(paint, self.tessellate(&path, None)?)
                        }
                        ColorPaintCommand::Push(path) => {
                            ColorDraw::Push(self.tessellate(&path, None)?)
                        }
                        ColorPaintCommand::Pop => ColorDraw::Pop,
                        ColorPaintCommand::BeginGroup(mode) => ColorDraw::BeginGroup(mode),
                        ColorPaintCommand::EndGroup => ColorDraw::EndGroup,
                    });
                }
                let linear_group = layers
                    .iter()
                    .any(|draw| matches!(draw, ColorDraw::BeginGroup(_)))
                    || layers
                        .iter()
                        .filter(|draw| {
                            matches!(draw, ColorDraw::Fill(..) | ColorDraw::Gradient(..))
                        })
                        .count()
                        > 1;
                return Ok(Geometry::Color {
                    layers,
                    linear_group,
                });
            }
            let mut layers = Vec::new();
            for layer in color_glyph::layers(face, glyph)? {
                let (glyph, paint) = layer?;
                let path = glyph_outline(face, glyph, size, oblique, false)?;
                layers.push(ColorDraw::Fill(paint, 1.0, self.tessellate(&path, None)?));
            }
            return Ok(Geometry::Color {
                layers,
                linear_group: false,
            });
        }
        let strikes = face.bitmap_strikes();
        if let Some(bitmap) = strikes.glyph_for_size(Size::new(size), glyph) {
            let upem = face
                .head()
                .map_err(|_| "invalid font header")?
                .units_per_em();
            return Ok(Geometry::Bitmap(Bitmap::decode(
                bitmap,
                strikes.format(),
                upem,
                size,
                oblique,
            )?));
        }
        let path = glyph_outline(face, glyph, size, oblique, bold)?;
        Ok(Geometry::Mesh(self.tessellate(&path, None)?))
    }

    fn tessellate(
        &mut self,
        path: &Path,
        stroke: Option<&StrokeGeometry>,
    ) -> Result<VertexBuffers<LyonPoint, u32>, &'static str> {
        if let Some(StrokeGeometry::Dashed(style)) = stroke {
            let bezier = PathConverter::new().convert(path);
            let outline = kurbo::stroke(bezier, style, &kurbo::StrokeOpts::default(), 0.1);
            return self.tessellate(&from_outline(&outline), None);
        }
        let lyon_path = convert_path(path)?;
        let mut buffers = VertexBuffers::new();
        if let Some(&StrokeGeometry::Solid {
            width,
            cap,
            join,
            miter_limit,
        }) = stroke
        {
            let options = StrokeOptions::tolerance(0.1)
                .with_line_width(width)
                .with_line_cap(cap)
                .with_line_join(join)
                .with_miter_limit(miter_limit);
            self.stroke
                .tessellate_path(
                    &lyon_path,
                    &options,
                    &mut BuffersBuilder::new(&mut buffers, |vertex: StrokeVertex| {
                        vertex.position()
                    }),
                )
                .map_err(|_| "stroke tessellation failed")?;
        } else {
            let rule = match path.fill_rule() {
                FillRule::NonZero => lyon_tessellation::path::FillRule::NonZero,
                FillRule::EvenOdd => lyon_tessellation::path::FillRule::EvenOdd,
            };
            let options = FillOptions::tolerance(0.1).with_fill_rule(rule);
            self.fill
                .tessellate_path(
                    &lyon_path,
                    &options,
                    &mut BuffersBuilder::new(&mut buffers, |vertex: FillVertex| vertex.position()),
                )
                .map_err(|_| "fill tessellation failed")?;
        }
        Ok(buffers)
    }
}

#[allow(clippy::cast_possible_truncation)]
pub(super) fn from_outline(outline: &kurbo::BezPath) -> Path {
    let mut path = Path::new();
    let point = |p: kurbo::Point| Point::new(p.x as f32, p.y as f32);
    for element in outline.elements() {
        match *element {
            kurbo::PathEl::MoveTo(end) => path.move_to(point(end)),
            kurbo::PathEl::LineTo(end) => path.line_to(point(end)),
            kurbo::PathEl::QuadTo(control, end) => path.quad_to(point(control), point(end)),
            kurbo::PathEl::CurveTo(first, second, end) => {
                path.cubic_to(point(first), point(second), point(end))
            }
            kurbo::PathEl::ClosePath => path.close(),
        };
    }
    path
}

fn convert_path(path: &Path) -> Result<lyon_tessellation::path::Path, &'static str> {
    let mut builder = lyon_tessellation::path::Path::builder();
    let mut open = false;
    let position = |p: Point| {
        if p.x.is_finite() && p.y.is_finite() {
            Ok(point(p.x, p.y))
        } else {
            Err("non-finite path coordinate")
        }
    };
    for segment in path.segments() {
        match *segment {
            PathSegment::MoveTo(to) => {
                if open {
                    builder.end(false);
                }
                builder.begin(position(to)?);
                open = true;
            }
            PathSegment::LineTo(to) if open => {
                builder.line_to(position(to)?);
            }
            PathSegment::QuadTo { control, end } if open => {
                builder.quadratic_bezier_to(position(control)?, position(end)?);
            }
            PathSegment::CubicTo {
                control1,
                control2,
                end,
            } if open => {
                builder.cubic_bezier_to(position(control1)?, position(control2)?, position(end)?);
            }
            PathSegment::Close if open => {
                builder.end(true);
                open = false;
            }
            _ => return Err("path continuation without an open contour"),
        }
    }
    if open {
        builder.end(false);
    }
    Ok(builder.build())
}

fn glyph_outline(
    face: &FontRef<'_>,
    glyph: GlyphId,
    pixel_size: f32,
    oblique: bool,
    bold: bool,
) -> Result<Path, &'static str> {
    let size = Size::new(pixel_size);
    let shear = if oblique { FAUX_ITALIC_SKEW } else { 0.0 };
    let mut pen = GlyphPen {
        path: Path::new(),
        shear: if bold { 0.0 } else { shear as f32 },
        y_scale: if bold { 1.0 } else { -1.0 },
    };
    if let Some(outline) = face.outline_glyphs().get(glyph) {
        let settings = DrawSettings::unhinted(size, LocationRef::default());
        outline
            .draw(settings, &mut pen)
            .map_err(|_| "glyph outline failed")?;
    }
    if bold && !pen.path.is_empty() {
        // Expand in font coordinates before applying oblique and flipping the Y axis.
        let outline = PathConverter::new().convert(&pen.path);
        let amount = f64::from(pixel_size) * f64::from(SYNTHETIC_BOLD_OUTSET_FRACTION);
        let mut expanded = kurbo::expand_path(
            &outline,
            kurbo::Diagonal2::new(amount, amount),
            kurbo::Join::Miter,
            4.0,
            0.1,
        );
        expanded.apply_affine(kurbo::Affine::new([1.0, 0.0, shear, -1.0, 0.0, 0.0]));
        return Ok(from_outline(&expanded));
    }
    Ok(pen.path)
}

pub(super) fn unscaled_outline(
    face: &FontRef<'_>,
    glyph: GlyphId,
) -> Result<kurbo::BezPath, &'static str> {
    let mut pen = GlyphPen {
        path: Path::new(),
        shear: 0.0,
        y_scale: 1.0,
    };
    if let Some(outline) = face.outline_glyphs().get(glyph) {
        outline
            .draw(
                DrawSettings::unhinted(Size::unscaled(), LocationRef::default()),
                &mut pen,
            )
            .map_err(|_| "COLRv1 outline failed")?;
    }
    Ok(PathConverter::new().convert(&pen.path))
}
