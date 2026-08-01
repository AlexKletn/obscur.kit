#![doc = include_str!("stacking_context.doc/_module.md")]

use std::cmp::Ordering;

use obscur_identity::NodeId;

#[doc = include_str!("stacking_context.doc/StackingContext.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StackingContext {
    #[doc = include_str!("stacking_context.doc/root.md")]
    root: NodeId,
    #[doc = include_str!("stacking_context.doc/z_index.md")]
    z_index: i32,
}

impl StackingContext {
    #[doc = include_str!("stacking_context.doc/new.md")]
    #[must_use]
    pub const fn new(root: NodeId, z_index: i32) -> Self {
        Self { root, z_index }
    }

    #[doc = include_str!("stacking_context.doc/root_2.md")]
    #[must_use]
    pub const fn root(self) -> NodeId {
        self.root
    }

    #[doc = include_str!("stacking_context.doc/z_index_2.md")]
    #[must_use]
    pub const fn z_index(self) -> i32 {
        self.z_index
    }

    #[doc = include_str!("stacking_context.doc/paint_order.md")]
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
        let mut v = [ctx(1, 10), ctx(2, -5), ctx(3, 0), ctx(4, 10)];
        v.sort_by(|a, b| a.paint_order(*b));
        let zs: Vec<i32> = v.iter().map(|c| c.z_index()).collect();
        assert_eq!(zs, vec![-5, 0, 10, 10]);
        // Equal z=10 ordered by ascending node id: id 1 before id 4.
        assert_eq!(v[2].root(), NodeId::from_raw(1));
        assert_eq!(v[3].root(), NodeId::from_raw(4));
    }
}
