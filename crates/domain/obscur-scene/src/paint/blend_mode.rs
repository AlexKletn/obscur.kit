//! How a shape's color composites with the content beneath it.

#[doc = include_str!("../../docs/api/paint/blend_mode.doc/BlendMode.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlendMode {
    /// Source-over compositing: the source replaces the backdrop where opaque.
    Normal,
    /// Multiplies source and backdrop channels; the result is always darker.
    Multiply,
    /// Inverse-multiplies source and backdrop; the result is always lighter.
    Screen,
    /// Multiplies or screens depending on the backdrop, increasing contrast.
    Overlay,
    /// Keeps the darker of source and backdrop per channel.
    Darken,
    /// Keeps the lighter of source and backdrop per channel.
    Lighten,
    /// Brightens the backdrop to reflect the source.
    ColorDodge,
    /// Darkens the backdrop to reflect the source.
    ColorBurn,
    #[doc = include_str!("../../docs/api/paint/blend_mode.doc/HardLight.md")]
    HardLight,
    #[doc = include_str!("../../docs/api/paint/blend_mode.doc/SoftLight.md")]
    SoftLight,
    /// Subtracts the darker from the lighter per channel.
    Difference,
    #[doc = include_str!("../../docs/api/paint/blend_mode.doc/Exclusion.md")]
    Exclusion,
    /// Uses the source hue and the backdrop saturation and luminosity.
    Hue,
    /// Uses the source saturation and the backdrop hue and luminosity.
    Saturation,
    /// Uses source hue and saturation with backdrop luminosity.
    Color,
    /// Uses source luminosity with backdrop hue and saturation.
    Luminosity,
}

impl BlendMode {
    /// Returns whether this is the [`BlendMode::Normal`] (source-over) mode.
    #[must_use]
    pub const fn is_normal(self) -> bool {
        matches!(self, Self::Normal)
    }
}

impl Default for BlendMode {
    /// The default blend mode is [`BlendMode::Normal`].
    fn default() -> Self {
        Self::Normal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_normal() {
        assert_eq!(BlendMode::default(), BlendMode::Normal);
        assert!(BlendMode::default().is_normal());
    }

    #[test]
    fn non_normal_is_not_normal() {
        assert!(!BlendMode::Multiply.is_normal());
    }
}
