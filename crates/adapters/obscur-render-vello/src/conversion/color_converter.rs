#![doc = include_str!("../../docs/api/conversion/color_converter.doc/_module.md")]

use obscur_scene::color::Color;
use vello::peniko;

#[doc = include_str!("../../docs/api/conversion/color_converter.doc/ColorConverter.md")]
#[derive(Debug, Default, Clone, Copy)]
pub struct ColorConverter;

impl ColorConverter {
    /// Creates a color converter.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    #[doc = include_str!("../../docs/api/conversion/color_converter.doc/convert.md")]
    #[must_use]
    pub fn convert(&self, color: Color) -> peniko::Color {
        peniko::Color::new([
            color.r.clamp(0.0, 1.0),
            color.g.clamp(0.0, 1.0),
            color.b.clamp(0.0, 1.0),
            color.a.clamp(0.0, 1.0),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_color_round_trips_through_rgba8() {
        let converted = ColorConverter::new().convert(Color::rgba(1.0, 0.0, 0.0, 1.0));
        let rgba8 = converted.to_rgba8();
        assert_eq!((rgba8.r, rgba8.g, rgba8.b, rgba8.a), (255, 0, 0, 255));
    }

    #[test]
    fn mid_channels_map_to_expected_bytes() {
        let converted = ColorConverter::new().convert(Color::rgba(0.0, 1.0, 0.0, 0.5));
        let rgba8 = converted.to_rgba8();
        assert_eq!(rgba8.r, 0);
        assert_eq!(rgba8.g, 255);
        assert_eq!(rgba8.b, 0);
        // 0.5 straight alpha rounds to 128.
        assert_eq!(rgba8.a, 128);
    }

    #[test]
    fn out_of_range_components_are_clamped() {
        let converted = ColorConverter::new().convert(Color::rgba(2.0, -1.0, 0.5, 3.0));
        let rgba8 = converted.to_rgba8();
        assert_eq!(rgba8.r, 255);
        assert_eq!(rgba8.g, 0);
        assert_eq!(rgba8.a, 255);
    }
}
