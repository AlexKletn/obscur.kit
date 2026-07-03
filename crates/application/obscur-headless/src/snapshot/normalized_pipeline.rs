//! A stable, order-independent form of a pipeline snapshot: [`NormalizedPipeline`].

use obscur_identity::NodeId;

use super::NormalizedNode;

/// A stable, deterministically-ordered form of a
/// [`PipelineSnapshot`](super::PipelineSnapshot), for reproducible golden diffs
/// (spec §7.14).
///
/// A raw [`PipelineSnapshot`](super::PipelineSnapshot) stores its per-node stage
/// data in [`HashMap`](std::collections::HashMap)s whose iteration order is
/// unstable, so it is not directly suitable as golden output. A
/// `NormalizedPipeline` flattens that data into a `NodeId`-**sorted**
/// [`Vec<NormalizedNode>`] plus a few scalar summaries. Because the sort is
/// total and deterministic, two normalizations of the same frame produce
/// byte-for-byte-equal output — the property golden tests rely on (§7.14).
///
/// Produce one with [`SnapshotNormalizer`](super::SnapshotNormalizer).
#[derive(Debug, Clone, PartialEq)]
pub struct NormalizedPipeline {
    /// The retained-tree root at capture, if any.
    root: Option<NodeId>,
    /// The number of nodes in the retained-tree snapshot.
    node_count: usize,
    /// One row per node present in the computed styles or the layout boxes,
    /// sorted by [`NodeId`].
    nodes: Vec<NormalizedNode>,
    /// The number of paint commands (fills) in the composed scene.
    fill_count: usize,
    /// Whether the composed scene's display list is balanced.
    scene_balanced: bool,
}

impl NormalizedPipeline {
    /// Builds a normalized pipeline from its already-computed parts.
    ///
    /// `nodes` is expected to be sorted by [`NodeId`];
    /// [`SnapshotNormalizer`](super::SnapshotNormalizer) guarantees this. The
    /// constructor stores the vector as given and does not re-sort.
    #[must_use]
    pub fn new(
        root: Option<NodeId>,
        node_count: usize,
        nodes: Vec<NormalizedNode>,
        fill_count: usize,
        scene_balanced: bool,
    ) -> Self {
        Self {
            root,
            node_count,
            nodes,
            fill_count,
            scene_balanced,
        }
    }

    /// Returns the retained-tree root at capture, if any.
    #[must_use]
    pub fn root(&self) -> Option<NodeId> {
        self.root
    }

    /// Returns the number of nodes in the retained-tree snapshot.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.node_count
    }

    /// Returns the `NodeId`-sorted per-node rows.
    #[must_use]
    pub fn nodes(&self) -> &[NormalizedNode] {
        &self.nodes
    }

    /// Returns the number of paint commands (fills) in the composed scene.
    #[must_use]
    pub fn fill_count(&self) -> usize {
        self.fill_count
    }

    /// Returns whether the composed scene's display list is balanced.
    #[must_use]
    pub fn scene_balanced(&self) -> bool {
        self.scene_balanced
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accessors_read_fields() {
        let rows = vec![NormalizedNode::new(NodeId::from_raw(1), None, None)];
        let normalized =
            NormalizedPipeline::new(Some(NodeId::from_raw(0)), 3, rows.clone(), 2, true);
        assert_eq!(normalized.root(), Some(NodeId::from_raw(0)));
        assert_eq!(normalized.node_count(), 3);
        assert_eq!(normalized.nodes(), rows.as_slice());
        assert_eq!(normalized.fill_count(), 2);
        assert!(normalized.scene_balanced());
    }

    #[test]
    fn equal_pipelines_compare_equal() {
        let a = NormalizedPipeline::new(None, 0, Vec::new(), 0, true);
        let b = NormalizedPipeline::new(None, 0, Vec::new(), 0, true);
        assert_eq!(a, b);
    }
}
