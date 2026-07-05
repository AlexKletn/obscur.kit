//! Snapshot module.
mod pipeline_snapshot;
pub use pipeline_snapshot::PipelineSnapshot;
mod normalized_node;
pub use normalized_node::NormalizedNode;
mod normalized_pipeline;
pub use normalized_pipeline::NormalizedPipeline;
mod snapshot_normalizer;
pub use snapshot_normalizer::SnapshotNormalizer;
mod snapshot_stage;
pub use snapshot_stage::SnapshotStage;
mod render_pass;
pub use render_pass::RenderPass;
mod render_graph_summary;
pub use render_graph_summary::RenderGraphSummary;
mod image_snapshot;
pub use image_snapshot::ImageSnapshot;
