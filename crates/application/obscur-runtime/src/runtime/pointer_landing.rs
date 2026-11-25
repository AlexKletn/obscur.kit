//! Where a pointer event landed: [`PointerLanding`], resolved by
//! [`PointerLandingResolver`].

use super::pointer_landing_result::PointerLanding;

use obscur_geometry::Point;
use obscur_identity::{FastMap, NodeId};
use obscur_interaction::pointer::{PointerEvent, PointerId};
use obscur_interaction::spatial::{HitChain, HitTest};
use obscur_tree::tree::RetainedTree;

use super::{
    AncestorChain, HitTesting, PointerSessions, ScrollOffsets, ScrollTargets, SurfaceGeometry,
};
use crate::orchestration::{ComputedStyles, LayoutBoxes, effective_stacking};

#[doc = include_str!("../../docs/api/runtime/pointer_landing.doc/PointerLandingResolver.md")]
pub(crate) struct PointerLandingResolver<'a> {
    /// The surface the event's physical coordinates are mapped through.
    surface: SurfaceGeometry,
    /// The retained tree the capture is validated against and the hit test runs
    /// over.
    tree: &'a RetainedTree,
    /// This frame's computed styles: which boxes clip, and how.
    computed: &'a ComputedStyles,
    /// This frame's arranged boxes, in unscrolled layout space.
    boxes: &'a LayoutBoxes,
    /// Every scroll container's live vertical offset.
    scroll: &'a ScrollOffsets,
    /// The z-order index the topmost hit is resolved through.
    hits: &'a HitTesting,
}

impl<'a> PointerLandingResolver<'a> {
    /// Bundles the retained frame and its live offsets for one event.
    pub(crate) const fn new(
        surface: SurfaceGeometry,
        tree: &'a RetainedTree,
        computed: &'a ComputedStyles,
        boxes: &'a LayoutBoxes,
        scroll: &'a ScrollOffsets,
        hits: &'a HitTesting,
    ) -> Self {
        Self {
            surface,
            tree,
            computed,
            boxes,
            scroll,
            hits,
        }
    }

    #[doc = include_str!("../../docs/api/runtime/pointer_landing.doc/resolve.md")]
    pub(crate) fn resolve(
        &self,
        event: &PointerEvent,
        pointers: &mut PointerSessions,
    ) -> PointerLanding {
        let screen = self.surface.to_content_point(event.position());
        let targets = ScrollTargets::new(self.tree, self.computed, self.boxes, self.scroll);
        let scrolls = self.scroll_points(&targets);
        let target = self
            .captured_target(pointers, event.pointer())
            .or_else(|| self.target_at(screen, &scrolls));
        // The fold belongs to the node that took the event, not to the pointer:
        // a fixed toolbar over a scrolled list, or the scroll container's own
        // border, rides no scroll and must be told where the pointer really is.
        let folded = target.map_or_else(
            || targets.fold_point(screen),
            |node| HitTesting::content_point(self.tree, &scrolls, node, screen),
        );
        PointerLanding {
            screen,
            folded,
            target,
        }
    }

    /// Queries the visual target and maps it to its logical event ancestry.
    pub(crate) fn hit_chain(&self, position: Point) -> HitChain {
        let screen = self.surface.to_content_point(position);
        let targets = ScrollTargets::new(self.tree, self.computed, self.boxes, self.scroll);
        let scrolls = self.scroll_points(&targets);
        let Some(target) = self.target_at(screen, &scrolls) else {
            return HitChain::new();
        };
        let stacking = effective_stacking(self.tree, self.computed);
        HitChain::from_hits(
            AncestorChain::of(self.tree, Some(target))
                .into_iter()
                .map(|node| {
                    let folded = HitTesting::folded_point(self.tree, &scrolls, node, screen);
                    let point = self.hits.local_point(node, folded);
                    let origin = self
                        .boxes
                        .get(&node)
                        .map_or(Point::ORIGIN, |bounds| bounds.origin());
                    let local = Point::new(point.x - origin.x, point.y - origin.y);
                    HitTest::new(node, local, stacking.z.get(&node).copied().unwrap_or(0))
                })
                .collect(),
        )
    }

    /// Resolves the visual target at a content-space point.
    fn target_at(&self, screen: Point, scrolls: &FastMap<NodeId, Point>) -> Option<NodeId> {
        self.hits
            .resolve(self.tree, self.computed, self.boxes, screen, scrolls)
    }

    /// Resolves the visual offsets of scroll containers.
    fn scroll_points(&self, targets: &ScrollTargets<'_>) -> FastMap<NodeId, Point> {
        targets
            .paint(self.computed)
            .into_iter()
            .map(|(node, paint)| (node, paint.offset()))
            .collect()
    }

    #[doc = include_str!("../../docs/api/runtime/pointer_landing.doc/captured_target.md")]
    fn captured_target(
        &self,
        pointers: &mut PointerSessions,
        pointer: PointerId,
    ) -> Option<NodeId> {
        pointers.live_capture_target(pointer, self.tree)
    }
}

#[cfg(test)]
mod tests {
    use obscur_geometry::Size;
    use obscur_interaction::pointer::{PointerDevice, PointerEventKind};
    use obscur_tree::node::NodeKind;

    use super::*;

    /// A frame with nothing in it: no tree, no boxes, no scroll container, and a
    /// surface at scale `1.0` with no content offset.
    fn empty() -> (
        RetainedTree,
        ComputedStyles,
        LayoutBoxes,
        ScrollOffsets,
        HitTesting,
    ) {
        (
            RetainedTree::new(),
            ComputedStyles::default(),
            LayoutBoxes::default(),
            ScrollOffsets::new(),
            HitTesting::new(),
        )
    }

    fn down_at(point: Point) -> PointerEvent {
        PointerEvent::new(
            PointerId::MOUSE,
            PointerDevice::Mouse,
            PointerEventKind::Down,
            point,
        )
    }

    #[test]
    fn an_empty_frame_lands_the_point_unchanged_on_nothing() {
        let (tree, computed, boxes, scroll, hits) = empty();
        let surface = SurfaceGeometry::new(Size::new(800.0, 600.0));
        let resolver =
            PointerLandingResolver::new(surface, &tree, &computed, &boxes, &scroll, &hits);
        let mut pointers = PointerSessions::new();
        let landing = resolver.resolve(&down_at(Point::new(7.0, 11.0)), &mut pointers);
        // Both spaces coincide with no scroll container and no surface transform,
        // which is what keeps a non-scrolling document's hit test byte-identical
        // to one built before scroll containers existed.
        assert_eq!(landing.screen, Point::new(7.0, 11.0));
        assert_eq!(landing.folded, Point::new(7.0, 11.0));
        assert_eq!(landing.target, None);
    }

    #[test]
    fn a_capture_targets_its_captor_wherever_the_pointer_is() {
        let (mut tree, computed, boxes, scroll, hits) = empty();
        let node = tree.alloc(NodeKind::Root);
        tree.set_root(node);
        let surface = SurfaceGeometry::new(Size::new(800.0, 600.0));
        let mut pointers = PointerSessions::new();
        pointers.capture(PointerId::MOUSE, node);
        let landing =
            PointerLandingResolver::new(surface, &tree, &computed, &boxes, &scroll, &hits)
                .resolve(&down_at(Point::new(9999.0, 9999.0)), &mut pointers);
        // The index is empty, so the only way to land on `node` is the capture.
        assert_eq!(landing.target, Some(node));
    }

    /// `root → [scroller(overflow: scroll, 200x400) → item(200x2000),
    /// toolbar(position: fixed, 200x50)]`, scrolled by `offset`.
    fn scrolled_page_with_fixed_toolbar(
        offset: f32,
    ) -> (
        RetainedTree,
        ComputedStyles,
        LayoutBoxes,
        ScrollOffsets,
        HitTesting,
        NodeId,
        NodeId,
    ) {
        use crate::orchestration::scroll_containers;
        use crate::runtime::FoldInputs;
        use obscur_layout::model::LayoutBox;
        use obscur_style::computed::ComputedStyle;
        use obscur_style::computed::ComputedValue;
        use obscur_style::property::{PropertyId, layout_keyword};
        use obscur_tree::node::LogicalParent;

        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let kid = |tree: &mut RetainedTree, parent: NodeId| {
            let id = tree.alloc(NodeKind::Element {
                component: obscur_identity::ComponentId::from_raw(1),
            });
            tree.get_mut(id)
                .expect("just allocated")
                .set_logical_parent(LogicalParent::of(parent));
            tree.get_mut(parent).expect("in tree").push_child(id);
            id
        };
        let scroller = kid(&mut tree, root);
        let item = kid(&mut tree, scroller);
        let toolbar = kid(&mut tree, root);

        let mut boxes = LayoutBoxes::default();
        let at = |w: f32, h: f32| {
            LayoutBox::new(obscur_geometry::Rect::new(
                Point::new(0.0, 0.0),
                Size::new(w, h),
            ))
        };
        boxes.insert(scroller, at(200.0, 400.0));
        boxes.insert(item, at(200.0, 2000.0));
        boxes.insert(toolbar, at(200.0, 50.0));

        let keyword = |id: PropertyId, sym| {
            let mut style = ComputedStyle::new();
            style.set(id, ComputedValue::keyword(sym));
            style
        };
        let mut computed = ComputedStyles::default();
        computed.insert(
            scroller,
            keyword(PropertyId::OverflowY, layout_keyword::OVERFLOW_SCROLL),
        );
        computed.insert(item, ComputedStyle::new());
        computed.insert(
            toolbar,
            keyword(PropertyId::Position, layout_keyword::POSITION_FIXED),
        );

        let containers = scroll_containers(&tree, &computed, &boxes);
        let container = *containers
            .iter()
            .find(|c| c.node == scroller)
            .expect("the scroller overflows, so it is a scroll container");
        let mut scroll = ScrollOffsets::new();
        scroll.set(
            container,
            offset,
            FoldInputs {
                tree: &tree,
                boxes: &boxes,
                containers: &containers,
            },
        );

        let mut hits = HitTesting::new();
        hits.rebuild(&tree, &computed, &boxes);
        (tree, computed, boxes, scroll, hits, item, toolbar)
    }

    #[test]
    fn a_fixed_toolbar_takes_the_click_where_it_is_drawn_at_any_offset() {
        // The whole-frame fold put the probe at (50, 20 + offset) — off the
        // toolbar entirely — and the scrolled row behind it took the click. It
        // also reported that shifted point as the event's position, so even the
        // handler that did run was told the wrong place.
        for offset in [0.0_f32, 120.0, 900.0] {
            let (tree, computed, boxes, scroll, hits, item, toolbar) =
                scrolled_page_with_fixed_toolbar(offset);
            let surface = SurfaceGeometry::new(Size::new(400.0, 400.0));
            let resolver =
                PointerLandingResolver::new(surface, &tree, &computed, &boxes, &scroll, &hits);
            let mut pointers = PointerSessions::new();

            let on_toolbar = Point::new(50.0, 20.0);
            let landing = resolver.resolve(&down_at(on_toolbar), &mut pointers);
            assert_eq!(landing.target, Some(toolbar), "at scroll offset {offset}");
            assert_eq!(
                landing.folded, on_toolbar,
                "the toolbar rides no scroll, so its event is where the pointer \
                 is (offset {offset})"
            );

            // The control: the scrolled content below the toolbar still folds,
            // both for the hit and for the point the handler is told.
            let below = Point::new(50.0, 300.0);
            let landing = resolver.resolve(&down_at(below), &mut pointers);
            assert_eq!(landing.target, Some(item), "at scroll offset {offset}");
            assert_eq!(landing.folded, Point::new(50.0, 300.0 + offset));
        }
    }

    #[test]
    fn a_capture_on_a_node_that_left_the_tree_releases_and_falls_back() {
        let (tree, computed, boxes, scroll, hits) = empty();
        let surface = SurfaceGeometry::new(Size::new(800.0, 600.0));
        let mut pointers = PointerSessions::new();
        // A node id that was never in this tree stands for one that has been
        // removed from it: §2.4 releases the capture rather than swallowing the
        // event.
        pointers.capture(PointerId::MOUSE, NodeId::from_raw(42));
        let landing =
            PointerLandingResolver::new(surface, &tree, &computed, &boxes, &scroll, &hits)
                .resolve(&down_at(Point::new(1.0, 2.0)), &mut pointers);
        assert_eq!(landing.target, None);
        assert_eq!(pointers.capture_target(PointerId::MOUSE), None);
    }
}
