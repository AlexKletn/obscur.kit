use crate::event::{AccessibilityAction, AccessibilityFocus};
use crate::tree::{SemanticSnapshot, SemanticUpdate};

use super::AccessibilityBridge;

#[doc = include_str!("../../docs/api/port/accessibility_bridge.doc/RecordingAccessibilityBridge.md")]
#[derive(Debug, Default, Clone)]
pub struct RecordingAccessibilityBridge {
    /// The number of full-snapshot pushes received.
    pub tree_updates: u64,
    /// The number of incremental updates applied.
    pub updates_applied: u64,
    /// The most recent announcement message, if any.
    pub last_announcement: Option<Box<str>>,
    /// The most recent focus change reflected, if any.
    pub last_focus: Option<AccessibilityFocus>,
    /// The queued inbound action requests, drained by `poll_actions`.
    queued: Vec<AccessibilityAction>,
}

impl RecordingAccessibilityBridge {
    /// Creates an empty recording bridge with zeroed counters.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Queues an inbound `action` for a later
    /// [`poll_actions`](AccessibilityBridge::poll_actions) to drain.
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
