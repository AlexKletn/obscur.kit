//! What a contact going down arms, asked of the frame it landed on:
//! [`GestureArming`].

use obscur_geometry::Point;
use obscur_identity::NodeId;
use obscur_interaction::pointer::PointerEvent;
use obscur_tree::tree::RetainedTree;
use obscur_view::view::EVENT_LONG_PRESS;

use super::{
    ActiveHold, CommandDispatch, PointerSessions, ScrollOffsets, ScrollTargets, StructuralStates,
    VirtualLists,
};
use crate::orchestration::{ComputedStyles, LayoutBoxes};

#[doc = include_str!("../../docs/api/runtime/gesture_arming.doc/GestureArming.md")]
pub(crate) struct GestureArming<'a> {
    /// The tree the long-press binding and the disabled gate are resolved over.
    tree: &'a RetainedTree,
    /// This frame's computed styles: which boxes scroll, and how.
    computed: &'a ComputedStyles,
    /// This frame's arranged boxes, in unscrolled layout space.
    boxes: &'a LayoutBoxes,
    /// Every scroll container's live vertical offset.
    scroll: &'a ScrollOffsets,
    /// The realized virtual-list windows, which say which row a point is over.
    virtual_lists: &'a VirtualLists,
}

impl<'a> GestureArming<'a> {
    /// Bundles the retained frame and its live offsets for one contact.
    pub(crate) const fn new(
        tree: &'a RetainedTree,
        computed: &'a ComputedStyles,
        boxes: &'a LayoutBoxes,
        scroll: &'a ScrollOffsets,
        virtual_lists: &'a VirtualLists,
    ) -> Self {
        Self {
            tree,
            computed,
            boxes,
            scroll,
            virtual_lists,
        }
    }

    /// Find the horizontal and vertical scroll targets under the start of a direct contact.
    pub(crate) fn pan_containers(
        &self,
        event: &PointerEvent,
        screen_point: Point,
    ) -> [Option<NodeId>; 2] {
        if !event.device().is_direct() {
            return [None; 2];
        }
        let targets = ScrollTargets::new(self.tree, self.computed, self.boxes, self.scroll);
        [Point::new(1.0, 0.0), Point::new(0.0, 1.0)]
            .map(|delta| targets.scrollable_at(screen_point, delta))
    }

    #[doc = include_str!("../../docs/api/runtime/gesture_arming.doc/hold.md")]
    pub(crate) fn hold(
        &self,
        event: &PointerEvent,
        screen_point: Point,
        target: NodeId,
    ) -> Option<ActiveHold> {
        if !event.device().is_direct() {
            return None;
        }
        if StructuralStates::of(self.tree).is_within_disabled(target) {
            return None;
        }
        // Two ways to be holdable: a binding in the ancestor chain, or a realized
        // row of a virtual list whose owner registered a row handler. The second
        // is not a special case so much as the main one on a tablet — a file
        // listing's rows are exactly what a user wants to act on, and they are
        // precisely the nodes that cannot carry a binding.
        let row = self
            .virtual_lists
            .holdable_row_at(screen_point, self.boxes, self.scroll);
        if row.is_none()
            && CommandDispatch::binding_for(self.tree, target, EVENT_LONG_PRESS).is_none()
        {
            return None;
        }
        Some(PointerSessions::new_hold(
            event.pointer(),
            target,
            screen_point,
            event.trust().is_trusted(),
            row,
        ))
    }
}

#[cfg(test)]
mod tests {
    use obscur_interaction::pointer::{PointerDevice, PointerEventKind, PointerId};

    use super::*;

    /// A frame with nothing in it: no tree, no boxes, no scroll container and no
    /// realized list.
    fn empty() -> (
        RetainedTree,
        ComputedStyles,
        LayoutBoxes,
        ScrollOffsets,
        VirtualLists,
    ) {
        (
            RetainedTree::new(),
            ComputedStyles::default(),
            LayoutBoxes::default(),
            ScrollOffsets::new(),
            VirtualLists::new(),
        )
    }

    fn down(device: PointerDevice) -> PointerEvent {
        PointerEvent::new(
            PointerId::MOUSE,
            device,
            PointerEventKind::Down,
            Point::new(4.0, 4.0),
        )
    }

    #[test]
    fn a_mouse_arms_neither_gesture() {
        let (tree, computed, boxes, scroll, lists) = empty();
        let arming = GestureArming::new(&tree, &computed, &boxes, &scroll, &lists);
        let event = down(PointerDevice::Mouse);
        let point = Point::new(4.0, 4.0);
        // The wheel is the mouse's scroll gesture and its second button is what a
        // hold reaches, so the direct-only gate rules both out before the frame is
        // consulted at all.
        assert_eq!(arming.pan_containers(&event, point), [None; 2]);
        assert!(arming.hold(&event, point, NodeId::from_raw(1)).is_none());
    }

    #[test]
    fn a_touch_over_nothing_scrollable_and_nothing_bound_arms_nothing() {
        let (tree, computed, boxes, scroll, lists) = empty();
        let arming = GestureArming::new(&tree, &computed, &boxes, &scroll, &lists);
        let event = down(PointerDevice::Touch);
        let point = Point::new(4.0, 4.0);
        assert_eq!(arming.pan_containers(&event, point), [None; 2]);
        // Neither a bound `EVENT_LONG_PRESS` nor a holdable row: arming here would
        // spend half a second of animation frames to conclude nothing.
        assert!(arming.hold(&event, point, NodeId::from_raw(1)).is_none());
    }
}
