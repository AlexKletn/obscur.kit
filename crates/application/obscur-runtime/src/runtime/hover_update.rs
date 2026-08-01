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
    #[doc = include_str!("hover_update.doc/changed.md")]
    changed: bool,
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
        let hover_target = if event.device() == PointerDevice::Touch && contact_ended {
            None
        } else {
            target
        };
        let chain = HoverChain::from_nodes(AncestorChain::of(tree, hover_target).into_vec());
        let changed = HoverTransition::between(current, &chain).has_changes();
        Self { chain, changed }
    }

    #[doc = include_str!("hover_update.doc/changed_2.md")]
    pub(crate) const fn changed(&self) -> bool {
        self.changed
    }

    #[doc = include_str!("hover_update.doc/into_chain.md")]
    pub(crate) fn into_chain(self) -> HoverChain {
        self.chain
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
        assert!(update.into_chain().nodes().is_empty());
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
        assert_eq!(update.into_chain().nodes(), &[node]);
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
        assert_eq!(update.into_chain().nodes(), &[node]);
    }
}
