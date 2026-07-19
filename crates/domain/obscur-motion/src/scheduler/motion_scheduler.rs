//! The [`MotionScheduler`] domain motion scheduler (spec §4.17 "motion engine",
//! event-driven scheduling / frame demand).

use crate::scheduler::ActiveMotionSet;
use crate::timeline::{Timeline, TimelineId};

/// Advances active motions and reports frame demand (spec §4.17).
///
/// The `MotionScheduler` is the domain-level scheduler of the motion engine: it
/// owns the [`ActiveMotionSet`], advances every active timeline by an explicit
/// `delta` each frame, drops finished ones, and reports whether another frame is
/// needed. It is **pure and deterministic** — it reads no wall-clock and reacts
/// only to explicit calls — and stays smooth under event-driven scheduling and
/// **skipped frames**, since a skipped frame is simply a larger `delta` absorbed
/// by [`Timeline::advance`]'s multi-iteration carry.
///
/// While any timeline is still running, [`needs_frame`](Self::needs_frame)
/// returns `true`; the runtime maps that signal to
/// `FrameReason::Animation` so the frame scheduler keeps requesting animation
/// frames until the last motion finishes (spec §4.17 "active animations keep
/// requesting frames"). This is the domain scheduler; the runtime
/// `FrameScheduler` merely consumes this demand signal.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MotionScheduler {
    /// The set of currently-tracked timelines.
    active: ActiveMotionSet,
}

impl MotionScheduler {
    /// Creates an empty scheduler with no active motions.
    #[must_use]
    pub fn new() -> Self {
        Self {
            active: ActiveMotionSet::new(),
        }
    }

    /// Registers `timeline` into the active set, keyed by its
    /// [`id`](Timeline::id).
    ///
    /// The timeline is inserted **as-is**: its own lifecycle state governs
    /// advancement, so the caller is responsible for having
    /// [`start`](Timeline::start)ed it. Registering a timeline whose id already
    /// exists replaces the previous one.
    pub fn register(&mut self, timeline: Timeline) {
        self.active.insert(timeline);
    }

    /// Advances every active timeline by `delta` logical seconds, then drops the
    /// finished ones and **returns** them.
    ///
    /// Each timeline is advanced via [`Timeline::advance`] in deterministic order
    /// (ascending by [`TimelineId`], from the backing `BTreeMap`), after which any
    /// that finished is pulled out of the active set. A large `delta` — e.g.
    /// spanning several skipped frames — is handled by the timeline's own
    /// multi-iteration overshoot, so scheduling stays smooth regardless of frame
    /// cadence.
    ///
    /// The returned timelines are the ones that finished **on this call**, in
    /// ascending id order. A caller that has to keep applying a finished
    /// animation's last value needs the position it settled at, and pruning would
    /// otherwise destroy it — a reversed final iteration settles at progress `0`,
    /// not `1`, so "it finished, assume the end" is wrong.
    pub fn advance(&mut self, delta: f32) -> Vec<Timeline> {
        // Collect ids first (deterministic ascending order) so each timeline can
        // be advanced through the set's `get_mut`; the borrow of `iter` ends
        // before the mutation begins.
        let ids: Vec<TimelineId> = self.active.iter().map(|(id, _)| *id).collect();
        let mut finished = Vec::new();
        for id in ids {
            if let Some(timeline) = self.active.get_mut(id) {
                timeline.advance(delta);
                if timeline.is_finished()
                    && let Some(done) = self.active.remove(id)
                {
                    finished.push(done);
                }
            }
        }
        finished
    }

    /// Returns whether another frame is needed — `true` while any motion is still
    /// active (spec §4.17).
    ///
    /// After [`advance`](Self::advance) has pruned finished timelines, a non-empty
    /// active set means work remains, so an animation frame is still wanted. This
    /// is the signal the runtime maps to `FrameReason::Animation`.
    #[must_use]
    pub fn needs_frame(&self) -> bool {
        !self.active.is_empty()
    }

    /// Returns the active motion set.
    #[must_use]
    pub const fn active(&self) -> &ActiveMotionSet {
        &self.active
    }

    /// Returns the number of active timelines.
    #[must_use]
    pub fn active_count(&self) -> usize {
        self.active.len()
    }

    /// Removes and returns the timeline with `id`, if present (spec §4.17
    /// interruption).
    ///
    /// This pulls a motion out of the active set — for example when an animation
    /// is interrupted or superseded before it finishes.
    pub fn remove(&mut self, id: TimelineId) -> Option<Timeline> {
        self.active.remove(id)
    }

    /// Removes every tracked timeline, leaving the scheduler idle.
    pub fn clear(&mut self) {
        self.active = ActiveMotionSet::new();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn running(id: u64, duration: f32) -> Timeline {
        let mut timeline = Timeline::new(TimelineId::from_raw(id), duration);
        timeline.start();
        timeline
    }

    #[test]
    fn new_is_empty_and_needs_no_frame() {
        let scheduler = MotionScheduler::new();
        assert_eq!(scheduler.active_count(), 0);
        assert!(!scheduler.needs_frame());
    }

    #[test]
    fn register_tracks_timeline_and_needs_frame() {
        let mut scheduler = MotionScheduler::new();
        scheduler.register(running(1, 1.0));
        assert_eq!(scheduler.active_count(), 1);
        assert!(scheduler.needs_frame());
    }

    #[test]
    fn advance_drops_finished_and_frame_demand_falls() {
        let mut scheduler = MotionScheduler::new();
        scheduler.register(running(1, 1.0));
        scheduler.register(running(2, 2.0));
        // One frame partway through: nothing finishes yet.
        scheduler.advance(1.0);
        assert_eq!(scheduler.active_count(), 1); // timeline 1 finished, 2 remains
        assert!(scheduler.needs_frame());
        // A second frame finishes the remainder.
        scheduler.advance(1.0);
        assert_eq!(scheduler.active_count(), 0);
        assert!(!scheduler.needs_frame());
    }

    #[test]
    fn advance_returns_the_timelines_it_pruned() {
        let mut scheduler = MotionScheduler::new();
        scheduler.register(running(1, 1.0));
        scheduler.register(running(2, 2.0));
        let finished = scheduler.advance(1.0);
        assert_eq!(finished.len(), 1);
        assert_eq!(finished[0].id(), TimelineId::from_raw(1));
        assert_eq!(finished[0].progress(), 1.0);
        // Already pruned: a second advance does not report it again.
        assert!(scheduler.advance(0.1).is_empty());
    }

    #[test]
    fn remove_pulls_timeline_out() {
        let mut scheduler = MotionScheduler::new();
        scheduler.register(running(1, 1.0));
        assert!(scheduler.remove(TimelineId::from_raw(1)).is_some());
        assert!(scheduler.remove(TimelineId::from_raw(1)).is_none());
        assert!(!scheduler.needs_frame());
    }

    #[test]
    fn clear_empties_the_scheduler() {
        let mut scheduler = MotionScheduler::new();
        scheduler.register(running(1, 1.0));
        scheduler.register(running(2, 1.0));
        scheduler.clear();
        assert_eq!(scheduler.active_count(), 0);
        assert!(!scheduler.needs_frame());
    }

    #[test]
    fn skipped_frame_large_delta_finishes_in_one_advance() {
        let mut scheduler = MotionScheduler::new();
        scheduler.register(running(1, 1.0));
        // A single large delta (several skipped frames) still completes it.
        scheduler.advance(10.0);
        assert!(!scheduler.needs_frame());
    }

    #[test]
    fn identical_deltas_are_deterministic() {
        let deltas = [0.1, 0.4, 0.05, 1.2, 0.3];
        let mut a = MotionScheduler::new();
        let mut b = MotionScheduler::new();
        for id in 1..=3 {
            a.register(running(id, 1.0));
            b.register(running(id, 1.0));
        }
        for &d in &deltas {
            a.advance(d);
            b.advance(d);
            assert_eq!(a, b);
            assert_eq!(a.needs_frame(), b.needs_frame());
        }
    }
}
