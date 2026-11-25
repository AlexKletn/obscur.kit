#![doc = include_str!("../../../docs/api/runtime/ui_runtime/accessibility.doc/_module.md")]

use std::cell::RefCell;
use std::rc::Rc;

use obscur_accessibility::event::AccessibilityAction;
use obscur_accessibility::port::AccessibilityBridge;
use obscur_accessibility::semantic::SemanticAction;
use obscur_accessibility::tree::{SemanticSnapshot, SemanticTree};
use obscur_geometry::{AffineTransform, Point, Rect};
use obscur_identity::InternedSymbol;
use obscur_interaction::event::{EventOrigin, UiEvent};
use obscur_view::view::ViewValue;

use crate::frame::FrameReason;
use crate::orchestration::scroll_containers;

use crate::runtime::{AncestorChain, SemanticTreeBuilder};

use super::UiRuntime;

impl UiRuntime {
    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/accessibility.doc/semantic_tree.md")]
    #[must_use]
    pub fn semantic_tree(&self) -> SemanticTree {
        let mut tree = SemanticTreeBuilder::build_visible(&self.tree, self.focused, |node| {
            !self.is_css_hidden(node)
        });
        if let Some(session) = &self.editing
            && self.tree.get(session.node()).is_some_and(|node| {
                !node.props().iter().any(|prop| {
                    prop.name() == InternedSymbol::named("type")
                        && matches!(prop.value(), ViewValue::Text(value) if value.eq_ignore_ascii_case("password"))
                })
            })
        {
            let selection = session.selection();
            tree.project_editing(
                session.node(),
                session.text(),
                [selection.anchor(), selection.focus()],
            );
        }
        let scrolls = scroll_containers(&self.tree, &self.computed, &self.boxes);
        let scale = self.device_scale();
        let physical = AffineTransform::scale(scale, scale);
        tree.project_bounds(|node| {
            let size = self.boxes.get(&node)?.size();
            let transform = physical.multiply(self.presented_local_to_window(node, &scrolls)?);
            let bounds = transform.map_rect(Rect::new(Point::ORIGIN, size));
            [
                bounds.origin.x,
                bounds.origin.y,
                bounds.right(),
                bounds.bottom(),
            ]
            .iter()
            .all(|value| value.is_finite())
            .then_some(bounds)
        });
        tree
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/accessibility.doc/semantic_snapshot.md")]
    #[must_use]
    pub fn semantic_snapshot(&self) -> SemanticSnapshot {
        SemanticSnapshot::from_tree(&self.semantic_tree())
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/accessibility.doc/has_accessibility_bridge.md")]
    #[must_use]
    pub fn has_accessibility_bridge(&self) -> bool {
        self.accessibility.is_installed()
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/accessibility.doc/set_accessibility_bridge.md")]
    pub fn set_accessibility_bridge(&mut self, bridge: Rc<RefCell<dyn AccessibilityBridge>>) {
        self.accessibility.install(bridge);
        self.publish_accessibility_now();
    }

    /// Drains the host bridge and executes supported actions on the retained UI. Returns the number of actions that changed the UI or ran a command. Hosts wake the UI loop after queuing an action.
    pub fn poll_accessibility_actions(&mut self) -> usize {
        let Some(bridge) = self.accessibility.installed() else {
            return 0;
        };
        let actions = bridge.borrow_mut().poll_actions();
        if actions.is_empty() {
            return 0;
        }
        let snapshot = self.semantic_snapshot();
        actions
            .iter()
            .filter(|action| self.dispatch_accessibility_action(&snapshot, action))
            .count()
    }

    fn dispatch_accessibility_action(
        &mut self,
        snapshot: &SemanticSnapshot,
        request: &AccessibilityAction,
    ) -> bool {
        let node = request.target();
        let Some(semantic) = snapshot.get(node) else {
            return false;
        };
        let state = semantic.state();
        if state.disabled || !self.focus_trap_admits(Some(node)) {
            return false;
        }
        if AncestorChain::of(&self.tree, Some(node))
            .into_iter()
            .any(|ancestor| {
                snapshot
                    .get(ancestor)
                    .is_some_and(|node| node.state().hidden)
            })
        {
            return false;
        }
        let action = request.action();
        if action != SemanticAction::ScrollIntoView && !semantic.supports_action(action) {
            return false;
        }
        let event = UiEvent::new(EventOrigin::Accessibility, node).as_trusted();
        let handled = match action {
            SemanticAction::Activate => self.activate(node, event),
            SemanticAction::Focus => self.move_focus_to(Some(node), event),
            SemanticAction::ScrollIntoView => self.scroll_into_view(node),
            SemanticAction::SetTextSelection => {
                let Some([anchor, focus]) = request.text_selection() else {
                    return false;
                };
                if !self.editing.as_ref().is_some_and(|session| {
                    session.node() == node
                        && session.text().is_char_boundary(anchor)
                        && session.text().is_char_boundary(focus)
                }) {
                    return false;
                }
                self.apply_edit(obscur_text::editing::EditCommand::SetSelection(
                    obscur_text::selection::TextSelection::new(anchor, focus),
                ));
                true
            }
            _ => false,
        };
        if handled {
            self.request_frame(FrameReason::Input);
        }
        handled
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/accessibility.doc/push_accessibility.md")]
    pub fn push_accessibility(&mut self, bridge: &mut dyn AccessibilityBridge) {
        let focused = self.focused;
        let snapshot = self.semantic_snapshot();
        self.accessibility.push(bridge, focused, &snapshot);
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/accessibility.doc/publish_accessibility.md")]
    pub(super) fn publish_accessibility(&mut self, tree_changed: bool) {
        if !self
            .accessibility
            .should_publish(tree_changed, self.focused)
        {
            return;
        }
        self.publish_accessibility_now();
    }

    /// Pushes to the installed bridge unconditionally; a no-op with none
    /// installed.
    fn publish_accessibility_now(&mut self) {
        if self.accessibility.is_installed() {
            let snapshot = self.semantic_snapshot();
            self.accessibility.publish(self.focused, snapshot);
        }
    }
}
