#![doc = include_str!("../../docs/api/observer/resize_observer.doc/_module.md")]

use obscur_geometry::Size;
use obscur_identity::NodeId;

use super::ResizeChange;

/// Watches one node's [`Size`], firing when the size changes and ignoring
/// position-only moves (spec §4.4).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResizeObserver {
    node: NodeId,
    last: Option<Size>,
}

impl ResizeObserver {
    /// Creates an observer for `node`. It has not yet observed any size, so the
    /// first [`observe`](Self::observe) fires with the initial value.
    #[must_use]
    pub const fn new(node: NodeId) -> Self {
        Self { node, last: None }
    }

    /// The observed node.
    #[must_use]
    pub const fn node(self) -> NodeId {
        self.node
    }

    /// The last observed size, or `None` before the first observation.
    #[must_use]
    pub const fn last(self) -> Option<Size> {
        self.last
    }

    #[doc = include_str!("../../docs/api/observer/resize_observer.doc/observe.md")]
    #[must_use]
    pub fn observe(&mut self, current: Size) -> Option<ResizeChange> {
        if self.last == Some(current) {
            return None;
        }
        let previous = self.last;
        self.last = Some(current);
        Some(ResizeChange::new(self.node, previous, current))
    }

    /// Forgets the last observed size, so the next observation fires afresh.
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
