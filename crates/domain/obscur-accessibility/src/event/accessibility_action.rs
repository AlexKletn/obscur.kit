//! An inbound action request from assistive technology (spec §7.1).

use obscur_identity::NodeId;

use crate::semantic::{SemanticAction, SemanticValue};

#[doc = include_str!("../../docs/api/event/accessibility_action.doc/AccessibilityAction.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct AccessibilityAction {
    /// The retained node the action targets.
    target: NodeId,
    /// The requested action.
    action: SemanticAction,
    /// The value payload, present for value-setting actions.
    value: Option<SemanticValue>,
    text_selection: Option<[usize; 2]>,
}

impl AccessibilityAction {
    /// Creates a request for `action` on `target` with no value payload.
    #[must_use]
    pub const fn new(target: NodeId, action: SemanticAction) -> Self {
        Self {
            target,
            action,
            value: None,
            text_selection: None,
        }
    }

    /// Returns a copy with `value` attached as the payload (e.g. for
    /// [`SemanticAction::SetValue`]).
    #[must_use]
    pub fn with_value(mut self, value: SemanticValue) -> Self {
        self.value = Some(value);
        self
    }

    /// Attaches directional UTF-8 byte offsets in anchor/focus order.
    #[must_use]
    pub const fn with_text_selection(mut self, selection: [usize; 2]) -> Self {
        self.text_selection = Some(selection);
        self
    }

    /// Returns the directional UTF-8 byte offsets carried by this request.
    #[must_use]
    pub const fn text_selection(&self) -> Option<[usize; 2]> {
        self.text_selection
    }

    /// Returns the retained node the action targets.
    #[must_use]
    pub const fn target(&self) -> NodeId {
        self.target
    }

    /// Returns the requested action.
    #[must_use]
    pub const fn action(&self) -> SemanticAction {
        self.action
    }

    /// Returns the value payload, if any.
    #[must_use]
    pub const fn value(&self) -> Option<&SemanticValue> {
        self.value.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_has_no_value() {
        let a = AccessibilityAction::new(NodeId::from_raw(2), SemanticAction::Activate);
        assert_eq!(a.target(), NodeId::from_raw(2));
        assert_eq!(a.action(), SemanticAction::Activate);
        assert_eq!(a.value(), None);
    }

    #[test]
    fn with_value_attaches_payload() {
        let a = AccessibilityAction::new(NodeId::from_raw(2), SemanticAction::SetValue)
            .with_value(SemanticValue::text("hello"));
        assert_eq!(a.action(), SemanticAction::SetValue);
        assert_eq!(a.value(), Some(&SemanticValue::text("hello")));
    }
}
