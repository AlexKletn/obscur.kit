//! The [`TransitionTrack`] single from→to eased track (spec §4.17).

use crate::track::{Easing, MotionTrack};
use crate::value::MotionValue;

/// A single `from` → `to` interpolation shaped by an easing curve (spec §4.17).
///
/// This is the native compile target of a CSS `transition`: one start value, one
/// end value, and a timing function. Sampling blends `from` toward `to` by the
/// eased progress. Because the two endpoints must share a [`MotionValue`]
/// variant to blend, a *type mismatch* (e.g. a scalar transitioning to a point)
/// cannot interpolate; sampling then falls back to the `to` endpoint rather than
/// panicking or fabricating a blend.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransitionTrack {
    /// The value at `t == 0`.
    from: MotionValue,
    /// The value at `t == 1`.
    to: MotionValue,
    /// The easing curve applied to progress before interpolating.
    easing: Easing,
}

impl TransitionTrack {
    /// Creates a transition from `from` to `to` with [`Easing::Linear`].
    #[must_use]
    pub const fn new(from: MotionValue, to: MotionValue) -> Self {
        Self {
            from,
            to,
            easing: Easing::Linear,
        }
    }

    /// Returns this transition with `easing` applied (fluent setter).
    #[must_use]
    pub const fn with_easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    /// Returns the start (`t == 0`) value.
    #[must_use]
    pub const fn from(&self) -> MotionValue {
        self.from
    }

    /// Returns the end (`t == 1`) value.
    #[must_use]
    pub const fn to(&self) -> MotionValue {
        self.to
    }

    /// Returns the easing curve applied before interpolation.
    #[must_use]
    pub const fn easing(&self) -> Easing {
        self.easing
    }
}

impl MotionTrack for TransitionTrack {
    /// Samples the eased blend from `from` toward `to` at progress `t`.
    ///
    /// The easing curve clamps `t` to `[0, 1]`, so `t <= 0` yields `from` and
    /// `t >= 1` yields `to`. On a variant mismatch between the endpoints the
    /// blend is undefined and this falls back to `to`.
    fn sample(&self, t: f32) -> MotionValue {
        self.from
            .interpolate_checked(&self.to, self.easing.apply(t))
            .unwrap_or(self.to)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_geometry::Point;

    #[test]
    fn new_defaults_to_linear() {
        let track = TransitionTrack::new(MotionValue::scalar(0.0), MotionValue::scalar(1.0));
        assert_eq!(track.easing(), Easing::Linear);
        assert_eq!(track.from(), MotionValue::scalar(0.0));
        assert_eq!(track.to(), MotionValue::scalar(1.0));
    }

    #[test]
    fn samples_endpoints() {
        let track = TransitionTrack::new(MotionValue::scalar(2.0), MotionValue::scalar(6.0));
        assert_eq!(track.sample(0.0), MotionValue::scalar(2.0));
        assert_eq!(track.sample(1.0), MotionValue::scalar(6.0));
    }

    #[test]
    fn clamps_out_of_range_progress_to_endpoints() {
        let track = TransitionTrack::new(MotionValue::scalar(2.0), MotionValue::scalar(6.0));
        assert_eq!(track.sample(-1.0), MotionValue::scalar(2.0));
        assert_eq!(track.sample(5.0), MotionValue::scalar(6.0));
    }

    #[test]
    fn eased_midpoint() {
        // ease-in at t = 0.5 is 0.25, so 0..8 samples to 2.0.
        let track = TransitionTrack::new(MotionValue::scalar(0.0), MotionValue::scalar(8.0))
            .with_easing(Easing::EaseIn);
        assert_eq!(track.sample(0.5), MotionValue::scalar(2.0));
    }

    #[test]
    fn type_mismatch_falls_back_to_to() {
        let to = MotionValue::point(Point::new(3.0, 4.0));
        let track = TransitionTrack::new(MotionValue::scalar(0.0), to);
        // Endpoints are different variants: the blend is impossible, so `to` wins.
        assert_eq!(track.sample(0.5), to);
    }

    #[test]
    fn is_object_safe() {
        let track: Box<dyn MotionTrack> =
            Box::new(TransitionTrack::new(MotionValue::scalar(0.0), MotionValue::scalar(1.0)));
        assert_eq!(track.sample(1.0), MotionValue::scalar(1.0));
    }
}
