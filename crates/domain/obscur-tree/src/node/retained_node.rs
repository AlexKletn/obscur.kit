#![doc = include_str!("retained_node.doc/_module.md")]

use obscur_identity::{InternedSymbol, NodeId, StableKey};
use obscur_view::view::{StyleAttributes, ViewEventBinding, ViewFragment, ViewProperty};

use super::{LogicalParent, NodeFlags, NodeGeneration, NodeKind, VisualParent};

#[doc = include_str!("retained_node.doc/RetainedNode.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct RetainedNode {
    #[doc = include_str!("retained_node.doc/id.md")]
    pub id: NodeId,
    #[doc = include_str!("retained_node.doc/generation.md")]
    pub generation: NodeGeneration,
    #[doc = include_str!("retained_node.doc/kind.md")]
    pub kind: NodeKind,
    #[doc = include_str!("retained_node.doc/flags.md")]
    pub flags: NodeFlags,
    #[doc = include_str!("retained_node.doc/logical_parent.md")]
    pub logical_parent: LogicalParent,
    #[doc = include_str!("retained_node.doc/visual_parent.md")]
    pub visual_parent: VisualParent,
    #[doc = include_str!("retained_node.doc/children.md")]
    pub children: Vec<NodeId>,
    #[doc = include_str!("retained_node.doc/element_children.md")]
    pub element_children: Vec<NodeId>,
    #[doc = include_str!("retained_node.doc/key.md")]
    pub key: Option<StableKey>,
    #[doc = include_str!("retained_node.doc/attributes.md")]
    pub attributes: StyleAttributes,
    #[doc = include_str!("retained_node.doc/event_bindings.md")]
    pub event_bindings: Box<[ViewEventBinding]>,
    #[doc = include_str!("retained_node.doc/props.md")]
    pub props: Box<[ViewProperty]>,
    #[doc = include_str!("retained_node.doc/slot_content.md")]
    pub slot_content: ViewFragment,
}

impl RetainedNode {
    #[doc = include_str!("retained_node.doc/new.md")]
    #[must_use]
    pub fn new(id: NodeId, kind: NodeKind) -> Self {
        Self {
            id,
            generation: NodeGeneration::FIRST,
            kind,
            flags: NodeFlags::empty(),
            logical_parent: LogicalParent::root(),
            visual_parent: VisualParent::root(),
            children: Vec::new(),
            element_children: Vec::new(),
            key: None,
            attributes: StyleAttributes::new(),
            event_bindings: Box::default(),
            props: Box::default(),
            slot_content: ViewFragment::new(),
        }
    }

    #[doc = include_str!("retained_node.doc/id_2.md")]
    #[must_use]
    pub const fn id(&self) -> NodeId {
        self.id
    }

    #[doc = include_str!("retained_node.doc/generation_2.md")]
    #[must_use]
    pub const fn generation(&self) -> NodeGeneration {
        self.generation
    }

    #[doc = include_str!("retained_node.doc/kind_2.md")]
    #[must_use]
    pub const fn kind(&self) -> &NodeKind {
        &self.kind
    }

    #[doc = include_str!("retained_node.doc/flags_2.md")]
    #[must_use]
    pub const fn flags(&self) -> NodeFlags {
        self.flags
    }

    #[doc = include_str!("retained_node.doc/logical_parent_2.md")]
    #[must_use]
    pub const fn logical_parent(&self) -> LogicalParent {
        self.logical_parent
    }

    #[doc = include_str!("retained_node.doc/visual_parent_2.md")]
    #[must_use]
    pub const fn visual_parent(&self) -> VisualParent {
        self.visual_parent
    }

    #[doc = include_str!("retained_node.doc/key_2.md")]
    #[must_use]
    pub const fn key(&self) -> Option<&StableKey> {
        self.key.as_ref()
    }

    #[doc = include_str!("retained_node.doc/attributes_2.md")]
    #[must_use]
    pub const fn attributes(&self) -> &StyleAttributes {
        &self.attributes
    }

    #[doc = include_str!("retained_node.doc/event_bindings_2.md")]
    #[must_use]
    pub fn event_bindings(&self) -> &[ViewEventBinding] {
        &self.event_bindings
    }

    #[doc = include_str!("retained_node.doc/slot_content_2.md")]
    #[must_use]
    pub const fn slot_content(&self) -> &ViewFragment {
        &self.slot_content
    }

    #[doc = include_str!("retained_node.doc/set_slot_content.md")]
    pub fn set_slot_content(&mut self, slot_content: ViewFragment) {
        self.slot_content = slot_content;
    }

    #[doc = include_str!("retained_node.doc/command_for_event.md")]
    #[must_use]
    pub fn command_for_event(&self, event: InternedSymbol) -> Option<InternedSymbol> {
        self.event_bindings
            .iter()
            .find(|binding| binding.event() == event)
            .map(ViewEventBinding::command)
    }

    #[doc = include_str!("retained_node.doc/bump_generation.md")]
    pub fn bump_generation(&mut self) {
        self.generation = self.generation.next();
    }

    #[doc = include_str!("retained_node.doc/set_logical_parent.md")]
    pub fn set_logical_parent(&mut self, parent: LogicalParent) {
        self.logical_parent = parent;
    }

    #[doc = include_str!("retained_node.doc/set_visual_parent.md")]
    pub fn set_visual_parent(&mut self, parent: VisualParent) {
        self.visual_parent = parent;
    }

    #[doc = include_str!("retained_node.doc/set_key.md")]
    pub fn set_key(&mut self, key: Option<StableKey>) {
        self.key = key;
    }

    #[doc = include_str!("retained_node.doc/set_attributes.md")]
    pub fn set_attributes(&mut self, attributes: StyleAttributes) {
        self.attributes = attributes;
    }

    #[doc = include_str!("retained_node.doc/set_event_bindings.md")]
    pub fn set_event_bindings(&mut self, event_bindings: Box<[ViewEventBinding]>) {
        self.event_bindings = event_bindings;
    }

    #[doc = include_str!("retained_node.doc/props_2.md")]
    #[must_use]
    pub fn props(&self) -> &[ViewProperty] {
        &self.props
    }

    #[doc = include_str!("retained_node.doc/set_props.md")]
    pub fn set_props(&mut self, props: Box<[ViewProperty]>) {
        self.props = props;
    }

    #[doc = include_str!("retained_node.doc/children_2.md")]
    #[must_use]
    pub fn children(&self) -> &[NodeId] {
        &self.element_children
    }

    #[doc = include_str!("retained_node.doc/child_nodes.md")]
    #[must_use]
    pub fn child_nodes(&self) -> &[NodeId] {
        &self.children
    }

    #[doc = include_str!("retained_node.doc/push_child.md")]
    pub fn push_child(&mut self, child: NodeId) {
        self.children.push(child);
    }

    #[doc = include_str!("retained_node.doc/remove_child.md")]
    pub fn remove_child(&mut self, child: NodeId) -> bool {
        if let Some(pos) = self.children.iter().position(|c| *c == child) {
            self.children.remove(pos);
            true
        } else {
            false
        }
    }

    #[doc = include_str!("retained_node.doc/insert_flags.md")]
    pub fn insert_flags(&mut self, flags: NodeFlags) {
        self.flags.insert(flags);
    }

    #[doc = include_str!("retained_node.doc/remove_flags.md")]
    pub fn remove_flags(&mut self, flags: NodeFlags) {
        self.flags.remove(flags);
    }

    #[doc = include_str!("retained_node.doc/has_flags.md")]
    #[must_use]
    pub fn has_flags(&self, flags: NodeFlags) -> bool {
        self.flags.contains(flags)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_identity::ComponentId;

    fn element(raw: u64) -> RetainedNode {
        RetainedNode::new(
            NodeId::from_raw(raw),
            NodeKind::Element {
                component: ComponentId::from_raw(raw),
            },
        )
    }

    #[test]
    fn new_is_empty_and_first_generation() {
        let n = element(1);
        assert_eq!(n.id(), NodeId::from_raw(1));
        assert_eq!(n.generation(), NodeGeneration::FIRST);
        assert!(n.flags().is_empty());
        assert!(n.logical_parent().is_root());
        assert!(n.visual_parent().is_root());
        assert!(n.children().is_empty());
        assert_eq!(n.key(), None);
    }

    #[test]
    fn bump_generation_advances() {
        let mut n = element(1);
        let g0 = n.generation();
        n.bump_generation();
        assert_eq!(n.generation(), g0.next());
    }

    #[test]
    fn logical_and_visual_parent_are_independent() {
        let mut n = element(1);
        let logical = LogicalParent::of(NodeId::from_raw(10));
        let visual = VisualParent::of(NodeId::from_raw(20));
        n.set_logical_parent(logical);
        n.set_visual_parent(visual);
        assert_eq!(n.logical_parent(), logical);
        assert_eq!(n.visual_parent(), visual);
        // A portal: logical scope kept, paint location moved.
        assert_ne!(n.logical_parent().parent(), n.visual_parent().parent());
    }

    #[test]
    fn child_management_preserves_order() {
        let mut n = element(1);
        let a = NodeId::from_raw(100);
        let b = NodeId::from_raw(101);
        let c = NodeId::from_raw(102);
        n.push_child(a);
        n.push_child(b);
        n.push_child(c);
        assert_eq!(n.child_nodes(), &[a, b, c]);

        assert!(n.remove_child(b));
        assert_eq!(n.child_nodes(), &[a, c]);
        assert!(!n.remove_child(b));
    }

    #[test]
    fn flag_helpers() {
        let mut n = element(1);
        n.insert_flags(NodeFlags::HIDDEN);
        assert!(n.has_flags(NodeFlags::HIDDEN));
        n.remove_flags(NodeFlags::HIDDEN);
        assert!(!n.has_flags(NodeFlags::HIDDEN));
    }

    #[test]
    fn set_key_roundtrips() {
        let mut n = element(1);
        n.set_key(Some(StableKey::Index(5)));
        assert_eq!(n.key(), Some(&StableKey::Index(5)));
        n.set_key(None);
        assert_eq!(n.key(), None);
    }

    #[test]
    fn new_starts_with_empty_attributes() {
        let n = element(1);
        assert!(n.attributes().is_empty());
    }

    #[test]
    fn set_attributes_roundtrips() {
        use obscur_view::view::{CLASS, ID, StyleAttributes, ViewProperty, ViewValue};

        let mut n = element(1);
        let attrs = StyleAttributes::extract(&[
            ViewProperty::new(ID, ViewValue::Text("main".into())),
            ViewProperty::new(CLASS, ViewValue::Text("counter primary".into())),
        ]);
        n.set_attributes(attrs.clone());
        assert_eq!(n.attributes(), &attrs);
        assert_eq!(n.attributes().id(), Some("main"));
        assert!(n.attributes().has_class("counter"));

        n.set_attributes(StyleAttributes::new());
        assert!(n.attributes().is_empty());
    }

    #[test]
    fn new_starts_with_no_event_bindings() {
        let n = element(1);
        assert!(n.event_bindings().is_empty());
        assert_eq!(n.command_for_event(InternedSymbol::from_raw(1)), None);
    }

    #[test]
    fn set_event_bindings_roundtrips_and_resolves_command() {
        let click = InternedSymbol::from_raw(10);
        let increment = InternedSymbol::from_raw(20);
        let mut n = element(1);
        n.set_event_bindings(Box::from([ViewEventBinding::new(click, increment)]));
        assert_eq!(n.event_bindings().len(), 1);
        assert_eq!(n.command_for_event(click), Some(increment));
        assert_eq!(n.command_for_event(InternedSymbol::from_raw(99)), None);

        n.set_event_bindings(Box::default());
        assert!(n.event_bindings().is_empty());
    }
}
