//! `Reconciler`: the engine that updates the retained tree from a desired tree.

mod sibling_reconciliation;

use sibling_reconciliation::SiblingReconciliation;

use std::collections::HashMap;

use obscur_identity::{ComponentId, FastMap, NodeId, StableKey};
use obscur_view::view::{
    StyleAttributes, View, ViewEventBinding, ViewFragment, ViewNode, ViewProperty, assigns_slots,
    is_outlet, may_project_content,
};

use crate::node::{LogicalParent, NodeFlags, NodeKind, VisualParent};
use crate::tree::RetainedTree;

use super::{ChildMatcher, IdentityMatch, ReconciliationOp, ReconciliationPlan};

#[doc = include_str!("../../docs/api/reconciliation/reconciler.doc/Reconciler.md")]
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

    #[doc = include_str!("../../docs/api/reconciliation/reconciler.doc/reconcile_root.md")]
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

    #[doc = include_str!("../../docs/api/reconciliation/reconciler.doc/reconcile.md")]
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

    /// Reconciles each sibling group in depth-first order with an explicit work stack.
    /// A stale parent is ignored.
    fn reconcile_into(
        self,
        tree: &mut RetainedTree,
        parent: NodeId,
        desired: &[ViewNode],
        plan: &mut ReconciliationPlan,
    ) {
        let Some(root) = self.siblings(tree, parent, desired, plan) else {
            return;
        };
        let mut pending = vec![root];
        while let Some(frame) = pending.last_mut() {
            let Some(decision) = frame.matches.next() else {
                pending
                    .pop()
                    .expect("active sibling group")
                    .finish(tree, plan);
                continue;
            };
            let node = frame
                .desired
                .find(|node| !is_outlet(node))
                .expect("each desired child has a match");
            let index = frame.new_children.len();
            let parent = frame.parent;
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
                        ViewNode::Element {
                            component,
                            props,
                            events,
                            children,
                            ..
                        } => {
                            // Refresh the authored properties (and the attributes
                            // derived from them), the event bindings, and the §2.12
                            // slot content from the latest desired node. Combine
                            // into a single Update op so a node yields at most one.
                            let props_changed = apply_props(tree, id, props);
                            let events_changed = apply_events(tree, id, events);
                            // Slot content is diffed too, and for a reason of its
                            // own: an `Update` is what marks a mounted component
                            // stale, and content that changed while the props did
                            // not (`<h1 slot="header">A</h1>` → `…B…`) must still
                            // re-render the host, or the projection stays stale.
                            let slots_changed = apply_slot_content(tree, id, *component, children);
                            if props_changed || events_changed || slots_changed {
                                plan.push(ReconciliationOp::Update { id });
                            }
                        }
                    }
                    frame.reused_order.push(id);
                    id
                }
                IdentityMatch::Create => {
                    let id = tree.alloc(node_kind_of(node));
                    // Newly created element nodes carry their authored properties
                    // (and the attributes derived from them), event bindings, and
                    // §2.12 slot content.
                    //
                    // Slot content is captured on the node's FIRST pass, not once
                    // it is known to host a component: a component is mounted from
                    // this plan, so the flag `hosts_component` reads is set after
                    // this runs, and waiting for it would drop the authored content
                    // of every component's opening frame.
                    if let ViewNode::Element {
                        component,
                        props,
                        events,
                        children,
                        ..
                    } = node
                    {
                        apply_props(tree, id, props);
                        apply_events(tree, id, events);
                        apply_slot_content(tree, id, *component, children);
                    }
                    plan.push(ReconciliationOp::Insert { id, parent, index });
                    id
                }
            };

            apply_identity(tree, id, parent, desired_key(node));

            frame.new_children.push(id);

            // A component host owns its rendered children. Authored children
            // are kept as slot content by apply_slot_content instead.
            if let ViewNode::Element { children, .. } = node
                && !hosts_component(tree, id)
                && let Some(child_frame) = self.siblings(tree, id, children.nodes(), plan)
            {
                pending.push(child_frame);
            }
        }
    }

    /// Prepares a sibling group for matching before traversing its children.
    fn siblings<'a>(
        self,
        tree: &RetainedTree,
        parent: NodeId,
        desired: &'a [ViewNode],
        plan: &mut ReconciliationPlan,
    ) -> Option<SiblingReconciliation<'a>> {
        let existing = tree.get(parent)?.child_nodes();
        // Stray slot outlets never materialize in the retained tree.
        let authored = || desired.iter().filter(|node| !is_outlet(node));
        report_duplicate_keys(parent, authored(), plan);
        let (matches, removed) = self
            .matcher
            .match_desired(tree, existing, authored())
            .into_parts();
        let new_children = Vec::with_capacity(matches.len());
        let old_rank = existing
            .iter()
            .enumerate()
            .map(|(rank, &id)| (id, rank))
            .collect();
        Some(SiblingReconciliation {
            parent,
            desired: desired.iter(),
            matches: matches.into_iter(),
            removed,
            old_rank,
            new_children,
            reused_order: Vec::new(),
        })
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

/// Records any key that more than one desired child claims (spec §2.4).
fn report_duplicate_keys<'a>(
    parent: NodeId,
    desired: impl Iterator<Item = &'a ViewNode>,
    plan: &mut ReconciliationPlan,
) {
    let mut counts: FastMap<StableKey, usize> = HashMap::default();
    for node in desired {
        if let Some(key) = desired_key(node) {
            *counts.entry(key).or_insert(0) += 1;
        }
    }
    for (key, count) in counts {
        if count > 1 {
            let () = plan.push_duplicate_key(parent, key, count);
        }
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

#[doc = include_str!("../../docs/api/reconciliation/reconciler.doc/apply_slot_content.md")]
fn apply_slot_content(
    tree: &mut RetainedTree,
    id: NodeId,
    component: ComponentId,
    children: &ViewFragment,
) -> bool {
    let Some(node) = tree.get_mut(id) else {
        return false;
    };
    // Diff before cloning, in both directions: a component host is re-reconciled
    // every structural frame, and the overwhelmingly common outcome is that its
    // authored children are byte-for-byte what they already were. Cloning first
    // and comparing after would deep-copy the content's whole subtree on every one
    // of those frames to discover it had nothing to do.
    if !assigns_slots(children.nodes()) && !may_project_content(component) {
        if node.slot_content().is_empty() {
            return false;
        }
        node.set_slot_content(ViewFragment::new());
        return true;
    }
    if node.slot_content() == children {
        return false;
    }
    node.set_slot_content(children.clone());
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
            .map(|n| n.child_nodes().to_vec())
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
    fn deeply_nested_view_reconciles_and_reuses_every_node() {
        const DEPTH: usize = 20_000;
        let mut fragment = ViewFragment::new();
        for _ in 0..DEPTH {
            fragment = ViewFragment::from_nodes(vec![el(1).with_children(fragment)]);
        }
        let view = View::from_fragment(fragment);
        let mut tree = RetainedTree::new();
        let reconciler = Reconciler::new();
        let first = reconciler.reconcile_root(&mut tree, &view);
        assert_eq!(first.len(), DEPTH + 1);
        assert_eq!(tree.len(), DEPTH + 1);
        assert!(reconciler.reconcile_root(&mut tree, &view).is_empty());

        // Dismantle the fixture independently of the tree walk under test.
        let mut fragments = vec![view.into_fragment()];
        while let Some(fragment) = fragments.pop() {
            fragments.extend(
                fragment
                    .into_nodes()
                    .into_iter()
                    .filter_map(|node| match node {
                        ViewNode::Element { children, .. } => Some(children),
                        ViewNode::Text { .. } => None,
                    }),
            );
        }
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
                ReconciliationOp::Remove { id, .. } => Some(*id),
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
        use obscur_view::view::{CLASS, ID, LOCAL_NAME, ViewProperty, ViewValue};

        let (mut tree, root) = tree_with_root();
        let styled = el(1)
            .with_prop(ViewProperty::new(
                LOCAL_NAME,
                ViewValue::Text("container".into()),
            ))
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
        let node =
            |text: &str| el(1).with_prop(ViewProperty::new(label, ViewValue::Text(text.into())));

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
        assert!(
            plan.is_empty(),
            "an unchanged prop must not report a change"
        );

        // Changing ONLY the non-CSS prop now reports an Update and takes effect.
        let plan = Reconciler::new().reconcile(
            &mut tree,
            root,
            &ViewFragment::from_nodes(vec![node("Cancel")]),
        );
        assert_eq!(
            plan.iter()
                .filter(|op| matches!(op, ReconciliationOp::Update { .. }))
                .count(),
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

        let _ = recon.reconcile(
            &mut tree,
            root,
            &ViewFragment::from_nodes(vec![with_src("a.png")]),
        );
        let id = child_ids(&tree, root)[0];
        assert_eq!(
            tree.get(id).expect("live").attributes().image_source(),
            Some("a.png")
        );

        // Changing the src re-applies the attributes and emits one Update op.
        let plan = recon.reconcile(
            &mut tree,
            root,
            &ViewFragment::from_nodes(vec![with_src("b.png")]),
        );
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
        assert_eq!(
            node.command_for_event(InternedSymbol::from_raw(0xDEAD)),
            None
        );
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
        let bound = |command| keyed_el(1, "e").with_event(ViewEventBinding::new(click, command));

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
