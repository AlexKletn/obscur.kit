#![doc = include_str!("semantic_node.doc/_module.md")]

use obscur_geometry::Rect;
use obscur_identity::NodeId;

use super::{SemanticAction, SemanticRelation, SemanticRole, SemanticState, SemanticValue};

#[doc = include_str!("semantic_node.doc/SemanticNode.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct SemanticNode {
    #[doc = include_str!("semantic_node.doc/node.md")]
    pub node: NodeId,
    #[doc = include_str!("semantic_node.doc/role.md")]
    pub role: SemanticRole,
    #[doc = include_str!("semantic_node.doc/name.md")]
    pub name: Option<Box<str>>,
    #[doc = include_str!("semantic_node.doc/description.md")]
    pub description: Option<Box<str>>,
    #[doc = include_str!("semantic_node.doc/state.md")]
    pub state: SemanticState,
    #[doc = include_str!("semantic_node.doc/value.md")]
    pub value: SemanticValue,
    #[doc = include_str!("semantic_node.doc/actions.md")]
    pub actions: Vec<SemanticAction>,
    #[doc = include_str!("semantic_node.doc/relations.md")]
    pub relations: Vec<SemanticRelation>,
    #[doc = include_str!("semantic_node.doc/bounds.md")]
    pub bounds: Option<Rect>,
    #[doc = include_str!("semantic_node.doc/decorative.md")]
    pub decorative: bool,
}

impl SemanticNode {
    #[doc = include_str!("semantic_node.doc/new.md")]
    #[must_use]
    pub fn new(node: NodeId, role: SemanticRole) -> Self {
        Self {
            node,
            role,
            name: None,
            description: None,
            state: SemanticState::new(),
            value: SemanticValue::None,
            actions: Vec::new(),
            relations: Vec::new(),
            bounds: None,
            decorative: false,
        }
    }

    #[doc = include_str!("semantic_node.doc/with_name.md")]
    #[must_use]
    pub fn with_name(mut self, name: impl Into<Box<str>>) -> Self {
        self.name = Some(name.into());
        self
    }

    #[doc = include_str!("semantic_node.doc/with_description.md")]
    #[must_use]
    pub fn with_description(mut self, description: impl Into<Box<str>>) -> Self {
        self.description = Some(description.into());
        self
    }

    #[doc = include_str!("semantic_node.doc/with_state.md")]
    #[must_use]
    pub fn with_state(mut self, state: SemanticState) -> Self {
        self.state = state;
        self
    }

    #[doc = include_str!("semantic_node.doc/with_value.md")]
    #[must_use]
    pub fn with_value(mut self, value: SemanticValue) -> Self {
        self.value = value;
        self
    }

    #[doc = include_str!("semantic_node.doc/with_bounds.md")]
    #[must_use]
    pub fn with_bounds(mut self, bounds: Rect) -> Self {
        self.bounds = Some(bounds);
        self
    }

    #[doc = include_str!("semantic_node.doc/decorative_2.md")]
    #[must_use]
    pub fn decorative(mut self, decorative: bool) -> Self {
        self.decorative = decorative;
        self
    }

    #[doc = include_str!("semantic_node.doc/add_action.md")]
    pub fn add_action(&mut self, action: SemanticAction) {
        self.actions.push(action);
    }

    #[doc = include_str!("semantic_node.doc/with_action.md")]
    #[must_use]
    pub fn with_action(mut self, action: SemanticAction) -> Self {
        self.add_action(action);
        self
    }

    #[doc = include_str!("semantic_node.doc/add_relation.md")]
    pub fn add_relation(&mut self, relation: SemanticRelation) {
        self.relations.push(relation);
    }

    #[doc = include_str!("semantic_node.doc/with_relation.md")]
    #[must_use]
    pub fn with_relation(mut self, relation: SemanticRelation) -> Self {
        self.add_relation(relation);
        self
    }

    #[doc = include_str!("semantic_node.doc/node_2.md")]
    #[must_use]
    pub const fn node(&self) -> NodeId {
        self.node
    }

    #[doc = include_str!("semantic_node.doc/role_2.md")]
    #[must_use]
    pub const fn role(&self) -> SemanticRole {
        self.role
    }

    #[doc = include_str!("semantic_node.doc/name_2.md")]
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    #[doc = include_str!("semantic_node.doc/description_2.md")]
    #[must_use]
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    #[doc = include_str!("semantic_node.doc/state_2.md")]
    #[must_use]
    pub const fn state(&self) -> &SemanticState {
        &self.state
    }

    #[doc = include_str!("semantic_node.doc/value_2.md")]
    #[must_use]
    pub const fn value(&self) -> &SemanticValue {
        &self.value
    }

    #[doc = include_str!("semantic_node.doc/actions_2.md")]
    #[must_use]
    pub fn actions(&self) -> &[SemanticAction] {
        &self.actions
    }

    #[doc = include_str!("semantic_node.doc/relations_2.md")]
    #[must_use]
    pub fn relations(&self) -> &[SemanticRelation] {
        &self.relations
    }

    #[doc = include_str!("semantic_node.doc/bounds_2.md")]
    #[must_use]
    pub const fn bounds(&self) -> Option<Rect> {
        self.bounds
    }

    #[doc = include_str!("semantic_node.doc/supports_action.md")]
    #[must_use]
    pub fn supports_action(&self, action: SemanticAction) -> bool {
        self.actions.contains(&action)
    }

    #[doc = include_str!("semantic_node.doc/is_ignored.md")]
    #[must_use]
    pub const fn is_ignored(&self) -> bool {
        self.decorative || self.state.hidden
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node() -> SemanticNode {
        SemanticNode::new(NodeId::from_raw(7), SemanticRole::Button)
    }

    #[test]
    fn new_defaults_empty() {
        let n = node();
        assert_eq!(n.node(), NodeId::from_raw(7));
        assert_eq!(n.role(), SemanticRole::Button);
        assert_eq!(n.name(), None);
        assert_eq!(n.description(), None);
        assert_eq!(n.state(), &SemanticState::new());
        assert_eq!(n.value(), &SemanticValue::None);
        assert!(n.actions().is_empty());
        assert!(n.relations().is_empty());
        assert_eq!(n.bounds(), None);
        assert!(!n.decorative);
        assert!(!n.is_ignored());
    }

    #[test]
    fn builder_populates_fields() {
        let n = node()
            .with_name("OK")
            .with_description("Confirm the dialog")
            .with_state(SemanticState::new().focusable(true))
            .with_value(SemanticValue::bool(true))
            .with_action(SemanticAction::Activate)
            .with_action(SemanticAction::Focus)
            .with_relation(SemanticRelation::LabelledBy(NodeId::from_raw(9)));
        assert_eq!(n.name(), Some("OK"));
        assert_eq!(n.description(), Some("Confirm the dialog"));
        assert!(n.state().focusable);
        assert_eq!(n.value(), &SemanticValue::Bool(true));
        assert_eq!(n.actions().len(), 2);
        assert_eq!(n.relations().len(), 1);
    }

    #[test]
    fn add_action_and_relation_mutate() {
        let mut n = node();
        n.add_action(SemanticAction::Activate);
        n.add_relation(SemanticRelation::Controls(NodeId::from_raw(3)));
        assert_eq!(n.actions(), &[SemanticAction::Activate]);
        assert_eq!(
            n.relations(),
            &[SemanticRelation::Controls(NodeId::from_raw(3))]
        );
    }

    #[test]
    fn supports_action_checks_membership() {
        let n = node().with_action(SemanticAction::Activate);
        assert!(n.supports_action(SemanticAction::Activate));
        assert!(!n.supports_action(SemanticAction::Focus));
    }

    #[test]
    fn is_ignored_when_decorative() {
        assert!(node().decorative(true).is_ignored());
    }

    #[test]
    fn is_ignored_when_hidden() {
        let n = node().with_state(SemanticState::new().hidden(true));
        assert!(n.is_ignored());
    }
}
