use std::collections::HashMap;

use obscur_geometry::AffineTransform;
use obscur_identity::{FastMap, NodeId};
use obscur_tree::tree::RetainedTree;

use crate::orchestration::{ComputedStyles, LayoutBoxes, resolved_transform};

pub(super) struct AncestorGeometry {
    memo: FastMap<NodeId, (i32, Option<AffineTransform>)>,
    pending: Vec<NodeId>,
}

impl AncestorGeometry {
    pub(super) fn new(nodes: usize) -> Self {
        Self {
            memo: HashMap::with_capacity_and_hasher(nodes, Default::default()),
            pending: Vec::new(),
        }
    }

    #[doc = include_str!("../../docs/api/runtime/hit_testing.doc/accumulated_transform.md")]
    pub(super) fn for_node(
        &mut self,
        tree: &RetainedTree,
        computed: &ComputedStyles,
        boxes: &LayoutBoxes,
        node: NodeId,
    ) -> (i32, Option<AffineTransform>) {
        if let Some(&cached) = self.memo.get(&node) {
            return cached;
        }
        self.pending.clear();
        let mut depth = 0_i32;
        let mut chain = None;
        let mut current = node;
        let limit = i32::try_from(tree.len()).unwrap_or(i32::MAX);
        for step in 0..=tree.len() {
            if let Some(&(known_depth, known_transform)) = self.memo.get(&current) {
                depth = known_depth.saturating_add(1).min(limit);
                chain = known_transform;
                break;
            }
            self.pending.push(current);
            let Some(parent) = tree.visual_parent_of(current) else {
                break;
            };
            if step == tree.len() {
                depth = limit;
            }
            current = parent;
        }
        for &ancestor in self.pending.iter().rev() {
            let own = boxes.get(&ancestor).and_then(|layout_box| {
                resolved_transform(computed.get(&ancestor), layout_box.bounds)
            });
            chain = match (chain, own) {
                (Some(outer), Some(inner)) => Some(outer.multiply(inner)),
                (outer, None) => outer,
                (None, inner) => inner,
            };
            self.memo.insert(ancestor, (depth, chain));
            depth = depth.saturating_add(1).min(limit);
        }
        self.memo[&node]
    }
}
