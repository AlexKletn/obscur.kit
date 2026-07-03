//! The stacking context a `LAYER_ROOT` node establishes.

use std::cmp::Ordering;

use obscur_identity::NodeId;

/// Describes the stacking context a `LAYER_ROOT` node establishes (spec §5.1).
///
/// A stacking context groups a node and the subtree it composites, and fixes
/// that group's paint order relative to sibling contexts via a `z_index`:
/// higher values paint on top. This is the **descriptive** retained-tree model
/// only — no actual painting or compositing happens here; that lives in
/// `obscur-scene` / `obscur-render` later.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StackingContext {
    /// The node that establishes this stacking context.
    root: NodeId,
    /// Paint order among siblings; higher paints on top.
    z_index: i32,
}

impl StackingContext {
    /// Creates a stacking context established by `root` at paint order
    /// `z_index`.
    #[must_use]
    pub const fn new(root: NodeId, z_index: i32) -> Self {
        Self { root, z_index }
    }

    /// Returns the node that establishes this stacking context.
    #[must_use]
    pub const fn root(self) -> NodeId {
        self.root
    }

    /// Returns the paint order (`z_index`); higher paints on top.
    #[must_use]
    pub const fn z_index(self) -> i32 {
        self.z_index
    }

    /// Compares two contexts for **paint order**, from bottom-most to top-most.
    ///
    /// The primary key is `z_index` ascending (a higher `z_index` paints later,
    /// i.e. on top). Ties are broken deterministically by the establishing
    /// node's raw [`NodeId`] so the ordering is a total order and paint order is
    /// always well-defined for a given set of contexts.
    #[must_use]
    pub fn paint_order(self, other: Self) -> Ordering {
        self.z_index
            .cmp(&other.z_index)
            .then_with(|| self.root.raw().cmp(&other.root.raw()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(root: u64, z: i32) -> StackingContext {
        StackingContext::new(NodeId::from_raw(root), z)
    }

    #[test]
    fn accessors_roundtrip() {
        let c = ctx(3, 7);
        assert_eq!(c.root(), NodeId::from_raw(3));
        assert_eq!(c.z_index(), 7);
    }

    #[test]
    fn higher_z_index_paints_later() {
        let low = ctx(1, 0);
        let high = ctx(2, 10);
        assert_eq!(low.paint_order(high), Ordering::Less);
        assert_eq!(high.paint_order(low), Ordering::Greater);
    }

    #[test]
    fn ties_broken_by_node_id_deterministically() {
        let a = ctx(1, 5);
        let b = ctx(2, 5);
        assert_eq!(a.paint_order(b), Ordering::Less);
        assert_eq!(b.paint_order(a), Ordering::Greater);
        assert_eq!(a.paint_order(a), Ordering::Equal);
    }

    #[test]
    fn sorts_into_bottom_to_top_order() {
        let mut v = vec![ctx(1, 10), ctx(2, -5), ctx(3, 0), ctx(4, 10)];
        v.sort_by(|a, b| a.paint_order(*b));
        let zs: Vec<i32> = v.iter().map(|c| c.z_index()).collect();
        assert_eq!(zs, vec![-5, 0, 10, 10]);
        // Equal z=10 ordered by ascending node id: id 1 before id 4.
        assert_eq!(v[2].root(), NodeId::from_raw(1));
        assert_eq!(v[3].root(), NodeId::from_raw(4));
    }
}
