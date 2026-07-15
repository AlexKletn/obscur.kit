//! `Reconciler`: the engine that updates the retained tree from a desired tree.

use std::collections::HashMap;

use obscur_identity::{NodeId, StableKey};
use obscur_view::view::{
    StyleAttributes, View, ViewEventBinding, ViewFragment, ViewNode, ViewProperty,
};

use crate::node::{LogicalParent, NodeFlags, NodeKind, VisualParent};
use crate::tree::{RetainedTree, Subtree};

use super::{ChildMatcher, IdentityMatch, ReconciliationOp, ReconciliationPlan};

/// Transforms a desired declarative tree into the retained tree, preserving node
/// identity across moves via keys.
///
/// Reconciliation is exactly the desired → retained transform (spec §2.2):
/// updates are expressed as new desired state, and this engine is the sole
/// writer that turns that state into retained-tree structure. It never invents a
/// competing identity — it reuses an existing [`NodeId`] whenever a desired child
/// is compatible with an existing one (matching by key across reorders, per spec
/// §2.11), and mints a fresh id only when it must.
///
/// **Structure**, **identity**, **text content**, an element's **full authored
/// property list** (plus the CSS-relevant [`StyleAttributes`] derived from it),
/// and its **event bindings** (`event → command`) are all reconciled here.
///
/// Properties are retained rather than projected away because values flow *down*
/// (spec §3.1) and the desired tree is gone by the time downstream phases run: a
/// component's typed props are decoded from the retained list after reconcile,
/// exactly as a fired event finds its command in the retained bindings. The
/// change check compares the whole list, so a change to *any* authored prop — not
/// just a CSS-relevant one — yields an `Update`.
///
/// Every pass returns a [`ReconciliationPlan`] describing what it did. The engine
/// is deterministic and panic-free, and is defensive about stale ids.
#[derive(Debug, Clone, Copy, Default)]
pub struct Reconciler {
    matcher: ChildMatcher,
}

impl Reconciler {
    /// Creates a reconciler.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            matcher: ChildMatcher::new(),
        }
    }

    /// Ensures a root node exists and reconciles its children against `desired`.
    ///
    /// If the tree already has a live root it is reused; otherwise a
    /// [`NodeKind::Root`] node is allocated and designated as the root (recorded
    /// as an [`ReconciliationOp::Insert`] with the root as its own parent). The
    /// desired [`View`]'s top-level fragment then reconciles into that root's
    /// children.
    #[must_use]
    pub fn reconcile_root(&self, tree: &mut RetainedTree, desired: &View) -> ReconciliationPlan {
        let mut plan = ReconciliationPlan::new();
        let root = match tree.root() {
            Some(id) if tree.contains(id) => id,
            _ => {
                let id = tree.alloc(NodeKind::Root);
                tree.set_root(id);
                plan.push(ReconciliationOp::Insert {
                    id,
                    parent: id,
                    index: 0,
                });
                id
            }
        };
        self.reconcile_into(tree, root, desired.root().nodes(), &mut plan);
        plan
    }

    /// Reconciles the children of an existing retained `parent` against a desired
    /// fragment, updating the tree and returning the op log.
    ///
    /// If `parent` is not a live node the pass is a no-op and an empty plan is
    /// returned.
    #[must_use]
    pub fn reconcile(
        &self,
        tree: &mut RetainedTree,
        parent: NodeId,
        desired: &ViewFragment,
    ) -> ReconciliationPlan {
        let mut plan = ReconciliationPlan::new();
        self.reconcile_into(tree, parent, desired.nodes(), &mut plan);
        plan
    }

    /// Reconciles `desired` into the children of `parent`, appending ops to
    /// `plan`. Recurses into each element child. A no-op if `parent` is stale.
    fn reconcile_into(
        self,
        tree: &mut RetainedTree,
        parent: NodeId,
        desired: &[ViewNode],
        plan: &mut ReconciliationPlan,
    ) {
        // Read the parent's current children; bail out defensively if stale.
        let Some(existing) = tree.get(parent).map(|node| node.children().to_vec()) else {
            return;
        };

        let matching = self.matcher.match_children(tree, &existing, desired);
        let (matches, removed) = matching.into_parts();

        // Old relative order of the reused nodes, for reorder detection.
        let old_rank: HashMap<NodeId, usize> = existing
            .iter()
            .enumerate()
            .map(|(rank, &id)| (id, rank))
            .collect();

        let mut new_children: Vec<NodeId> = Vec::with_capacity(desired.len());
        let mut reused_order: Vec<NodeId> = Vec::new();

        for (index, (node, decision)) in desired.iter().zip(matches).enumerate() {
            let id = match decision {
                IdentityMatch::Reuse(id) => {
                    // In-place content updates: text content for text nodes, and
                    // the CSS-relevant style attributes for elements (props can
                    // change between renders). Each variant yields at most one
                    // Update op, so there is no duplicate-Update concern.
                    match node {
                        ViewNode::Text { content, .. } => {
                            if apply_text_content(tree, id, content) {
                                plan.push(ReconciliationOp::Update { id });
                            }
                        }
                        ViewNode::Element { props, events, .. } => {
                            // Refresh the authored properties (and the attributes
                            // derived from them) and the event bindings from the
                            // latest desired node. Combine into a single Update op
                            // so a node yields at most one.
                            let props_changed = apply_props(tree, id, props);
                            let events_changed = apply_events(tree, id, events);
                            if props_changed || events_changed {
                                plan.push(ReconciliationOp::Update { id });
                            }
                        }
                    }
                    reused_order.push(id);
                    id
                }
                IdentityMatch::Create => {
                    let id = tree.alloc(node_kind_of(node));
                    // Newly created element nodes carry their authored properties
                    // (and the attributes derived from them) and event bindings.
                    if let ViewNode::Element { props, events, .. } = node {
                        apply_props(tree, id, props);
                        apply_events(tree, id, events);
                    }
                    plan.push(ReconciliationOp::Insert { id, parent, index });
                    id
                }
            };

            apply_identity(tree, id, parent, desired_key(node));

            // Recurse into element children (created nodes start empty, so all
            // of their desired children become inserts).
            //
            // A COMPONENT_HOST is the exception: its children belong to the
            // mounted component's `render`, not to the desired tree. The desired
            // node names the component and carries its props but says nothing
            // about its output, so recursing here would delete that output every
            // frame — and take any nested component instance (and its local
            // state) with it. Authored children under a component are §2.12 slot
            // content, which is not lowered yet; they are ignored rather than
            // mistaken for the component's own output.
            if let ViewNode::Element { children, .. } = node
                && !hosts_component(tree, id)
            {
                self.reconcile_into(tree, id, children.nodes(), plan);
            }

            new_children.push(id);
        }

        // Record moves for reused nodes whose relative order changed. Nodes that
        // stay in increasing old-rank order are left in place (no-op); the rest
        // are repositioned to their final index.
        let mut last_rank: Option<usize> = None;
        for &id in &reused_order {
            let Some(&rank) = old_rank.get(&id) else {
                continue;
            };
            let in_order = last_rank.is_none_or(|last| rank > last);
            if in_order {
                last_rank = Some(rank);
            } else if let Some(index) = new_children.iter().position(|&c| c == id) {
                plan.push(ReconciliationOp::Move { id, index });
            }
        }

        // Commit the parent's child order to the desired order.
        if let Some(parent_node) = tree.get_mut(parent) {
            parent_node.children = new_children;
        }

        // Remove every unmatched existing child together with its subtree,
        // leaves before parents so a parent is never removed before its children.
        for &root in &removed {
            let mut ids = Subtree::new(root).descendants(tree);
            ids.reverse();
            for id in ids {
                if tree.remove(id).is_some() {
                    plan.push(ReconciliationOp::Remove { id });
                }
            }
        }
    }
}

/// Whether `id` is a live node flagged as hosting a mounted component, whose
/// children are therefore owned by that component rather than by the desired tree.
fn hosts_component(tree: &RetainedTree, id: NodeId) -> bool {
    tree.get(id)
        .is_some_and(|node| node.flags().contains(NodeFlags::COMPONENT_HOST))
}

/// Maps a desired node to the retained [`NodeKind`] it materializes as.
fn node_kind_of(node: &ViewNode) -> NodeKind {
    match node {
        ViewNode::Element { component, .. } => NodeKind::Element {
            component: *component,
        },
        ViewNode::Text { content, .. } => NodeKind::Text {
            content: content.clone(),
        },
    }
}

/// Extracts the retained key (a [`StableKey`]) a desired node carries, if any.
fn desired_key(node: &ViewNode) -> Option<StableKey> {
    node.key().map(|key| key.stable_key().clone())
}

/// Sets a node's logical and visual parent to `parent` and its reconciliation
/// key. A no-op if `id` is stale.
fn apply_identity(tree: &mut RetainedTree, id: NodeId, parent: NodeId, key: Option<StableKey>) {
    if let Some(node) = tree.get_mut(id) {
        node.set_logical_parent(LogicalParent::of(parent));
        node.set_visual_parent(VisualParent::of(parent));
        node.set_key(key);
    }
}

/// Updates a reused text node's content in place, returning `true` if the
/// content actually changed. A no-op (returning `false`) if `id` is stale or is
/// not a text node.
fn apply_text_content(tree: &mut RetainedTree, id: NodeId, content: &str) -> bool {
    let Some(node) = tree.get_mut(id) else {
        return false;
    };
    // Resolve whether the content differs before taking a mutable borrow of the
    // node's kind, so the read borrow is released before the write.
    let changed = match node.kind() {
        NodeKind::Text { content: current } => current.as_ref() != content,
        _ => false,
    };
    if changed {
        node.kind = NodeKind::Text {
            content: content.into(),
        };
    }
    changed
}

/// Refreshes a node's CSS-relevant style attributes from a desired element's
/// props, returning `true` if they actually changed. A no-op (returning
/// `false`) if `id` is stale or the extracted attributes equal the current ones.
fn apply_props(tree: &mut RetainedTree, id: NodeId, props: &[ViewProperty]) -> bool {
    let Some(node) = tree.get_mut(id) else {
        return false;
    };
    // Compare the FULL property list, not the derived attributes. Attributes are
    // only the CSS-relevant projection, so diffing them would miss a change to
    // any other authored prop (a component's `label`, say) and report the node
    // unchanged — which would leave the component rendering a stale value, and
    // would wrongly let the frame skip layout.
    if node.props() == props {
        return false;
    }
    node.set_props(Box::from(props));
    // Attributes are derived from the props, so they are re-extracted together
    // and can never drift out of step with them.
    node.set_attributes(StyleAttributes::extract(props));
    true
}

/// Refreshes a node's event bindings from a desired element's events, returning
/// `true` if they actually changed. A no-op (returning `false`) if `id` is stale
/// or the events equal the current bindings.
fn apply_events(tree: &mut RetainedTree, id: NodeId, events: &[ViewEventBinding]) -> bool {
    let Some(node) = tree.get_mut(id) else {
        return false;
    };
    if node.event_bindings() == events {
        return false;
    }
    node.set_event_bindings(Box::from(events));
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_identity::ComponentId;
    use obscur_view::view::ViewKey;

    fn el(comp: u64) -> ViewNode {
        ViewNode::element(ComponentId::from_raw(comp))
    }

    fn keyed_el(comp: u64, key: &str) -> ViewNode {
        el(comp).with_key(ViewKey::from(key))
    }

    fn child_ids(tree: &RetainedTree, parent: NodeId) -> Vec<NodeId> {
        tree.get(parent)
            .map(|n| n.children().to_vec())
            .unwrap_or_default()
    }

    /// Builds a tree with a root node and returns it plus the root id.
    fn tree_with_root() -> (RetainedTree, NodeId) {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        (tree, root)
    }

    #[test]
    fn reconcile_root_creates_root_when_missing() {
        let mut tree = RetainedTree::new();
        let view = View::single(el(1));
        let plan = Reconciler::new().reconcile_root(&mut tree, &view);
        let root = tree.root().expect("root created");
        assert!(tree.get(root).expect("live").kind().is_root());
        assert_eq!(child_ids(&tree, root).len(), 1);
        // A root insert plus one child insert.
        assert_eq!(plan.iter().filter(|op| op.is_insert()).count(), 2);
    }

    #[test]
    fn initial_children_are_all_inserts() {
        let (mut tree, root) = tree_with_root();
        let fragment = ViewFragment::from_nodes(vec![el(1), el(2), ViewNode::text("hi")]);
        let plan = Reconciler::new().reconcile(&mut tree, root, &fragment);
        assert_eq!(child_ids(&tree, root).len(), 3);
        assert_eq!(plan.iter().filter(|op| op.is_insert()).count(), 3);
        assert!(plan.iter().all(|op| op.is_insert()));
    }

    #[test]
    fn parents_and_keys_are_set_on_children() {
        let (mut tree, root) = tree_with_root();
        let fragment = ViewFragment::from_nodes(vec![keyed_el(1, "a")]);
        let _ = Reconciler::new().reconcile(&mut tree, root, &fragment);
        let child = child_ids(&tree, root)[0];
        let node = tree.get(child).expect("live");
        assert_eq!(node.logical_parent(), LogicalParent::of(root));
        assert_eq!(node.visual_parent(), VisualParent::of(root));
        assert_eq!(node.key(), Some(&StableKey::from("a")));
    }

    #[test]
    fn reorder_of_keyed_list_preserves_identity() {
        let (mut tree, root) = tree_with_root();
        let recon = Reconciler::new();

        // Initial keyed list a, b, c.
        let first =
            ViewFragment::from_nodes(vec![keyed_el(1, "a"), keyed_el(2, "b"), keyed_el(3, "c")]);
        let _ = recon.reconcile(&mut tree, root, &first);
        let ids_before = child_ids(&tree, root);
        assert_eq!(ids_before.len(), 3);
        let (a, b, c) = (ids_before[0], ids_before[1], ids_before[2]);

        // Reorder to c, a, b — same keys, different order.
        let reordered =
            ViewFragment::from_nodes(vec![keyed_el(3, "c"), keyed_el(1, "a"), keyed_el(2, "b")]);
        let plan = recon.reconcile(&mut tree, root, &reordered);

        // Identity is preserved: the very same ids, now in the desired order.
        assert_eq!(child_ids(&tree, root), vec![c, a, b]);
        assert!(tree.contains(a) && tree.contains(b) && tree.contains(c));

        // The core property: a pure reorder produces only Move ops — no
        // Insert and no Remove touch identity.
        assert!(
            plan.iter().all(|op| op.is_move()),
            "reorder must emit only Move ops, got {:?}",
            plan.ops()
        );
        assert!(plan.iter().any(|op| op.is_move()));
        assert!(!plan.iter().any(|op| op.is_insert()));
        assert!(!plan.iter().any(|op| op.is_remove()));
    }

    #[test]
    fn unchanged_reconcile_is_a_no_op() {
        let (mut tree, root) = tree_with_root();
        let recon = Reconciler::new();
        let fragment = ViewFragment::from_nodes(vec![keyed_el(1, "a"), keyed_el(2, "b")]);
        let _ = recon.reconcile(&mut tree, root, &fragment);
        let ids = child_ids(&tree, root);
        let plan = recon.reconcile(&mut tree, root, &fragment);
        assert_eq!(child_ids(&tree, root), ids);
        assert!(
            plan.is_empty(),
            "unchanged pass must be a no-op, got {:?}",
            plan.ops()
        );
    }

    #[test]
    fn removed_child_and_its_subtree_are_removed() {
        let (mut tree, root) = tree_with_root();
        let recon = Reconciler::new();

        // parent element 'a' with a grandchild; sibling 'b'.
        let with_subtree = ViewFragment::from_nodes(vec![
            el(1)
                .with_key(ViewKey::from("a"))
                .with_children(ViewFragment::from_nodes(vec![ViewNode::text("deep")])),
            keyed_el(2, "b"),
        ]);
        let _ = recon.reconcile(&mut tree, root, &with_subtree);
        let ids = child_ids(&tree, root);
        let a = ids[0];
        let grandchild = child_ids(&tree, a)[0];
        let b = ids[1];
        assert!(tree.contains(grandchild));

        // Drop 'a' entirely; keep 'b'.
        let only_b = ViewFragment::from_nodes(vec![keyed_el(2, "b")]);
        let plan = recon.reconcile(&mut tree, root, &only_b);

        assert_eq!(child_ids(&tree, root), vec![b]);
        assert!(!tree.contains(a));
        assert!(!tree.contains(grandchild), "subtree must be removed too");
        // Both the parent and its grandchild are reported removed.
        let removed: Vec<NodeId> = plan
            .iter()
            .filter_map(|op| match op {
                ReconciliationOp::Remove { id } => Some(*id),
                _ => None,
            })
            .collect();
        assert!(removed.contains(&a));
        assert!(removed.contains(&grandchild));
    }

    #[test]
    fn incompatible_child_is_replaced_not_mutated() {
        let (mut tree, root) = tree_with_root();
        let recon = Reconciler::new();
        let _ = recon.reconcile(
            &mut tree,
            root,
            &ViewFragment::from_nodes(vec![keyed_el(1, "a")]),
        );
        let old = child_ids(&tree, root)[0];

        // Same key, different component -> replace (new id, old removed).
        let plan = recon.reconcile(
            &mut tree,
            root,
            &ViewFragment::from_nodes(vec![keyed_el(9, "a")]),
        );
        let new = child_ids(&tree, root)[0];
        assert_ne!(new, old);
        assert!(!tree.contains(old));
        assert!(plan.iter().any(|op| op.is_insert()));
        assert!(plan.iter().any(|op| op.is_remove()));
    }

    #[test]
    fn changed_text_content_is_updated_in_place() {
        let (mut tree, root) = tree_with_root();
        let recon = Reconciler::new();
        let _ = recon.reconcile(
            &mut tree,
            root,
            &ViewFragment::from_nodes(vec![ViewNode::text("before").with_key(ViewKey::from("t"))]),
        );
        let id = child_ids(&tree, root)[0];

        let plan = recon.reconcile(
            &mut tree,
            root,
            &ViewFragment::from_nodes(vec![ViewNode::text("after").with_key(ViewKey::from("t"))]),
        );
        // Same node, updated content, one Update op.
        assert_eq!(child_ids(&tree, root)[0], id);
        assert_eq!(tree.get(id).expect("live").kind().text(), Some("after"));
        assert!(plan.iter().any(|op| op.is_update()));
        assert!(!plan.iter().any(|op| op.is_insert()));
        assert!(!plan.iter().any(|op| op.is_remove()));
    }

    #[test]
    fn element_carries_style_attributes_after_reconcile() {
        use obscur_view::view::{CLASS, ID, TYPE, ViewProperty, ViewValue};

        let (mut tree, root) = tree_with_root();
        let styled = el(1)
            .with_prop(ViewProperty::new(TYPE, ViewValue::Text("container".into())))
            .with_prop(ViewProperty::new(ID, ViewValue::Text("main".into())))
            .with_prop(ViewProperty::new(
                CLASS,
                ViewValue::Text("counter primary".into()),
            ));
        let _ =
            Reconciler::new().reconcile(&mut tree, root, &ViewFragment::from_nodes(vec![styled]));
        let id = child_ids(&tree, root)[0];
        let attrs = tree.get(id).expect("live").attributes();
        assert_eq!(attrs.local_name(), Some("container"));
        assert_eq!(attrs.id(), Some("main"));
        let classes: Vec<&str> = attrs.classes().iter().map(AsRef::as_ref).collect();
        assert_eq!(classes, ["counter", "primary"]);
    }

    #[test]
    fn a_non_css_prop_survives_reconcile_and_its_change_yields_an_update() {
        // Before props were retained, only the CSS-relevant projection survived,
        // so an authored `label` was DROPPED — and changing it produced no
        // `Update` at all, because the derived attributes were identical. Values
        // flow down (§3.1), so the value has to still be there afterwards.
        use obscur_identity::InternedSymbol;
        use obscur_view::view::{ViewProperty, ViewValue};

        let label = InternedSymbol::from_raw(0x8000_00AA);
        let node = |text: &str| {
            el(1).with_prop(ViewProperty::new(label, ViewValue::Text(text.into())))
        };

        let (mut tree, root) = tree_with_root();
        let plan = Reconciler::new().reconcile(
            &mut tree,
            root,
            &ViewFragment::from_nodes(vec![node("OK")]),
        );
        assert_eq!(plan.iter().filter(|op| op.is_insert()).count(), 1);

        let id = child_ids(&tree, root)[0];
        assert_eq!(
            tree.get(id).expect("live").props(),
            &[ViewProperty::new(label, ViewValue::Text("OK".into()))],
            "the authored prop is retained, not projected away"
        );

        // An identical pass is still a no-op.
        let plan = Reconciler::new().reconcile(
            &mut tree,
            root,
            &ViewFragment::from_nodes(vec![node("OK")]),
        );
        assert!(plan.is_empty(), "an unchanged prop must not report a change");

        // Changing ONLY the non-CSS prop now reports an Update and takes effect.
        let plan = Reconciler::new().reconcile(
            &mut tree,
            root,
            &ViewFragment::from_nodes(vec![node("Cancel")]),
        );
        assert_eq!(
            plan.iter().filter(|op| matches!(op, ReconciliationOp::Update { .. })).count(),
            1,
            "a changed non-CSS prop must yield an Update"
        );
        assert_eq!(
            tree.get(id).expect("live").props()[0].value(),
            &ViewValue::Text("Cancel".into())
        );
    }

    #[test]
    fn element_carries_image_source_after_reconcile() {
        use obscur_view::view::{SRC, ViewProperty, ViewValue};

        let (mut tree, root) = tree_with_root();
        let img = el(1).with_prop(ViewProperty::new(SRC, ViewValue::Text("logo.png".into())));
        let _ = Reconciler::new().reconcile(&mut tree, root, &ViewFragment::from_nodes(vec![img]));
        let id = child_ids(&tree, root)[0];
        assert_eq!(
            tree.get(id).expect("live").attributes().image_source(),
            Some("logo.png")
        );
    }

    #[test]
    fn changed_image_source_is_updated_in_place() {
        use obscur_view::view::{SRC, ViewProperty, ViewValue};

        let (mut tree, root) = tree_with_root();
        let recon = Reconciler::new();
        let with_src = |src: &str| {
            keyed_el(1, "e").with_prop(ViewProperty::new(SRC, ViewValue::Text(src.into())))
        };

        let _ = recon.reconcile(&mut tree, root, &ViewFragment::from_nodes(vec![with_src("a.png")]));
        let id = child_ids(&tree, root)[0];
        assert_eq!(
            tree.get(id).expect("live").attributes().image_source(),
            Some("a.png")
        );

        // Changing the src re-applies the attributes and emits one Update op.
        let plan =
            recon.reconcile(&mut tree, root, &ViewFragment::from_nodes(vec![with_src("b.png")]));
        assert_eq!(child_ids(&tree, root)[0], id);
        assert_eq!(
            tree.get(id).expect("live").attributes().image_source(),
            Some("b.png")
        );
        assert!(plan.iter().any(|op| op.is_update()));
        assert!(!plan.iter().any(|op| op.is_insert()));
        assert!(!plan.iter().any(|op| op.is_remove()));
    }

    #[test]
    fn text_and_bare_element_carry_empty_attributes() {
        let (mut tree, root) = tree_with_root();
        let _ = Reconciler::new().reconcile(
            &mut tree,
            root,
            &ViewFragment::from_nodes(vec![el(1), ViewNode::text("hi")]),
        );
        let ids = child_ids(&tree, root);
        assert!(tree.get(ids[0]).expect("live").attributes().is_empty());
        assert!(tree.get(ids[1]).expect("live").attributes().is_empty());
    }

    #[test]
    fn changed_style_attributes_are_updated_in_place() {
        use obscur_view::view::{CLASS, ViewProperty, ViewValue};

        let (mut tree, root) = tree_with_root();
        let recon = Reconciler::new();
        let with_class = |class: &str| {
            keyed_el(1, "e").with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
        };

        let _ = recon.reconcile(
            &mut tree,
            root,
            &ViewFragment::from_nodes(vec![with_class("counter")]),
        );
        let id = child_ids(&tree, root)[0];
        assert!(
            tree.get(id)
                .expect("live")
                .attributes()
                .has_class("counter")
        );

        // Changing the class props updates the retained attributes and emits one
        // Update op.
        let plan = recon.reconcile(
            &mut tree,
            root,
            &ViewFragment::from_nodes(vec![with_class("primary")]),
        );
        assert_eq!(child_ids(&tree, root)[0], id);
        let attrs = tree.get(id).expect("live").attributes();
        assert!(attrs.has_class("primary"));
        assert!(!attrs.has_class("counter"));
        assert!(plan.iter().any(|op| op.is_update()));
        assert!(!plan.iter().any(|op| op.is_insert()));
        assert!(!plan.iter().any(|op| op.is_remove()));
    }

    #[test]
    fn identical_style_attributes_yield_no_update() {
        use obscur_view::view::{CLASS, ViewProperty, ViewValue};

        let (mut tree, root) = tree_with_root();
        let recon = Reconciler::new();
        let styled = || {
            keyed_el(1, "e").with_prop(ViewProperty::new(
                CLASS,
                ViewValue::Text("counter primary".into()),
            ))
        };
        let fragment = ViewFragment::from_nodes(vec![styled()]);
        let _ = recon.reconcile(&mut tree, root, &fragment);
        // Re-reconcile with identical props: no spurious change is reported.
        let plan = recon.reconcile(&mut tree, root, &ViewFragment::from_nodes(vec![styled()]));
        assert!(
            plan.is_empty(),
            "identical props must not report a change, got {:?}",
            plan.ops()
        );
    }

    #[test]
    fn element_carries_event_bindings_after_reconcile() {
        use obscur_identity::InternedSymbol;

        let click = InternedSymbol::from_raw(0x00E0_0001);
        let increment = InternedSymbol::from_raw(0x00C0_0001);

        let (mut tree, root) = tree_with_root();
        let bound = el(1).with_event(ViewEventBinding::new(click, increment));
        let _ =
            Reconciler::new().reconcile(&mut tree, root, &ViewFragment::from_nodes(vec![bound]));
        let id = child_ids(&tree, root)[0];
        let node = tree.get(id).expect("live");
        assert_eq!(
            node.event_bindings(),
            &[ViewEventBinding::new(click, increment)]
        );
        assert_eq!(node.command_for_event(click), Some(increment));
        assert_eq!(node.command_for_event(InternedSymbol::from_raw(0xDEAD)), None);
    }

    #[test]
    fn bare_element_and_text_carry_no_event_bindings() {
        let (mut tree, root) = tree_with_root();
        let _ = Reconciler::new().reconcile(
            &mut tree,
            root,
            &ViewFragment::from_nodes(vec![el(1), ViewNode::text("hi")]),
        );
        let ids = child_ids(&tree, root);
        // (c) a Text node has no bindings; a bare element has none either.
        assert!(tree.get(ids[0]).expect("live").event_bindings().is_empty());
        assert!(tree.get(ids[1]).expect("live").event_bindings().is_empty());
    }

    #[test]
    fn changed_event_binding_is_refreshed_in_place() {
        use obscur_identity::InternedSymbol;

        let click = InternedSymbol::from_raw(0x00E0_0001);
        let increment = InternedSymbol::from_raw(0x00C0_0001);
        let decrement = InternedSymbol::from_raw(0x00C0_0002);

        let (mut tree, root) = tree_with_root();
        let recon = Reconciler::new();
        let bound = |command| {
            keyed_el(1, "e").with_event(ViewEventBinding::new(click, command))
        };

        let _ = recon.reconcile(
            &mut tree,
            root,
            &ViewFragment::from_nodes(vec![bound(increment)]),
        );
        let id = child_ids(&tree, root)[0];
        assert_eq!(
            tree.get(id).expect("live").command_for_event(click),
            Some(increment)
        );

        // (b) Same key → same NodeId; changing the bound command refreshes the
        // binding on that node and emits one Update op.
        let plan = recon.reconcile(
            &mut tree,
            root,
            &ViewFragment::from_nodes(vec![bound(decrement)]),
        );
        assert_eq!(child_ids(&tree, root)[0], id);
        assert_eq!(
            tree.get(id).expect("live").command_for_event(click),
            Some(decrement)
        );
        assert!(plan.iter().any(|op| op.is_update()));
        assert!(!plan.iter().any(|op| op.is_insert()));
        assert!(!plan.iter().any(|op| op.is_remove()));
    }

    #[test]
    fn identical_event_bindings_yield_no_update() {
        use obscur_identity::InternedSymbol;

        let click = InternedSymbol::from_raw(0x00E0_0001);
        let increment = InternedSymbol::from_raw(0x00C0_0001);

        let (mut tree, root) = tree_with_root();
        let recon = Reconciler::new();
        let bound = || keyed_el(1, "e").with_event(ViewEventBinding::new(click, increment));
        let _ = recon.reconcile(&mut tree, root, &ViewFragment::from_nodes(vec![bound()]));
        let plan = recon.reconcile(&mut tree, root, &ViewFragment::from_nodes(vec![bound()]));
        assert!(
            plan.is_empty(),
            "identical events must not report a change, got {:?}",
            plan.ops()
        );
    }

    #[test]
    fn stale_parent_is_a_no_op() {
        let mut tree = RetainedTree::new();
        let ghost = tree.alloc(NodeKind::Root);
        tree.remove(ghost);
        let plan =
            Reconciler::new().reconcile(&mut tree, ghost, &ViewFragment::from_nodes(vec![el(1)]));
        assert!(plan.is_empty());
    }
}
