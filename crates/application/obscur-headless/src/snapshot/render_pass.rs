//! One backend-neutral render pass in a render-graph description: [`RenderPass`].

/// A single, backend-neutral render pass within a
/// [`RenderGraphSummary`](super::RenderGraphSummary) (spec §7.4).
///
/// A render pass is a structural, GPU-free description of *what* a backend would
/// draw for one stage of the frame: a stable [`label`](Self::label) and the
/// paint-command counts it covers. It fabricates no GPU data — the counts are
/// derived directly from the composed scene's display list, so the same frame
/// always yields the same pass.
///
/// The headless pipeline emits a single `"scene"` pass for an ordinary frame
/// (the composited display list). Extension/custom passes (spec §5.3) and
/// GPU-specific passes are backend-gated and are not modeled here; when a real
/// backend runs them, it may append its own passes to the graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderPass {
    /// A stable, human-readable label for the pass (e.g. `"scene"`).
    label: Box<str>,
    /// The number of draw commands (fills, strokes, text) this pass issues.
    draw_count: usize,
    /// The total number of paint commands this pass covers, including clip and
    /// layer push/pop markers.
    command_count: usize,
}

impl RenderPass {
    /// Builds a render pass labeled `label` covering `draw_count` draw commands
    /// out of `command_count` total commands.
    #[must_use]
    pub fn new(label: impl Into<Box<str>>, draw_count: usize, command_count: usize) -> Self {
        Self {
            label: label.into(),
            draw_count,
            command_count,
        }
    }

    /// The pass label, used for diagnostics and golden-file headers.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    /// The number of draw commands (fills, strokes, text) this pass issues.
    #[must_use]
    pub fn draw_count(&self) -> usize {
        self.draw_count
    }

    /// The total number of paint commands this pass covers.
    #[must_use]
    pub fn command_count(&self) -> usize {
        self.command_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accessors_read_fields() {
        let pass = RenderPass::new("scene", 3, 7);
        assert_eq!(pass.label(), "scene");
        assert_eq!(pass.draw_count(), 3);
        assert_eq!(pass.command_count(), 7);
    }

    #[test]
    fn equal_passes_compare_equal() {
        assert_eq!(
            RenderPass::new("scene", 1, 2),
            RenderPass::new("scene", 1, 2)
        );
        assert_ne!(
            RenderPass::new("scene", 1, 2),
            RenderPass::new("scene", 1, 3)
        );
    }
}
