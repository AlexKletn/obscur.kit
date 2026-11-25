use crate::path::Path;

use super::VectorPaint;

#[doc = include_str!("../../docs/api/paint/vector_shape.doc/VectorShape.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct VectorShape {
    /// The outline, in the document's own coordinate space.
    pub path: Path,
    /// The interior paint, or [`None`] for `fill="none"`.
    pub fill: Option<VectorPaint>,
    /// The outline paint and its width, or [`None`] for `stroke="none"`.
    pub stroke: Option<(VectorPaint, f32)>,
}
