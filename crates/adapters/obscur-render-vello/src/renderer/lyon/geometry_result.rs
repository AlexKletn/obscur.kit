use super::{bitmap_glyph::Bitmap, color_draw::ColorDraw, geometry::Contour};

pub(super) enum Geometry {
    Mesh(Contour),
    Bitmap(Bitmap),
    Color {
        layers: Vec<ColorDraw>,
        linear_group: bool,
    },
}
