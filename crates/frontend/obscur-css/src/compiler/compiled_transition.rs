use obscur_motion::track::{Easing, TransitionTrack};
use obscur_motion::value::MotionValue;
use obscur_style::property::PropertyId;

#[doc = include_str!("../../docs/api/compiler/transition_compiler.doc/CompiledTransition.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CompiledTransition {
    /// The property this transition animates.
    pub property: PropertyId,
    /// The transition duration, in seconds.
    pub duration_secs: f32,
    /// The delay before the transition starts, in seconds.
    pub delay_secs: f32,
    /// The timing function.
    pub easing: Easing,
}

impl CompiledTransition {
    /// Binds runtime `from`/`to` endpoints into a runnable [`TransitionTrack`],
    /// carrying this template's easing.
    #[must_use]
    pub fn build_track(&self, from: MotionValue, to: MotionValue) -> TransitionTrack {
        TransitionTrack::new(from, to).with_easing(self.easing)
    }
}
