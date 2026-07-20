//! A retained tree node — the anchor of UI identity.

use obscur_identity::{InternedSymbol, NodeId, StableKey};
use obscur_view::view::{StyleAttributes, ViewEventBinding, ViewFragment, ViewProperty};

use super::{LogicalParent, NodeFlags, NodeGeneration, NodeKind, VisualParent};

/// A node record in the retained tree, owned by the arena.
///
/// The retained tree is the single source of UI identity (spec §1.5): layout,
/// focus, selection, animation, DnD and caches all attach to a node's stable
/// [`NodeId`] plus its [`NodeGeneration`] (spec §2.2). Because the tree is an
/// arena, a node links to its parents and children **by `NodeId`**, never by
/// owning child nodes.
///
/// A node has an independently settable **logical** parent (structure /
/// style-theme scope) and **visual** parent (paint location). These coincide for
/// ordinary nodes but diverge for portalled nodes (spec §4.14).
/// Because the retained `props` carry a [`ViewValue`](obscur_view::view::ViewValue)
/// — which holds an `f64` — this type is `PartialEq` but not `Eq`.
#[derive(Debug, Clone, PartialEq)]
pub struct RetainedNode {
    /// Stable node identity.
    pub id: NodeId,
    /// Generation, bumped when the arena slot's identity is reused.
    pub generation: NodeGeneration,
    /// Classification of this node (element, text, root, synthetic).
    pub kind: NodeKind,
    /// Structural / behavioural flags (not invalidation state, spec §4.3).
    pub flags: NodeFlags,
    /// Logical parent (structure and style/theme scope).
    pub logical_parent: LogicalParent,
    /// Visual parent (paint location in the stacking tree).
    pub visual_parent: VisualParent,
    /// Ordered child links, by `NodeId` — **every** child node in document
    /// order, text and synthetic parts included. This is the all-nodes list
    /// internal traversals (layout, paint, hit-test, semantics, reconcile) walk;
    /// read it through [`child_nodes`](Self::child_nodes).
    pub children: Vec<NodeId>,
    /// The element-only projection of [`children`](Self::children), in document
    /// order — the **component tree** a host observes through
    /// [`children`](Self::children) (the accessor). Text and synthetic parts are
    /// content, not component-tree children (spec §4.16), so they are excluded
    /// here. Kept in step with `children` by the arena, which is the only thing
    /// that knows each child's [`NodeKind`]; a bare node cannot derive it alone.
    pub element_children: Vec<NodeId>,
    /// Explicit reconciliation key, if the desired node carried one.
    pub key: Option<StableKey>,
    /// CSS-relevant identity (local name, id, class list) for element nodes,
    /// extracted from the desired node's props during reconciliation. Empty for
    /// non-element nodes and elements that carry no style attributes.
    pub attributes: StyleAttributes,
    /// Event bindings (`event → command`) carried on element nodes, copied from
    /// the desired node's events during reconciliation. Actions flow *up*: the
    /// runtime looks a fired event up here to find the command it dispatches
    /// (spec §2.3, §3.1). Empty for non-element nodes and elements with no
    /// events.
    pub event_bindings: Box<[ViewEventBinding]>,
    /// The desired node's **full** property list, copied during reconciliation.
    ///
    /// [`attributes`](Self::attributes) above is only the CSS-relevant projection
    /// of these (local name, id, class list, boolean states); every other authored
    /// property — a component's `label`, `placeholder`, or any custom prop — lives
    /// only here. Values flow *down* (spec §3.1), and a component's typed props
    /// are decoded from this list after the frame's reconcile, so it is retained
    /// for exactly the reason [`event_bindings`](Self::event_bindings) is: the
    /// runtime must be able to look it up later, once the desired tree is gone.
    ///
    /// Empty for non-element nodes and elements that carry no properties.
    pub props: Box<[ViewProperty]>,
    /// The desired node's authored children, kept for §2.12 slot projection.
    ///
    /// Retained for the same reason [`props`](Self::props) is — the desired tree
    /// is gone by the time a component renders, and a component's output declares
    /// *where* its authored children go (see
    /// [`ViewNode::slot`](obscur_view::view::ViewNode::slot)), so the runtime must
    /// be able to look the content up again after the frame's reconcile.
    ///
    /// Populated **only** when at least one authored child carries a `slot="…"`
    /// assignment: keeping every element's children would clone the desired
    /// subtree once per level of depth, which is a cost a document that authors no
    /// slot must not pay. Empty for every other node, which is nearly all of them.
    pub slot_content: ViewFragment,
}

impl RetainedNode {
    /// Creates a retained node at its first generation with the given kind:
    /// no parents, no children, no key, and empty flags.
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

    /// Returns this node's stable identity.
    #[must_use]
    pub const fn id(&self) -> NodeId {
        self.id
    }

    /// Returns this node's current generation.
    #[must_use]
    pub const fn generation(&self) -> NodeGeneration {
        self.generation
    }

    /// Returns this node's classification.
    #[must_use]
    pub const fn kind(&self) -> &NodeKind {
        &self.kind
    }

    /// Returns this node's structural / behavioural flags.
    #[must_use]
    pub const fn flags(&self) -> NodeFlags {
        self.flags
    }

    /// Returns this node's logical parent.
    #[must_use]
    pub const fn logical_parent(&self) -> LogicalParent {
        self.logical_parent
    }

    /// Returns this node's visual parent.
    #[must_use]
    pub const fn visual_parent(&self) -> VisualParent {
        self.visual_parent
    }

    /// Returns this node's explicit reconciliation key, if any.
    #[must_use]
    pub const fn key(&self) -> Option<&StableKey> {
        self.key.as_ref()
    }

    /// Returns this node's CSS-relevant style attributes (local name, id,
    /// classes). Empty for non-element nodes.
    #[must_use]
    pub const fn attributes(&self) -> &StyleAttributes {
        &self.attributes
    }

    /// Returns this node's event bindings (`event → command`). Empty for
    /// non-element nodes and elements that carry no events.
    #[must_use]
    pub fn event_bindings(&self) -> &[ViewEventBinding] {
        &self.event_bindings
    }

    /// Returns the authored children kept for §2.12 slot projection — see
    /// [`slot_content`](Self::slot_content). Empty unless an authored child of
    /// this node carried a `slot="…"` assignment.
    #[must_use]
    pub const fn slot_content(&self) -> &ViewFragment {
        &self.slot_content
    }

    /// Sets the authored children kept for §2.12 slot projection, replacing any
    /// previous value.
    pub fn set_slot_content(&mut self, slot_content: ViewFragment) {
        self.slot_content = slot_content;
    }

    /// Returns the command bound to `event` on this node, if any — the first
    /// matching binding wins.
    #[must_use]
    pub fn command_for_event(&self, event: InternedSymbol) -> Option<InternedSymbol> {
        self.event_bindings
            .iter()
            .find(|binding| binding.event() == event)
            .map(ViewEventBinding::command)
    }

    /// Advances the generation, marking the arena slot as reused for a new
    /// occupant so that stale references can be detected.
    pub fn bump_generation(&mut self) {
        self.generation = self.generation.next();
    }

    /// Sets the logical parent (structure / style-theme scope). Independent of
    /// the visual parent (spec §4.14).
    pub fn set_logical_parent(&mut self, parent: LogicalParent) {
        self.logical_parent = parent;
    }

    /// Sets the visual parent (paint location). Independent of the logical
    /// parent (spec §4.14).
    pub fn set_visual_parent(&mut self, parent: VisualParent) {
        self.visual_parent = parent;
    }

    /// Sets or clears the explicit reconciliation key.
    pub fn set_key(&mut self, key: Option<StableKey>) {
        self.key = key;
    }

    /// Sets this node's CSS-relevant style attributes, replacing any previous
    /// value.
    pub fn set_attributes(&mut self, attributes: StyleAttributes) {
        self.attributes = attributes;
    }

    /// Sets this node's event bindings, replacing any previous value.
    pub fn set_event_bindings(&mut self, event_bindings: Box<[ViewEventBinding]>) {
        self.event_bindings = event_bindings;
    }

    /// Returns the node's full authored property list.
    #[must_use]
    pub fn props(&self) -> &[ViewProperty] {
        &self.props
    }

    /// Sets this node's properties, replacing any previous value.
    ///
    /// Callers that want [`attributes`](Self::attributes) to stay in step must
    /// re-derive them; the reconciler does both together.
    pub fn set_props(&mut self, props: Box<[ViewProperty]>) {
        self.props = props;
    }

    /// Returns the element-only child links — the **component tree** a host
    /// observes (spec §4.16). Text runs and synthetic parts are content, not
    /// component-tree children, so they are excluded; a `.badge` holding only the
    /// text "New" reports **zero** children here. Internal traversals that must
    /// see every child node (layout, paint, hit-test, semantics, reconcile) use
    /// [`child_nodes`](Self::child_nodes) instead.
    #[must_use]
    pub fn children(&self) -> &[NodeId] {
        &self.element_children
    }

    /// Returns **every** ordered child link — text and synthetic parts included.
    /// This is the all-nodes view the tree machinery walks; the element-only
    /// component-tree projection a host sees is [`children`](Self::children).
    #[must_use]
    pub fn child_nodes(&self) -> &[NodeId] {
        &self.children
    }

    /// Appends a child link, preserving order.
    pub fn push_child(&mut self, child: NodeId) {
        self.children.push(child);
    }

    /// Removes the first occurrence of `child` from the child links, preserving
    /// the order of the remaining children. Returns `true` if a child was
    /// removed.
    pub fn remove_child(&mut self, child: NodeId) -> bool {
        if let Some(pos) = self.children.iter().position(|c| *c == child) {
            self.children.remove(pos);
            true
        } else {
            false
        }
    }

    /// Adds `flags` to this node's flag set.
    pub fn insert_flags(&mut self, flags: NodeFlags) {
        self.flags.insert(flags);
    }

    /// Removes `flags` from this node's flag set.
    pub fn remove_flags(&mut self, flags: NodeFlags) {
        self.flags.remove(flags);
    }

    /// Returns `true` if every flag in `flags` is set.
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
