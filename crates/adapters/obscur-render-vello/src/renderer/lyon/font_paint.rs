use obscur_scene::color::Color;
use obscur_scene::paint::{ExtendMode, GradientStop};

type PaletteStop = (f32, Option<Color>, f32);

pub(super) struct FontPaint {
    pub(super) kind: f32,
    pub(super) extend: ExtendMode,
    pub(super) geometry: [[f32; 4]; 4],
    pub(super) stops: Vec<PaletteStop>,
}

impl FontPaint {
    pub(super) fn stops(&self, foreground: Color) -> Vec<GradientStop> {
        self.stops
            .iter()
            .map(|&(offset, color, alpha)| {
                let mut color = color.unwrap_or(foreground);
                color.a *= alpha;
                GradientStop::new(offset, color)
            })
            .collect()
    }
}
