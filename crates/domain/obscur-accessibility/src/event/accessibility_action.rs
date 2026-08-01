#![doc = include_str!("accessibility_action.doc/_module.md")]

use obscur_identity::NodeId;

use crate::semantic::{SemanticAction, SemanticValue};

#[doc = include_str!("accessibility_action.doc/AccessibilityAction.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct AccessibilityAction {
    #[doc = include_str!("accessibility_action.doc/target.md")]
    target: NodeId,
    #[doc = include_str!("accessibility_action.doc/action.md")]
    action: SemanticAction,
    #[doc = include_str!("accessibility_action.doc/value.md")]
    value: Option<SemanticValue>,
}

impl AccessibilityAction {
    #[doc = include_str!("accessibility_action.doc/new.md")]
    #[must_use]
    pub const fn new(target: NodeId, action: SemanticAction) -> Self {
        Self {
            target,
            action,
            value: None,
        }
    }

    #[doc = include_str!("accessibility_action.doc/with_value.md")]
    #[must_use]
    pub fn with_value(mut self, value: SemanticValue) -> Self {
        self.value = Some(value);
        self
    }

    #[doc = include_str!("accessibility_action.doc/target_2.md")]
    #[must_use]
    pub const fn target(&self) -> NodeId {
        self.target
    }

    #[doc = include_str!("accessibility_action.doc/action_2.md")]
    #[must_use]
    pub const fn action(&self) -> SemanticAction {
        self.action
    }

    #[doc = include_str!("accessibility_action.doc/value_2.md")]
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
