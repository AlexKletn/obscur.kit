//! The [`ActiveMotionSet`] deterministic set of running timelines (spec §4.17
//! "motion engine").

use std::collections::BTreeMap;

use crate::timeline::{Timeline, TimelineId};

/// The deterministic set of currently-running [`Timeline`]s, keyed by
/// [`TimelineId`] (spec §4.17).
///
/// The motion engine tracks every active timeline in one place so the
/// [`MotionScheduler`](super::MotionScheduler) can advance them, drop finished
/// ones, and report frame demand. Timelines are stored in a [`BTreeMap`] keyed by
/// their stable [`id`](Timeline::id), which gives a **deterministic** iteration
/// order (ascending by id) regardless of insertion order — a requirement for the
/// engine's reproducibility (spec §4.17). Feeding two sets the same operations
/// yields identical iteration and advancement.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ActiveMotionSet {
    /// The active timelines, keyed by id for deterministic iteration order.
    timelines: BTreeMap<TimelineId, Timeline>,
}

impl ActiveMotionSet {
    /// Creates an empty active set.
    #[must_use]
    pub fn new() -> Self {
        Self {
            timelines: BTreeMap::new(),
        }
    }

    /// Inserts `timeline`, keyed by its [`id`](Timeline::id).
    ///
    /// Returns the previous timeline registered under the same id, if it replaced
    /// one; otherwise [`None`].
    pub fn insert(&mut self, timeline: Timeline) -> Option<Timeline> {
        self.timelines.insert(timeline.id(), timeline)
    }

    /// Removes and returns the timeline with `id`, or [`None`] if absent.
    pub fn remove(&mut self, id: TimelineId) -> Option<Timeline> {
        self.timelines.remove(&id)
    }

    /// Returns a shared reference to the timeline with `id`, if present.
    #[must_use]
    pub fn get(&self, id: TimelineId) -> Option<&Timeline> {
        self.timelines.get(&id)
    }

    /// Returns a mutable reference to the timeline with `id`, if present.
    pub fn get_mut(&mut self, id: TimelineId) -> Option<&mut Timeline> {
        self.timelines.get_mut(&id)
    }

    /// Returns whether a timeline with `id` is in the set.
    #[must_use]
    pub fn contains(&self, id: TimelineId) -> bool {
        self.timelines.contains_key(&id)
    }

    /// Returns the number of active timelines.
    #[must_use]
    pub fn len(&self) -> usize {
        self.timelines.len()
    }

    /// Returns whether the set is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.timelines.is_empty()
    }

    /// Iterates the active timelines in deterministic order (ascending by
    /// [`TimelineId`]).
    pub fn iter(&self) -> impl Iterator<Item = (&TimelineId, &Timeline)> {
        self.timelines.iter()
    }

    /// Removes every finished timeline, returning how many were dropped.
    ///
    /// A timeline is dropped when [`Timeline::is_finished`] is `true`; those still
    /// running, paused or idle are retained. This is how the engine prunes
    /// completed animations from the active set (spec §4.17).
    pub fn retain_active(&mut self) -> usize {
        let before = self.timelines.len();
        self.timelines.retain(|_, timeline| !timeline.is_finished());
        before - self.timelines.len()
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
    fn insert_get_contains_len() {
        let mut set = ActiveMotionSet::new();
        assert!(set.is_empty());
        assert_eq!(set.insert(running(1, 1.0)), None);
        assert_eq!(set.len(), 1);
        assert!(set.contains(TimelineId::from_raw(1)));
        assert!(set.get(TimelineId::from_raw(1)).is_some());
        assert!(!set.contains(TimelineId::from_raw(2)));
    }

    #[test]
    fn insert_returns_replaced_timeline() {
        let mut set = ActiveMotionSet::new();
        set.insert(running(1, 1.0));
        let replaced = set.insert(running(1, 2.0));
        assert!(replaced.is_some());
        assert_eq!(replaced.unwrap().duration(), 1.0);
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn remove_pulls_timeline_out() {
        let mut set = ActiveMotionSet::new();
        set.insert(running(1, 1.0));
        assert!(set.remove(TimelineId::from_raw(1)).is_some());
        assert!(set.remove(TimelineId::from_raw(1)).is_none());
        assert!(set.is_empty());
    }

    #[test]
    fn get_mut_allows_advancing() {
        let mut set = ActiveMotionSet::new();
        set.insert(running(1, 1.0));
        set.get_mut(TimelineId::from_raw(1)).unwrap().advance(1.0);
        assert!(set.get(TimelineId::from_raw(1)).unwrap().is_finished());
    }

    #[test]
    fn iteration_is_sorted_by_id() {
        let mut set = ActiveMotionSet::new();
        // Insert ids out of order.
        set.insert(running(3, 1.0));
        set.insert(running(1, 1.0));
        set.insert(running(2, 1.0));
        let ids: Vec<u64> = set.iter().map(|(id, _)| id.raw()).collect();
        assert_eq!(ids, vec![1, 2, 3]);
    }

    #[test]
    fn retain_active_drops_finished() {
        let mut set = ActiveMotionSet::new();
        set.insert(running(1, 1.0));
        let mut finished = running(2, 1.0);
        finished.advance(1.0);
        assert!(finished.is_finished());
        set.insert(finished);
        assert_eq!(set.len(), 2);
        let removed = set.retain_active();
        assert_eq!(removed, 1);
        assert_eq!(set.len(), 1);
        assert!(set.contains(TimelineId::from_raw(1)));
        assert!(!set.contains(TimelineId::from_raw(2)));
    }
}
