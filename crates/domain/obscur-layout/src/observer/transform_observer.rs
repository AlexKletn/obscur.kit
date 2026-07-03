//! The [`TransformObserver`] — watches a node's transform (optional, spec §4.4).
//!
//! Like all geometry observers it reports the coalesced net change after layout
//! commit. The watched value is the node's [`AffineTransform`]; the observer
//! fires whenever the transform changes. `AffineTransform` is `Copy` and
//! `PartialEq` (six `f32` coefficients), so it is compared and stored directly.

use obscur_geometry::AffineTransform;
use obscur_identity::NodeId;

/// A transform change reported by a [`TransformObserver`] (spec §4.4).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransformChange {
    /// The observed node.
    pub node: NodeId,
    /// The newly-committed transform.
    pub current: AffineTransform,
}

impl TransformChange {
    /// Constructs a transform change event.
    #[must_use]
    pub const fn new(node: NodeId, current: AffineTransform) -> Self {
        Self { node, current }
    }
}

/// Watches one node's [`AffineTransform`], firing on any change (spec §4.4).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransformObserver {
    node: NodeId,
    last: Option<AffineTransform>,
}

impl TransformObserver {
    /// Creates an observer for `node`. It has not yet observed any transform, so
    /// the first [`observe`](Self::observe) fires with the initial value.
    #[must_use]
    pub const fn new(node: NodeId) -> Self {
        Self { node, last: None }
    }

    /// The observed node.
    #[must_use]
    pub const fn node(self) -> NodeId {
        self.node
    }

    /// The last observed transform, or `None` before the first observation.
    #[must_use]
    pub const fn last(self) -> Option<AffineTransform> {
        self.last
    }

    /// Records the newly-committed transform, returning a [`TransformChange`]
    /// when it differs from the last observed transform (or on the first
    /// observation).
    ///
    /// Returns `None` when the transform is unchanged. Updates the stored
    /// transform on a fire.
    #[must_use]
    pub fn observe(&mut self, current: AffineTransform) -> Option<TransformChange> {
        if self.last == Some(current) {
            return None;
        }
        self.last = Some(current);
        Some(TransformChange::new(self.node, current))
    }

    /// Forgets the last observed transform, so the next observation fires
    /// afresh.
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
