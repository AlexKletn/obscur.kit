//! The [`HitTest`] record for a single node hit at a point (spec §4.7).

use obscur_geometry::Point;
use obscur_identity::NodeId;

/// A single hit record: a node struck by a hit-test query at a point
/// (spec §4.7).
///
/// Hit testing maps a point in visual space back to the logical event tree.
/// Each record captures the node hit, the query point expressed in that node's
/// local coordinate space, and the node's `z_order` so that overlapping hits
/// can be ordered topmost-first.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HitTest {
    node: NodeId,
    local_point: Point,
    z_order: i32,
}

impl HitTest {
    /// Constructs a hit record for `node` struck at `local_point` (in the
    /// node's local space) with the given `z_order` (higher = on top).
    #[must_use]
    pub const fn new(node: NodeId, local_point: Point, z_order: i32) -> Self {
        Self {
            node,
            local_point,
            z_order,
        }
    }

    /// The node that was hit.
    #[must_use]
    pub const fn node(&self) -> NodeId {
        self.node
    }

    /// The query point translated into the node's local coordinate space.
    #[must_use]
    pub const fn local_point(&self) -> Point {
        self.local_point
    }

    /// The node's z-order; higher values are drawn on top.
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
