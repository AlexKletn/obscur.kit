#![doc = include_str!("motion_coordinator.doc/_module.md")]

use obscur_motion::track::{Keyframe, KeyframeTrack};
use obscur_motion::value::MotionValue;

#[doc = include_str!("motion_coordinator.doc/MotionCoordinator.md")]
#[derive(Debug, Clone, Copy, Default)]
pub struct MotionCoordinator;

impl MotionCoordinator {
    #[doc = include_str!("motion_coordinator.doc/sample_keyframes.md")]
    #[must_use]
    pub fn sample_keyframes(track: &KeyframeTrack, t: f32) -> Option<MotionValue> {
        let frames = track.keyframes();
        let first = frames.first()?;
        // Safe: `first` proved the slice is non-empty, so `last` exists.
        let last = &frames[frames.len() - 1];
        let t = t.clamp(0.0, 1.0);

        // Hold the endpoints outside the keyframe range.
        if t <= first.offset() {
            return Some(first.value());
        }
        if t >= last.offset() {
            return Some(last.value());
        }

        // Locate the last keyframe whose offset is <= t; its successor brackets t.
        let mut i = 0;
        while i + 1 < frames.len() && frames[i + 1].offset() <= t {
            i += 1;
        }
        let lower: &Keyframe = &frames[i];
        let upper: &Keyframe = &frames[i + 1];
        let span = upper.offset() - lower.offset();
        let local = if span == 0.0 {
            0.0
        } else {
            (t - lower.offset()) / span
        };
        let eased = lower.easing().apply(local);
        Some(
            lower
                .value()
                .interpolate_checked(&upper.value(), eased)
                .unwrap_or_else(|| upper.value()),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_motion::track::Easing;

    fn track(pairs: &[(f32, f32)]) -> KeyframeTrack {
        KeyframeTrack::from_keyframes(
            pairs
                .iter()
                .map(|&(offset, value)| Keyframe::new(offset, MotionValue::scalar(value)))
                .collect(),
        )
    }

    #[test]
    fn empty_track_samples_to_none() {
        assert_eq!(
            MotionCoordinator::sample_keyframes(&KeyframeTrack::new(), 0.5),
            None
        );
    }

    #[test]
    fn two_keyframes_linear_endpoints_and_midpoint() {
        let t = track(&[(0.0, 0.0), (1.0, 10.0)]);
        assert_eq!(
            MotionCoordinator::sample_keyframes(&t, 0.0),
            Some(MotionValue::scalar(0.0))
        );
        assert_eq!(
            MotionCoordinator::sample_keyframes(&t, 0.5),
            Some(MotionValue::scalar(5.0))
        );
        assert_eq!(
            MotionCoordinator::sample_keyframes(&t, 1.0),
            Some(MotionValue::scalar(10.0))
        );
    }

    #[test]
    fn out_of_range_progress_clamps_to_endpoints() {
        let t = track(&[(0.0, 3.0), (1.0, 9.0)]);
        assert_eq!(
            MotionCoordinator::sample_keyframes(&t, -2.0),
            Some(MotionValue::scalar(3.0))
        );
        assert_eq!(
            MotionCoordinator::sample_keyframes(&t, 5.0),
            Some(MotionValue::scalar(9.0))
        );
    }

    #[test]
    fn three_keyframes_pick_the_right_segment() {
        let t = track(&[(0.0, 0.0), (0.5, 10.0), (1.0, 0.0)]);
        assert_eq!(
            MotionCoordinator::sample_keyframes(&t, 0.25),
            Some(MotionValue::scalar(5.0))
        );
        assert_eq!(
            MotionCoordinator::sample_keyframes(&t, 0.5),
            Some(MotionValue::scalar(10.0))
        );
        assert_eq!(
            MotionCoordinator::sample_keyframes(&t, 0.75),
            Some(MotionValue::scalar(5.0))
        );
    }

    #[test]
    fn eased_segment_uses_lower_keyframe_easing() {
        let t = KeyframeTrack::from_keyframes(vec![
            Keyframe::new(0.0, MotionValue::scalar(0.0)).with_easing(Easing::EaseIn),
            Keyframe::new(1.0, MotionValue::scalar(8.0)),
        ]);
        // Local t 0.5 eased-in -> 0.25 -> blends 0..8 to 2.0.
        assert_eq!(
            MotionCoordinator::sample_keyframes(&t, 0.5),
            Some(MotionValue::scalar(2.0))
        );
    }
}
