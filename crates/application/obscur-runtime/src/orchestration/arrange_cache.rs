#![doc = include_str!("arrange_cache.doc/_module.md")]

use std::collections::HashSet;

use obscur_geometry::{BoxConstraints, Point, Rect};
use obscur_identity::{FastMap, FastSet, NodeId};
use obscur_tree::tree::RetainedTree;

use super::layout_coordinator::LayoutBoxes;

#[doc = include_str!("arrange_cache.doc/ArrangeKey.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct ArrangeKey {
    #[doc = include_str!("arrange_cache.doc/origin.md")]
    origin: (u32, u32),
    #[doc = include_str!("arrange_cache.doc/constraints.md")]
    constraints: (u32, u32, u32, u32),
    #[doc = include_str!("arrange_cache.doc/containing_block.md")]
    containing_block: (u32, u32, u32, u32),
    #[doc = include_str!("arrange_cache.doc/viewport.md")]
    viewport: (u32, u32, u32, u32),
    #[doc = include_str!("arrange_cache.doc/scroll.md")]
    scroll: (u32, u32),
}

impl ArrangeKey {
    #[doc = include_str!("arrange_cache.doc/new.md")]
    pub(super) fn new(
        origin: Point,
        constraints: BoxConstraints,
        containing_block: Rect,
        viewport: Rect,
        scroll: (f32, f32),
    ) -> Self {
        let rect = |r: Rect| {
            (
                r.origin.x.to_bits(),
                r.origin.y.to_bits(),
                r.size.width.to_bits(),
                r.size.height.to_bits(),
            )
        };
        Self {
            origin: (origin.x.to_bits(), origin.y.to_bits()),
            constraints: (
                constraints.min.width.to_bits(),
                constraints.min.height.to_bits(),
                constraints.max.width.to_bits(),
                constraints.max.height.to_bits(),
            ),
            containing_block: rect(containing_block),
            viewport: rect(viewport),
            scroll: (scroll.0.to_bits(), scroll.1.to_bits()),
        }
    }
}

#[doc = include_str!("arrange_cache.doc/ArrangeCache.md")]
#[derive(Debug, Default)]
pub struct ArrangeCache {
    #[doc = include_str!("arrange_cache.doc/keys.md")]
    keys: std::cell::RefCell<FastMap<NodeId, ArrangeKey>>,
    #[doc = include_str!("arrange_cache.doc/retained.md")]
    retained: std::cell::RefCell<LayoutBoxes>,
    #[doc = include_str!("arrange_cache.doc/arranged.md")]
    arranged: std::cell::RefCell<Vec<NodeId>>,
}

impl ArrangeCache {
    #[doc = include_str!("arrange_cache.doc/new_2.md")]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[doc = include_str!("arrange_cache.doc/begin_pass.md")]
    pub(super) fn begin_pass(&self) {
        self.arranged.borrow_mut().clear();
    }

    #[doc = include_str!("arrange_cache.doc/end_pass.md")]
    pub(super) fn end_pass(&self, boxes: &LayoutBoxes) {
        self.retained.borrow_mut().clone_from(boxes);
    }

    #[doc = include_str!("arrange_cache.doc/clear.md")]
    pub fn clear(&self) {
        self.keys.borrow_mut().clear();
        self.retained.borrow_mut().clear();
        self.arranged.borrow_mut().clear();
    }

    #[doc = include_str!("arrange_cache.doc/evict.md")]
    pub fn evict(&self, nodes: &FastSet<NodeId>) {
        self.keys
            .borrow_mut()
            .retain(|node, _| !nodes.contains(node));
    }

    #[doc = include_str!("arrange_cache.doc/get.md")]
    pub(super) fn get(&self, node: NodeId) -> Option<ArrangeKey> {
        self.keys.borrow().get(&node).copied()
    }

    #[doc = include_str!("arrange_cache.doc/record.md")]
    pub(super) fn record(&self, node: NodeId, key: ArrangeKey) {
        self.keys.borrow_mut().insert(node, key);
        self.arranged.borrow_mut().push(node);
    }

    #[doc = include_str!("arrange_cache.doc/serve.md")]
    pub(super) fn serve(&self, tree: &RetainedTree, node: NodeId, boxes: &mut LayoutBoxes) -> bool {
        let retained = self.retained.borrow();
        if !retained.contains_key(&node) {
            return false;
        }
        // Cycle-safe like every other walk here: `visited` emits each node once.
        let mut visited: FastSet<NodeId> = HashSet::default();
        let mut stack: Vec<NodeId> = vec![node];
        while let Some(current) = stack.pop() {
            if !visited.insert(current) {
                continue;
            }
            if let Some(&retained_box) = retained.get(&current) {
                boxes.insert(current, retained_box);
            }
            if let Some(record) = tree.get(current) {
                stack.extend(record.child_nodes().iter().copied());
            }
        }
        true
    }

    #[doc = include_str!("arrange_cache.doc/arranged_2.md")]
    #[must_use]
    pub fn arranged(&self, node: NodeId) -> bool {
        self.arranged.borrow().contains(&node)
    }

    #[doc = include_str!("arrange_cache.doc/arranged_count.md")]
    #[must_use]
    pub fn arranged_count(&self) -> usize {
        self.arranged.borrow().len()
    }
}
