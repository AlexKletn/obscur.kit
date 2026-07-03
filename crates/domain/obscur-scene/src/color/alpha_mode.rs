//! Whether a color's or image's alpha is premultiplied into its channels.

/// Whether a color's or image's alpha is premultiplied into its color channels
/// (spec §5.7).
///
/// Authored colors are [`AlphaMode::Straight`]: the color channels are
/// independent of the alpha channel. Compositing operates on
/// [`AlphaMode::Premultiplied`] data, where each color channel has already been
/// scaled by alpha, which makes source-over blending a simple linear
/// combination.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AlphaMode {
    /// Color channels are independent of alpha (as authored).
    Straight,
    /// Color channels have been multiplied by alpha (compositing form).
    Premultiplied,
}

impl AlphaMode {
    /// Returns whether the color channels are premultiplied by alpha.
    #[must_use]
    pub const fn is_premultiplied(self) -> bool {
        matches!(self, Self::Premultiplied)
    }
}

impl Default for AlphaMode {
    /// The default alpha mode is [`AlphaMode::Straight`], matching authored
    /// colors.
    fn default() -> Self {
        Self::Straight
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_straight() {
        assert_eq!(AlphaMode::default(), AlphaMode::Straight);
    }

    #[test]
    fn is_premultiplied() {
        assert!(AlphaMode::Premultiplied.is_premultiplied());
        assert!(!AlphaMode::Straight.is_premultiplied());
    }
}
