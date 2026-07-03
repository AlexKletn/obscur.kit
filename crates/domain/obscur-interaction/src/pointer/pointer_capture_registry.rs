//! The [`PointerCaptureRegistry`] tracking which node captured each pointer
//! (spec §4.7).

use std::collections::BTreeMap;

use obscur_identity::NodeId;

use super::{PointerCapture, PointerId};

/// Tracks which node, if any, has captured each pointer (spec §4.7).
///
/// A captured pointer routes all its subsequent events to its capturing node
/// instead of the hit-tested target: capture *overrides* hit-testing. The
/// registry maps each captured [`PointerId`] to the [`NodeId`] holding it,
/// backed by a [`BTreeMap`] for deterministic ordering.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PointerCaptureRegistry {
    captures: BTreeMap<PointerId, NodeId>,
}

impl PointerCaptureRegistry {
    /// Creates an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            captures: BTreeMap::new(),
        }
    }

    /// Captures `pointer` to `node`, setting or replacing any existing capture.
    ///
    /// Returns the node that was previously capturing `pointer`, if any.
    pub fn capture(&mut self, pointer: PointerId, node: NodeId) -> Option<NodeId> {
        self.captures.insert(pointer, node)
    }

    /// Releases any capture on `pointer`, returning the node that held it.
    pub fn release(&mut self, pointer: PointerId) -> Option<NodeId> {
        self.captures.remove(&pointer)
    }

    /// Releases every pointer captured by `node` — for example when the node is
    /// removed — and returns the number of captures released.
    pub fn release_node(&mut self, node: NodeId) -> usize {
        let before = self.captures.len();
        self.captures.retain(|_, captor| *captor != node);
        before - self.captures.len()
    }

    /// Returns the node capturing `pointer`, if any. A captured pointer routes
    /// here instead of its hit-tested target.
    #[must_use]
    pub fn target(&self, pointer: PointerId) -> Option<NodeId> {
        self.captures.get(&pointer).copied()
    }

    /// Returns whether `pointer` is currently captured.
    #[must_use]
    pub fn is_captured(&self, pointer: PointerId) -> bool {
        self.captures.contains_key(&pointer)
    }

    /// Returns the [`PointerCapture`] binding for `pointer`, if it is captured.
    #[must_use]
    pub fn capture_for(&self, pointer: PointerId) -> Option<PointerCapture> {
        self.captures
            .get(&pointer)
            .map(|node| PointerCapture::new(pointer, *node))
    }

    /// Returns the number of captured pointers.
    #[must_use]
    pub fn len(&self) -> usize {
        self.captures.len()
    }

    /// Returns whether no pointer is captured.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.captures.is_empty()
    }

    /// Releases all captures.
    pub fn clear(&mut self) {
        self.captures.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(raw: u64) -> NodeId {
        NodeId::from_raw(raw)
    }

    #[test]
    fn new_is_empty() {
        let r = PointerCaptureRegistry::new();
        assert!(r.is_empty());
        assert_eq!(r.len(), 0);
        assert_eq!(r, PointerCaptureRegistry::default());
    }

    #[test]
    fn capture_returns_previous() {
        let mut r = PointerCaptureRegistry::new();
        assert_eq!(r.capture(PointerId::MOUSE, node(1)), None);
        assert_eq!(r.capture(PointerId::MOUSE, node(2)), Some(node(1)));
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn target_overrides_hit_testing() {
        let mut r = PointerCaptureRegistry::new();
        r.capture(PointerId::MOUSE, node(5));
        assert_eq!(r.target(PointerId::MOUSE), Some(node(5)));
        assert!(r.is_captured(PointerId::MOUSE));
        assert_eq!(r.target(PointerId::from_raw(9)), None);
        assert!(!r.is_captured(PointerId::from_raw(9)));
    }

    #[test]
    fn capture_for_returns_binding() {
        let mut r = PointerCaptureRegistry::new();
        r.capture(PointerId::MOUSE, node(3));
        assert_eq!(
            r.capture_for(PointerId::MOUSE),
            Some(PointerCapture::new(PointerId::MOUSE, node(3)))
        );
        assert_eq!(r.capture_for(PointerId::from_raw(4)), None);
    }

    #[test]
    fn release_returns_released_node() {
        let mut r = PointerCaptureRegistry::new();
        r.capture(PointerId::MOUSE, node(1));
        assert_eq!(r.release(PointerId::MOUSE), Some(node(1)));
        assert_eq!(r.release(PointerId::MOUSE), None);
        assert!(r.is_empty());
    }

    #[test]
    fn release_node_releases_all_for_that_node() {
        let mut r = PointerCaptureRegistry::new();
        r.capture(PointerId::from_raw(1), node(10));
        r.capture(PointerId::from_raw(2), node(10));
        r.capture(PointerId::from_raw(3), node(20));

        assert_eq!(r.release_node(node(10)), 2);
        assert!(!r.is_captured(PointerId::from_raw(1)));
        assert!(!r.is_captured(PointerId::from_raw(2)));
        assert_eq!(r.target(PointerId::from_raw(3)), Some(node(20)));
        assert_eq!(r.release_node(node(99)), 0);
    }

    #[test]
    fn clear_removes_everything() {
        let mut r = PointerCaptureRegistry::new();
        r.capture(PointerId::MOUSE, node(1));
        r.capture(PointerId::from_raw(2), node(2));
        r.clear();
        assert!(r.is_empty());
    }
}
