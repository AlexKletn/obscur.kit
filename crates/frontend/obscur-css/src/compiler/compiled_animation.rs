use obscur_motion::timeline::{FillMode, PlaybackDirection, RepeatCount};
use obscur_motion::track::KeyframeTrack;
use obscur_style::matching::RuleCondition;
use obscur_style::property::PropertyId;

#[doc = include_str!("../../docs/api/compiler/motion_rule.doc/CompiledAnimation.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct CompiledAnimation {
    /// Index of the animation reference within the originating style rule.
    pub reference_index: usize,
    /// Condition of the referenced keyframe definition, if any.
    pub condition: Option<RuleCondition>,
    /// The per-property keyframe tracks (cloned from the referenced
    /// `@keyframes`), in first-seen property order.
    pub tracks: Vec<(PropertyId, KeyframeTrack)>,
    /// The duration to play the tracks over, in seconds.
    pub duration_secs: f32,
    /// `animation-delay`, in seconds (negative seeks into the playback).
    pub delay_secs: f32,
    /// `animation-iteration-count`.
    pub repeat: RepeatCount,
    /// The direction iteration 1 plays in.
    pub direction: PlaybackDirection,
    /// Whether iterations alternate direction.
    pub alternates: bool,
    /// `animation-fill-mode`.
    pub fill: FillMode,
}
