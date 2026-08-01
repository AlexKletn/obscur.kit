#![doc = include_str!("scroll_offsets.doc/_module.md")]

use std::collections::HashMap;

use obscur_identity::NodeId;
use obscur_tree::tree::RetainedTree;

use crate::orchestration::{LayoutBoxes, ScrollContainer};

#[doc = include_str!("scroll_offsets.doc/FoldInputs.md")]
#[derive(Clone, Copy)]
pub(crate) struct FoldInputs<'a> {
    #[doc = include_str!("scroll_offsets.doc/tree.md")]
    pub(crate) tree: &'a RetainedTree,
    #[doc = include_str!("scroll_offsets.doc/boxes.md")]
    pub(crate) boxes: &'a LayoutBoxes,
    #[doc = include_str!("scroll_offsets.doc/containers.md")]
    pub(crate) containers: &'a [ScrollContainer],
}

#[doc = include_str!("scroll_offsets.doc/ScrollOffsets.md")]
pub(crate) struct ScrollOffsets {
    #[doc = include_str!("scroll_offsets.doc/offsets.md")]
    offsets: HashMap<NodeId, f32>,
    #[doc = include_str!("scroll_offsets.doc/folded.md")]
    folded: LayoutBoxes,
    #[doc = include_str!("scroll_offsets.doc/folded_active.md")]
    folded_active: bool,
}

impl ScrollOffsets {
    #[doc = include_str!("scroll_offsets.doc/new.md")]
    pub(crate) fn new() -> Self {
        Self {
            offsets: HashMap::new(),
            folded: LayoutBoxes::new(),
            folded_active: false,
        }
    }

    #[doc = include_str!("scroll_offsets.doc/clamped.md")]
    pub(crate) fn clamped(&self, c: ScrollContainer) -> f32 {
        self.offsets
            .get(&c.node)
            .copied()
            .unwrap_or(0.0)
            .clamp(0.0, c.max_scroll_top())
    }

    #[doc = include_str!("scroll_offsets.doc/stored.md")]
    pub(crate) fn stored(&self, node: NodeId) -> f32 {
        self.offsets.get(&node).copied().unwrap_or(0.0).max(0.0)
    }

    #[doc = include_str!("scroll_offsets.doc/set.md")]
    pub(crate) fn set(&mut self, c: ScrollContainer, px: f32, fold: FoldInputs<'_>) {
        if !px.is_finite() {
            return;
        }
        self.offsets
            .insert(c.node, px.clamp(0.0, c.max_scroll_top()));
        self.refold(fold);
    }

    #[doc = include_str!("scroll_offsets.doc/shift.md")]
    pub(crate) fn shift(&mut self, c: ScrollContainer, dy: f32, fold: FoldInputs<'_>) -> f32 {
        let next = (self.clamped(c) + dy).clamp(0.0, c.max_scroll_top());
        self.offsets.insert(c.node, next);
        self.refold(fold);
        next
    }

    #[doc = include_str!("scroll_offsets.doc/seat.md")]
    pub(crate) fn seat(&mut self, node: NodeId, px: f32) {
        self.offsets.insert(node, px);
    }

    #[doc = include_str!("scroll_offsets.doc/refold.md")]
    pub(crate) fn refold(&mut self, fold: FoldInputs<'_>) {
        if fold.containers.is_empty() {
            self.folded_active = false;
            self.folded.clear();
            return;
        }
        // The live scroll offset of each overflowing scroll container, so an
        // ancestor walk can add every crossed container's contribution.
        let scroll_of: HashMap<NodeId, f32> = fold
            .containers
            .iter()
            .map(|c| (c.node, self.clamped(*c)))
            .collect();
        let mut folded = fold.boxes.clone();
        let cap = fold.tree.len();
        for (&node, layout_box) in &mut folded {
            let mut dy = 0.0;
            let mut current = node;
            // Fold every scroll-container ancestor, exclusive of the node itself: a
            // scroll container reports its own unscrolled position.
            for _ in 0..cap {
                let Some(parent) = fold
                    .tree
                    .get(current)
                    .and_then(|n| n.logical_parent().parent())
                else {
                    break;
                };
                if let Some(offset) = scroll_of.get(&parent) {
                    dy += *offset;
                }
                current = parent;
            }
            layout_box.bounds.origin.y -= dy;
        }
        self.folded = folded;
        self.folded_active = true;
    }

    #[doc = include_str!("scroll_offsets.doc/projection.md")]
    pub(crate) fn projection<'a>(&'a self, boxes: &'a LayoutBoxes) -> &'a LayoutBoxes {
        if self.folded_active {
            &self.folded
        } else {
            boxes
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{FoldInputs, ScrollOffsets};
    use crate::orchestration::{LayoutBoxes, ScrollContainer};
    use obscur_geometry::{Point, Rect, Size};
    use obscur_identity::{ComponentId, NodeId};
    use obscur_layout::model::LayoutBox;
    use obscur_tree::node::{LogicalParent, NodeKind};
    use obscur_tree::tree::RetainedTree;

    /// A scroll container at `node` whose scrollport is `height` tall at `top` and
    /// whose content is `content` tall.
    fn container(node: NodeId, top: f32, height: f32, content: f32) -> ScrollContainer {
        ScrollContainer {
            node,
            scrollport: Rect::new(Point::new(0.0, top), Size::new(100.0, height)),
            content_height: content,
            depth: 0,
        }
    }

    fn layout_box(top: f32) -> LayoutBox {
        LayoutBox::new(Rect::new(Point::new(0.0, top), Size::new(100.0, 10.0)))
    }

    /// A chain `root → a → b → c` of plain elements, so a fold can be measured
    /// against a real ancestor walk. Returns the four ids in order.
    fn chain() -> (RetainedTree, [NodeId; 4]) {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let mut ids = [root; 4];
        let mut parent = root;
        for slot in &mut ids[1..] {
            let child = tree.alloc(NodeKind::Element {
                component: ComponentId::from_raw(1),
            });
            tree.get_mut(child)
                .expect("just allocated")
                .set_logical_parent(LogicalParent::of(parent));
            tree.get_mut(parent).expect("in tree").push_child(child);
            *slot = child;
            parent = child;
        }
        (tree, ids)
    }

    #[test]
    fn an_offset_is_kept_as_written_and_clamped_on_every_read() {
        // The point of clamping on read rather than on write: a relayout that
        // shrinks the content must not surface the old over-scroll, but must also
        // not destroy the position the user had — growing back restores it.
        let node = NodeId::from_raw(7);
        let mut offsets = ScrollOffsets::new();
        let tree = RetainedTree::new();
        let boxes = LayoutBoxes::new();
        let tall = container(node, 0.0, 100.0, 500.0);
        offsets.set(
            tall,
            400.0,
            FoldInputs {
                tree: &tree,
                boxes: &boxes,
                containers: &[tall],
            },
        );
        assert_eq!(offsets.clamped(tall), 400.0);

        // The content shrinks to two viewports: the read re-clamps.
        let short = container(node, 0.0, 100.0, 200.0);
        assert_eq!(offsets.clamped(short), 100.0);
        // …and the stored offset survives, so growing back restores it.
        assert_eq!(offsets.clamped(tall), 400.0);
    }

    #[test]
    fn a_non_finite_offset_never_reaches_the_map() {
        let node = NodeId::from_raw(7);
        let c = container(node, 0.0, 100.0, 500.0);
        let tree = RetainedTree::new();
        let boxes = LayoutBoxes::new();
        let fold = FoldInputs {
            tree: &tree,
            boxes: &boxes,
            containers: &[c],
        };
        let mut offsets = ScrollOffsets::new();
        offsets.set(c, 120.0, fold);
        offsets.set(c, f32::NAN, fold);
        assert_eq!(offsets.clamped(c), 120.0);
        offsets.set(c, f32::INFINITY, fold);
        assert_eq!(offsets.clamped(c), 120.0);
    }

    /// Names the defect recorded in `docs/runtime-decomposition.md` (the
    /// "reported, not changed" table, `scroll_by` row): [`set`] rejects a
    /// non-finite offset, but [`shift`] does not check its **delta**, and
    /// `(current + NaN).clamp(0, max)` is `NaN` in Rust rather than a panic — so a
    /// `NaN` wheel delta poisons the map and reaches `ScrollPaint`.
    ///
    /// This test asserts the behaviour that is **wrong**, on purpose and as a
    /// tripwire, because the campaign that found it deliberately did not change
    /// it: a refactor that silently alters behaviour cannot be reviewed. When the
    /// guard lands, this test must be **inverted** — assert the delta is ignored
    /// and the offset keeps its previous value, as [`set`]'s test already does —
    /// and the ledger row deleted. Do not delete this test to make a fix pass.
    ///
    /// [`set`]: ScrollOffsets::set
    /// [`shift`]: ScrollOffsets::shift
    #[test]
    fn a_non_finite_shift_delta_poisons_the_offset_and_should_not() {
        let node = NodeId::from_raw(7);
        let c = container(node, 0.0, 100.0, 500.0);
        let tree = RetainedTree::new();
        let boxes = LayoutBoxes::new();
        let fold = FoldInputs {
            tree: &tree,
            boxes: &boxes,
            containers: &[c],
        };
        let mut offsets = ScrollOffsets::new();
        offsets.set(c, 120.0, fold);
        let landed = offsets.shift(c, f32::NAN, fold);
        assert!(
            landed.is_nan(),
            "the defect: a NaN delta clamps to NaN rather than being rejected"
        );
        assert!(
            offsets.clamped(c).is_nan(),
            "and it is stored, so it reaches the paint transform and the scrollbar"
        );
    }

    #[test]
    fn a_shift_clamps_at_both_ends_and_reports_where_it_landed() {
        let node = NodeId::from_raw(7);
        let c = container(node, 0.0, 100.0, 300.0); // max 200
        let tree = RetainedTree::new();
        let boxes = LayoutBoxes::new();
        let fold = FoldInputs {
            tree: &tree,
            boxes: &boxes,
            containers: &[c],
        };
        let mut offsets = ScrollOffsets::new();
        assert_eq!(offsets.shift(c, 50.0, fold), 50.0);
        assert_eq!(offsets.shift(c, 1_000.0, fold), 200.0);
        assert_eq!(offsets.shift(c, -1_000.0, fold), 0.0);
    }

    #[test]
    fn nothing_overflowing_hands_the_content_boxes_straight_back() {
        // The no-clone guarantee, asserted on the pointer rather than on the
        // values: an unscrolled tree must not pay for the projection.
        let mut offsets = ScrollOffsets::new();
        let tree = RetainedTree::new();
        let mut boxes = LayoutBoxes::new();
        boxes.insert(NodeId::from_raw(1), layout_box(40.0));
        offsets.refold(FoldInputs {
            tree: &tree,
            boxes: &boxes,
            containers: &[],
        });
        assert!(std::ptr::eq(offsets.projection(&boxes), &raw const boxes));
    }

    #[test]
    fn a_write_refolds_in_the_same_call() {
        // The invariant itself: there is no window in which the map has moved and
        // the projection has not.
        let (tree, [_root, scroller, row, _leaf]) = chain();
        let mut boxes = LayoutBoxes::new();
        boxes.insert(scroller, layout_box(0.0));
        boxes.insert(row, layout_box(300.0));
        let c = container(scroller, 0.0, 100.0, 900.0);
        let fold = FoldInputs {
            tree: &tree,
            boxes: &boxes,
            containers: &[c],
        };
        let mut offsets = ScrollOffsets::new();
        offsets.set(c, 250.0, fold);

        let folded = offsets.projection(&boxes);
        assert!(
            !std::ptr::eq(folded, &raw const boxes),
            "a container overflows, so the projection is distinct"
        );
        // The row is painted 250px up; the scroll container reports its own,
        // unscrolled box.
        assert_eq!(folded[&row].bounds.origin.y, 50.0);
        assert_eq!(folded[&scroller].bounds.origin.y, 0.0);
    }

    #[test]
    fn a_nested_scroller_folds_its_whole_ancestor_chain() {
        // `root → outer → inner → leaf`, both scrollers scrolled: the leaf must
        // lose both offsets, the inner scroller only its ancestor's.
        let (tree, [_root, outer, inner, leaf]) = chain();
        let mut boxes = LayoutBoxes::new();
        boxes.insert(outer, layout_box(0.0));
        boxes.insert(inner, layout_box(100.0));
        boxes.insert(leaf, layout_box(400.0));
        let co = container(outer, 0.0, 200.0, 1_000.0);
        let ci = container(inner, 100.0, 100.0, 1_000.0);
        let fold = FoldInputs {
            tree: &tree,
            boxes: &boxes,
            containers: &[co, ci],
        };
        let mut offsets = ScrollOffsets::new();
        offsets.set(co, 30.0, fold);
        offsets.set(ci, 70.0, fold);

        let folded = offsets.projection(&boxes);
        assert_eq!(
            folded[&outer].bounds.origin.y, 0.0,
            "the outermost is fixed"
        );
        assert_eq!(
            folded[&inner].bounds.origin.y, 70.0,
            "the inner scroller moves with its ancestor only"
        );
        assert_eq!(
            folded[&leaf].bounds.origin.y, 300.0,
            "the leaf loses 30 + 70"
        );
    }

    #[test]
    fn a_cyclic_parent_chain_terminates() {
        // Not reachable through reconciliation, but the fold walks parent links it
        // does not own: a cycle must bound out rather than hang the frame that
        // publishes geometry.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = tree.alloc(NodeKind::Root);
        let b = tree.alloc(NodeKind::Root);
        tree.get_mut(a)
            .expect("in tree")
            .set_logical_parent(LogicalParent::of(b));
        tree.get_mut(b)
            .expect("in tree")
            .set_logical_parent(LogicalParent::of(a));
        let mut boxes = LayoutBoxes::new();
        boxes.insert(a, layout_box(0.0));
        let c = container(root, 0.0, 10.0, 100.0);
        let mut offsets = ScrollOffsets::new();
        offsets.refold(FoldInputs {
            tree: &tree,
            boxes: &boxes,
            containers: &[c],
        });
        // It returned at all; and no container on the (cyclic) chain means no shift.
        assert_eq!(offsets.projection(&boxes)[&a].bounds.origin.y, 0.0);
    }

    #[test]
    fn a_seated_offset_is_read_back_floored_at_zero() {
        // `seat` is the caret-follow / virtual-reset route: it writes a value its
        // caller already clamped, and `stored` is what the virtual paths read.
        let node = NodeId::from_raw(3);
        let mut offsets = ScrollOffsets::new();
        assert_eq!(offsets.stored(node), 0.0, "an untouched node is at the top");
        offsets.seat(node, 42.0);
        assert_eq!(offsets.stored(node), 42.0);
        offsets.seat(node, 0.0);
        assert_eq!(offsets.stored(node), 0.0);
    }
}
