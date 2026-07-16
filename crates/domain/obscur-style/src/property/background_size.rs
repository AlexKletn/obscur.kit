//! The computed value of CSS `background-size` (spec §5.2, CSS Backgrounds §3).

use crate::property::Length;

/// The computed value of `background-size`: how one copy of a
/// `background-image` is scaled before it is placed and (optionally) tiled.
///
/// The scene coordinator resolves this against the painted box and the image's
/// intrinsic pixel size to a concrete tile width/height at paint time:
///
/// * [`BackgroundSize::Auto`] — draw the image at its intrinsic pixel size (the
///   CSS initial). When both axes are `auto` the intrinsic size is used
///   directly.
/// * [`BackgroundSize::Cover`] — scale the image (preserving its aspect ratio)
///   to the smallest size that fully covers the box; overflow is clipped by the
///   background paint region.
/// * [`BackgroundSize::Contain`] — scale the image (preserving its aspect ratio)
///   to the largest size that fits entirely inside the box.
/// * [`BackgroundSize::Explicit`] — an explicit per-axis size, each component a
///   [`Length`] (`px` / `%` of the box extent) or [`Length::Auto`]. A single
///   `auto` axis is derived from the other axis to preserve the intrinsic aspect
///   ratio; both `auto` collapse to the intrinsic size (like
///   [`BackgroundSize::Auto`]).
///
/// Because [`Length`] carries `f32` data this derives [`PartialEq`] but not
/// [`Eq`]/[`Hash`].
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
    /// Whether either explicit axis is a `ch` term — the question
    /// [`PropertyValue::uses_ch`](crate::property::PropertyValue::uses_ch) asks
    /// before paying for a `0`-glyph measurement.
    ///
    /// The keyword forms carry no length, exactly as in
    /// `absolutize_background_size`.
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
