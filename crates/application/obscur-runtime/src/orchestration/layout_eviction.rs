#![doc = include_str!("layout_eviction.doc/_module.md")]

use std::collections::{HashMap, HashSet};

use obscur_identity::{FastMap, FastSet, NodeId};
use obscur_layout::invalidation::LayoutInvalidation;
use obscur_tree::tree::RetainedTree;

use super::layout_coordinator::MAX_DEPTH;

#[doc = include_str!("layout_eviction.doc/LayoutEviction.md")]
pub(super) struct LayoutEviction {
    #[doc = include_str!("layout_eviction.doc/measures.md")]
    pub(super) measures: FastSet<NodeId>,
    #[doc = include_str!("layout_eviction.doc/arranges.md")]
    pub(super) arranges: FastSet<NodeId>,
}

#[doc = include_str!("layout_eviction.doc/invalidated_layout.md")]
pub(super) fn invalidated_layout(
    tree: &RetainedTree,
    dirty: &LayoutInvalidation,
) -> Option<LayoutEviction> {
    if dirty.is_empty() {
        return Some(LayoutEviction {
            measures: HashSet::default(),
            arranges: HashSet::default(),
        });
    }
    // No root: nothing was arranged, so nothing retained can be trusted.
    let root = tree.root()?;
    // The root's `font-size` is the `rem` base of every node's lengths and text
    // metrics, so a change to it can move any box in the tree. Unboundable.
    if dirty.is_invalidated(root) {
        return None;
    }

    // Parent links derived from `children()` — the exact links the measure
    // recursion follows (see [`children_of`](super::formatting_context::children_of)) — rather than from
    // `logical_parent()`, which is a separately-set field that nothing forces to
    // stay the inverse of the child links. (The reconciler does set both together,
    // and `Portal::apply` moves only the VISUAL parent, so today they agree; this
    // walk simply does not need them to.) Cycle-safe: `visited` emits each node
    // once.
    let mut parent_of: FastMap<NodeId, NodeId> = HashMap::default();
    let mut visited: FastSet<NodeId> = HashSet::default();
    let mut order: Vec<NodeId> = Vec::new();
    let mut stack: Vec<NodeId> = vec![root];
    while let Some(node) = stack.pop() {
        if !visited.insert(node) {
            continue;
        }
        order.push(node);
        let Some(record) = tree.get(node) else {
            continue;
        };
        for &child in record.child_nodes() {
            if !visited.contains(&child) {
                parent_of.insert(child, node);
                stack.push(child);
            }
        }
    }
    // A dirty node the walk never reached has no known ancestor chain, so the set
    // of measures it can have changed is unknown.
    if dirty.iter().any(|(node, _)| !visited.contains(node)) {
        return None;
    }

    let mut evict: FastSet<NodeId> = HashSet::default();
    for (&node, _) in dirty.iter() {
        evict.insert(node);
        // Upward: every context that measured *through* this node.
        let mut current = node;
        for _ in 0..MAX_DEPTH {
            let Some(&parent) = parent_of.get(&current) else {
                break;
            };
            evict.insert(parent);
            current = parent;
        }
    }
    // A text node reads its container's computed `text-overflow`, so a dirty
    // node's children re-measure. Both link kinds are checked (see above).
    for &node in &order {
        let structural = parent_of.get(&node).copied();
        let logical = tree.get(node).and_then(|n| n.logical_parent().parent());
        if structural.is_some_and(|parent| dirty.is_invalidated(parent))
            || logical.is_some_and(|parent| dirty.is_invalidated(parent))
        {
            evict.insert(node);
        }
    }

    // `arrange(N)` writes every box in N's subtree, so it must re-run when
    // ANYTHING in that subtree can have changed: the upward closure of `measures`
    // (see [`LayoutEviction::arranges`]). Same walk, same parent links, same
    // `MAX_DEPTH` bound as the upward pass above.
    let mut arranges: FastSet<NodeId> = HashSet::default();
    for &node in &evict {
        arranges.insert(node);
        let mut current = node;
        for _ in 0..MAX_DEPTH {
            let Some(&parent) = parent_of.get(&current) else {
                break;
            };
            arranges.insert(parent);
            current = parent;
        }
    }
    Some(LayoutEviction {
        measures: evict,
        arranges,
    })
}
