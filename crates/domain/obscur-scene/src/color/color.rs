//! A typed color with premultiplied-alpha semantics (skeleton).

/// An sRGB color with straight alpha, as authored. Compositing converts to
/// linear premultiplied form at paint time (spec §5.7).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Color {
    /// Red channel, `0.0..=1.0`.
    pub r: f32,
    /// Green channel, `0.0..=1.0`.
    pub g: f32,
    /// Blue channel, `0.0..=1.0`.
    pub b: f32,
    /// Alpha channel, `0.0..=1.0`.
    pub a: f32,
}

impl Color {
    /// Fully transparent.
    pub const TRANSPARENT: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };

    /// Opaque black.
    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    /// Creates a color from straight-alpha sRGB components.
    #[must_use]
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
}
