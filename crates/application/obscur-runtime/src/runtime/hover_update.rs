#![doc = include_str!("hover_update.doc/_module.md")]

use obscur_identity::NodeId;
use obscur_interaction::hover::{HoverChain, HoverTransition};
use obscur_interaction::pointer::{PointerDevice, PointerEvent, PointerEventKind};
use obscur_tree::tree::RetainedTree;

use super::AncestorChain;

#[doc = include_str!("hover_update.doc/HoverUpdate.md")]
#[derive(Debug, Clone)]
pub(crate) struct HoverUpdate {
    #[doc = include_str!("hover_update.doc/chain.md")]
    chain: HoverChain,
    #[doc = include_str!("hover_update.doc/crossed.md")]
    crossed: HoverTransition,
}

impl HoverUpdate {
    #[doc = include_str!("hover_update.doc/for_event.md")]
    pub(crate) fn for_event(
        tree: &RetainedTree,
        current: &HoverChain,
        event: &PointerEvent,
        target: Option<NodeId>,
    ) -> Self {
        let contact_ended = matches!(
            event.kind(),
            PointerEventKind::Up | PointerEventKind::Cancel
        );
        // A `Leave` means the pointer is off the surface, so nothing is under it
        // — whatever the hit test says about the last position it reported.
        // Without this the chain would keep the node the cursor was over as it
        // crossed the window edge, which is exactly the stuck `:hover` a host
        // that never delivered a leave event used to live with: the row the
        // pointer exited by stayed highlighted for as long as the window did.
        let left_surface = event.kind() == PointerEventKind::Leave;
        let hover_target =
            if left_surface || (event.device() == PointerDevice::Touch && contact_ended) {
                None
            } else {
                target
            };
        let chain = HoverChain::from_nodes(AncestorChain::of(tree, hover_target).into_vec());
        // The transition is KEPT rather than reduced to a bool. It was computed
        // and thrown away here for as long as `:hover` was the only thing that
        // cared; `pointerenter` / `pointerleave` are dispatched from exactly the
        // two lists it already holds, so carrying it out costs the caller
        // nothing it was not already paying.
        let crossed = HoverTransition::between(current, &chain);
        Self { chain, crossed }
    }

    #[doc = include_str!("hover_update.doc/changed_2.md")]
    pub(crate) fn changed(&self) -> bool {
        self.crossed.has_changes()
    }

    #[doc = include_str!("hover_update.doc/into_parts.md")]
    pub(crate) fn into_parts(self) -> (HoverChain, HoverTransition) {
        (self.chain, self.crossed)
    }
}

#[cfg(test)]
mod tests {
    use obscur_geometry::Point;
    use obscur_interaction::pointer::PointerId;

    use super::*;

    fn event(device: PointerDevice, kind: PointerEventKind) -> PointerEvent {
        PointerEvent::new(PointerId::MOUSE, device, kind, Point::new(1.0, 1.0))
    }

    #[test]
    fn a_lifted_finger_leaves_nothing_hovered() {
        let tree = RetainedTree::new();
        let node = NodeId::from_raw(1);
        let current = HoverChain::from_nodes(vec![node]);
        let update = HoverUpdate::for_event(
            &tree,
            &current,
            &event(PointerDevice::Touch, PointerEventKind::Up),
            Some(node),
        );
        // The lift still hit-tests to the row it lifted from; the finger has no
        // resting cursor, so nothing stays `:hover` after it.
        assert!(update.changed());
        let (chain, crossed) = update.into_parts();
        assert!(chain.nodes().is_empty());
        // And §2.4's "touch does not synthesize persistent hover" is what the
        // crossing events say too: the lift LEAVES the row it entered on contact,
        // so a tap is an enter and then a leave rather than a row left entered
        // for the rest of the session.
        assert_eq!(crossed.left(), &[node]);
        assert!(crossed.entered().is_empty());
    }

    #[test]
    fn a_pen_lifting_off_keeps_hovering_what_it_is_over() {
        let tree = RetainedTree::new();
        let node = NodeId::from_raw(1);
        let current = HoverChain::from_nodes(vec![node]);
        let update = HoverUpdate::for_event(
            &tree,
            &current,
            &event(PointerDevice::Pen, PointerEventKind::Up),
            Some(node),
        );
        // A pen hovers above the surface, so the same event leaves the chain
        // exactly where the hit test put it — and nothing changed, so no restyle.
        assert!(!update.changed());
        let (chain, crossed) = update.into_parts();
        assert_eq!(chain.nodes(), &[node]);
        // Nothing crossed a boundary, so nothing is entered or left either — a
        // pen resting still must not emit a crossing per report.
        assert!(crossed.is_empty());
    }

    /// The pointer crossing the window edge, which no host in this workspace
    /// used to report: `WindowEvent::CursorLeft` was unhandled, so `:hover`
    /// never cleared and the row the cursor exited by stayed highlighted.
    #[test]
    fn a_pointer_that_left_the_surface_hovers_nothing() {
        let tree = RetainedTree::new();
        let node = NodeId::from_raw(1);
        let current = HoverChain::from_nodes(vec![node]);
        // The hit test still answers with the node under the last reported
        // position — a leave carries no new one — so the target is deliberately
        // `Some` here. Honouring it is the bug.
        let update = HoverUpdate::for_event(
            &tree,
            &current,
            &event(PointerDevice::Mouse, PointerEventKind::Leave),
            Some(node),
        );

        assert!(update.changed());
        let (chain, crossed) = update.into_parts();
        assert!(chain.nodes().is_empty(), "nothing is under a gone pointer");
        assert_eq!(
            crossed.left(),
            &[node],
            "and the node it was over hears its own pointerleave"
        );
        assert!(crossed.entered().is_empty());
    }

    /// The other direction, so the rule above is "a leave empties the chain" and
    /// not "a mouse event empties the chain": the matching `Enter` seats what it
    /// landed on, because a pointer that arrived *is* over something.
    #[test]
    fn a_pointer_entering_the_surface_hovers_what_it_landed_on() {
        let tree = RetainedTree::new();
        let node = NodeId::from_raw(1);
        let update = HoverUpdate::for_event(
            &tree,
            &HoverChain::new(),
            &event(PointerDevice::Mouse, PointerEventKind::Enter),
            Some(node),
        );

        assert!(update.changed());
        let (chain, crossed) = update.into_parts();
        assert_eq!(chain.nodes(), &[node]);
        assert_eq!(crossed.entered(), &[node]);
    }

    #[test]
    fn a_touch_contact_still_hovers_while_it_is_down() {
        let tree = RetainedTree::new();
        let node = NodeId::from_raw(1);
        let update = HoverUpdate::for_event(
            &tree,
            &HoverChain::new(),
            &event(PointerDevice::Touch, PointerEventKind::Down),
            Some(node),
        );
        assert!(update.changed());
        let (chain, crossed) = update.into_parts();
        assert_eq!(chain.nodes(), &[node]);
        assert_eq!(crossed.entered(), &[node]);
    }
}
