//! A stable, order-independent form of a pipeline snapshot: [`NormalizedPipeline`].

use obscur_identity::NodeId;

use super::{ImageSnapshot, NormalizedNode};

/// A stable, deterministically-ordered form of a
/// [`PipelineSnapshot`](super::PipelineSnapshot), for reproducible golden diffs
/// (spec §7.14).
///
/// A raw [`PipelineSnapshot`](super::PipelineSnapshot) stores its per-node stage
/// data in [`HashMap`](std::collections::HashMap)s whose iteration order is
/// unstable, so it is not directly suitable as golden output. A
/// `NormalizedPipeline` flattens that data into a `NodeId`-**sorted**
/// [`Vec<NormalizedNode>`] plus a few scalar summaries — one per §7.4 stage.
/// Because the ordering is total and deterministic, two normalizations of the
/// same frame produce byte-for-byte-equal output — the property golden tests
/// rely on (§7.14).
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
    /// The number of paint commands in the paint-command stream (spec §7.4).
    paint_command_count: usize,
    /// The number of nodes in the accessibility (semantic) snapshot (spec §7.1).
    accessibility_node_count: usize,
    /// The accessibility tree root at capture, if any (spec §7.1).
    accessibility_root: Option<NodeId>,
    /// The number of passes in the backend-neutral render graph (spec §7.4).
    render_pass_count: usize,
    /// The rendered image, if a backend attached one (spec §7.4).
    image: Option<ImageSnapshot>,
}

impl NormalizedPipeline {
    /// Builds a normalized pipeline from its already-computed parts.
    ///
    /// `nodes` is expected to be sorted by [`NodeId`];
    /// [`SnapshotNormalizer`](super::SnapshotNormalizer) guarantees this. The
    /// constructor stores the vector as given and does not re-sort.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        root: Option<NodeId>,
        node_count: usize,
        nodes: Vec<NormalizedNode>,
        fill_count: usize,
        scene_balanced: bool,
        paint_command_count: usize,
        accessibility_node_count: usize,
        accessibility_root: Option<NodeId>,
        render_pass_count: usize,
        image: Option<ImageSnapshot>,
    ) -> Self {
        Self {
            root,
            node_count,
            nodes,
            fill_count,
            scene_balanced,
            paint_command_count,
            accessibility_node_count,
            accessibility_root,
            render_pass_count,
            image,
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

    /// Returns the number of paint commands in the paint-command stream
    /// (spec §7.4).
    #[must_use]
    pub fn paint_command_count(&self) -> usize {
        self.paint_command_count
    }

    /// Returns the number of nodes in the accessibility snapshot (spec §7.1).
    #[must_use]
    pub fn accessibility_node_count(&self) -> usize {
        self.accessibility_node_count
    }

    /// Returns the accessibility tree root at capture, if any (spec §7.1).
    #[must_use]
    pub fn accessibility_root(&self) -> Option<NodeId> {
        self.accessibility_root
    }

    /// Returns the number of passes in the render graph (spec §7.4).
    #[must_use]
    pub fn render_pass_count(&self) -> usize {
        self.render_pass_count
    }

    /// Returns the rendered image, if a backend attached one (spec §7.4).
    #[must_use]
    pub fn image(&self) -> Option<ImageSnapshot> {
        self.image
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(root: Option<NodeId>, image: Option<ImageSnapshot>) -> NormalizedPipeline {
        NormalizedPipeline::new(
            root,
            3,
            vec![NormalizedNode::new(NodeId::from_raw(1), None, None)],
            2,
            true,
            5,
            3,
            root,
            1,
            image,
        )
    }

    #[test]
    fn accessors_read_fields() {
        let image = ImageSnapshot::from_parts(4, 4, 42);
        let normalized = sample(Some(NodeId::from_raw(0)), Some(image));
        assert_eq!(normalized.root(), Some(NodeId::from_raw(0)));
        assert_eq!(normalized.node_count(), 3);
        assert_eq!(normalized.nodes().len(), 1);
        assert_eq!(normalized.fill_count(), 2);
        assert!(normalized.scene_balanced());
        assert_eq!(normalized.paint_command_count(), 5);
        assert_eq!(normalized.accessibility_node_count(), 3);
        assert_eq!(normalized.accessibility_root(), Some(NodeId::from_raw(0)));
        assert_eq!(normalized.render_pass_count(), 1);
        assert_eq!(normalized.image(), Some(image));
    }

    #[test]
    fn equal_pipelines_compare_equal() {
        assert_eq!(sample(None, None), sample(None, None));
    }
}
