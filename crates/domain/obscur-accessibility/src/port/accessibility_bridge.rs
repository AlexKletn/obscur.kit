//! The replaceable port platform accessibility adapters implement (§7.1).

use crate::event::{AccessibilityAction, AccessibilityFocus};
use crate::tree::{SemanticSnapshot, SemanticUpdate};

#[doc = include_str!("../../docs/api/port/accessibility_bridge.doc/AccessibilityBridge.md")]
pub trait AccessibilityBridge {
    /// Pushes a full [`SemanticSnapshot`] to the platform accessibility tree.
    fn update_tree(&mut self, snapshot: &SemanticSnapshot);

    /// Applies a single incremental [`SemanticUpdate`] to the platform tree.
    fn apply_update(&mut self, update: &SemanticUpdate);

    /// Reflects an [`AccessibilityFocus`] change to the platform.
    fn set_focus(&mut self, focus: &AccessibilityFocus);

    /// Emits a live-region announcement. `assertive` requests an interrupting
    /// (assertive) announcement rather than a polite one.
    fn announce(&mut self, message: &str, assertive: bool);

    /// Drains and returns the inbound assistive-technology action requests
    /// queued since the last poll. Returns an empty [`Vec`] when none are
    /// pending.
    fn poll_actions(&mut self) -> Vec<AccessibilityAction>;
}

#[cfg(test)]
mod tests {
    use super::super::{NullAccessibilityBridge, RecordingAccessibilityBridge};
    use super::*;
    use crate::semantic::{SemanticAction, SemanticNode, SemanticRole};
    use crate::tree::SemanticTree;
    use obscur_identity::NodeId;

    fn snapshot() -> SemanticSnapshot {
        let mut tree = SemanticTree::new();
        tree.insert(
            SemanticNode::new(NodeId::from_raw(1), SemanticRole::Button),
            None,
        );
        SemanticSnapshot::from_tree(&tree)
    }

    #[test]
    fn accessibility_bridge_is_object_safe() {
        let _: Box<dyn AccessibilityBridge> = Box::new(NullAccessibilityBridge);
    }

    #[test]
    fn null_bridge_is_all_noops() {
        let mut bridge = NullAccessibilityBridge::new();
        bridge.update_tree(&snapshot());
        bridge.apply_update(&SemanticUpdate::NodeChanged(NodeId::from_raw(1)));
        bridge.set_focus(&AccessibilityFocus::cleared(None));
        bridge.announce("hi", true);
        assert!(bridge.poll_actions().is_empty());
    }

    #[test]
    fn recording_bridge_counts_and_captures() {
        let mut bridge = RecordingAccessibilityBridge::new();
        bridge.update_tree(&snapshot());
        bridge.update_tree(&snapshot());
        bridge.apply_update(&SemanticUpdate::NodeChanged(NodeId::from_raw(1)));
        bridge.set_focus(&AccessibilityFocus::moved_to(NodeId::from_raw(1), None));
        bridge.announce("saved", false);

        assert_eq!(bridge.tree_updates, 2);
        assert_eq!(bridge.updates_applied, 1);
        assert_eq!(bridge.last_announcement.as_deref(), Some("saved"));
        assert_eq!(
            bridge.last_focus,
            Some(AccessibilityFocus::moved_to(NodeId::from_raw(1), None))
        );
    }

    #[test]
    fn recording_bridge_queue_action_drains_in_order() {
        let mut bridge = RecordingAccessibilityBridge::new();
        bridge.queue_action(AccessibilityAction::new(
            NodeId::from_raw(1),
            SemanticAction::Activate,
        ));
        bridge.queue_action(AccessibilityAction::new(
            NodeId::from_raw(2),
            SemanticAction::Focus,
        ));

        let drained = bridge.poll_actions();
        assert_eq!(drained.len(), 2);
        assert_eq!(drained[0].target(), NodeId::from_raw(1));
        assert_eq!(drained[1].action(), SemanticAction::Focus);
        // Draining empties the queue.
        assert!(bridge.poll_actions().is_empty());
    }
}
