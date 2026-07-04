//! A retained tree node — the anchor of UI identity.

use obscur_identity::{NodeId, StableKey};
use obscur_view::view::StyleAttributes;

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
#[derive(Debug, Clone, PartialEq, Eq)]
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
    /// Ordered child links, by `NodeId`.
    pub children: Vec<NodeId>,
    /// Explicit reconciliation key, if the desired node carried one.
    pub key: Option<StableKey>,
    /// CSS-relevant identity (local name, id, class list) for element nodes,
    /// extracted from the desired node's props during reconciliation. Empty for
    /// non-element nodes and elements that carry no style attributes.
    pub attributes: StyleAttributes,
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
            key: None,
            attributes: StyleAttributes::new(),
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

    /// Returns the ordered child links.
    #[must_use]
    pub fn children(&self) -> &[NodeId] {
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
        assert_eq!(n.children(), &[a, b, c]);

        assert!(n.remove_child(b));
        assert_eq!(n.children(), &[a, c]);
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
}
