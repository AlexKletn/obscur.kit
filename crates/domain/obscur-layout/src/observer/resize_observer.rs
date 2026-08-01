#![doc = include_str!("resize_observer.doc/_module.md")]

use obscur_geometry::Size;
use obscur_identity::NodeId;

#[doc = include_str!("resize_observer.doc/ResizeChange.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResizeChange {
    #[doc = include_str!("resize_observer.doc/node.md")]
    pub node: NodeId,
    #[doc = include_str!("resize_observer.doc/previous.md")]
    pub previous: Option<Size>,
    #[doc = include_str!("resize_observer.doc/current.md")]
    pub current: Size,
}

impl ResizeChange {
    #[doc = include_str!("resize_observer.doc/new.md")]
    #[must_use]
    pub const fn new(node: NodeId, previous: Option<Size>, current: Size) -> Self {
        Self {
            node,
            previous,
            current,
        }
    }
}

#[doc = include_str!("resize_observer.doc/ResizeObserver.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResizeObserver {
    node: NodeId,
    last: Option<Size>,
}

impl ResizeObserver {
    #[doc = include_str!("resize_observer.doc/new_2.md")]
    #[must_use]
    pub const fn new(node: NodeId) -> Self {
        Self { node, last: None }
    }

    #[doc = include_str!("resize_observer.doc/node_2.md")]
    #[must_use]
    pub const fn node(self) -> NodeId {
        self.node
    }

    #[doc = include_str!("resize_observer.doc/last.md")]
    #[must_use]
    pub const fn last(self) -> Option<Size> {
        self.last
    }

    #[doc = include_str!("resize_observer.doc/observe.md")]
    #[must_use]
    pub fn observe(&mut self, current: Size) -> Option<ResizeChange> {
        if self.last == Some(current) {
            return None;
        }
        let previous = self.last;
        self.last = Some(current);
        Some(ResizeChange::new(self.node, previous, current))
    }

    #[doc = include_str!("resize_observer.doc/reset.md")]
    pub const fn reset(&mut self) {
        self.last = None;
    }
}

#[cfg(test)]
mod tests {
    use super::{ResizeChange, ResizeObserver};
    use obscur_geometry::Size;
    use obscur_identity::NodeId;

    #[test]
    fn first_observe_fires_with_initial_value() {
        let node = NodeId::from_raw(1);
        let mut obs = ResizeObserver::new(node);
        let s = Size::new(10.0, 20.0);
        assert_eq!(obs.observe(s), Some(ResizeChange::new(node, None, s)));
    }

    #[test]
    fn unchanged_size_yields_none() {
        let mut obs = ResizeObserver::new(NodeId::from_raw(1));
        let s = Size::new(5.0, 5.0);
        assert!(obs.observe(s).is_some());
        assert_eq!(obs.observe(s), None);
    }

    #[test]
    fn real_change_reports_previous_and_current() {
        let node = NodeId::from_raw(3);
        let mut obs = ResizeObserver::new(node);
        let a = Size::new(10.0, 10.0);
        let b = Size::new(10.0, 12.0);
        let _ = obs.observe(a);
        assert_eq!(obs.observe(b), Some(ResizeChange::new(node, Some(a), b)));
    }

    #[test]
    fn position_only_move_is_ignored() {
        // The observer is fed only the size, so a move that keeps the same size
        // never fires.
        let mut obs = ResizeObserver::new(NodeId::from_raw(2));
        let s = Size::new(30.0, 40.0);
        assert!(obs.observe(s).is_some());
        assert_eq!(obs.observe(s), None);
    }

    #[test]
    fn reset_makes_next_observe_fire() {
        let mut obs = ResizeObserver::new(NodeId::from_raw(1));
        let s = Size::new(1.0, 1.0);
        let _ = obs.observe(s);
        obs.reset();
        assert!(obs.observe(s).is_some());
    }
}
