//! `UiIrNode`: one node of the IR tree, carrying a stable id (spec §2.5).

use crate::document::{UiIrEvent, UiIrExpression, UiIrProperty};

/// The category of a [`UiIrNode`] (spec §2.5).
///
/// A node's kind fixes how it is interpreted: a raw element, a text leaf, a
/// component instance, a transparent fragment, or the control-flow
/// [`Conditional`](UiIrNodeKind::Conditional)/[`Loop`](UiIrNodeKind::Loop)
/// forms. Every kind except [`Text`](UiIrNodeKind::Text) can hold children;
/// [`is_container`](UiIrNodeKind::is_container) reports that.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiIrNodeKind {
    /// A raw element identified by tag.
    Element,
    /// A text leaf carrying literal content.
    Text,
    /// An instance of a named component.
    Component,
    /// A transparent grouping of children with no element of its own.
    Fragment,
    /// A node rendered conditionally.
    Conditional,
    /// A node repeated over a collection.
    Loop,
}

impl UiIrNodeKind {
    /// Returns whether nodes of this kind can hold children — every kind except
    /// [`Text`](UiIrNodeKind::Text).
    #[must_use]
    pub const fn is_container(self) -> bool {
        matches!(
            self,
            Self::Element | Self::Component | Self::Fragment | Self::Conditional | Self::Loop
        )
    }
}

/// One node of the UI IR tree, carrying a compiler-assigned stable id
/// (spec §2.5).
///
/// The IR tree mirrors the runtime view tree but is the serializable compile
/// artifact: every node has a stable [`id`](UiIrNode::id) (used by source maps
/// and diagnostics), a [`kind`](UiIrNode::kind), and — depending on kind — a
/// tag, text content, an optional stable [`key`](UiIrNode::key) (mandatory on a
/// [`Loop`](UiIrNodeKind::Loop) lowered from a `<For>`, spec §2.11), properties
/// flowed down, event bindings flowing up, and recursively nested children.
#[derive(Debug, Clone, PartialEq)]
pub struct UiIrNode {
    id: u64,
    kind: UiIrNodeKind,
    tag: Box<str>,
    text: Box<str>,
    key: Option<UiIrExpression>,
    properties: Vec<UiIrProperty>,
    events: Vec<UiIrEvent>,
    children: Vec<UiIrNode>,
}

impl UiIrNode {
    /// Creates an element node with stable `id` and element `tag`.
    #[must_use]
    pub fn element(id: u64, tag: impl Into<Box<str>>) -> Self {
        Self::bare(id, UiIrNodeKind::Element, tag.into(), Box::from(""))
    }

    /// Creates a component-instance node with stable `id` and component `name`.
    #[must_use]
    pub fn component(id: u64, name: impl Into<Box<str>>) -> Self {
        Self::bare(id, UiIrNodeKind::Component, name.into(), Box::from(""))
    }

    /// Creates a text node with stable `id` holding `content`.
    #[must_use]
    pub fn text(id: u64, content: impl Into<Box<str>>) -> Self {
        Self::bare(id, UiIrNodeKind::Text, Box::from(""), content.into())
    }

    /// Creates a fragment node with stable `id` and no tag or text.
    #[must_use]
    pub fn fragment(id: u64) -> Self {
        Self::bare(id, UiIrNodeKind::Fragment, Box::from(""), Box::from(""))
    }

    /// Creates a [`Loop`](UiIrNodeKind::Loop) control-flow node with stable `id`
    /// and control keyword `tag` (`"For"` or `"VirtualFor"`, spec §2.11).
    ///
    /// The loop's mandatory stable key is attached with [`with_key`](Self::with_key)
    /// and its iterated collection flows in as an `each` property; the repeated
    /// body is its children.
    #[must_use]
    pub fn loop_node(id: u64, tag: impl Into<Box<str>>) -> Self {
        Self::bare(id, UiIrNodeKind::Loop, tag.into(), Box::from(""))
    }

    /// Creates a [`Conditional`](UiIrNodeKind::Conditional) control-flow node with
    /// stable `id` and control keyword `tag` (`"If"`, `"ElseIf"` or `"Else"`,
    /// spec §2.11).
    ///
    /// The branch condition flows in as a `cond` property (absent on a bare
    /// `<Else>`); the guarded body is its children.
    #[must_use]
    pub fn conditional(id: u64, tag: impl Into<Box<str>>) -> Self {
        Self::bare(id, UiIrNodeKind::Conditional, tag.into(), Box::from(""))
    }

    /// Builds a node with empty property/event/child collections and no key.
    fn bare(id: u64, kind: UiIrNodeKind, tag: Box<str>, text: Box<str>) -> Self {
        Self {
            id,
            kind,
            tag,
            text,
            key: None,
            properties: Vec::new(),
            events: Vec::new(),
            children: Vec::new(),
        }
    }

    /// Attaches the stable key expression `key` to this node, returning the
    /// updated node.
    ///
    /// A [`Loop`](UiIrNodeKind::Loop) carries the mandatory `key={…}` of a
    /// `<For>` (spec §2.11) so reconciliation can preserve per-item identity; any
    /// keyed node may carry one.
    #[must_use]
    pub fn with_key(mut self, key: UiIrExpression) -> Self {
        self.key = Some(key);
        self
    }

    /// Appends `property`, returning the updated node.
    #[must_use]
    pub fn with_property(mut self, property: UiIrProperty) -> Self {
        self.properties.push(property);
        self
    }

    /// Appends `event`, returning the updated node.
    #[must_use]
    pub fn with_event(mut self, event: UiIrEvent) -> Self {
        self.events.push(event);
        self
    }

    /// Appends `child`, returning the updated node.
    #[must_use]
    pub fn with_child(mut self, child: UiIrNode) -> Self {
        self.children.push(child);
        self
    }

    /// Appends all nodes from `children`, returning the updated node.
    #[must_use]
    pub fn with_children(mut self, children: impl IntoIterator<Item = UiIrNode>) -> Self {
        self.children.extend(children);
        self
    }

    /// Returns this node's stable id.
    #[must_use]
    pub const fn id(&self) -> u64 {
        self.id
    }

    /// Returns this node's kind.
    #[must_use]
    pub const fn kind(&self) -> UiIrNodeKind {
        self.kind
    }

    /// Returns the element tag or component name (empty for text/fragment
    /// nodes).
    #[must_use]
    pub fn tag(&self) -> &str {
        &self.tag
    }

    /// Returns this node's stable key expression, if one is attached.
    ///
    /// A [`Loop`](UiIrNodeKind::Loop) lowered from a `<For key={…}>` carries the
    /// key here (spec §2.11); most other nodes have none.
    #[must_use]
    pub const fn key(&self) -> Option<&UiIrExpression> {
        self.key.as_ref()
    }

    /// Returns the text content for text nodes (empty for non-text nodes).
    ///
    /// Named `content` rather than `text` because [`text`](UiIrNode::text) is
    /// the text-node constructor; the two cannot share a name.
    #[must_use]
    pub fn content(&self) -> &str {
        &self.text
    }

    /// Returns the properties flowed down into this node.
    #[must_use]
    pub fn properties(&self) -> &[UiIrProperty] {
        &self.properties
    }

    /// Returns the event bindings on this node.
    #[must_use]
    pub fn events(&self) -> &[UiIrEvent] {
        &self.events
    }

    /// Returns this node's direct children.
    #[must_use]
    pub fn children(&self) -> &[UiIrNode] {
        &self.children
    }

    /// Returns the number of direct children.
    #[must_use]
    pub fn child_count(&self) -> usize {
        self.children.len()
    }

    /// Returns whether this node has no children.
    #[must_use]
    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }

    /// Returns the value expression of the property named `name`, if present.
    #[must_use]
    pub fn property(&self, name: &str) -> Option<&UiIrExpression> {
        self.properties
            .iter()
            .find(|property| property.name() == name)
            .map(UiIrProperty::value)
    }

    /// Returns the total number of nodes in the subtree rooted at this node —
    /// this node plus every descendant.
    #[must_use]
    pub fn node_count(&self) -> usize {
        1 + self
            .children
            .iter()
            .map(UiIrNode::node_count)
            .sum::<usize>()
    }

    /// Finds the node with stable `id` in the subtree rooted here, searching
    /// depth-first. Returns `None` if no node has that id.
    #[must_use]
    pub fn find(&self, id: u64) -> Option<&UiIrNode> {
        if self.id == id {
            return Some(self);
        }
        self.children.iter().find_map(|child| child.find(id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::UiIrValue;

    #[test]
    fn element_component_text_and_fragment_constructors() {
        let el = UiIrNode::element(1, "div");
        assert_eq!(el.id(), 1);
        assert_eq!(el.kind(), UiIrNodeKind::Element);
        assert_eq!(el.tag(), "div");
        assert_eq!(el.content(), "");

        let comp = UiIrNode::component(2, "Button");
        assert_eq!(comp.kind(), UiIrNodeKind::Component);
        assert_eq!(comp.tag(), "Button");

        let txt = UiIrNode::text(3, "hi");
        assert_eq!(txt.kind(), UiIrNodeKind::Text);
        assert_eq!(txt.content(), "hi");
        assert_eq!(txt.tag(), "");

        let frag = UiIrNode::fragment(4);
        assert_eq!(frag.kind(), UiIrNodeKind::Fragment);
    }

    #[test]
    fn builders_append_properties_events_and_children() {
        let node = UiIrNode::element(1, "div")
            .with_property(UiIrProperty::literal("x", UiIrValue::Int(1)))
            .with_event(UiIrEvent::new("click", "submit"))
            .with_child(UiIrNode::text(2, "a"))
            .with_children([UiIrNode::text(3, "b"), UiIrNode::text(4, "c")]);
        assert_eq!(node.properties().len(), 1);
        assert_eq!(node.events().len(), 1);
        assert_eq!(node.child_count(), 3);
        assert!(!node.is_leaf());
    }

    #[test]
    fn property_lookup_by_name() {
        let node = UiIrNode::element(1, "div")
            .with_property(UiIrProperty::literal("count", UiIrValue::Int(5)));
        assert_eq!(node.property("count"), Some(&UiIrExpression::int(5)));
        assert_eq!(node.property("missing"), None);
    }

    #[test]
    fn node_count_counts_whole_subtree() {
        let tree = UiIrNode::element(1, "div")
            .with_child(UiIrNode::text(2, "a"))
            .with_child(UiIrNode::text(3, "b"));
        assert_eq!(tree.node_count(), 3);
    }

    #[test]
    fn find_hits_and_misses() {
        let tree = UiIrNode::element(1, "div")
            .with_child(UiIrNode::element(2, "span").with_child(UiIrNode::text(3, "deep")));
        assert_eq!(tree.find(1).map(UiIrNode::id), Some(1));
        assert_eq!(tree.find(3).map(UiIrNode::id), Some(3));
        assert!(tree.find(99).is_none());
    }

    #[test]
    fn is_leaf_reports_no_children() {
        assert!(UiIrNode::text(1, "x").is_leaf());
        assert!(
            !UiIrNode::element(1, "div")
                .with_child(UiIrNode::text(2, "y"))
                .is_leaf()
        );
    }

    #[test]
    fn kind_container_predicate() {
        assert!(UiIrNodeKind::Element.is_container());
        assert!(UiIrNodeKind::Fragment.is_container());
        assert!(!UiIrNodeKind::Text.is_container());
    }

    #[test]
    fn loop_node_carries_kind_tag_and_key() {
        let node = UiIrNode::loop_node(1, "For").with_key(UiIrExpression::binding("row.id"));
        assert_eq!(node.kind(), UiIrNodeKind::Loop);
        assert_eq!(node.tag(), "For");
        assert_eq!(node.key(), Some(&UiIrExpression::binding("row.id")));
    }

    #[test]
    fn conditional_node_carries_kind_and_tag() {
        let node = UiIrNode::conditional(1, "If");
        assert_eq!(node.kind(), UiIrNodeKind::Conditional);
        assert_eq!(node.tag(), "If");
        assert_eq!(node.key(), None);
    }
}
