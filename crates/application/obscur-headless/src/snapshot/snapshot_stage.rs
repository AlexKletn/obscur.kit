//! Identifies which pipeline stage a snapshot captures: [`SnapshotStage`].

/// The pipeline stage a snapshot captures (spec §7.4).
///
/// Headless mode can output every stage of the frame pipeline for inspection
/// and golden testing (spec §7.4). §7.4 enumerates eight stages, and this enum
/// names all of them so a snapshot, a diff, or a diagnostic can refer to a stage
/// without string dispatch:
///
/// 1. the reconciled retained tree;
/// 2. the computed styles;
/// 3. the layout boxes;
/// 4. the composed scene;
/// 5. the paint-command stream;
/// 6. the accessibility (semantic) tree;
/// 7. the render graph;
/// 8. the rendered image.
///
/// # Ordering
///
/// The variants are declared in **pipeline order** — the exact order §7.4 lists
/// them — and the derived [`Ord`]/[`PartialOrd`] follow that declaration order.
/// Comparing two stages therefore tells you which one runs earlier in the frame.
///
/// # Backend dependence
///
/// The first seven stages are backend-independent and are captured
/// deterministically without a GPU (see
/// [`PipelineSnapshot`](super::PipelineSnapshot)). Only [`Image`](Self::Image) is
/// backend-dependent: a raster is produced by a concrete renderer (e.g. the
/// software `obscur-render-cpu` backend), so the headless pipeline models it as
/// an optional stage carried by
/// [`ImageSnapshot`](super::ImageSnapshot) rather than deriving it in
/// [`capture`](super::PipelineSnapshot::capture).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SnapshotStage {
    /// The reconciled retained tree — the structural result of
    /// `View → reconcile` (spec §5.13).
    RetainedTree,
    /// The per-node computed styles resolved from the tree (spec §4.14).
    ComputedStyle,
    /// The per-node absolute layout boxes arranged under the viewport
    /// (spec §4.2).
    Layout,
    /// The composed, paint-ordered scene display list (spec §5.1).
    Scene,
    /// The flat paint-command stream a rasterizer consumes, in paint order
    /// (spec §5.1, §7.4).
    Paint,
    /// The semantic accessibility tree linked to the retained tree
    /// (spec §7.1, §7.4).
    Accessibility,
    /// The backend-neutral render-graph description: the ordered render passes
    /// and their target (spec §7.4).
    RenderGraph,
    /// The rendered raster image — the one backend-dependent stage, produced by
    /// a concrete renderer (spec §7.4).
    Image,
}

impl SnapshotStage {
    /// Returns a stable, human-readable name for this stage, suitable for
    /// diagnostics and golden-file headers.
    ///
    /// The strings are part of the type's stable contract: they do not change
    /// with build configuration and are safe to embed in reproducible output.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::RetainedTree => "retained-tree",
            Self::ComputedStyle => "computed-style",
            Self::Layout => "layout",
            Self::Scene => "scene",
            Self::Paint => "paint",
            Self::Accessibility => "accessibility",
            Self::RenderGraph => "render-graph",
            Self::Image => "image",
        }
    }

    /// Returns whether this stage is backend-dependent, i.e. it can only be
    /// produced by a concrete renderer (spec §7.4).
    ///
    /// Every stage except [`Image`](Self::Image) is derived deterministically
    /// from the retained tree with no GPU; the rendered image requires a
    /// rasterizing backend.
    #[must_use]
    pub const fn is_backend_dependent(self) -> bool {
        matches!(self, Self::Image)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_stable() {
        assert_eq!(SnapshotStage::RetainedTree.name(), "retained-tree");
        assert_eq!(SnapshotStage::ComputedStyle.name(), "computed-style");
        assert_eq!(SnapshotStage::Layout.name(), "layout");
        assert_eq!(SnapshotStage::Scene.name(), "scene");
        assert_eq!(SnapshotStage::Paint.name(), "paint");
        assert_eq!(SnapshotStage::Accessibility.name(), "accessibility");
        assert_eq!(SnapshotStage::RenderGraph.name(), "render-graph");
        assert_eq!(SnapshotStage::Image.name(), "image");
    }

    #[test]
    fn ordering_follows_pipeline_order() {
        assert!(SnapshotStage::RetainedTree < SnapshotStage::ComputedStyle);
        assert!(SnapshotStage::ComputedStyle < SnapshotStage::Layout);
        assert!(SnapshotStage::Layout < SnapshotStage::Scene);
        assert!(SnapshotStage::Scene < SnapshotStage::Paint);
        assert!(SnapshotStage::Paint < SnapshotStage::Accessibility);
        assert!(SnapshotStage::Accessibility < SnapshotStage::RenderGraph);
        assert!(SnapshotStage::RenderGraph < SnapshotStage::Image);
    }

    #[test]
    fn only_image_is_backend_dependent() {
        assert!(SnapshotStage::Image.is_backend_dependent());
        for stage in [
            SnapshotStage::RetainedTree,
            SnapshotStage::ComputedStyle,
            SnapshotStage::Layout,
            SnapshotStage::Scene,
            SnapshotStage::Paint,
            SnapshotStage::Accessibility,
            SnapshotStage::RenderGraph,
        ] {
            assert!(
                !stage.is_backend_dependent(),
                "{stage:?} is backend-neutral"
            );
        }
    }

    #[test]
    fn is_copy_eq_and_hashable() {
        use std::collections::HashSet;

        let stage = SnapshotStage::Layout;
        let copy = stage;
        assert_eq!(stage, copy);

        let mut set = HashSet::new();
        set.insert(SnapshotStage::Scene);
        set.insert(SnapshotStage::Scene);
        assert_eq!(set.len(), 1);
    }
}
