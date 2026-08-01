#![doc = include_str!("motion_rule.doc/_module.md")]

use obscur_motion::timeline::{FillMode, PlaybackDirection, RepeatCount};
use obscur_motion::track::KeyframeTrack;
use obscur_style::property::PropertyId;
use obscur_style::selector::SelectorList;

use super::transition_compiler::CompiledTransition;

#[doc = include_str!("motion_rule.doc/CompiledAnimation.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct CompiledAnimation {
    #[doc = include_str!("motion_rule.doc/tracks.md")]
    pub tracks: Vec<(PropertyId, KeyframeTrack)>,
    #[doc = include_str!("motion_rule.doc/duration_secs.md")]
    pub duration_secs: f32,
    #[doc = include_str!("motion_rule.doc/delay_secs.md")]
    pub delay_secs: f32,
    #[doc = include_str!("motion_rule.doc/repeat.md")]
    pub repeat: RepeatCount,
    #[doc = include_str!("motion_rule.doc/direction.md")]
    pub direction: PlaybackDirection,
    #[doc = include_str!("motion_rule.doc/alternates.md")]
    pub alternates: bool,
    #[doc = include_str!("motion_rule.doc/fill.md")]
    pub fill: FillMode,
}

#[doc = include_str!("motion_rule.doc/CompiledMotionRule.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct CompiledMotionRule {
    #[doc = include_str!("motion_rule.doc/selectors.md")]
    pub selectors: SelectorList,
    #[doc = include_str!("motion_rule.doc/transitions.md")]
    pub transitions: Vec<CompiledTransition>,
    #[doc = include_str!("motion_rule.doc/animations.md")]
    pub animations: Vec<CompiledAnimation>,
}
