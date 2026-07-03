//! CSS color as modelled in style (not the renderer's paint color).
//!
//! [`StyleColor`] is the *authored* color the cascade works with: sRGB with
//! straight (non-premultiplied) alpha, each channel in `0.0..=1.0`. It is a
//! pure style concept and deliberately has **no dependency on the scene**.
//!
//! Conversion to the renderer's paint color — which is premultiplied and in a
//! linear working space — happens later, in a dedicated scene converter. Keeping
//! that conversion out of this type preserves the authored values exactly as the
//! stylesheet expressed them.

/// An sRGB color with straight alpha, as authored in a stylesheet.
///
/// Each channel (`r`, `g`, `b`, `a`) is nominally in the range `0.0..=1.0`.
/// Values are stored as-authored and are **not** premultiplied; alpha is kept
/// separate from the color channels. This is the style-side representation only;
/// see the module docs for the conversion to renderer paint color.
///
/// Because the channels are `f32`, this type derives [`PartialEq`] but not
/// [`Eq`]/[`Hash`].
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

    /// Constructs a color from straight sRGB channels and alpha.
    ///
    /// Channels are expected to be in `0.0..=1.0`; out-of-range inputs are
    /// stored verbatim (clamping, if any, is the caller's responsibility).
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
