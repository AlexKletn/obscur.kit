#![doc = include_str!("ui_ir_node.doc/_module.md")]

use crate::document::{UiIrEvent, UiIrExpression, UiIrProperty};

#[doc = include_str!("ui_ir_node.doc/UiIrNodeKind.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiIrNodeKind {
    #[doc = include_str!("ui_ir_node.doc/Element.md")]
    Element,
    #[doc = include_str!("ui_ir_node.doc/Text.md")]
    Text,
    #[doc = include_str!("ui_ir_node.doc/Component.md")]
    Component,
    #[doc = include_str!("ui_ir_node.doc/Fragment.md")]
    Fragment,
    #[doc = include_str!("ui_ir_node.doc/Conditional.md")]
    Conditional,
    #[doc = include_str!("ui_ir_node.doc/Loop.md")]
    Loop,
}

impl UiIrNodeKind {
    #[doc = include_str!("ui_ir_node.doc/is_container.md")]
    #[must_use]
    pub const fn is_container(self) -> bool {
        matches!(
            self,
            Self::Element | Self::Component | Self::Fragment | Self::Conditional | Self::Loop
        )
    }
}

#[doc = include_str!("ui_ir_node.doc/INTERPOLATION_TAG.md")]
const INTERPOLATION_TAG: &str = "Expression";

#[doc = include_str!("ui_ir_node.doc/INTERPOLATION_VALUE.md")]
const INTERPOLATION_VALUE: &str = "value";

#[doc = include_str!("ui_ir_node.doc/UiIrNode.md")]
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
    #[doc = include_str!("ui_ir_node.doc/element.md")]
    #[must_use]
    pub fn element(id: u64, tag: impl Into<Box<str>>) -> Self {
        Self::bare(id, UiIrNodeKind::Element, tag.into(), Box::from(""))
    }

    #[doc = include_str!("ui_ir_node.doc/component.md")]
    #[must_use]
    pub fn component(id: u64, name: impl Into<Box<str>>) -> Self {
        Self::bare(id, UiIrNodeKind::Component, name.into(), Box::from(""))
    }

    #[doc = include_str!("ui_ir_node.doc/text.md")]
    #[must_use]
    pub fn text(id: u64, content: impl Into<Box<str>>) -> Self {
        Self::bare(id, UiIrNodeKind::Text, Box::from(""), content.into())
    }

    #[doc = include_str!("ui_ir_node.doc/fragment.md")]
    #[must_use]
    pub fn fragment(id: u64) -> Self {
        Self::bare(id, UiIrNodeKind::Fragment, Box::from(""), Box::from(""))
    }

    #[doc = include_str!("ui_ir_node.doc/loop_node.md")]
    #[must_use]
    pub fn loop_node(id: u64, tag: impl Into<Box<str>>) -> Self {
        Self::bare(id, UiIrNodeKind::Loop, tag.into(), Box::from(""))
    }

    #[doc = include_str!("ui_ir_node.doc/conditional.md")]
    #[must_use]
    pub fn conditional(id: u64, tag: impl Into<Box<str>>) -> Self {
        Self::bare(id, UiIrNodeKind::Conditional, tag.into(), Box::from(""))
    }

    #[doc = include_str!("ui_ir_node.doc/interpolation.md")]
    #[must_use]
    pub fn interpolation(id: u64, value: UiIrExpression) -> Self {
        Self::bare(
            id,
            UiIrNodeKind::Element,
            Box::from(INTERPOLATION_TAG),
            Box::from(""),
        )
        .with_property(UiIrProperty::new(INTERPOLATION_VALUE, value))
    }

    #[doc = include_str!("ui_ir_node.doc/interpolated_value.md")]
    #[must_use]
    pub fn interpolated_value(&self) -> Option<&UiIrExpression> {
        (self.kind == UiIrNodeKind::Element && &*self.tag == INTERPOLATION_TAG)
            .then(|| self.property(INTERPOLATION_VALUE))
            .flatten()
    }

    #[doc = include_str!("ui_ir_node.doc/bare.md")]
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

    #[doc = include_str!("ui_ir_node.doc/with_key.md")]
    #[must_use]
    pub fn with_key(mut self, key: UiIrExpression) -> Self {
        self.key = Some(key);
        self
    }

    #[doc = include_str!("ui_ir_node.doc/with_property.md")]
    #[must_use]
    pub fn with_property(mut self, property: UiIrProperty) -> Self {
        self.properties.push(property);
        self
    }

    #[doc = include_str!("ui_ir_node.doc/with_event.md")]
    #[must_use]
    pub fn with_event(mut self, event: UiIrEvent) -> Self {
        self.events.push(event);
        self
    }

    #[doc = include_str!("ui_ir_node.doc/with_child.md")]
    #[must_use]
    pub fn with_child(mut self, child: UiIrNode) -> Self {
        self.children.push(child);
        self
    }

    #[doc = include_str!("ui_ir_node.doc/with_children.md")]
    #[must_use]
    pub fn with_children(mut self, children: impl IntoIterator<Item = UiIrNode>) -> Self {
        self.children.extend(children);
        self
    }

    #[doc = include_str!("ui_ir_node.doc/id.md")]
    #[must_use]
    pub const fn id(&self) -> u64 {
        self.id
    }

    #[doc = include_str!("ui_ir_node.doc/kind.md")]
    #[must_use]
    pub const fn kind(&self) -> UiIrNodeKind {
        self.kind
    }

    #[doc = include_str!("ui_ir_node.doc/tag.md")]
    #[must_use]
    pub fn tag(&self) -> &str {
        &self.tag
    }

    #[doc = include_str!("ui_ir_node.doc/key.md")]
    #[must_use]
    pub const fn key(&self) -> Option<&UiIrExpression> {
        self.key.as_ref()
    }

    #[doc = include_str!("ui_ir_node.doc/content.md")]
    #[must_use]
    pub fn content(&self) -> &str {
        &self.text
    }

    #[doc = include_str!("ui_ir_node.doc/properties.md")]
    #[must_use]
    pub fn properties(&self) -> &[UiIrProperty] {
        &self.properties
    }

    #[doc = include_str!("ui_ir_node.doc/events.md")]
    #[must_use]
    pub fn events(&self) -> &[UiIrEvent] {
        &self.events
    }

    #[doc = include_str!("ui_ir_node.doc/children.md")]
    #[must_use]
    pub fn children(&self) -> &[UiIrNode] {
        &self.children
    }

    #[doc = include_str!("ui_ir_node.doc/child_count.md")]
    #[must_use]
    pub fn child_count(&self) -> usize {
        self.children.len()
    }

    #[doc = include_str!("ui_ir_node.doc/is_leaf.md")]
    #[must_use]
    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }

    #[doc = include_str!("ui_ir_node.doc/property.md")]
    #[must_use]
    pub fn property(&self, name: &str) -> Option<&UiIrExpression> {
        self.properties
            .iter()
            .find(|property| property.name() == name)
            .map(UiIrProperty::value)
    }

    #[doc = include_str!("ui_ir_node.doc/node_count.md")]
    #[must_use]
    pub fn node_count(&self) -> usize {
        1 + self
            .children
            .iter()
            .map(UiIrNode::node_count)
            .sum::<usize>()
    }

    #[doc = include_str!("ui_ir_node.doc/find.md")]
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
