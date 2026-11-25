use std::collections::{HashMap, HashSet};

use obscur_identity::{FastMap, FastSet, NodeId};
use obscur_layout::invalidation::LayoutInvalidation;
use obscur_tree::tree::RetainedTree;

use super::layout_coordinator::MAX_DEPTH;
use super::layout_eviction::LayoutEviction;

#[derive(Debug)]
pub(super) struct LayoutTopology {
    root: NodeId,
    parents: FastMap<NodeId, NodeId>,
    reachable: FastSet<NodeId>,
    text_by_parent: FastMap<NodeId, Vec<NodeId>>,
}

impl LayoutTopology {
    pub(super) fn new(tree: &RetainedTree, root: NodeId) -> Self {
        let mut topology = Self {
            root,
            parents: HashMap::default(),
            reachable: HashSet::default(),
            text_by_parent: HashMap::default(),
        };
        // Layout follows retained child edges. Text styling also reads its
        // logical parent's text-overflow, so index that dependency separately.
        let mut stack = vec![root];
        while let Some(node) = stack.pop() {
            if !topology.reachable.insert(node) {
                continue;
            }
            let Some(record) = tree.get(node) else {
                continue;
            };
            if record.kind().text().is_some()
                && let Some(parent) = tree.visual_parent_of(node)
            {
                topology
                    .text_by_parent
                    .entry(parent)
                    .or_default()
                    .push(node);
            }
            for child in tree.visual_children(node) {
                if !topology.reachable.contains(&child) {
                    topology.parents.insert(child, node);
                    stack.push(child);
                }
            }
        }
        topology
    }

    pub(super) fn evictions(&self, dirty: &LayoutInvalidation) -> Option<LayoutEviction> {
        if dirty.is_invalidated(self.root)
            || dirty.iter().any(|(node, _)| !self.reachable.contains(node))
        {
            return None;
        }
        let mut measures = HashSet::default();
        for (&node, _) in dirty.iter() {
            self.add_ancestors(node, &mut measures);
            if let Some(text) = self.text_by_parent.get(&node) {
                measures.extend(text.iter().copied());
            }
        }
        // Arrange writes every box in a subtree, so it must rerun for all
        // ancestors of affected measurements, including dependent text.
        let mut arranges = HashSet::default();
        for &node in &measures {
            self.add_ancestors(node, &mut arranges);
        }
        Some(LayoutEviction { measures, arranges })
    }

    fn add_ancestors(&self, node: NodeId, out: &mut FastSet<NodeId>) {
        out.insert(node);
        let mut current = node;
        for _ in 0..MAX_DEPTH {
            let Some(&parent) = self.parents.get(&current) else {
                break;
            };
            out.insert(parent);
            current = parent;
        }
    }
}
