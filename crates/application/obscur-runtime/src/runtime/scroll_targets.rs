#![doc = include_str!("scroll_targets.doc/_module.md")]

use obscur_geometry::Point;
use obscur_identity::{FastMap, NodeId};
use obscur_tree::tree::RetainedTree;

use super::{AncestorChain, ScrollOffsets};
use crate::orchestration::{
    ComputedStyles, LayoutBoxes, ScrollContainer, ScrollPaint, overflow_scrolls, scroll_containers,
};

#[doc = include_str!("scroll_targets.doc/ScrollTargets.md")]
pub(crate) struct ScrollTargets<'a> {
    #[doc = include_str!("scroll_targets.doc/tree.md")]
    tree: &'a RetainedTree,
    #[doc = include_str!("scroll_targets.doc/computed.md")]
    computed: &'a ComputedStyles,
    #[doc = include_str!("scroll_targets.doc/boxes.md")]
    boxes: &'a LayoutBoxes,
    #[doc = include_str!("scroll_targets.doc/scroll.md")]
    scroll: &'a ScrollOffsets,
}

impl<'a> ScrollTargets<'a> {
    #[doc = include_str!("scroll_targets.doc/new.md")]
    pub(crate) fn new(
        tree: &'a RetainedTree,
        computed: &'a ComputedStyles,
        boxes: &'a LayoutBoxes,
        scroll: &'a ScrollOffsets,
    ) -> Self {
        Self {
            tree,
            computed,
            boxes,
            scroll,
        }
    }

    #[doc = include_str!("scroll_targets.doc/scrollable_at.md")]
    pub(crate) fn scrollable_at(&self, point: Point) -> Option<NodeId> {
        let containers: Vec<_> = scroll_containers(self.tree, self.computed, self.boxes)
            .into_iter()
            .filter(|c| overflow_scrolls(self.computed, c.node))
            .collect();
        if containers.is_empty() {
            return None;
        }
        self.fold_through(point, &containers).1
    }

    #[doc = include_str!("scroll_targets.doc/enclosing_scrollable.md")]
    pub(crate) fn enclosing_scrollable(&self, node: NodeId) -> Option<ScrollContainer> {
        let containers = scroll_containers(self.tree, self.computed, self.boxes);
        AncestorChain::of(self.tree, Some(node))
            .into_iter()
            .find_map(|ancestor| {
                containers
                    .iter()
                    .find(|c| c.node == ancestor && overflow_scrolls(self.computed, c.node))
                    .copied()
            })
    }

    #[doc = include_str!("scroll_targets.doc/fold_point.md")]
    pub(crate) fn fold_point(&self, point: Point) -> Point {
        let containers = scroll_containers(self.tree, self.computed, self.boxes);
        if containers.is_empty() {
            return point;
        }
        self.fold_through(point, &containers).0
    }

    #[doc = include_str!("scroll_targets.doc/paint.md")]
    pub(crate) fn paint(&self, computed: &ComputedStyles) -> FastMap<NodeId, ScrollPaint> {
        scroll_containers(self.tree, computed, self.boxes)
            .into_iter()
            .map(|c| {
                (
                    c.node,
                    ScrollPaint {
                        scroll_top: self.scroll.clamped(c),
                        content_height: c.content_height,
                        // `hidden` clips and scrolls programmatically, but shows no
                        // scrollbar — advertising one would promise a scroll the user
                        // cannot perform.
                        shows_scrollbar: overflow_scrolls(computed, c.node),
                    },
                )
            })
            .collect()
    }

    #[doc = include_str!("scroll_targets.doc/fold_through.md")]
    fn fold_through(
        &self,
        mut point: Point,
        containers: &[ScrollContainer],
    ) -> (Point, Option<NodeId>) {
        let mut innermost = None;
        for &c in containers {
            if c.scrollport.contains(point) {
                point = Point::new(point.x, point.y + self.scroll.clamped(c));
                innermost = Some(c.node);
            }
        }
        (point, innermost)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty() -> (RetainedTree, ComputedStyles, LayoutBoxes, ScrollOffsets) {
        (
            RetainedTree::new(),
            ComputedStyles::default(),
            LayoutBoxes::default(),
            ScrollOffsets::new(),
        )
    }

    #[test]
    fn a_frame_with_no_scroll_container_scrolls_nothing_and_folds_nothing() {
        let (tree, computed, boxes, scroll) = empty();
        let targets = ScrollTargets::new(&tree, &computed, &boxes, &scroll);
        let point = Point::new(7.0, 11.0);
        assert_eq!(targets.scrollable_at(point), None);
        // The identity, which is what keeps a non-scrolling document's hit-test
        // byte-identical to one built before scroll containers existed.
        assert_eq!(targets.fold_point(point), point);
        assert!(targets.paint(&computed).is_empty());
    }

    #[test]
    fn a_node_with_no_ancestors_has_no_enclosing_scrollable() {
        let (tree, computed, boxes, scroll) = empty();
        let targets = ScrollTargets::new(&tree, &computed, &boxes, &scroll);
        assert!(targets.enclosing_scrollable(NodeId::from_raw(4)).is_none());
    }
}
