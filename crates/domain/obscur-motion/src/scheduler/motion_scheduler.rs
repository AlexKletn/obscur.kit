#![doc = include_str!("motion_scheduler.doc/_module.md")]

use crate::scheduler::ActiveMotionSet;
use crate::timeline::{Timeline, TimelineId};

#[doc = include_str!("motion_scheduler.doc/MotionScheduler.md")]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MotionScheduler {
    #[doc = include_str!("motion_scheduler.doc/active.md")]
    active: ActiveMotionSet,
}

impl MotionScheduler {
    #[doc = include_str!("motion_scheduler.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self {
            active: ActiveMotionSet::new(),
        }
    }

    #[doc = include_str!("motion_scheduler.doc/register.md")]
    pub fn register(&mut self, timeline: Timeline) {
        self.active.insert(timeline);
    }

    #[doc = include_str!("motion_scheduler.doc/advance.md")]
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

    #[doc = include_str!("motion_scheduler.doc/needs_frame.md")]
    #[must_use]
    pub fn needs_frame(&self) -> bool {
        !self.active.is_empty()
    }

    #[doc = include_str!("motion_scheduler.doc/active_2.md")]
    #[must_use]
    pub const fn active(&self) -> &ActiveMotionSet {
        &self.active
    }

    #[doc = include_str!("motion_scheduler.doc/active_count.md")]
    #[must_use]
    pub fn active_count(&self) -> usize {
        self.active.len()
    }

    #[doc = include_str!("motion_scheduler.doc/remove.md")]
    pub fn remove(&mut self, id: TimelineId) -> Option<Timeline> {
        self.active.remove(id)
    }

    #[doc = include_str!("motion_scheduler.doc/clear.md")]
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
