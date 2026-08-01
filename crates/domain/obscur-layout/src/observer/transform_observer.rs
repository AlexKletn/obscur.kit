#![doc = include_str!("transform_observer.doc/_module.md")]

use obscur_geometry::AffineTransform;
use obscur_identity::NodeId;

#[doc = include_str!("transform_observer.doc/TransformChange.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransformChange {
    #[doc = include_str!("transform_observer.doc/node.md")]
    pub node: NodeId,
    #[doc = include_str!("transform_observer.doc/current.md")]
    pub current: AffineTransform,
}

impl TransformChange {
    #[doc = include_str!("transform_observer.doc/new.md")]
    #[must_use]
    pub const fn new(node: NodeId, current: AffineTransform) -> Self {
        Self { node, current }
    }
}

#[doc = include_str!("transform_observer.doc/TransformObserver.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransformObserver {
    node: NodeId,
    last: Option<AffineTransform>,
}

impl TransformObserver {
    #[doc = include_str!("transform_observer.doc/new_2.md")]
    #[must_use]
    pub const fn new(node: NodeId) -> Self {
        Self { node, last: None }
    }

    #[doc = include_str!("transform_observer.doc/node_2.md")]
    #[must_use]
    pub const fn node(self) -> NodeId {
        self.node
    }

    #[doc = include_str!("transform_observer.doc/last.md")]
    #[must_use]
    pub const fn last(self) -> Option<AffineTransform> {
        self.last
    }

    #[doc = include_str!("transform_observer.doc/observe.md")]
    #[must_use]
    pub fn observe(&mut self, current: AffineTransform) -> Option<TransformChange> {
        if self.last == Some(current) {
            return None;
        }
        self.last = Some(current);
        Some(TransformChange::new(self.node, current))
    }

    #[doc = include_str!("transform_observer.doc/reset.md")]
    pub const fn reset(&mut self) {
        self.last = None;
    }
}

#[cfg(test)]
mod tests {
    use super::{TransformChange, TransformObserver};
    use obscur_geometry::AffineTransform;
    use obscur_identity::NodeId;

    #[test]
    fn first_observe_fires_with_initial_value() {
        let node = NodeId::from_raw(1);
        let mut obs = TransformObserver::new(node);
        let t = AffineTransform::IDENTITY;
        assert_eq!(obs.observe(t), Some(TransformChange::new(node, t)));
    }

    #[test]
    fn unchanged_transform_yields_none() {
        let mut obs = TransformObserver::new(NodeId::from_raw(1));
        let t = AffineTransform::translate(3.0, 4.0);
        assert!(obs.observe(t).is_some());
        assert_eq!(obs.observe(t), None);
    }

    #[test]
    fn real_change_reports_current() {
        let node = NodeId::from_raw(9);
        let mut obs = TransformObserver::new(node);
        let _ = obs.observe(AffineTransform::IDENTITY);
        let moved = AffineTransform::translate(5.0, 0.0);
        assert_eq!(obs.observe(moved), Some(TransformChange::new(node, moved)));
    }

    #[test]
    fn reset_makes_next_observe_fire() {
        let mut obs = TransformObserver::new(NodeId::from_raw(1));
        let t = AffineTransform::IDENTITY;
        let _ = obs.observe(t);
        obs.reset();
        assert!(obs.observe(t).is_some());
    }
}
