#![doc = include_str!("hit_test.doc/_module.md")]

use obscur_geometry::Point;
use obscur_identity::NodeId;

#[doc = include_str!("hit_test.doc/HitTest.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HitTest {
    node: NodeId,
    local_point: Point,
    z_order: i32,
}

impl HitTest {
    #[doc = include_str!("hit_test.doc/new.md")]
    #[must_use]
    pub const fn new(node: NodeId, local_point: Point, z_order: i32) -> Self {
        Self {
            node,
            local_point,
            z_order,
        }
    }

    #[doc = include_str!("hit_test.doc/node.md")]
    #[must_use]
    pub const fn node(&self) -> NodeId {
        self.node
    }

    #[doc = include_str!("hit_test.doc/local_point.md")]
    #[must_use]
    pub const fn local_point(&self) -> Point {
        self.local_point
    }

    #[doc = include_str!("hit_test.doc/z_order.md")]
    #[must_use]
    pub const fn z_order(&self) -> i32 {
        self.z_order
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(raw: u64) -> NodeId {
        NodeId::from_raw(raw)
    }

    #[test]
    fn accessors_return_constructor_values() {
        let h = HitTest::new(node(7), Point::new(3.0, 4.0), 5);
        assert_eq!(h.node(), node(7));
        assert_eq!(h.local_point(), Point::new(3.0, 4.0));
        assert_eq!(h.z_order(), 5);
    }

    #[test]
    fn equality_is_field_wise() {
        let a = HitTest::new(node(1), Point::new(0.0, 0.0), 0);
        let b = HitTest::new(node(1), Point::new(0.0, 0.0), 0);
        let c = HitTest::new(node(1), Point::new(0.0, 0.0), 1);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
