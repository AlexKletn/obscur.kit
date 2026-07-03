//! Converts an obscur-scene [`Color`] into a Vello/peniko color.
//!
//! Pure, backend-local conversion (spec §2.7): the domain authors colors as
//! straight-alpha sRGB `f32` components; this maps them to peniko's
//! `AlphaColor<Srgb>` (spec §5.2 — geometry and paint order are preserved by the
//! adapter; only rasterization is delegated to Vello).

use obscur_scene::color::Color;
use vello::peniko;

/// A stateless converter from an obscur-scene [`Color`] to a [`peniko::Color`].
///
/// obscur-scene stores colors as straight-alpha sRGB `f32` components that are
/// deliberately *not* clamped (to allow wide-gamut / HDR authoring). peniko's
/// `AlphaColor<Srgb>` is likewise an `f32` sRGB color, so the conversion is a
/// direct component copy; components are clamped to `[0.0, 1.0]` here because the
/// Vello pipeline expects displayable sRGB values.
#[derive(Debug, Default, Clone, Copy)]
pub struct ColorConverter;

impl ColorConverter {
    /// Creates a color converter.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Converts an obscur-scene [`Color`] into a [`peniko::Color`].
    ///
    /// Each channel is clamped to `[0.0, 1.0]`; the alpha is straight (not
    /// premultiplied), matching peniko's `AlphaColor<Srgb>`.
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
