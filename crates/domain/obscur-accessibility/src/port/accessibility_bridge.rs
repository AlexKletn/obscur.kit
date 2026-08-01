#![doc = include_str!("accessibility_bridge.doc/_module.md")]

use crate::event::{AccessibilityAction, AccessibilityFocus};
use crate::tree::{SemanticSnapshot, SemanticUpdate};

#[doc = include_str!("accessibility_bridge.doc/AccessibilityBridge.md")]
pub trait AccessibilityBridge {
    #[doc = include_str!("accessibility_bridge.doc/update_tree.md")]
    fn update_tree(&mut self, snapshot: &SemanticSnapshot);

    #[doc = include_str!("accessibility_bridge.doc/apply_update.md")]
    fn apply_update(&mut self, update: &SemanticUpdate);

    #[doc = include_str!("accessibility_bridge.doc/set_focus.md")]
    fn set_focus(&mut self, focus: &AccessibilityFocus);

    #[doc = include_str!("accessibility_bridge.doc/announce.md")]
    fn announce(&mut self, message: &str, assertive: bool);

    #[doc = include_str!("accessibility_bridge.doc/poll_actions.md")]
    fn poll_actions(&mut self) -> Vec<AccessibilityAction>;
}

#[doc = include_str!("accessibility_bridge.doc/NullAccessibilityBridge.md")]
#[derive(Debug, Default, Clone)]
pub struct NullAccessibilityBridge;

impl NullAccessibilityBridge {
    #[doc = include_str!("accessibility_bridge.doc/new.md")]
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl AccessibilityBridge for NullAccessibilityBridge {
    fn update_tree(&mut self, _snapshot: &SemanticSnapshot) {}

    fn apply_update(&mut self, _update: &SemanticUpdate) {}

    fn set_focus(&mut self, _focus: &AccessibilityFocus) {}

    fn announce(&mut self, _message: &str, _assertive: bool) {}

    fn poll_actions(&mut self) -> Vec<AccessibilityAction> {
        Vec::new()
    }
}

#[doc = include_str!("accessibility_bridge.doc/RecordingAccessibilityBridge.md")]
#[derive(Debug, Default, Clone)]
pub struct RecordingAccessibilityBridge {
    #[doc = include_str!("accessibility_bridge.doc/tree_updates.md")]
    pub tree_updates: u64,
    #[doc = include_str!("accessibility_bridge.doc/updates_applied.md")]
    pub updates_applied: u64,
    #[doc = include_str!("accessibility_bridge.doc/last_announcement.md")]
    pub last_announcement: Option<Box<str>>,
    #[doc = include_str!("accessibility_bridge.doc/last_focus.md")]
    pub last_focus: Option<AccessibilityFocus>,
    #[doc = include_str!("accessibility_bridge.doc/queued.md")]
    queued: Vec<AccessibilityAction>,
}

impl RecordingAccessibilityBridge {
    #[doc = include_str!("accessibility_bridge.doc/new_2.md")]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[doc = include_str!("accessibility_bridge.doc/queue_action.md")]
    pub fn queue_action(&mut self, action: AccessibilityAction) {
        self.queued.push(action);
    }
}

impl AccessibilityBridge for RecordingAccessibilityBridge {
    fn update_tree(&mut self, _snapshot: &SemanticSnapshot) {
        self.tree_updates += 1;
    }

    fn apply_update(&mut self, _update: &SemanticUpdate) {
        self.updates_applied += 1;
    }

    fn set_focus(&mut self, focus: &AccessibilityFocus) {
        self.last_focus = Some(*focus);
    }

    fn announce(&mut self, message: &str, _assertive: bool) {
        self.last_announcement = Some(Box::from(message));
    }

    fn poll_actions(&mut self) -> Vec<AccessibilityAction> {
        std::mem::take(&mut self.queued)
    }
}

#[cfg(test)]
mod tests {
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
