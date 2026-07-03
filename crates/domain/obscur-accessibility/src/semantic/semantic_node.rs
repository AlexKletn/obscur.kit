//! One node of the semantic accessibility tree (spec §7.1).

use obscur_geometry::Rect;
use obscur_identity::NodeId;

use super::{SemanticAction, SemanticRelation, SemanticRole, SemanticState, SemanticValue};

/// One node of the semantic accessibility tree (spec §7.1).
///
/// A semantic node describes a single retained-tree node — identified by its
/// [`node`](Self::node) [`NodeId`] — for assistive technology. It bundles the
/// node's [`role`](Self::role), accessible [`name`](Self::name) and
/// [`description`](Self::description), [`state`](Self::state),
/// [`value`](Self::value), supported [`actions`](Self::actions), typed
/// [`relations`](Self::relations) to other nodes, on-screen
/// [`bounds`](Self::bounds) for hit-testing and highlighting, and a
/// [`decorative`](Self::decorative) flag.
///
/// Platform adapters project this data onto UI Automation, AX, AT-SPI, or web
/// accessibility APIs. A node that [`is_ignored`](Self::is_ignored) is not
/// exposed to assistive technology at all.
#[derive(Debug, Clone, PartialEq)]
pub struct SemanticNode {
    /// The retained-tree node this semantic node describes.
    pub node: NodeId,
    /// The node's semantic role.
    pub role: SemanticRole,
    /// The accessible name, if any.
    pub name: Option<Box<str>>,
    /// The accessible description, if any.
    pub description: Option<Box<str>>,
    /// The node's state flags.
    pub state: SemanticState,
    /// The node's current value.
    pub value: SemanticValue,
    /// The actions the node supports or can receive.
    pub actions: Vec<SemanticAction>,
    /// Typed relations from this node to others.
    pub relations: Vec<SemanticRelation>,
    /// On-screen bounds for hit-testing and highlighting, if known.
    pub bounds: Option<Rect>,
    /// Whether the node is purely decorative and hidden from assistive tech
    /// entirely (spec §7.1 decorative hiding).
    pub decorative: bool,
}

impl SemanticNode {
    /// Creates a node for the retained [`NodeId`] with the given role and every
    /// other field defaulted or empty (spec §7.1).
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

    /// Returns a copy with the accessible [`name`](Self::name) set.
    #[must_use]
    pub fn with_name(mut self, name: impl Into<Box<str>>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Returns a copy with the accessible [`description`](Self::description) set.
    #[must_use]
    pub fn with_description(mut self, description: impl Into<Box<str>>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Returns a copy with the [`state`](Self::state) replaced.
    #[must_use]
    pub fn with_state(mut self, state: SemanticState) -> Self {
        self.state = state;
        self
    }

    /// Returns a copy with the [`value`](Self::value) replaced.
    #[must_use]
    pub fn with_value(mut self, value: SemanticValue) -> Self {
        self.value = value;
        self
    }

    /// Returns a copy with the on-screen [`bounds`](Self::bounds) set.
    #[must_use]
    pub fn with_bounds(mut self, bounds: Rect) -> Self {
        self.bounds = Some(bounds);
        self
    }

    /// Returns a copy with the [`decorative`](Self::decorative) flag set.
    #[must_use]
    pub fn decorative(mut self, decorative: bool) -> Self {
        self.decorative = decorative;
        self
    }

    /// Appends a supported [`SemanticAction`] to this node.
    pub fn add_action(&mut self, action: SemanticAction) {
        self.actions.push(action);
    }

    /// Returns a copy with `action` appended to [`actions`](Self::actions).
    #[must_use]
    pub fn with_action(mut self, action: SemanticAction) -> Self {
        self.add_action(action);
        self
    }

    /// Appends a [`SemanticRelation`] to this node.
    pub fn add_relation(&mut self, relation: SemanticRelation) {
        self.relations.push(relation);
    }

    /// Returns a copy with `relation` appended to
    /// [`relations`](Self::relations).
    #[must_use]
    pub fn with_relation(mut self, relation: SemanticRelation) -> Self {
        self.add_relation(relation);
        self
    }

    /// Returns the retained-tree [`NodeId`] this node describes.
    #[must_use]
    pub const fn node(&self) -> NodeId {
        self.node
    }

    /// Returns the node's [`role`](Self::role).
    #[must_use]
    pub const fn role(&self) -> SemanticRole {
        self.role
    }

    /// Returns the accessible name, if any.
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Returns the accessible description, if any.
    #[must_use]
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Returns the node's [`state`](Self::state).
    #[must_use]
    pub const fn state(&self) -> &SemanticState {
        &self.state
    }

    /// Returns the node's [`value`](Self::value).
    #[must_use]
    pub const fn value(&self) -> &SemanticValue {
        &self.value
    }

    /// Returns the node's supported actions.
    #[must_use]
    pub fn actions(&self) -> &[SemanticAction] {
        &self.actions
    }

    /// Returns the node's relations.
    #[must_use]
    pub fn relations(&self) -> &[SemanticRelation] {
        &self.relations
    }

    /// Returns the on-screen bounds, if known.
    #[must_use]
    pub const fn bounds(&self) -> Option<Rect> {
        self.bounds
    }

    /// Returns whether the node supports the given [`SemanticAction`].
    #[must_use]
    pub fn supports_action(&self, action: SemanticAction) -> bool {
        self.actions.contains(&action)
    }

    /// Returns whether the node is not exposed to assistive technology.
    ///
    /// A node is ignored when it is [`decorative`](Self::decorative) or its
    /// [`state.hidden`](SemanticState::hidden) flag is set (spec §7.1
    /// decorative hiding). Adapters skip ignored nodes entirely.
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
