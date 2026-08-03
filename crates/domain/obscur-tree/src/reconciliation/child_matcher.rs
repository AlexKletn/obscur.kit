#![doc = include_str!("child_matcher.doc/_module.md")]

use std::collections::{HashMap, HashSet};

use obscur_identity::{NodeId, StableKey};
use obscur_view::view::{ViewKey, ViewNode};

use crate::node::NodeKind;
use crate::tree::RetainedTree;

use super::{ChildMatching, IdentityMatch};

#[doc = include_str!("child_matcher.doc/ChildMatcher.md")]
#[derive(Debug, Clone, Copy, Default)]
pub struct ChildMatcher;

impl ChildMatcher {
    #[doc = include_str!("child_matcher.doc/new.md")]
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    #[doc = include_str!("child_matcher.doc/match_children.md")]
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
            let decision = if let Some(key) = node.key().map(ViewKey::stable_key) {
                match key_index.get(key) {
                    Some(&(id, kind)) if !used.contains(&id) && compatible(node, kind) => {
                        used.insert(id);
                        IdentityMatch::Reuse(id)
                    }
                    _ => IdentityMatch::Create,
                }
            } else {
                match next_unused_compatible(&unkeyed, &used, node, cursor) {
                    Some(index) => {
                        let (eid, _) = unkeyed[index];
                        used.insert(eid);
                        cursor = index + 1;
                        IdentityMatch::Reuse(eid)
                    }
                    None => IdentityMatch::Create,
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

fn next_unused_compatible(
    unkeyed: &[(NodeId, &NodeKind)],
    used: &HashSet<NodeId>,
    desired: &ViewNode,
    from: usize,
) -> Option<usize> {
    let mut index = from;
    while index < unkeyed.len() {
        let (id, kind) = unkeyed[index];
        if !used.contains(&id) && compatible(desired, kind) {
            return Some(index);
        }
        index += 1;
    }
    None
}

#[doc = include_str!("child_matcher.doc/compatible.md")]
fn compatible(desired: &ViewNode, kind: &NodeKind) -> bool {
    match (desired, kind) {
        (
            ViewNode::Element { component, .. },
            NodeKind::Element {
                component: existing,
            },
        ) => component == existing,
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
        let a = tree.alloc(NodeKind::Text {
            content: "x".into(),
        });
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
        let t = tree.alloc(NodeKind::Text {
            content: "t".into(),
        });
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
