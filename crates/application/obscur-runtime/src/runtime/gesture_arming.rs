#![doc = include_str!("gesture_arming.doc/_module.md")]

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

#[doc = include_str!("gesture_arming.doc/GestureArming.md")]
pub(crate) struct GestureArming<'a> {
    #[doc = include_str!("gesture_arming.doc/tree.md")]
    tree: &'a RetainedTree,
    #[doc = include_str!("gesture_arming.doc/computed.md")]
    computed: &'a ComputedStyles,
    #[doc = include_str!("gesture_arming.doc/boxes.md")]
    boxes: &'a LayoutBoxes,
    #[doc = include_str!("gesture_arming.doc/scroll.md")]
    scroll: &'a ScrollOffsets,
    #[doc = include_str!("gesture_arming.doc/virtual_lists.md")]
    virtual_lists: &'a VirtualLists,
}

impl<'a> GestureArming<'a> {
    #[doc = include_str!("gesture_arming.doc/new.md")]
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

    #[doc = include_str!("gesture_arming.doc/pan_container.md")]
    pub(crate) fn pan_container(
        &self,
        event: &PointerEvent,
        screen_point: Point,
    ) -> Option<NodeId> {
        if !event.device().is_direct() {
            return None;
        }
        ScrollTargets::new(self.tree, self.computed, self.boxes, self.scroll)
            .scrollable_at(screen_point)
    }

    #[doc = include_str!("gesture_arming.doc/hold.md")]
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
        assert_eq!(arming.pan_container(&event, point), None);
        assert!(arming.hold(&event, point, NodeId::from_raw(1)).is_none());
    }

    #[test]
    fn a_touch_over_nothing_scrollable_and_nothing_bound_arms_nothing() {
        let (tree, computed, boxes, scroll, lists) = empty();
        let arming = GestureArming::new(&tree, &computed, &boxes, &scroll, &lists);
        let event = down(PointerDevice::Touch);
        let point = Point::new(4.0, 4.0);
        assert_eq!(arming.pan_container(&event, point), None);
        // Neither a bound `EVENT_LONG_PRESS` nor a holdable row: arming here would
        // spend half a second of animation frames to conclude nothing.
        assert!(arming.hold(&event, point, NodeId::from_raw(1)).is_none());
    }
}
