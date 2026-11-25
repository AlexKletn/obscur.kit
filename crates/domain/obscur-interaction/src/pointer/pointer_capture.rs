//! The [`PointerCapture`] binding that routes a pointer to a node (spec §4.7).

use obscur_identity::NodeId;

use super::PointerId;

#[doc = include_str!("../../docs/api/pointer/pointer_capture.doc/PointerCapture.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PointerCapture {
    /// The captured pointer.
    pub pointer: PointerId,
    /// The node the pointer is captured to.
    pub node: NodeId,
}

impl PointerCapture {
    /// Creates a capture binding `pointer` to `node`.
    #[must_use]
    pub const fn new(pointer: PointerId, node: NodeId) -> Self {
        Self { pointer, node }
    }

    /// Returns the captured pointer.
    #[must_use]
    pub const fn pointer(&self) -> PointerId {
        self.pointer
    }

    /// Returns the node the pointer is captured to.
    #[must_use]
    pub const fn node(&self) -> NodeId {
        self.node
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accessors_return_fields() {
        let c = PointerCapture::new(PointerId::from_raw(2), NodeId::from_raw(7));
        assert_eq!(c.pointer(), PointerId::from_raw(2));
        assert_eq!(c.node(), NodeId::from_raw(7));
    }

    #[test]
    fn equal_bindings_compare_equal() {
        let a = PointerCapture::new(PointerId::MOUSE, NodeId::from_raw(1));
        let b = PointerCapture::new(PointerId::MOUSE, NodeId::from_raw(1));
        assert_eq!(a, b);
    }
}
