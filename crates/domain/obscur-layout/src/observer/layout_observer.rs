#![doc = include_str!("layout_observer.doc/_module.md")]

use obscur_identity::NodeId;

use crate::model::LayoutBox;

#[doc = include_str!("layout_observer.doc/LayoutChange.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutChange {
    #[doc = include_str!("layout_observer.doc/node.md")]
    pub node: NodeId,
    #[doc = include_str!("layout_observer.doc/previous.md")]
    pub previous: Option<LayoutBox>,
    #[doc = include_str!("layout_observer.doc/current.md")]
    pub current: LayoutBox,
}

impl LayoutChange {
    #[doc = include_str!("layout_observer.doc/new.md")]
    #[must_use]
    pub const fn new(node: NodeId, previous: Option<LayoutBox>, current: LayoutBox) -> Self {
        Self {
            node,
            previous,
            current,
        }
    }
}

#[doc = include_str!("layout_observer.doc/LayoutObserver.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutObserver {
    node: NodeId,
    last: Option<LayoutBox>,
}

impl LayoutObserver {
    #[doc = include_str!("layout_observer.doc/new_2.md")]
    #[must_use]
    pub const fn new(node: NodeId) -> Self {
        Self { node, last: None }
    }

    #[doc = include_str!("layout_observer.doc/node_2.md")]
    #[must_use]
    pub const fn node(self) -> NodeId {
        self.node
    }

    #[doc = include_str!("layout_observer.doc/last.md")]
    #[must_use]
    pub const fn last(self) -> Option<LayoutBox> {
        self.last
    }

    #[doc = include_str!("layout_observer.doc/observe.md")]
    #[must_use]
    pub fn observe(&mut self, current: LayoutBox) -> Option<LayoutChange> {
        if self.last == Some(current) {
            return None;
        }
        let previous = self.last;
        self.last = Some(current);
        Some(LayoutChange::new(self.node, previous, current))
    }

    #[doc = include_str!("layout_observer.doc/reset.md")]
    pub const fn reset(&mut self) {
        self.last = None;
    }
}

#[cfg(test)]
mod tests {
    use super::{LayoutChange, LayoutObserver};
    use crate::model::LayoutBox;
    use obscur_geometry::{Point, Rect, Size};
    use obscur_identity::NodeId;

    fn box_at(x: f32, y: f32, w: f32, h: f32) -> LayoutBox {
        LayoutBox::new(Rect::new(Point::new(x, y), Size::new(w, h)))
    }

    #[test]
    fn first_observe_fires_with_initial_value() {
        let node = NodeId::from_raw(1);
        let mut obs = LayoutObserver::new(node);
        let b = box_at(0.0, 0.0, 10.0, 10.0);
        assert_eq!(obs.observe(b), Some(LayoutChange::new(node, None, b)));
        assert_eq!(obs.last(), Some(b));
    }

    #[test]
    fn unchanged_box_yields_none() {
        let mut obs = LayoutObserver::new(NodeId::from_raw(1));
        let b = box_at(1.0, 2.0, 3.0, 4.0);
        assert!(obs.observe(b).is_some());
        assert_eq!(obs.observe(b), None);
    }

    #[test]
    fn real_change_reports_previous_and_current() {
        let node = NodeId::from_raw(7);
        let mut obs = LayoutObserver::new(node);
        let a = box_at(0.0, 0.0, 10.0, 10.0);
        let b = box_at(5.0, 0.0, 10.0, 10.0);
        let _ = obs.observe(a);
        assert_eq!(obs.observe(b), Some(LayoutChange::new(node, Some(a), b)));
    }

    #[test]
    fn reset_makes_next_observe_fire() {
        let mut obs = LayoutObserver::new(NodeId::from_raw(1));
        let b = box_at(0.0, 0.0, 1.0, 1.0);
        let _ = obs.observe(b);
        obs.reset();
        assert_eq!(obs.last(), None);
        assert!(obs.observe(b).is_some());
    }
}
