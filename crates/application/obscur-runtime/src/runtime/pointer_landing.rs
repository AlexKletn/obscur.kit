#![doc = include_str!("pointer_landing.doc/_module.md")]

use obscur_geometry::Point;
use obscur_identity::NodeId;
use obscur_interaction::pointer::{PointerEvent, PointerId};
use obscur_tree::tree::RetainedTree;

use super::{HitTesting, PointerSessions, ScrollOffsets, ScrollTargets, SurfaceGeometry};
use crate::orchestration::{ComputedStyles, LayoutBoxes};

#[doc = include_str!("pointer_landing.doc/PointerLanding.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PointerLanding {
    #[doc = include_str!("pointer_landing.doc/screen.md")]
    pub(crate) screen: Point,
    #[doc = include_str!("pointer_landing.doc/folded.md")]
    pub(crate) folded: Point,
    #[doc = include_str!("pointer_landing.doc/target.md")]
    pub(crate) target: Option<NodeId>,
}

#[doc = include_str!("pointer_landing.doc/PointerLandingResolver.md")]
pub(crate) struct PointerLandingResolver<'a> {
    #[doc = include_str!("pointer_landing.doc/surface.md")]
    surface: SurfaceGeometry,
    #[doc = include_str!("pointer_landing.doc/tree.md")]
    tree: &'a RetainedTree,
    #[doc = include_str!("pointer_landing.doc/computed.md")]
    computed: &'a ComputedStyles,
    #[doc = include_str!("pointer_landing.doc/boxes.md")]
    boxes: &'a LayoutBoxes,
    #[doc = include_str!("pointer_landing.doc/scroll.md")]
    scroll: &'a ScrollOffsets,
    #[doc = include_str!("pointer_landing.doc/hits.md")]
    hits: &'a HitTesting,
}

impl<'a> PointerLandingResolver<'a> {
    #[doc = include_str!("pointer_landing.doc/new.md")]
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

    #[doc = include_str!("pointer_landing.doc/resolve.md")]
    pub(crate) fn resolve(
        &self,
        event: &PointerEvent,
        pointers: &mut PointerSessions,
    ) -> PointerLanding {
        let screen = self.surface.to_content_point(event.position());
        let folded = ScrollTargets::new(self.tree, self.computed, self.boxes, self.scroll)
            .fold_point(screen);
        let target = self.captured_target(pointers, event.pointer()).or_else(|| {
            self.hits
                .resolve(self.tree, self.computed, self.boxes, folded, screen)
        });
        PointerLanding {
            screen,
            folded,
            target,
        }
    }

    #[doc = include_str!("pointer_landing.doc/captured_target.md")]
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
            ComputedStyles::new(),
            LayoutBoxes::new(),
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
