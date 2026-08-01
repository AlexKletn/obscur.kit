#![doc = include_str!("normalized_pipeline.doc/_module.md")]

use obscur_identity::NodeId;

use super::{ImageSnapshot, NormalizedNode};

#[doc = include_str!("normalized_pipeline.doc/NormalizedPipeline.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct NormalizedPipeline {
    #[doc = include_str!("normalized_pipeline.doc/root.md")]
    root: Option<NodeId>,
    #[doc = include_str!("normalized_pipeline.doc/node_count.md")]
    node_count: usize,
    #[doc = include_str!("normalized_pipeline.doc/nodes.md")]
    nodes: Vec<NormalizedNode>,
    #[doc = include_str!("normalized_pipeline.doc/fill_count.md")]
    fill_count: usize,
    #[doc = include_str!("normalized_pipeline.doc/scene_balanced.md")]
    scene_balanced: bool,
    #[doc = include_str!("normalized_pipeline.doc/paint_command_count.md")]
    paint_command_count: usize,
    #[doc = include_str!("normalized_pipeline.doc/accessibility_node_count.md")]
    accessibility_node_count: usize,
    #[doc = include_str!("normalized_pipeline.doc/accessibility_root.md")]
    accessibility_root: Option<NodeId>,
    #[doc = include_str!("normalized_pipeline.doc/render_pass_count.md")]
    render_pass_count: usize,
    #[doc = include_str!("normalized_pipeline.doc/image.md")]
    image: Option<ImageSnapshot>,
}

impl NormalizedPipeline {
    #[doc = include_str!("normalized_pipeline.doc/new.md")]
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

    #[doc = include_str!("normalized_pipeline.doc/root_2.md")]
    #[must_use]
    pub fn root(&self) -> Option<NodeId> {
        self.root
    }

    #[doc = include_str!("normalized_pipeline.doc/node_count_2.md")]
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.node_count
    }

    #[doc = include_str!("normalized_pipeline.doc/nodes_2.md")]
    #[must_use]
    pub fn nodes(&self) -> &[NormalizedNode] {
        &self.nodes
    }

    #[doc = include_str!("normalized_pipeline.doc/fill_count_2.md")]
    #[must_use]
    pub fn fill_count(&self) -> usize {
        self.fill_count
    }

    #[doc = include_str!("normalized_pipeline.doc/scene_balanced_2.md")]
    #[must_use]
    pub fn scene_balanced(&self) -> bool {
        self.scene_balanced
    }

    #[doc = include_str!("normalized_pipeline.doc/paint_command_count_2.md")]
    #[must_use]
    pub fn paint_command_count(&self) -> usize {
        self.paint_command_count
    }

    #[doc = include_str!("normalized_pipeline.doc/accessibility_node_count_2.md")]
    #[must_use]
    pub fn accessibility_node_count(&self) -> usize {
        self.accessibility_node_count
    }

    #[doc = include_str!("normalized_pipeline.doc/accessibility_root_2.md")]
    #[must_use]
    pub fn accessibility_root(&self) -> Option<NodeId> {
        self.accessibility_root
    }

    #[doc = include_str!("normalized_pipeline.doc/render_pass_count_2.md")]
    #[must_use]
    pub fn render_pass_count(&self) -> usize {
        self.render_pass_count
    }

    #[doc = include_str!("normalized_pipeline.doc/image_2.md")]
    #[must_use]
    pub fn image(&self) -> Option<&ImageSnapshot> {
        self.image.as_ref()
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
        let normalized = sample(Some(NodeId::from_raw(0)), Some(image.clone()));
        assert_eq!(normalized.root(), Some(NodeId::from_raw(0)));
        assert_eq!(normalized.node_count(), 3);
        assert_eq!(normalized.nodes().len(), 1);
        assert_eq!(normalized.fill_count(), 2);
        assert!(normalized.scene_balanced());
        assert_eq!(normalized.paint_command_count(), 5);
        assert_eq!(normalized.accessibility_node_count(), 3);
        assert_eq!(normalized.accessibility_root(), Some(NodeId::from_raw(0)));
        assert_eq!(normalized.render_pass_count(), 1);
        assert_eq!(normalized.image(), Some(&image));
    }

    #[test]
    fn equal_pipelines_compare_equal() {
        assert_eq!(sample(None, None), sample(None, None));
    }
}
