//! A single sizing input for a layout node (width, height, flex-basis, ...).

/// A sizing input expressed in layout's own vocabulary (spec §4.1).
///
/// A [`Dimension`] describes how a length is specified before resolution:
/// automatic (content/context driven), an absolute number of logical pixels,
/// or a percentage of some base length. All values are fractional logical
/// pixels; no rounding is applied here (spec §4.6).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Dimension {
    /// The length is determined by content/context rather than an explicit value.
    Auto,
    /// An absolute length, in logical pixels.
    Px(f32),
    /// A percentage (`0.0..=100.0` style, e.g. `50.0` means 50%) of a base length.
    Percent(f32),
}

impl Dimension {
    /// Resolves this dimension against a `base` length, in logical pixels.
    ///
    /// * [`Dimension::Px`] resolves to its stored value.
    /// * [`Dimension::Percent`] resolves to `base * percent / 100.0`.
    /// * [`Dimension::Auto`] has no resolvable value and yields [`None`].
    #[must_use]
    pub fn resolve(self, base: f32) -> Option<f32> {
        match self {
            Self::Auto => None,
            Self::Px(value) => Some(value),
            Self::Percent(percent) => Some(base * percent / 100.0),
        }
    }

    /// Returns `true` if this dimension is [`Dimension::Auto`].
    #[must_use]
    pub const fn is_auto(self) -> bool {
        matches!(self, Self::Auto)
    }
}

#[cfg(test)]
mod tests {
    use super::Dimension;

    #[test]
    fn px_resolves_to_its_value() {
        assert_eq!(Dimension::Px(12.5).resolve(100.0), Some(12.5));
    }

    #[test]
    fn percent_resolves_against_base() {
        assert_eq!(Dimension::Percent(50.0).resolve(80.0), Some(40.0));
    }

    #[test]
    fn auto_never_resolves() {
        assert_eq!(Dimension::Auto.resolve(100.0), None);
        assert!(Dimension::Auto.is_auto());
        assert!(!Dimension::Px(0.0).is_auto());
    }
}
