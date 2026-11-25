//! [`ScrollOffsets`]: every scroll container's live vertical offset, and the
//! viewport-relative box projection derived from it.

use std::collections::HashMap;

use obscur_geometry::Point;
use obscur_identity::{FastMap, NodeId};
use obscur_tree::tree::RetainedTree;

use super::FoldInputs;
use super::scroll_motion::ScrollMotion;
use crate::orchestration::{LayoutBoxes, ScrollContainer};

#[doc = include_str!("../../docs/api/runtime/scroll_offsets.doc/ScrollOffsets.md")]
pub(crate) struct ScrollOffsets {
    /// Each scroll container's stored vertical offset in logical CSS px. Stored
    /// unclamped-by-later-layout (see the type doc) and floored at zero on every
    /// write.
    offsets: FastMap<NodeId, Point>,
    /// In-flight programmatic scrolls keyed by container.
    motions: FastMap<NodeId, ScrollMotion>,
    /// Accumulated offset changes awaiting the next motion tick.
    motion_deltas: FastMap<NodeId, Point>,
    /// Measured scroll velocities from the latest positive motion tick.
    velocities: FastMap<NodeId, Point>,
    /// The scroll-chain-adjusted (viewport-relative) view of the last folded
    /// boxes, which the public geometry accessor exposes for observers (spec
    /// §4.4). Populated only while [`folded_active`](Self::folded_active) is set —
    /// i.e. some scroll container overflows — so an unscrolled tree keeps
    /// returning the content-space boxes by reference with no per-frame clone.
    folded: LayoutBoxes,
    /// Whether [`folded`](Self::folded) currently holds a distinct (scroll-folded)
    /// map. `false` when no scroll container overflows, in which case
    /// [`projection`](Self::projection) returns the content-space boxes unchanged.
    folded_active: bool,
}

impl ScrollOffsets {
    /// An unscrolled document: no container has an offset, and the projection is
    /// inactive, so [`projection`](Self::projection) hands back whatever boxes it
    /// is given.
    pub(crate) fn new() -> Self {
        Self {
            offsets: HashMap::default(),
            motions: HashMap::default(),
            motion_deltas: HashMap::default(),
            velocities: HashMap::default(),
            folded: LayoutBoxes::default(),
            folded_active: false,
        }
    }

    #[doc = include_str!("../../docs/api/runtime/scroll_offsets.doc/retain_live.md")]
    pub(crate) fn retain_live(&mut self, tree: &RetainedTree) {
        self.retain_nodes(|node| tree.contains(node));
    }

    /// Discards stored offsets and motion for nodes outside the retained scroll set.
    pub(crate) fn retain_nodes(&mut self, keep: impl Fn(NodeId) -> bool) {
        self.offsets.retain(|node, _| keep(*node));
        self.motions.retain(|node, _| keep(*node));
        self.motion_deltas.retain(|node, _| keep(*node));
        self.velocities.retain(|node, _| keep(*node));
    }

    #[cfg(test)]
    /// Counts stored node scroll offsets for retention tests.
    pub(crate) fn stored_offset_count(&self) -> usize {
        self.offsets.len()
    }

    /// The scroll container `c`'s stored offset, re-clamped to `c`'s **live**
    /// range — so a layout change that shrinks the content never surfaces a stale
    /// over-scroll.
    pub(crate) fn clamped(&self, c: ScrollContainer) -> f32 {
        self.offset(c).y
    }

    pub(crate) fn offset(&self, c: ScrollContainer) -> Point {
        let offset = self.offsets.get(&c.node).copied().unwrap_or_default();
        Point::new(
            c.clamp_left(offset.x),
            offset.y.clamp(0.0, c.max_scroll_top()),
        )
    }

    /// Returns a container’s latest sampled velocity in logical pixels per second.
    pub(crate) fn velocity(&self, node: NodeId) -> Point {
        self.velocities.get(&node).copied().unwrap_or_default()
    }

    /// Publishes the combined displacement since the previous motion tick.
    pub(crate) fn sample_velocities(&mut self, elapsed: f32) {
        if !elapsed.is_finite() || elapsed <= 0.0 {
            return;
        }
        self.velocities.clear();
        self.velocities
            .extend(self.motion_deltas.drain().filter_map(|(node, delta)| {
                let velocity = Point::new(delta.x / elapsed, delta.y / elapsed);
                (velocity.x.is_finite() && velocity.y.is_finite()).then_some((node, velocity))
            }));
    }

    /// Accumulates scrolling displacement for the next velocity sample.
    fn record_motion(
        deltas: &mut FastMap<NodeId, Point>,
        node: NodeId,
        before: Point,
        after: Point,
    ) {
        if before != after {
            let delta = deltas.entry(node).or_default();
            delta.x += after.x - before.x;
            delta.y += after.y - before.y;
        }
    }

    /// Discards motion samples after a programmatic jump.
    fn clear_velocity(&mut self, node: NodeId) {
        self.motion_deltas.remove(&node);
        self.velocities.remove(&node);
    }

    /// Starts or replaces the motion for the container.
    pub(crate) fn animate(&mut self, container: ScrollContainer, offset: Point, duration: f32) {
        let start = self.offset(container);
        let end = Point::new(
            container.clamp_left(offset.x),
            offset.y.clamp(0.0, container.max_scroll_top()),
        );
        self.motions.remove(&container.node);
        if start != end {
            self.motions
                .insert(container.node, ScrollMotion::new(start, end, duration));
        }
    }

    /// Reports whether any programmatic scrolls are in flight.
    pub(crate) fn has_motion(&self) -> bool {
        !self.motions.is_empty()
    }

    /// Samples active programmatic scrolls and refreshes folded geometry once.
    pub(crate) fn advance_motion(&mut self, delta: f32, fold: FoldInputs<'_>) -> bool {
        let mut sampled = false;
        self.motions.retain(|node, motion| {
            let Some(container) = fold
                .containers
                .iter()
                .find(|container| container.node == *node)
            else {
                return false;
            };
            let before = self.offsets.get(node).copied().unwrap_or_default();
            let (offset, active) = motion.advance(delta);
            let clamp = |point: Point| {
                Point::new(
                    container.clamp_left(point.x),
                    point.y.clamp(0.0, container.max_scroll_top()),
                )
            };
            let offset = clamp(offset);
            self.offsets.insert(*node, offset);
            Self::record_motion(&mut self.motion_deltas, *node, clamp(before), offset);
            sampled = true;
            active
        });
        if sampled {
            self.refold(fold);
        }
        sampled
    }

    pub(crate) fn shift_offset(
        &mut self,
        c: ScrollContainer,
        delta: Point,
        fold: FoldInputs<'_>,
    ) -> Point {
        let offset = self.shift_unfolded(c, delta);
        self.refold(fold);
        offset
    }

    /// Shifts one container and leaves folded geometry refresh to the caller.
    pub(crate) fn shift_unfolded(&mut self, c: ScrollContainer, delta: Point) -> Point {
        let current = self.offset(c);
        let offset = Point::new(
            if delta.x.is_finite() {
                c.clamp_left(current.x + delta.x)
            } else {
                current.x
            },
            if delta.y.is_finite() {
                (current.y + delta.y).clamp(0.0, c.max_scroll_top())
            } else {
                current.y
            },
        );
        if offset != current {
            self.motions.remove(&c.node);
        }
        Self::record_motion(&mut self.motion_deltas, c.node, current, offset);
        let stored = self.offsets.entry(c.node).or_default();
        if delta.x.is_finite() && delta.x != 0.0 {
            stored.x = offset.x;
        }
        if delta.y.is_finite() && delta.y != 0.0 {
            stored.y = offset.y;
        }
        offset
    }

    pub(crate) fn set_left(&mut self, c: ScrollContainer, px: f32, fold: FoldInputs<'_>) {
        if px.is_finite() {
            self.motions.remove(&c.node);
            self.clear_velocity(c.node);
            self.offsets.entry(c.node).or_default().x = c.clamp_left(px);
            self.refold(fold);
        }
    }

    /// Clamps both persisted axes to the current content and refolds once.
    pub(crate) fn restore(&mut self, c: ScrollContainer, offset: Point, fold: FoldInputs<'_>) {
        self.restore_unfolded(c, offset);
        self.refold(fold);
    }

    /// Restores an offset before the caller refreshes folded geometry.
    pub(crate) fn restore_unfolded(&mut self, c: ScrollContainer, offset: Point) {
        self.motions.remove(&c.node);
        self.clear_velocity(c.node);
        self.offsets.insert(
            c.node,
            Point::new(
                c.clamp_left(offset.x),
                offset.y.clamp(0.0, c.max_scroll_top()),
            ),
        );
    }

    #[doc = include_str!("../../docs/api/runtime/scroll_offsets.doc/stored.md")]
    pub(crate) fn stored(&self, node: NodeId) -> f32 {
        self.offsets
            .get(&node)
            .map_or(0.0, |offset| offset.y.max(0.0))
    }

    #[doc = include_str!("../../docs/api/runtime/scroll_offsets.doc/set.md")]
    pub(crate) fn set(&mut self, c: ScrollContainer, px: f32, fold: FoldInputs<'_>) {
        if !px.is_finite() {
            return;
        }
        self.motions.remove(&c.node);
        self.clear_velocity(c.node);
        self.offsets.entry(c.node).or_default().y = px.clamp(0.0, c.max_scroll_top());
        self.refold(fold);
    }

    /// Moves `c`'s offset by `dy` logical px (positive = toward the content's
    /// end), clamped to the scrollable range, refolds the projection against
    /// `fold`, and returns the resulting offset.
    pub(crate) fn shift(&mut self, c: ScrollContainer, dy: f32, fold: FoldInputs<'_>) -> f32 {
        self.motions.remove(&c.node);
        self.clear_velocity(c.node);
        let next = (self.clamped(c) + dy).clamp(0.0, c.max_scroll_top());
        self.offsets.entry(c.node).or_default().y = next;
        self.refold(fold);
        next
    }

    #[doc = include_str!("../../docs/api/runtime/scroll_offsets.doc/seat.md")]
    pub(crate) fn seat(&mut self, node: NodeId, px: f32) {
        self.motions.remove(&node);
        self.clear_velocity(node);
        self.offsets.entry(node).or_default().y = px;
    }

    #[doc = include_str!("../../docs/api/runtime/scroll_offsets.doc/refold.md")]
    pub(crate) fn refold(&mut self, fold: FoldInputs<'_>) {
        if fold.containers.is_empty() {
            self.folded_active = false;
            self.folded.clear();
            return;
        }
        // The live scroll offset of each overflowing scroll container, so an
        // ancestor walk can add every crossed container's contribution.
        let scroll_of: FastMap<NodeId, Point> = fold
            .containers
            .iter()
            .map(|c| (c.node, self.offset(*c)))
            .collect();
        let mut folded = fold.boxes.clone();
        let ancestor_walk_limit = fold.tree.len();
        for (&node, layout_box) in &mut folded {
            let mut offset = Point::ORIGIN;
            let mut current = node;
            // Fold every scroll-container ancestor, exclusive of the node itself: a
            // scroll container reports its own unscrolled position.
            for _ in 0..ancestor_walk_limit {
                let Some(parent) = fold.tree.visual_parent_of(current) else {
                    break;
                };
                if let Some(parent_offset) = scroll_of.get(&parent) {
                    offset.x += parent_offset.x;
                    offset.y += parent_offset.y;
                }
                current = parent;
            }
            layout_box.bounds.origin.x -= offset.x;
            layout_box.bounds.origin.y -= offset.y;
        }
        self.folded = folded;
        self.folded_active = true;
    }

    #[doc = include_str!("../../docs/api/runtime/scroll_offsets.doc/projection.md")]
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
            content_width: 100.0,
            rtl: false,
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
        let boxes = LayoutBoxes::default();
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
        let boxes = LayoutBoxes::default();
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
        let boxes = LayoutBoxes::default();
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
        let boxes = LayoutBoxes::default();
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
        let mut boxes = LayoutBoxes::default();
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
        let mut boxes = LayoutBoxes::default();
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
        let mut boxes = LayoutBoxes::default();
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
        let mut boxes = LayoutBoxes::default();
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
