//! The §4.2 layout cache: [`ArrangeCache`], and the [`ArrangeKey`] it retains a
//! subtree under.

use std::collections::HashSet;

use obscur_identity::{FastMap, FastSet, NodeId};
use obscur_layout::invalidation::LayoutInvalidation;
use obscur_tree::tree::RetainedTree;

use super::arrange_key::ArrangeKey;
use super::layout_coordinator::LayoutBoxes;
use super::layout_eviction::{LayoutEviction, invalidated_layout};
use super::layout_topology::LayoutTopology;

#[doc = include_str!("../../docs/api/orchestration/arrange_cache.doc/ArrangeCache.md")]
#[derive(Debug, Default)]
pub struct ArrangeCache {
    topology: std::cell::RefCell<Option<LayoutTopology>>,
    /// The key each node was last arranged under.
    keys: std::cell::RefCell<FastMap<NodeId, ArrangeKey>>,
    /// Every absolute box the last completed pass produced.
    retained: std::cell::RefCell<LayoutBoxes>,
    /// The nodes whose box the current pass actually computed. Reset per pass.
    arranged: std::cell::RefCell<Vec<NodeId>>,
}

impl ArrangeCache {
    /// Creates an empty cache.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Starts a pass: forgets what the previous pass arranged, keeping the
    /// retained keys and boxes.
    pub(super) fn begin_pass(&self) {
        self.arranged.borrow_mut().clear();
    }

    /// Ends a pass: `boxes` becomes what a later pass may serve subtrees from.
    pub(super) fn end_pass(&self, boxes: &LayoutBoxes) {
        self.retained.borrow_mut().clone_from(boxes);
    }

    pub(crate) fn forget_topology(&self) {
        self.topology.borrow_mut().take();
    }

    pub(crate) fn retain_topology(&self, tree: &RetainedTree) {
        static REBUILD: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        if !*REBUILD.get_or_init(|| {
            std::env::var_os("OBSCUR_FULL_LAYOUT_TOPOLOGY").is_some_and(|value| value == "1")
        }) {
            let mut topology = self.topology.borrow_mut();
            if topology.is_none() {
                *topology = tree.root().map(|root| LayoutTopology::new(tree, root));
            }
        }
    }

    pub(super) fn invalidated_layout(
        &self,
        tree: &RetainedTree,
        dirty: &LayoutInvalidation,
    ) -> Option<LayoutEviction> {
        let topology = self.topology.borrow();
        match topology.as_ref() {
            Some(topology) => topology.evictions(dirty),
            None => invalidated_layout(tree, dirty),
        }
    }

    /// Forgets everything retained.
    pub fn clear(&self) {
        self.forget_topology();
        self.keys.borrow_mut().clear();
        self.retained.borrow_mut().clear();
        self.arranged.borrow_mut().clear();
    }

    #[doc = include_str!("../../docs/api/orchestration/arrange_cache.doc/evict.md")]
    pub fn evict(&self, nodes: &FastSet<NodeId>) {
        self.keys
            .borrow_mut()
            .retain(|node, _| !nodes.contains(node));
    }

    /// The key `node` was last arranged under, if it may still be served.
    pub(super) fn get(&self, node: NodeId) -> Option<ArrangeKey> {
        self.keys.borrow().get(&node).copied()
    }

    /// Records that this pass arranged `node` under `key`.
    pub(super) fn record(&self, node: NodeId, key: ArrangeKey) {
        self.keys.borrow_mut().insert(node, key);
        self.arranged.borrow_mut().push(node);
    }

    #[doc = include_str!("../../docs/api/orchestration/arrange_cache.doc/serve.md")]
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

    /// Whether the last pass **re-arranged** `node`, as opposed to serving its
    /// subtree from retained boxes or never reaching it.
    #[must_use]
    pub fn arranged(&self, node: NodeId) -> bool {
        self.arranged.borrow().contains(&node)
    }

    /// How many nodes the last pass re-arranged.
    #[must_use]
    pub fn arranged_count(&self) -> usize {
        self.arranged.borrow().len()
    }
}
