//! The replaceable port platform accessibility adapters implement (§7.1).

use crate::event::{AccessibilityAction, AccessibilityFocus};
use crate::tree::{SemanticSnapshot, SemanticUpdate};

/// The replaceable port platform accessibility adapters implement (spec §7.1).
///
/// A single trait through which the pure accessibility domain drives the outside
/// platform accessibility API — UI Automation, AX, AT-SPI, or web a11y. An
/// adapter implements it once per platform and the domain stores it as
/// `Box<dyn AccessibilityBridge>`, so the trait is deliberately **object-safe**:
/// no generic methods and no `Self`-returning methods.
///
/// The bridge pushes tree state downward — a full [`SemanticSnapshot`] or an
/// incremental [`SemanticUpdate`] — reflects [`AccessibilityFocus`] and live
/// announcements, and drains inbound [`AccessibilityAction`] requests upward so
/// they can be dispatched through the same command/event model as ordinary
/// input.
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

/// A reference, no-op [`AccessibilityBridge`] for headless or unsupported
/// platforms (spec §7.1).
///
/// `NullAccessibilityBridge` owns nothing platform-specific: every downward push
/// ([`update_tree`](AccessibilityBridge::update_tree),
/// [`apply_update`](AccessibilityBridge::apply_update),
/// [`set_focus`](AccessibilityBridge::set_focus),
/// [`announce`](AccessibilityBridge::announce)) is a no-op, and
/// [`poll_actions`](AccessibilityBridge::poll_actions) returns nothing because
/// no assistive technology is attached. Its silence is correct behavior for a
/// platform without an accessibility API, not a stub awaiting implementation.
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

/// A deterministic recording [`AccessibilityBridge`] for tests (spec §7.1).
///
/// `RecordingAccessibilityBridge` observes what the domain pushes — counting
/// [`tree_updates`](Self::tree_updates) and [`updates_applied`](Self::updates_applied),
/// retaining the [`last_announcement`](Self::last_announcement) and
/// [`last_focus`](Self::last_focus) — and lets a test seed inbound requests with
/// [`queue_action`](Self::queue_action) so
/// [`poll_actions`](AccessibilityBridge::poll_actions) drains them in order. It
/// makes the accessibility command loop testable without any real platform.
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
