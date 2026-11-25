//! The [`KeyframeTrack`] ordered-keyframe track and its [`Keyframe`] (spec §4.17).

use crate::track::MotionTrack;
use crate::value::MotionValue;

use super::Keyframe;

#[doc = include_str!("../../docs/api/track/keyframe_track.doc/KeyframeTrack.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct KeyframeTrack {
    /// The keyframes, kept sorted ascending by [`Keyframe::offset`].
    keyframes: Vec<Keyframe>,
}

impl KeyframeTrack {
    /// Creates an empty track with no keyframes.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            keyframes: Vec::new(),
        }
    }

    /// Creates a track from `keyframes`, sorting them (stably) by offset.
    #[must_use]
    pub fn from_keyframes(mut keyframes: Vec<Keyframe>) -> Self {
        keyframes.sort_by(|a, b| {
            a.offset()
                .partial_cmp(&b.offset())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        Self { keyframes }
    }

    #[doc = include_str!("../../docs/api/track/keyframe_track.doc/from_endpoints.md")]
    #[must_use]
    pub fn from_endpoints(start: MotionValue, end: MotionValue) -> Self {
        Self {
            keyframes: vec![Keyframe::new(0.0, start), Keyframe::new(1.0, end)],
        }
    }

    #[doc = include_str!("../../docs/api/track/keyframe_track.doc/with_start_value.md")]
    #[must_use]
    pub fn with_start_value(&self, start: MotionValue) -> Self {
        let mut keyframes = self.keyframes.clone();
        if let Some(first) = keyframes.first_mut() {
            *first = Keyframe::new(first.offset(), start).with_easing(first.easing());
        }
        Self { keyframes }
    }

    #[doc = include_str!("../../docs/api/track/keyframe_track.doc/push.md")]
    pub fn push(&mut self, keyframe: Keyframe) {
        let index = self
            .keyframes
            .partition_point(|k| k.offset() <= keyframe.offset());
        self.keyframes.insert(index, keyframe);
    }

    /// Returns the keyframes in ascending offset order.
    #[must_use]
    pub fn keyframes(&self) -> &[Keyframe] {
        &self.keyframes
    }

    /// Returns the number of keyframes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.keyframes.len()
    }

    /// Returns whether the track holds no keyframes.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.keyframes.is_empty()
    }
}

impl Default for KeyframeTrack {
    fn default() -> Self {
        Self::new()
    }
}

impl MotionTrack for KeyframeTrack {
    #[doc = include_str!("../../docs/api/track/keyframe_track.doc/sample.md")]
    fn sample(&self, t: f32) -> MotionValue {
        let t = t.clamp(0.0, 1.0);
        let frames = &self.keyframes;
        let Some(first) = frames.first() else {
            return MotionValue::scalar(0.0);
        };
        // Safe: non-empty, so `last` exists.
        let last = frames[frames.len() - 1];
        if t <= first.offset() {
            return first.value();
        }
        if t >= last.offset() {
            return last.value();
        }
        let lower_index = last_keyframe_at_or_before(frames, t);
        let lower = frames[lower_index];
        let upper = frames[lower_index + 1];
        let span = upper.offset() - lower.offset();
        let local = if span == 0.0 {
            0.0
        } else {
            (t - lower.offset()) / span
        };
        let eased = lower.easing().apply(local);
        lower
            .value()
            .interpolate_checked(&upper.value(), eased)
            .unwrap_or(upper.value())
    }
}

fn last_keyframe_at_or_before(frames: &[Keyframe], t: f32) -> usize {
    let mut index = 0;
    while index + 1 < frames.len() && frames[index + 1].offset() <= t {
        index += 1;
    }
    index
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::track::Easing;
    use obscur_geometry::Point;

    #[test]
    fn empty_track_is_degenerate_zero() {
        let track = KeyframeTrack::new();
        assert!(track.is_empty());
        assert_eq!(track.len(), 0);
        assert_eq!(track.sample(0.5), MotionValue::scalar(0.0));
    }

    #[test]
    fn single_keyframe_holds_its_value() {
        let mut track = KeyframeTrack::new();
        track.push(Keyframe::new(0.5, MotionValue::scalar(7.0)));
        assert_eq!(track.len(), 1);
        assert_eq!(track.sample(0.0), MotionValue::scalar(7.0));
        assert_eq!(track.sample(0.5), MotionValue::scalar(7.0));
        assert_eq!(track.sample(1.0), MotionValue::scalar(7.0));
    }

    #[test]
    fn three_keyframe_bracketing_sample() {
        let track = KeyframeTrack::from_keyframes(vec![
            Keyframe::new(0.0, MotionValue::scalar(0.0)),
            Keyframe::new(0.5, MotionValue::scalar(10.0)),
            Keyframe::new(1.0, MotionValue::scalar(0.0)),
        ]);
        // Exact offsets return the stored values.
        assert_eq!(track.sample(0.0), MotionValue::scalar(0.0));
        assert_eq!(track.sample(0.5), MotionValue::scalar(10.0));
        assert_eq!(track.sample(1.0), MotionValue::scalar(0.0));
        // Midpoint of the first segment (linear): halfway from 0 to 10.
        assert_eq!(track.sample(0.25), MotionValue::scalar(5.0));
        // Midpoint of the second segment: halfway from 10 back to 0.
        assert_eq!(track.sample(0.75), MotionValue::scalar(5.0));
    }

    #[test]
    fn out_of_range_progress_clamps_to_ends() {
        let track = KeyframeTrack::from_keyframes(vec![
            Keyframe::new(0.25, MotionValue::scalar(3.0)),
            Keyframe::new(0.75, MotionValue::scalar(9.0)),
        ]);
        assert_eq!(track.sample(-2.0), MotionValue::scalar(3.0));
        assert_eq!(track.sample(0.1), MotionValue::scalar(3.0));
        assert_eq!(track.sample(0.9), MotionValue::scalar(9.0));
        assert_eq!(track.sample(2.0), MotionValue::scalar(9.0));
    }

    #[test]
    fn from_keyframes_sorts_by_offset() {
        let track = KeyframeTrack::from_keyframes(vec![
            Keyframe::new(1.0, MotionValue::scalar(2.0)),
            Keyframe::new(0.0, MotionValue::scalar(0.0)),
            Keyframe::new(0.5, MotionValue::scalar(1.0)),
        ]);
        let offsets: Vec<f32> = track.keyframes().iter().map(Keyframe::offset).collect();
        assert_eq!(offsets, vec![0.0, 0.5, 1.0]);
    }

    #[test]
    fn eased_segment_uses_lower_keyframe_easing() {
        // Lower keyframe eases-in; the segment midpoint (local t = 0.5 -> 0.25)
        // blends 0..8 to 2.0.
        let track = KeyframeTrack::from_keyframes(vec![
            Keyframe::new(0.0, MotionValue::scalar(0.0)).with_easing(Easing::EaseIn),
            Keyframe::new(1.0, MotionValue::scalar(8.0)),
        ]);
        assert_eq!(track.sample(0.5), MotionValue::scalar(2.0));
    }

    #[test]
    fn variant_mismatch_falls_back_to_upper() {
        let upper = MotionValue::point(Point::new(1.0, 2.0));
        let track = KeyframeTrack::from_keyframes(vec![
            Keyframe::new(0.0, MotionValue::scalar(0.0)),
            Keyframe::new(1.0, upper),
        ]);
        assert_eq!(track.sample(0.5), upper);
    }

    #[test]
    fn is_object_safe() {
        let track: Box<dyn MotionTrack> = Box::new(KeyframeTrack::new());
        assert_eq!(track.sample(0.5), MotionValue::scalar(0.0));
    }

    #[test]
    fn from_endpoints_is_a_linear_two_stop_track() {
        let track =
            KeyframeTrack::from_endpoints(MotionValue::scalar(20.0), MotionValue::scalar(80.0));
        assert_eq!(track.len(), 2);
        assert_eq!(track.sample(0.0), MotionValue::scalar(20.0));
        assert_eq!(track.sample(0.5), MotionValue::scalar(50.0));
        assert_eq!(track.sample(1.0), MotionValue::scalar(80.0));
    }

    #[test]
    fn with_start_value_rebases_the_first_keyframe_only() {
        let base = KeyframeTrack::from_keyframes(vec![
            Keyframe::new(0.0, MotionValue::scalar(0.0)).with_easing(Easing::EaseIn),
            Keyframe::new(1.0, MotionValue::scalar(100.0)),
        ]);
        let rebased = base.with_start_value(MotionValue::scalar(40.0));
        // The start value moved to 40; the target (100) and easing are preserved.
        assert_eq!(rebased.sample(0.0), MotionValue::scalar(40.0));
        assert_eq!(rebased.sample(1.0), MotionValue::scalar(100.0));
        assert_eq!(rebased.keyframes()[0].easing(), Easing::EaseIn);
        // The original is untouched.
        assert_eq!(base.sample(0.0), MotionValue::scalar(0.0));
    }

    #[test]
    fn with_start_value_on_empty_track_is_empty() {
        let rebased = KeyframeTrack::new().with_start_value(MotionValue::scalar(5.0));
        assert!(rebased.is_empty());
    }
}
