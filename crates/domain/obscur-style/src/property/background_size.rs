//! The computed value of CSS `background-size` (spec §5.2, CSS Backgrounds §3).

use crate::property::Length;

#[doc = include_str!("../../docs/api/property/background_size.doc/BackgroundSize.md")]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum BackgroundSize {
    /// Draw the image at its intrinsic pixel size (the CSS initial `auto`).
    #[default]
    Auto,
    /// Scale to the smallest size that fully covers the box (aspect preserved).
    Cover,
    /// Scale to the largest size that fits inside the box (aspect preserved).
    Contain,
    /// An explicit per-axis size. Each component is a concrete [`Length`]
    /// (`px` / `%`) or [`Length::Auto`] (derived from the other axis).
    Explicit {
        /// The horizontal size component (`Length::Auto` = derive from height).
        width: Length,
        /// The vertical size component (`Length::Auto` = derive from width).
        height: Length,
    },
}

impl BackgroundSize {
    #[doc = include_str!("../../docs/api/property/background_size.doc/uses_ch.md")]
    #[must_use]
    pub const fn uses_ch(&self) -> bool {
        match self {
            Self::Explicit { width, height } => width.is_ch() || height.is_ch(),
            Self::Auto | Self::Cover | Self::Contain => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_auto() {
        assert_eq!(BackgroundSize::default(), BackgroundSize::Auto);
    }

    #[test]
    fn explicit_carries_both_axes() {
        let size = BackgroundSize::Explicit {
            width: Length::Percent(50.0),
            height: Length::Auto,
        };
        assert_eq!(
            size,
            BackgroundSize::Explicit {
                width: Length::Percent(50.0),
                height: Length::Auto,
            }
        );
        assert_ne!(size, BackgroundSize::Cover);
    }
}
