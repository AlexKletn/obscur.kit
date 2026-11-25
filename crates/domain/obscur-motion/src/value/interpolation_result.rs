//! The `InterpolationResult` outcome of an interpolation attempt (spec §4.17).

use super::MotionValue;

#[doc = include_str!("../../docs/api/value/interpolation_result.doc/InterpolationResult.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InterpolationResult {
    /// The endpoints blended successfully to this value.
    Interpolated(MotionValue),
    /// The endpoints held different variants and could not be blended.
    Incompatible {
        /// The `self`/start endpoint of the attempted blend.
        from: MotionValue,
        /// The `other`/end endpoint of the attempted blend.
        to: MotionValue,
    },
}

impl InterpolationResult {
    /// Wraps a successfully interpolated value.
    #[must_use]
    pub const fn interpolated(value: MotionValue) -> Self {
        Self::Interpolated(value)
    }

    /// Records a failed blend between two type-incompatible endpoints.
    #[must_use]
    pub const fn incompatible(from: MotionValue, to: MotionValue) -> Self {
        Self::Incompatible { from, to }
    }

    #[doc = include_str!("../../docs/api/value/interpolation_result.doc/between.md")]
    #[must_use]
    pub fn between(from: MotionValue, to: MotionValue, t: f32) -> InterpolationResult {
        match from.interpolate_checked(&to, t) {
            Some(value) => Self::Interpolated(value),
            None => Self::Incompatible { from, to },
        }
    }

    /// Returns the interpolated value, or `None` when the endpoints were
    /// incompatible.
    #[must_use]
    pub const fn value(&self) -> Option<MotionValue> {
        match self {
            Self::Interpolated(value) => Some(*value),
            Self::Incompatible { .. } => None,
        }
    }

    /// Returns whether the blend succeeded.
    #[must_use]
    pub const fn is_interpolated(&self) -> bool {
        matches!(self, Self::Interpolated(_))
    }

    /// Returns whether the endpoints were incompatible.
    #[must_use]
    pub const fn is_incompatible(&self) -> bool {
        matches!(self, Self::Incompatible { .. })
    }

    /// Returns the interpolated value, or `fallback` when the endpoints were
    /// incompatible.
    #[must_use]
    pub const fn unwrap_or(&self, fallback: MotionValue) -> MotionValue {
        match self {
            Self::Interpolated(value) => *value,
            Self::Incompatible { .. } => fallback,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_geometry::Size;

    #[test]
    fn between_interpolates_matching_variants() {
        let result =
            InterpolationResult::between(MotionValue::scalar(0.0), MotionValue::scalar(10.0), 0.5);
        assert_eq!(
            result,
            InterpolationResult::interpolated(MotionValue::scalar(5.0))
        );
        assert!(result.is_interpolated());
        assert!(!result.is_incompatible());
        assert_eq!(result.value(), Some(MotionValue::scalar(5.0)));
    }

    #[test]
    fn between_reports_incompatible_endpoints() {
        let from = MotionValue::scalar(0.0);
        let to = MotionValue::size(Size::new(4.0, 4.0));
        let result = InterpolationResult::between(from, to, 0.5);
        assert_eq!(result, InterpolationResult::incompatible(from, to));
        assert!(result.is_incompatible());
        assert_eq!(result.value(), None);
    }

    #[test]
    fn unwrap_or_uses_fallback_only_when_incompatible() {
        let ok =
            InterpolationResult::between(MotionValue::scalar(0.0), MotionValue::scalar(10.0), 1.0);
        let fallback = MotionValue::scalar(-1.0);
        assert_eq!(ok.unwrap_or(fallback), MotionValue::scalar(10.0));

        let bad = InterpolationResult::incompatible(
            MotionValue::scalar(0.0),
            MotionValue::size(Size::new(1.0, 1.0)),
        );
        assert_eq!(bad.unwrap_or(fallback), fallback);
    }
}
