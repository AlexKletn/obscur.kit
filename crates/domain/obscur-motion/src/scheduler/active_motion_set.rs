#![doc = include_str!("active_motion_set.doc/_module.md")]

use std::collections::BTreeMap;

use crate::timeline::{Timeline, TimelineId};

#[doc = include_str!("active_motion_set.doc/ActiveMotionSet.md")]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ActiveMotionSet {
    #[doc = include_str!("active_motion_set.doc/timelines.md")]
    timelines: BTreeMap<TimelineId, Timeline>,
}

impl ActiveMotionSet {
    #[doc = include_str!("active_motion_set.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self {
            timelines: BTreeMap::new(),
        }
    }

    #[doc = include_str!("active_motion_set.doc/insert.md")]
    pub fn insert(&mut self, timeline: Timeline) -> Option<Timeline> {
        self.timelines.insert(timeline.id(), timeline)
    }

    #[doc = include_str!("active_motion_set.doc/remove.md")]
    pub fn remove(&mut self, id: TimelineId) -> Option<Timeline> {
        self.timelines.remove(&id)
    }

    #[doc = include_str!("active_motion_set.doc/get.md")]
    #[must_use]
    pub fn get(&self, id: TimelineId) -> Option<&Timeline> {
        self.timelines.get(&id)
    }

    #[doc = include_str!("active_motion_set.doc/get_mut.md")]
    pub fn get_mut(&mut self, id: TimelineId) -> Option<&mut Timeline> {
        self.timelines.get_mut(&id)
    }

    #[doc = include_str!("active_motion_set.doc/contains.md")]
    #[must_use]
    pub fn contains(&self, id: TimelineId) -> bool {
        self.timelines.contains_key(&id)
    }

    #[doc = include_str!("active_motion_set.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.timelines.len()
    }

    #[doc = include_str!("active_motion_set.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.timelines.is_empty()
    }

    #[doc = include_str!("active_motion_set.doc/iter.md")]
    pub fn iter(&self) -> impl Iterator<Item = (&TimelineId, &Timeline)> {
        self.timelines.iter()
    }

    #[doc = include_str!("active_motion_set.doc/retain_active.md")]
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
