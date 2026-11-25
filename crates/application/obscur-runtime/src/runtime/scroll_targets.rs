//! The frame's live scroll containers, seen from outside: [`ScrollTargets`].

use obscur_geometry::Point;
use obscur_identity::{FastMap, NodeId};
use obscur_tree::tree::RetainedTree;

use super::ScrollOffsets;
use crate::orchestration::{
    ComputedStyles, LayoutBoxes, ScrollContainer, ScrollPaint, overflow_scrolls,
    overflow_scrolls_x, scroll_containers,
};

#[doc = include_str!("../../docs/api/runtime/scroll_targets.doc/ScrollTargets.md")]
pub(crate) struct ScrollTargets<'a> {
    /// The retained tree the containers and the ancestor walk are found in.
    tree: &'a RetainedTree,
    /// This frame's computed styles: which boxes overflow, and how.
    computed: &'a ComputedStyles,
    /// This frame's arranged boxes, in unscrolled layout space.
    boxes: &'a LayoutBoxes,
    /// Every container's live vertical offset.
    scroll: &'a ScrollOffsets,
}

impl<'a> ScrollTargets<'a> {
    /// Bundles the retained frame and the live offsets for one question.
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

    /// Find the innermost scroll container under a point that accepts at least one requested axis.
    pub(crate) fn scrollable_at(&self, point: Point, delta: Point) -> Option<NodeId> {
        self.scrollable_in(
            point,
            delta,
            &scroll_containers(self.tree, self.computed, self.boxes),
        )
    }

    /// Selects the initial scroll target from the supplied scroll containers.
    pub(crate) fn scrollable_in(
        &self,
        mut point: Point,
        delta: Point,
        containers: &[ScrollContainer],
    ) -> Option<NodeId> {
        let mut target = None;
        for &c in containers {
            if c.scrollport.contains(point) {
                if (delta.x.is_finite()
                    && delta.x != 0.0
                    && c.horizontal_range() > 0.0
                    && overflow_scrolls_x(self.computed, c.node))
                    || (delta.y.is_finite()
                        && delta.y != 0.0
                        && c.max_scroll_top() > 0.0
                        && overflow_scrolls(self.computed, c.node))
                {
                    target = Some(c.node);
                }
                let offset = self.scroll.offset(c);
                point = Point::new(point.x + offset.x, point.y + offset.y);
            }
        }
        target
    }

    #[doc = include_str!("../../docs/api/runtime/scroll_targets.doc/enclosing_scrollable.md")]
    pub(crate) fn enclosing_scrollable(&self, node: NodeId) -> Option<ScrollContainer> {
        let containers = scroll_containers(self.tree, self.computed, self.boxes);
        let mut current = Some(node);
        for _ in 0..=self.tree.len() {
            let node = current?;
            if let Some(container) = containers.iter().find(|container| {
                container.node == node
                    && ((container.max_scroll_top() > 0.0 && overflow_scrolls(self.computed, node))
                        || (container.horizontal_range() > 0.0
                            && overflow_scrolls_x(self.computed, node)))
            }) {
                return Some(*container);
            }
            current = self.tree.visual_parent_of(node);
        }
        None
    }

    #[doc = include_str!("../../docs/api/runtime/scroll_targets.doc/fold_point.md")]
    pub(crate) fn fold_point(&self, point: Point) -> Point {
        let containers = scroll_containers(self.tree, self.computed, self.boxes);
        if containers.is_empty() {
            return point;
        }
        self.fold_through(point, &containers).0
    }

    #[doc = include_str!("../../docs/api/runtime/scroll_targets.doc/paint.md")]
    pub(crate) fn paint(&self, computed: &ComputedStyles) -> FastMap<NodeId, ScrollPaint> {
        scroll_containers(self.tree, computed, self.boxes)
            .into_iter()
            .map(|c| {
                (
                    c.node,
                    ScrollPaint {
                        scroll_top: self.scroll.clamped(c),
                        scroll_left: self.scroll.offset(c).x,
                        content_width: c.content_width,
                        rtl: c.rtl,
                        shows_horizontal_scrollbar: overflow_scrolls_x(computed, c.node),
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

    #[doc = include_str!("../../docs/api/runtime/scroll_targets.doc/fold_through.md")]
    fn fold_through(
        &self,
        mut point: Point,
        containers: &[ScrollContainer],
    ) -> (Point, Option<NodeId>) {
        let mut innermost = None;
        for &c in containers {
            if c.scrollport.contains(point) {
                let offset = self.scroll.offset(c);
                point = Point::new(point.x + offset.x, point.y + offset.y);
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
        assert_eq!(targets.scrollable_at(point, Point::new(0.0, 1.0)), None);
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
