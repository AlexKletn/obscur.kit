//! How a gradient extends beyond its defined `[0, 1]` range.

/// Controls how a gradient is sampled outside its nominal `[0.0, 1.0]` offset
/// range (spec §5.1).
///
/// The mode determines the color used for positions before the first stop or
/// after the last stop of a gradient.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExtendMode {
    /// Clamp to the edge stops: positions below `0.0` take the first stop's
    /// color and positions above `1.0` take the last stop's color.
    Pad,
    /// Tile the gradient, repeating the `[0, 1]` pattern periodically.
    Repeat,
    /// Tile the gradient, mirroring every other repetition so adjacent tiles
    /// meet seamlessly.
    Reflect,
}

impl ExtendMode {
    /// Returns whether this is the [`ExtendMode::Pad`] mode.
    #[must_use]
    pub const fn is_pad(self) -> bool {
        matches!(self, Self::Pad)
    }
}

impl Default for ExtendMode {
    /// The default extend mode is [`ExtendMode::Pad`].
    fn default() -> Self {
        Self::Pad
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_pad() {
        assert_eq!(ExtendMode::default(), ExtendMode::Pad);
        assert!(ExtendMode::default().is_pad());
    }

    #[test]
    fn variants_distinct() {
        assert_ne!(ExtendMode::Repeat, ExtendMode::Reflect);
    }
}
