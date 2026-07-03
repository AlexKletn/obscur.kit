//! Track module.
mod keyframe_track;
pub use keyframe_track::{Keyframe, KeyframeTrack};
mod motion_track;
pub use motion_track::{Easing, MotionTrack};
mod spring_track;
pub use spring_track::SpringTrack;
mod transition_track;
pub use transition_track::TransitionTrack;
