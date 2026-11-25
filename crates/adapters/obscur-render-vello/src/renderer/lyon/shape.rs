use obscur_scene::path::Path;

use super::stroke_geometry::StrokeGeometry;

#[derive(PartialEq)]
pub(super) enum Shape<'a> {
    Path(&'a Path, Option<&'a StrokeGeometry>),
    Glyph {
        font: u32,
        glyph: u32,
        size: f32,
        oblique: bool,
        bold: bool,
    },
}
