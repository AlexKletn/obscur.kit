use obscur_scene::color::Color;

use super::{font_paint::FontPaint, geometry::Contour};

pub(super) enum ColorDraw {
    Fill(Option<Color>, f32, Contour),
    Gradient(FontPaint, Contour),
    Push(Contour),
    Pop,
    BeginGroup(Option<u32>),
    EndGroup,
}
