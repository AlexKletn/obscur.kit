use super::Easing;
use crate::value::MotionValue;

#[doc = include_str!("../../docs/api/track/keyframe_track.doc/Keyframe.md")]
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
