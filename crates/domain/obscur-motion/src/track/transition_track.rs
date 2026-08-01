#![doc = include_str!("transition_track.doc/_module.md")]

use crate::track::{Easing, MotionTrack};
use crate::value::MotionValue;

#[doc = include_str!("transition_track.doc/TransitionTrack.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransitionTrack {
    #[doc = include_str!("transition_track.doc/from.md")]
    from: MotionValue,
    #[doc = include_str!("transition_track.doc/to.md")]
    to: MotionValue,
    #[doc = include_str!("transition_track.doc/easing.md")]
    easing: Easing,
}

impl TransitionTrack {
    #[doc = include_str!("transition_track.doc/new.md")]
    #[must_use]
    pub const fn new(from: MotionValue, to: MotionValue) -> Self {
        Self {
            from,
            to,
            easing: Easing::Linear,
        }
    }

    #[doc = include_str!("transition_track.doc/with_easing.md")]
    #[must_use]
    pub const fn with_easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    #[doc = include_str!("transition_track.doc/from_2.md")]
    #[must_use]
    pub const fn from(&self) -> MotionValue {
        self.from
    }

    #[doc = include_str!("transition_track.doc/to_2.md")]
    #[must_use]
    pub const fn to(&self) -> MotionValue {
        self.to
    }

    #[doc = include_str!("transition_track.doc/easing_2.md")]
    #[must_use]
    pub const fn easing(&self) -> Easing {
        self.easing
    }
}

impl MotionTrack for TransitionTrack {
    #[doc = include_str!("transition_track.doc/sample.md")]
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
        let track: Box<dyn MotionTrack> = Box::new(TransitionTrack::new(
            MotionValue::scalar(0.0),
            MotionValue::scalar(1.0),
        ));
        assert_eq!(track.sample(1.0), MotionValue::scalar(1.0));
    }
}
