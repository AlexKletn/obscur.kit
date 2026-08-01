#![doc = include_str!("pointer_capture.doc/_module.md")]

use obscur_identity::NodeId;

use super::PointerId;

#[doc = include_str!("pointer_capture.doc/PointerCapture.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PointerCapture {
    #[doc = include_str!("pointer_capture.doc/pointer.md")]
    pub pointer: PointerId,
    #[doc = include_str!("pointer_capture.doc/node.md")]
    pub node: NodeId,
}

impl PointerCapture {
    #[doc = include_str!("pointer_capture.doc/new.md")]
    #[must_use]
    pub const fn new(pointer: PointerId, node: NodeId) -> Self {
        Self { pointer, node }
    }

    #[doc = include_str!("pointer_capture.doc/pointer_2.md")]
    #[must_use]
    pub const fn pointer(&self) -> PointerId {
        self.pointer
    }

    #[doc = include_str!("pointer_capture.doc/node_2.md")]
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
