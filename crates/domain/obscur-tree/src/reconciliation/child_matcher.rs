//! `ChildMatcher`: the pure keyed-then-unkeyed child matching policy.

use std::collections::{HashMap, HashSet};

use obscur_identity::{NodeId, StableKey};
use obscur_view::view::{ViewKey, ViewNode};

use crate::node::NodeKind;
use crate::tree::RetainedTree;

use super::{ChildMatching, IdentityMatch};

/// Decides, for one parent, which existing retained children a desired child
/// sequence should reuse, create, and remove.
///
/// This is the identity-preservation heart of reconciliation (spec §2.2,
/// §2.11). It is a **pure** decision function: it only reads node `kind`/`key`
/// through a borrowed [`RetainedTree`] and never mutates the tree. The
/// [`Reconciler`](super::Reconciler) applies the resulting [`ChildMatching`].
///
/// # Matching policy
///
/// A desired child may only *reuse* an existing child that is **compatible**:
/// the same node variant, and — for elements — the same
/// [`ComponentId`](obscur_identity::ComponentId). A text child reuses a text
/// child. An incompatible pairing is never an in-place kind change: the old
/// node is removed and a new one is created (a replace).
///
/// Matching proceeds in two independent lanes:
///
/// * **Keyed** desired children (those carrying a [`ViewKey`]) match the
///   existing child with the *equal* [`StableKey`], regardless of position.
///   This is what preserves identity across a **reorder**. A key index is built
///   first (first existing occurrence of a duplicate key wins, deterministically).
///   If the keyed match is incompatible, or the key is unmatched, or already
///   claimed, the desired child is created.
/// * **Unkeyed** desired children match, scanning forward and never revisiting,
///   the next unused **compatible** unkeyed existing child; otherwise they are
///   created. Keyed existing children are reserved for key matching and are
///   never consumed by an unkeyed desired child.
///
/// Any existing child left unmatched after both lanes is reported for removal.
/// The whole procedure is deterministic and panic-free; stale existing ids that
/// no longer resolve in the tree are ignored.
#[derive(Debug, Clone, Copy, Default)]
pub struct ChildMatcher;

impl ChildMatcher {
    /// Creates a matcher. The matcher is stateless.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Computes the reuse/create/remove decision for a parent's children.
    ///
    /// `existing` is the parent's current ordered child ids (resolved against
    /// `tree`); `desired` is the parent's desired child sequence. The returned
    /// [`ChildMatching`] carries one decision per desired child in order, plus
    /// the unmatched existing children to remove.
    #[must_use]
    pub fn match_children(
        &self,
        tree: &RetainedTree,
        existing: &[NodeId],
        desired: &[ViewNode],
    ) -> ChildMatching {
        // Snapshot the resolvable existing children with their key and kind.
        // Stale ids (whose slot was reused or removed) are dropped here and are
        // therefore neither reused nor reported as removed.
        let mut infos: Vec<(NodeId, Option<&StableKey>, &NodeKind)> = Vec::new();
        for &id in existing {
            if let Some(node) = tree.get(id) {
                infos.push((id, node.key(), node.kind()));
            }
        }

        // Key index: first existing occurrence of a key wins (deterministic).
        let mut key_index: HashMap<&StableKey, (NodeId, &NodeKind)> = HashMap::new();
        for &(id, key, kind) in &infos {
            if let Some(k) = key {
                key_index.entry(k).or_insert((id, kind));
            }
        }

        // Unkeyed existing children, in order, are the only candidates for
        // unkeyed desired children.
        let unkeyed: Vec<(NodeId, &NodeKind)> = infos
            .iter()
            .filter(|(_, key, _)| key.is_none())
            .map(|&(id, _, kind)| (id, kind))
            .collect();

        let mut used: HashSet<NodeId> = HashSet::new();
        let mut matches: Vec<IdentityMatch> = Vec::with_capacity(desired.len());
        let mut cursor: usize = 0;

        for node in desired {
            let decision = match node.key().map(ViewKey::stable_key) {
                Some(key) => match key_index.get(key) {
                    Some(&(id, kind)) if !used.contains(&id) && compatible(node, kind) => {
                        used.insert(id);
                        IdentityMatch::Reuse(id)
                    }
                    _ => IdentityMatch::Create,
                },
                None => {
                    // Forward scan for the next unused compatible unkeyed child.
                    let mut found: Option<usize> = None;
                    let mut i = cursor;
                    while i < unkeyed.len() {
                        let (eid, ekind) = unkeyed[i];
                        if !used.contains(&eid) && compatible(node, ekind) {
                            found = Some(i);
                            break;
                        }
                        i += 1;
                    }
                    match found {
                        Some(i) => {
                            let (eid, _) = unkeyed[i];
                            used.insert(eid);
                            cursor = i + 1;
                            IdentityMatch::Reuse(eid)
                        }
                        None => IdentityMatch::Create,
                    }
                }
            };
            matches.push(decision);
        }

        // Everything resolvable and unmatched is removed, in existing order.
        let removed: Vec<NodeId> = infos
            .iter()
            .map(|&(id, _, _)| id)
            .filter(|id| !used.contains(id))
            .collect();

        ChildMatching::new(matches, removed)
    }
}

/// Returns `true` if `desired` may reuse an existing node of `kind` in place:
/// same variant, and the same component for elements.
fn compatible(desired: &ViewNode, kind: &NodeKind) -> bool {
    match (desired, kind) {
        (ViewNode::Element { component, .. }, NodeKind::Element { component: existing }) => {
            component == existing
        }
        (ViewNode::Text { .. }, NodeKind::Text { .. }) => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_identity::ComponentId;

    fn el_kind(raw: u64) -> NodeKind {
        NodeKind::Element {
            component: ComponentId::from_raw(raw),
        }
    }

    fn el_node(raw: u64) -> ViewNode {
        ViewNode::element(ComponentId::from_raw(raw))
    }

    /// Allocates an existing keyed element child and returns its id.
    fn alloc_keyed_el(tree: &mut RetainedTree, comp: u64, key: StableKey) -> NodeId {
        let id = tree.alloc(el_kind(comp));
        if let Some(node) = tree.get_mut(id) {
            node.set_key(Some(key));
        }
        id
    }

    #[test]
    fn keyed_children_match_by_key_across_reorder() {
        let mut tree = RetainedTree::new();
        let a = alloc_keyed_el(&mut tree, 1, StableKey::from("a"));
        let b = alloc_keyed_el(&mut tree, 2, StableKey::from("b"));
        let c = alloc_keyed_el(&mut tree, 3, StableKey::from("c"));

        // Desired order reversed; each keyed to the matching existing child.
        let desired = vec![
            el_node(3).with_key("c"),
            el_node(1).with_key("a"),
            el_node(2).with_key("b"),
        ];

        let matching = ChildMatcher::new().match_children(&tree, &[a, b, c], &desired);
        assert_eq!(
            matching.matches(),
            &[
                IdentityMatch::Reuse(c),
                IdentityMatch::Reuse(a),
                IdentityMatch::Reuse(b),
            ]
        );
        assert!(matching.removed().is_empty());
    }

    #[test]
    fn incompatible_keyed_match_is_replace() {
        let mut tree = RetainedTree::new();
        // Existing 'a' is a text node; desired 'a' is an element -> replace.
        let a = tree.alloc(NodeKind::Text { content: "x".into() });
        if let Some(n) = tree.get_mut(a) {
            n.set_key(Some(StableKey::from("a")));
        }
        let desired = vec![el_node(1).with_key("a")];
        let matching = ChildMatcher::new().match_children(&tree, &[a], &desired);
        assert_eq!(matching.matches(), &[IdentityMatch::Create]);
        assert_eq!(matching.removed(), &[a]);
    }

    #[test]
    fn unkeyed_matches_positionally_and_skips_incompatible() {
        let mut tree = RetainedTree::new();
        let e1 = tree.alloc(el_kind(1));
        let t = tree.alloc(NodeKind::Text { content: "t".into() });
        // desired: [text, element(1)] against existing [element(1), text]
        let desired = vec![ViewNode::text("t"), el_node(1)];
        let matching = ChildMatcher::new().match_children(&tree, &[e1, t], &desired);
        // First desired (text) scans forward past the incompatible element and
        // reuses the text; second desired (element) can no longer reach the
        // earlier element (cursor advanced) so it is created.
        assert_eq!(
            matching.matches(),
            &[IdentityMatch::Reuse(t), IdentityMatch::Create]
        );
        assert_eq!(matching.removed(), &[e1]);
    }

    #[test]
    fn unkeyed_reuses_compatible_prefix() {
        let mut tree = RetainedTree::new();
        let a = tree.alloc(el_kind(1));
        let b = tree.alloc(el_kind(1));
        let desired = vec![el_node(1), el_node(1), el_node(1)];
        let matching = ChildMatcher::new().match_children(&tree, &[a, b], &desired);
        assert_eq!(
            matching.matches(),
            &[
                IdentityMatch::Reuse(a),
                IdentityMatch::Reuse(b),
                IdentityMatch::Create,
            ]
        );
        assert!(matching.removed().is_empty());
    }

    #[test]
    fn duplicate_desired_key_creates_second() {
        let mut tree = RetainedTree::new();
        let a = alloc_keyed_el(&mut tree, 1, StableKey::from("a"));
        let desired = vec![el_node(1).with_key("a"), el_node(1).with_key("a")];
        let matching = ChildMatcher::new().match_children(&tree, &[a], &desired);
        assert_eq!(
            matching.matches(),
            &[IdentityMatch::Reuse(a), IdentityMatch::Create]
        );
    }

    #[test]
    fn stale_existing_ids_are_ignored() {
        let mut tree = RetainedTree::new();
        let a = tree.alloc(el_kind(1));
        tree.remove(a);
        let desired = vec![el_node(1)];
        let matching = ChildMatcher::new().match_children(&tree, &[a], &desired);
        assert_eq!(matching.matches(), &[IdentityMatch::Create]);
        assert!(matching.removed().is_empty());
    }
}
