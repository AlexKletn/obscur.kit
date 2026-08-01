#![doc = include_str!("accessibility.doc/_module.md")]

use std::cell::RefCell;
use std::rc::Rc;

use obscur_accessibility::port::AccessibilityBridge;
use obscur_accessibility::tree::{SemanticSnapshot, SemanticTree};

use crate::runtime::SemanticTreeBuilder;

use super::UiRuntime;

impl UiRuntime {
    #[doc = include_str!("accessibility.doc/semantic_tree.md")]
    #[must_use]
    pub fn semantic_tree(&self) -> SemanticTree {
        SemanticTreeBuilder::build(&self.tree, self.focused)
    }

    #[doc = include_str!("accessibility.doc/semantic_snapshot.md")]
    #[must_use]
    pub fn semantic_snapshot(&self) -> SemanticSnapshot {
        SemanticTreeBuilder::snapshot(&self.tree, self.focused)
    }

    #[doc = include_str!("accessibility.doc/has_accessibility_bridge.md")]
    #[must_use]
    pub fn has_accessibility_bridge(&self) -> bool {
        self.accessibility.is_installed()
    }

    #[doc = include_str!("accessibility.doc/set_accessibility_bridge.md")]
    pub fn set_accessibility_bridge(&mut self, bridge: Rc<RefCell<dyn AccessibilityBridge>>) {
        self.accessibility.install(bridge);
        self.publish_accessibility_now();
    }

    #[doc = include_str!("accessibility.doc/push_accessibility.md")]
    pub fn push_accessibility(&mut self, bridge: &mut dyn AccessibilityBridge) {
        let focused = self.focused;
        let snapshot = self.semantic_snapshot();
        self.accessibility.push(bridge, focused, &snapshot);
    }

    #[doc = include_str!("accessibility.doc/publish_accessibility.md")]
    pub(super) fn publish_accessibility(&mut self, tree_changed: bool) {
        if !self
            .accessibility
            .should_publish(tree_changed, self.focused)
        {
            return;
        }
        self.publish_accessibility_now();
    }

    #[doc = include_str!("accessibility.doc/publish_accessibility_now.md")]
    fn publish_accessibility_now(&mut self) {
        // Taking the handle first keeps the `&mut self` push disjoint from the
        // publisher's own borrow; the bridge is shared with the host by
        // construction, so this is a refcount bump rather than a copy of anything.
        let Some(bridge) = self.accessibility.installed() else {
            return;
        };
        self.push_accessibility(&mut *bridge.borrow_mut());
    }
}
