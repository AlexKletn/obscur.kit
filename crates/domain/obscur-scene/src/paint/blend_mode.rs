//! How a shape's color composites with the content beneath it.

/// The blend function used when compositing a shape over its backdrop
/// (spec §5.1).
///
/// This is a representative subset of the CSS `mix-blend-mode` values covering
/// the **separable** blend modes (each channel blends independently) plus
/// [`BlendMode::Normal`]. The full **non-separable** set — `Hue`, `Saturation`,
/// `Color`, and `Luminosity`, which mix whole colors rather than individual
/// channels — is a later addition.
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
    /// Multiplies or screens depending on the source, like [`Overlay`] with the
    /// roles of source and backdrop swapped.
    ///
    /// [`Overlay`]: BlendMode::Overlay
    HardLight,
    /// A softer variant of [`HardLight`] that darkens or lightens gently.
    ///
    /// [`HardLight`]: BlendMode::HardLight
    SoftLight,
    /// Subtracts the darker from the lighter per channel.
    Difference,
    /// Like [`Difference`] but with lower contrast in the mid-tones.
    ///
    /// [`Difference`]: BlendMode::Difference
    Exclusion,
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
