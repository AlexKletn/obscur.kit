#![doc = include_str!("../../../docs/api/runtime/ui_runtime/motion.doc/_module.md")]

use obscur_identity::NodeId;
use obscur_motion::timeline::TimelineId;
use obscur_motion::track::KeyframeTrack;
use obscur_motion::transition::TransitionInterruption;
use obscur_style::property::PropertyId;

use crate::orchestration::{AnimatedKind, CssMotionRules, MotionPlayback};

use super::UiRuntime;

impl UiRuntime {
    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/motion.doc/set_css_motion.md")]
    pub fn set_css_motion(&mut self, rules: CssMotionRules) {
        self.motion.set_css_rules(rules);
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/motion.doc/animate_with.md")]
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
        self.request_timeline_frame();
        id
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/motion.doc/animate.md")]
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

    /// Pauses a live timeline and presents the requested normalized progress. Further samples come from the caller until playback resumes. Returns false for a non-finite value or unknown timeline.
    pub fn set_animation_progress(&mut self, id: TimelineId, progress: f32) -> bool {
        if !progress.is_finite() || !self.motion.scrub(id, progress) {
            return false;
        }
        self.request_timeline_frame();
        true
    }

    /// Resumes time-based playback of a paused timeline at its current progress.
    pub fn resume_animation(&mut self, id: TimelineId) -> bool {
        if !self.motion.resume(id) {
            return false;
        }
        self.request_timeline_frame();
        true
    }

    /// Stop an active animation or release its retained final value. The animated
    /// property returns to its current authored value on the next frame. Returns
    /// whether the timeline was still bound to a property.
    pub fn cancel_animation(&mut self, timeline: TimelineId) -> bool {
        if !self.motion.cancel_animation(timeline) {
            return false;
        }
        self.animation_progress.retain(|(id, _)| *id != timeline);
        self.request_frame(crate::frame::FrameReason::UiCommit);
        true
    }

    /// Report a forward, one-shot timeline's progress as the runtime advances it.
    /// The terminal sample requests a view update so the host can remove its animation layers.
    pub fn observe_animation_progress(
        &mut self,
        timeline: TimelineId,
        observer: impl FnMut(f32) + 'static,
    ) {
        self.animation_progress.push((timeline, Box::new(observer)));
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/motion.doc/advance_motion.md")]
    pub fn advance_motion(&mut self, delta: f32) {
        if !delta.is_finite() {
            return;
        }
        if delta > 0.0 {
            self.timeline_waiting_for_tick = false;
        }
        // A flung list is motion too, and this is the runtime's one "time passed"
        // signal — the tick a host already drives. Giving the coast its own entry
        // point would mean every host had to learn to call a second clock, and
        // the one that forgot would have inertia only on the frames something
        // else happened to be animating.
        self.advance_flings(delta);
        self.advance_smooth_scrolls(delta);
        self.advance_drag_autoscroll(delta);
        self.advance_selection_autoscroll(delta);
        self.advance_scroll_snaps(delta);
        self.scroll.sample_velocities(delta);
        self.advance_holds(delta);
        // §5.3: a canvas that draws a moving thing reads this clock. It rides
        // the same signal for the same reason the flings and the holds do — a
        // host already drives one "time passed" tick, and a second clock would
        // be a second thing to forget.
        if delta > 0.0
            && let Ok(elapsed) = std::time::Duration::try_from_secs_f32(delta)
        {
            self.logical_time = self.logical_time.saturating_add(elapsed);
        }
        // Remember where each timeline that finished on this tick settled: the
        // scheduler prunes it, and the overlay still has to apply the value the
        // animation ended on, which for a reversed final iteration is progress
        // `0` and not `1` (css-animations-1 §3.9 `forwards`).
        if self.motion.advance(delta) && delta > 0.0 {
            self.request_timeline_frame();
        }
        let mut finished = false;
        self.animation_progress.retain_mut(|(timeline, observer)| {
            let progress = self.motion.timeline_progress(*timeline);
            observer(progress);
            finished |= progress >= 1.0;
            progress < 1.0
        });
        if finished {
            self.request_frame(crate::frame::FrameReason::UiCommit);
        }
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/motion.doc/active_animations.md")]
    #[must_use]
    pub fn active_animations(&self) -> usize {
        self.motion.active_animations()
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/motion.doc/active_animation_count.md")]
    #[must_use]
    pub fn active_animation_count(&self) -> usize {
        self.motion.active_count()
    }
}
