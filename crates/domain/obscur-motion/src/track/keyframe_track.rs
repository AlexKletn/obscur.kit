//! The [`KeyframeTrack`] ordered-keyframe track and its [`Keyframe`] (spec §4.17).

use crate::track::{Easing, MotionTrack};
use crate::value::MotionValue;

/// One stop in a [`KeyframeTrack`]: a value pinned at a normalized offset
/// (spec §4.17).
///
/// `offset` is the position within the track's `[0, 1]` progress range at which
/// `value` is reached. Following CSS `@keyframes` semantics, `easing` is the
/// timing function used to interpolate *from this keyframe to the next one*, not
/// into this one.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Keyframe {
    /// Normalized position of this stop within the track, in `[0, 1]`.
    offset: f32,
    /// The presentation value reached at `offset`.
    value: MotionValue,
    /// The easing applied on the segment leaving this keyframe toward the next.
    easing: Easing,
}

impl Keyframe {
    /// Creates a keyframe placing `value` at `offset`, with [`Easing::Linear`]
    /// on the outgoing segment.
    #[must_use]
    pub const fn new(offset: f32, value: MotionValue) -> Self {
        Self {
            offset,
            value,
            easing: Easing::Linear,
        }
    }

    /// Returns this keyframe with `easing` on its outgoing segment (fluent
    /// setter).
    #[must_use]
    pub const fn with_easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    /// Returns the normalized offset of this keyframe.
    #[must_use]
    pub const fn offset(&self) -> f32 {
        self.offset
    }

    /// Returns the value reached at this keyframe.
    #[must_use]
    pub const fn value(&self) -> MotionValue {
        self.value
    }

    /// Returns the easing applied on the outgoing segment.
    #[must_use]
    pub const fn easing(&self) -> Easing {
        self.easing
    }
}

/// An ordered list of [`Keyframe`]s sampled by normalized progress (spec §4.17).
///
/// This is the native compile target of CSS `@keyframes`: a set of value stops
/// at normalized offsets, with per-segment easing. The keyframes are kept sorted
/// by offset so sampling can walk them directly. Sampling clamps progress to the
/// first/last stop, then linearly locates the bracketing pair and blends the
/// lower value toward the upper one using the lower keyframe's easing. A blend
/// between mismatched [`MotionValue`] variants is impossible and falls back to
/// the upper keyframe's value.
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
            a.offset
                .partial_cmp(&b.offset)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        Self { keyframes }
    }

    /// Creates a two-stop linear track running `start` (offset `0`) to `end`
    /// (offset `1`) — the minimal track between two endpoint values (spec §4.17).
    ///
    /// Used to build interruption redirects, e.g. a
    /// [`Reverse`](crate::transition::TransitionInterruption::Reverse) that plays
    /// the element's current value back toward the interrupted transition's
    /// origin.
    #[must_use]
    pub fn from_endpoints(start: MotionValue, end: MotionValue) -> Self {
        Self {
            keyframes: vec![Keyframe::new(0.0, start), Keyframe::new(1.0, end)],
        }
    }

    /// Returns a copy of this track whose first (lowest-offset) keyframe holds
    /// `start`, leaving every other keyframe — offsets, easings and later values
    /// — untouched (spec §4.17 "interruption / blend").
    ///
    /// This is the primitive a
    /// [`Blend`](crate::transition::TransitionInterruption::Blend) interruption
    /// uses: an incoming transition is re-based so it *begins* at the element's
    /// current mid-animation value instead of the track's authored start,
    /// producing a seamless redirect with no visible jump. An empty track is
    /// returned unchanged.
    #[must_use]
    pub fn with_start_value(&self, start: MotionValue) -> Self {
        let mut keyframes = self.keyframes.clone();
        if let Some(first) = keyframes.first_mut() {
            *first = Keyframe::new(first.offset, start).with_easing(first.easing);
        }
        Self { keyframes }
    }

    /// Inserts `keyframe`, keeping the list sorted by offset.
    ///
    /// A keyframe with an offset equal to existing ones is inserted after them,
    /// preserving insertion order among equal offsets.
    pub fn push(&mut self, keyframe: Keyframe) {
        let index = self
            .keyframes
            .partition_point(|k| k.offset <= keyframe.offset);
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
    /// Samples the track at normalized progress `t` (spec §4.17).
    ///
    /// `t` is clamped to `[0, 1]`. An empty track is a degenerate case and
    /// yields `MotionValue::scalar(0.0)`. Progress at or before the first
    /// keyframe yields the first value; at or after the last yields the last.
    /// Between two keyframes the local progress is eased by the lower
    /// keyframe's easing and used to blend the two values, falling back to the
    /// upper value on a variant mismatch.
    fn sample(&self, t: f32) -> MotionValue {
        let t = t.clamp(0.0, 1.0);
        let frames = &self.keyframes;
        let Some(first) = frames.first() else {
            // Degenerate: a track with no keyframes has no value to present.
            return MotionValue::scalar(0.0);
        };
        // Safe: non-empty, so `last` exists.
        let last = frames[frames.len() - 1];
        if t <= first.offset {
            return first.value;
        }
        if t >= last.offset {
            return last.value;
        }
        // Find the last keyframe whose offset is <= t; its successor brackets t.
        let mut i = 0;
        while i + 1 < frames.len() && frames[i + 1].offset <= t {
            i += 1;
        }
        let lower = frames[i];
        let upper = frames[i + 1];
        let span = upper.offset - lower.offset;
        let local = if span == 0.0 {
            0.0
        } else {
            (t - lower.offset) / span
        };
        let eased = lower.easing.apply(local);
        lower
            .value
            .interpolate_checked(&upper.value, eased)
            .unwrap_or(upper.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        let track = KeyframeTrack::from_endpoints(MotionValue::scalar(20.0), MotionValue::scalar(80.0));
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
