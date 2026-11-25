use crate::event::{AccessibilityAction, AccessibilityFocus};
use crate::tree::{SemanticSnapshot, SemanticUpdate};

use super::AccessibilityBridge;

#[doc = include_str!("../../docs/api/port/accessibility_bridge.doc/NullAccessibilityBridge.md")]
#[derive(Debug, Default, Clone)]
pub struct NullAccessibilityBridge;

impl NullAccessibilityBridge {
    /// Creates the no-op bridge.
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
