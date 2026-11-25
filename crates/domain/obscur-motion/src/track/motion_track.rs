#![doc = include_str!("../../docs/api/track/motion_track.doc/_module.md")]

use crate::value::MotionValue;

#[doc = include_str!("../../docs/api/track/motion_track.doc/MotionTrack.md")]
pub trait MotionTrack {
    #[doc = include_str!("../../docs/api/track/motion_track.doc/sample.md")]
    #[must_use]
    fn sample(&self, t: f32) -> MotionValue;
}
