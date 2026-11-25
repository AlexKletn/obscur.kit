#![doc = include_str!("../../docs/api/property/style_color.doc/_module.md")]

#[doc = include_str!("../../docs/api/property/style_color.doc/StyleColor.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StyleColor {
    /// Red channel in `0.0..=1.0` (sRGB).
    pub r: f32,
    /// Green channel in `0.0..=1.0` (sRGB).
    pub g: f32,
    /// Blue channel in `0.0..=1.0` (sRGB).
    pub b: f32,
    /// Straight (non-premultiplied) alpha in `0.0..=1.0`.
    pub a: f32,
}

impl StyleColor {
    /// Fully transparent black (`rgba(0, 0, 0, 0)`), the used value of the CSS
    /// `transparent` keyword.
    pub const TRANSPARENT: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };

    /// Opaque black (`rgba(0, 0, 0, 1)`).
    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    #[doc = include_str!("../../docs/api/property/style_color.doc/rgba.md")]
    #[must_use]
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgba_stores_channels() {
        let c = StyleColor::rgba(0.25, 0.5, 0.75, 1.0);
        assert_eq!(c.r, 0.25);
        assert_eq!(c.g, 0.5);
        assert_eq!(c.b, 0.75);
        assert_eq!(c.a, 1.0);
    }

    #[test]
    fn transparent_is_zero_alpha() {
        assert_eq!(StyleColor::TRANSPARENT.a, 0.0);
    }

    #[test]
    fn black_is_opaque() {
        assert_eq!(StyleColor::BLACK, StyleColor::rgba(0.0, 0.0, 0.0, 1.0));
    }
}
