//! A single sizing input for a layout node (width, height, flex-basis, ...).

use std::sync::Arc;

use obscur_geometry::saturating_f32;

use super::TrackLimit;

#[doc = include_str!("../../docs/api/model/dimension.doc/Dimension.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum Dimension {
    /// The length is determined by content/context rather than an explicit value.
    Auto,
    /// An absolute length, in logical pixels.
    Px(f32),
    /// A percentage (`0.0..=100.0` style, e.g. `50.0` means 50%) of a base length.
    Percent(f32),
    /// A nonnegative size given by a pixel offset and a fraction of the containing axis.
    Affine {
        /// The fixed pixel offset.
        pixels: f32,
        /// The fraction of the containing axis.
        fraction: f32,
    },
    /// `min-content`: the smallest size the content can take without overflowing
    /// — in the inline axis, the widest unbreakable run (CSS Sizing 3 §4.1).
    MinContent,
    /// A CSS size expression resolved against the containing axis.
    Math(Arc<TrackLimit>),
    /// `max-content`: the size the content wants given unbounded space — in the
    /// inline axis, everything on one line (CSS Sizing 3 §4.1).
    MaxContent,
    /// `fit-content`: `min(max-content, max(min-content, stretch-fit))` — the
    /// classic shrink-to-fit (CSS Sizing 3 §4.1, CSS 2.1 §10.3.5).
    FitContent,
}

impl Dimension {
    #[doc = include_str!("../../docs/api/model/dimension.doc/resolve.md")]
    #[must_use]
    pub fn resolve(&self, base: f32) -> Option<f32> {
        let value = self.resolve_signed(base)?;
        Some(match self {
            Self::Affine { .. } | Self::Math(_) => value.max(0.0),
            _ => value,
        })
    }

    /// Resolves signed offsets and saturates overflow to the nearest finite bound. Undefined results remain unresolved.
    pub(crate) fn resolve_signed(&self, base: f32) -> Option<f32> {
        let value = self.resolve_wide(base)?;
        (!value.is_nan()).then(|| saturating_f32(value))
    }

    /// Resolves signed lengths in f64 so enclosing math can combine them before narrowing.
    pub(super) fn resolve_wide(&self, base: f32) -> Option<f64> {
        let value = match self {
            Self::Auto | Self::MinContent | Self::MaxContent | Self::FitContent => return None,
            Self::Px(value) => f64::from(*value),
            Self::Percent(percent) if base.is_finite() => {
                f64::from(base) * f64::from(*percent) / 100.0
            }
            Self::Affine { pixels, fraction } if base.is_finite() => {
                f64::from(*pixels) + f64::from(*fraction) * f64::from(base)
            }
            Self::Math(expression) => expression.resolve_wide(base)?,
            _ => return None,
        };
        Some(value)
    }

    /// Returns `true` if this dimension is [`Dimension::Auto`].
    #[must_use]
    pub const fn is_auto(&self) -> bool {
        matches!(self, Self::Auto)
    }

    /// Whether evaluation reads a percentage basis.
    pub(crate) fn uses_percentage(&self) -> bool {
        match self {
            Self::Percent(_) | Self::Affine { .. } => true,
            Self::Math(expression) => expression.uses_percentage(),
            _ => false,
        }
    }

    #[doc = include_str!("../../docs/api/model/dimension.doc/is_intrinsic.md")]
    #[must_use]
    pub const fn is_intrinsic(&self) -> bool {
        matches!(self, Self::MinContent | Self::MaxContent | Self::FitContent)
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

    /// Checks full-scale percentages, signed saturation, and preservation of subnormal lengths.
    #[test]
    fn large_percentages_resolve_to_finite_lengths() {
        assert_eq!(Dimension::Percent(100.0).resolve(f32::MAX), Some(f32::MAX));
        assert_eq!(Dimension::Percent(200.0).resolve(f32::MAX), Some(f32::MAX));
        assert_eq!(
            Dimension::Percent(-200.0).resolve_signed(f32::MAX),
            Some(f32::MIN)
        );
        assert_eq!(
            Dimension::Percent(100.0).resolve(f32::from_bits(1)),
            Some(f32::from_bits(1))
        );
    }

    /// Verifies that a finite length survives cancellation of oversized intermediate values.
    #[test]
    fn length_math_preserves_cancellation_before_narrowing() {
        use super::TrackLimit;
        use std::sync::Arc;

        let positive =
            TrackLimit::Scale(Box::new(TrackLimit::Length(Dimension::Px(f32::MAX))), 3.0);
        let negative =
            TrackLimit::Scale(Box::new(TrackLimit::Length(Dimension::Px(f32::MAX))), 2.0);
        let difference = Dimension::Math(Arc::new(TrackLimit::Difference(
            Box::new(positive),
            Box::new(negative),
        )));
        assert_eq!(difference.resolve(0.0), Some(f32::MAX));
    }

    #[test]
    fn auto_never_resolves() {
        assert_eq!(Dimension::Auto.resolve(100.0), None);
        assert!(Dimension::Auto.is_auto());
        assert!(!Dimension::Px(0.0).is_auto());
    }

    #[test]
    fn an_intrinsic_keyword_has_no_arithmetic_answer() {
        // Not resolvable against a base at any size: the answer lives in the
        // content. An algorithm must measure, or fall back to `auto`.
        for dim in [
            Dimension::MinContent,
            Dimension::MaxContent,
            Dimension::FitContent,
        ] {
            assert_eq!(dim.resolve(100.0), None);
            assert!(dim.is_intrinsic());
            assert!(
                !dim.is_auto(),
                "intrinsic is not `auto`, even though both resolve to None"
            );
        }
        assert!(!Dimension::Auto.is_intrinsic());
        assert!(!Dimension::Px(1.0).is_intrinsic());
        assert!(!Dimension::Percent(50.0).is_intrinsic());
    }

    #[test]
    fn a_percentage_against_an_indefinite_base_behaves_as_auto() {
        // CSS Sizing 3 §5.1. This is what keeps a `max-content` probe — which
        // offers an infinite inline axis — from producing an infinitely wide box.
        assert_eq!(Dimension::Percent(50.0).resolve(f32::INFINITY), None);
        assert_eq!(Dimension::Percent(50.0).resolve(f32::NAN), None);
        // An absolute length is unmoved: it never needed the base.
        assert_eq!(Dimension::Px(12.0).resolve(f32::INFINITY), Some(12.0));
        // And a definite base still resolves exactly as before.
        assert_eq!(Dimension::Percent(50.0).resolve(80.0), Some(40.0));
    }
}
