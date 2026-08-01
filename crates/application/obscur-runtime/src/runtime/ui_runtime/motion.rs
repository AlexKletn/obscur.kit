#![doc = include_str!("motion.doc/_module.md")]

use obscur_identity::NodeId;
use obscur_motion::timeline::TimelineId;
use obscur_motion::track::KeyframeTrack;
use obscur_motion::transition::TransitionInterruption;
use obscur_style::property::PropertyId;

use crate::frame::FrameReason;
use crate::orchestration::{AnimatedKind, CssMotionRules, MotionPlayback};

use super::UiRuntime;

impl UiRuntime {
    #[doc = include_str!("motion.doc/set_css_motion.md")]
    pub fn set_css_motion(&mut self, rules: CssMotionRules) {
        self.motion.set_css_rules(rules);
    }

    #[doc = include_str!("motion.doc/animate_with.md")]
    #[allow(clippy::too_many_arguments)]
    pub fn animate_with(
        &mut self,
        node: NodeId,
        property: PropertyId,
        track: KeyframeTrack,
        duration: f32,
        kind: AnimatedKind,
        interruption: TransitionInterruption,
        playback: MotionPlayback,
    ) -> TimelineId {
        let id = self.motion.animate_with(
            node,
            property,
            track,
            duration,
            kind,
            interruption,
            playback,
        );
        self.request_frame(FrameReason::Animation);
        id
    }

    #[doc = include_str!("motion.doc/animate.md")]
    pub fn animate(
        &mut self,
        node: NodeId,
        property: PropertyId,
        track: KeyframeTrack,
        duration: f32,
        kind: AnimatedKind,
        interruption: TransitionInterruption,
    ) -> TimelineId {
        self.animate_with(
            node,
            property,
            track,
            duration,
            kind,
            interruption,
            MotionPlayback::default(),
        )
    }

    #[doc = include_str!("motion.doc/advance_motion.md")]
    pub fn advance_motion(&mut self, delta: f32) {
        // A flung list is motion too, and this is the runtime's one "time passed"
        // signal — the tick a host already drives. Giving the coast its own entry
        // point would mean every host had to learn to call a second clock, and
        // the one that forgot would have inertia only on the frames something
        // else happened to be animating.
        self.advance_flings(delta);
        self.advance_holds(delta);
        // Remember where each timeline that finished on this tick settled: the
        // scheduler prunes it, and the overlay still has to apply the value the
        // animation ended on, which for a reversed final iteration is progress
        // `0` and not `1` (css-animations-1 §3.9 `forwards`).
        if self.motion.advance(delta) {
            self.request_frame(FrameReason::Animation);
        }
    }

    #[doc = include_str!("motion.doc/active_animations.md")]
    #[must_use]
    pub fn active_animations(&self) -> usize {
        self.motion.active_animations()
    }

    #[doc = include_str!("motion.doc/active_animation_count.md")]
    #[must_use]
    pub fn active_animation_count(&self) -> usize {
        self.motion.active_count()
    }
}
