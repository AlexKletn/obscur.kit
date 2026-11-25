use obscur_scene::{color::Color, path::Path};

use super::font_paint::FontPaint;

pub(super) enum ColorPaintCommand {
    Fill(Path, Option<Color>, f32),
    Gradient(Path, FontPaint),
    Push(Path),
    Pop,
    BeginGroup(Option<u32>),
    EndGroup,
}
