//! The host-driven AccessKit implementation of `AccessibilityBridge` (§7.1).

use accesskit::{ActionData, ActionRequest, Role, TextPosition, TreeUpdate};
use obscur_accessibility::event::{AccessibilityAction, AccessibilityFocus};
use obscur_accessibility::port::AccessibilityBridge;
use obscur_accessibility::semantic::SemanticAction;
use obscur_accessibility::tree::{SemanticSnapshot, SemanticUpdate};
use obscur_identity::NodeId;

use crate::conversion::{ActionConverter, TreeUpdateConverter};

#[doc = include_str!("../../docs/api/bridge/accesskit_bridge.doc/AccessKitBridge.md")]
#[derive(Debug, Default)]
pub struct AccessKitBridge {
    converter: TreeUpdateConverter,
    last_update: Option<TreeUpdate>,
    tree_revision: u64,
    pending_actions: Vec<AccessibilityAction>,
    announcements: Vec<(Box<str>, bool)>,
    focus_target: Option<NodeId>,
    dirty: bool,
}

impl AccessKitBridge {
    /// Creates an empty bridge with no stored update, focus, or queued work.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[doc = include_str!("../../docs/api/bridge/accesskit_bridge.doc/last_tree_update.md")]
    #[must_use]
    pub const fn last_tree_update(&self) -> Option<&TreeUpdate> {
        self.last_update.as_ref()
    }

    /// Changes when a new tree update is available to the host.
    #[must_use]
    pub const fn tree_revision(&self) -> u64 {
        self.tree_revision
    }

    #[doc = include_str!("../../docs/api/bridge/accesskit_bridge.doc/is_dirty.md")]
    #[must_use]
    pub const fn is_dirty(&self) -> bool {
        self.dirty
    }

    #[doc = include_str!("../../docs/api/bridge/accesskit_bridge.doc/push_action.md")]
    pub fn push_action(&mut self, action: AccessibilityAction) {
        self.pending_actions.push(action);
    }

    /// Queues platform requests for activation, focus, scrolling into view or
    /// changing a text selection. Selection endpoints must refer to valid text
    /// positions in the target field's latest tree update. Returns false for
    /// unsupported actions or invalid argument payloads.
    pub fn push_action_request(&mut self, request: ActionRequest) -> bool {
        let Some(action) = ActionConverter::new().from_accesskit(request.action) else {
            return false;
        };
        if action == SemanticAction::SetTextSelection {
            let Some(ActionData::SetTextSelection(selection)) = request.data else {
                return false;
            };
            let Some(update) = &self.last_update else {
                return false;
            };
            let Some(anchor) = selection_offset(update, request.target, selection.anchor) else {
                return false;
            };
            let Some(focus) = selection_offset(update, request.target, selection.focus) else {
                return false;
            };
            self.push_action(
                AccessibilityAction::new(NodeId::from_raw(request.target.0), action)
                    .with_text_selection([anchor, focus]),
            );
            return true;
        }
        if request.data.is_some()
            || !matches!(
                action,
                SemanticAction::Activate | SemanticAction::Focus | SemanticAction::ScrollIntoView
            )
        {
            return false;
        }
        self.push_action(AccessibilityAction::new(
            NodeId::from_raw(request.target.0),
            action,
        ));
        true
    }

    #[doc = include_str!("../../docs/api/bridge/accesskit_bridge.doc/take_announcements.md")]
    #[must_use]
    pub fn take_announcements(&mut self) -> Vec<(Box<str>, bool)> {
        std::mem::take(&mut self.announcements)
    }
}

fn selection_offset(
    update: &TreeUpdate,
    field: accesskit::NodeId,
    position: TextPosition,
) -> Option<usize> {
    let node = |id| {
        update
            .nodes
            .iter()
            .find(|(key, _)| *key == id)
            .map(|(_, node)| node)
    };
    let field = node(field)?;
    field.text_selection()?;
    let mut offset = 0;
    for &id in field.children() {
        let run = node(id)?;
        if run.role() != Role::TextRun {
            continue;
        }
        if id == position.node {
            return Some(
                offset
                    + run
                        .character_lengths()
                        .get(..position.character_index)?
                        .iter()
                        .map(|&len| usize::from(len))
                        .sum::<usize>(),
            );
        }
        offset += run.value()?.len();
    }
    None
}

impl AccessibilityBridge for AccessKitBridge {
    fn update_tree(&mut self, snapshot: &SemanticSnapshot) {
        // A rootless snapshot cannot form an AccessKit tree; swallow the
        // `MissingRoot` error and keep the previous update rather than panic.
        if let Ok(update) = self
            .converter
            .convert_with_focus(snapshot, self.focus_target)
        {
            self.last_update = Some(update);
            self.tree_revision = self.tree_revision.wrapping_add(1);
            self.dirty = false;
        }
    }

    fn apply_update(&mut self, _update: &SemanticUpdate) {
        // The adapter does not build partial AccessKit updates; it marks itself
        // dirty so the host re-pushes a full snapshot on the next frame.
        self.dirty = true;
    }

    fn set_focus(&mut self, focus: &AccessibilityFocus) {
        self.focus_target = focus.focused();
        // Reflect focus into the pending update immediately so a host that reads
        // `last_tree_update` before the next push sees the new focus. AccessKit
        // requires `focus` to name a real node, so a cleared focus falls back to
        // the tree root.
        if let Some(update) = self.last_update.as_mut() {
            let node = self.converter.node_converter();
            update.focus = match focus.focused() {
                Some(id) => node.node_id(id),
                None => update.tree.as_ref().map_or(update.focus, |tree| tree.root),
            };
        }
    }

    fn announce(&mut self, message: &str, assertive: bool) {
        self.announcements.push((Box::from(message), assertive));
    }

    fn poll_actions(&mut self) -> Vec<AccessibilityAction> {
        std::mem::take(&mut self.pending_actions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use accesskit::NodeId as AkNodeId;
    use obscur_accessibility::semantic::{SemanticAction, SemanticNode, SemanticRole};
    use obscur_accessibility::tree::SemanticTree;

    fn snapshot() -> SemanticSnapshot {
        let mut tree = SemanticTree::new();
        tree.insert(
            SemanticNode::new(NodeId::from_raw(1), SemanticRole::Group),
            None,
        );
        tree.insert(
            SemanticNode::new(NodeId::from_raw(2), SemanticRole::Button).with_name("OK"),
            Some(NodeId::from_raw(1)),
        );
        tree.set_root(NodeId::from_raw(1));
        SemanticSnapshot::from_tree(&tree)
    }

    #[test]
    fn update_tree_stores_last_update() {
        let mut bridge = AccessKitBridge::new();
        assert!(bridge.last_tree_update().is_none());
        bridge.update_tree(&snapshot());
        let update = bridge.last_tree_update().expect("stored");
        assert_eq!(update.nodes.len(), 2);
        assert_eq!(update.focus, AkNodeId(1));
    }

    #[test]
    fn rootless_snapshot_leaves_last_update_none() {
        let mut tree = SemanticTree::new();
        tree.insert(
            SemanticNode::new(NodeId::from_raw(1), SemanticRole::Group),
            None,
        );
        let mut bridge = AccessKitBridge::new();
        bridge.update_tree(&SemanticSnapshot::from_tree(&tree));
        assert!(bridge.last_tree_update().is_none());
    }

    #[test]
    fn poll_actions_drains_pushed_actions() {
        let mut bridge = AccessKitBridge::new();
        bridge.push_action(AccessibilityAction::new(
            NodeId::from_raw(2),
            SemanticAction::Activate,
        ));
        let drained = bridge.poll_actions();
        assert_eq!(drained.len(), 1);
        assert_eq!(drained[0].target(), NodeId::from_raw(2));
        assert!(bridge.poll_actions().is_empty());
    }

    #[test]
    fn announce_buffers_until_taken() {
        let mut bridge = AccessKitBridge::new();
        bridge.announce("saved", false);
        bridge.announce("error", true);
        let taken = bridge.take_announcements();
        assert_eq!(taken.len(), 2);
        assert_eq!(taken[0], (Box::from("saved"), false));
        assert_eq!(taken[1], (Box::from("error"), true));
        assert!(bridge.take_announcements().is_empty());
    }

    #[test]
    fn apply_update_marks_dirty_and_push_clears_it() {
        let mut bridge = AccessKitBridge::new();
        bridge.update_tree(&snapshot());
        assert!(!bridge.is_dirty());
        bridge.apply_update(&SemanticUpdate::NodeChanged(NodeId::from_raw(2)));
        assert!(bridge.is_dirty());
        bridge.update_tree(&snapshot());
        assert!(!bridge.is_dirty());
    }

    #[test]
    fn set_focus_updates_stored_focus() {
        let mut bridge = AccessKitBridge::new();
        bridge.update_tree(&snapshot());
        bridge.set_focus(&AccessibilityFocus::moved_to(NodeId::from_raw(2), None));
        assert_eq!(
            bridge.last_tree_update().expect("stored").focus,
            AkNodeId(2)
        );
    }

    #[test]
    fn set_focus_persists_across_next_push() {
        let mut bridge = AccessKitBridge::new();
        bridge.set_focus(&AccessibilityFocus::moved_to(NodeId::from_raw(2), None));
        bridge.update_tree(&snapshot());
        assert_eq!(
            bridge.last_tree_update().expect("stored").focus,
            AkNodeId(2)
        );
    }

    #[test]
    fn cleared_focus_falls_back_to_root() {
        let mut bridge = AccessKitBridge::new();
        bridge.update_tree(&snapshot());
        bridge.set_focus(&AccessibilityFocus::moved_to(NodeId::from_raw(2), None));
        bridge.set_focus(&AccessibilityFocus::cleared(Some(NodeId::from_raw(2))));
        assert_eq!(
            bridge.last_tree_update().expect("stored").focus,
            AkNodeId(1)
        );
    }

    #[test]
    fn is_object_safe() {
        let _: Box<dyn AccessibilityBridge> = Box::new(AccessKitBridge::new());
    }
}
