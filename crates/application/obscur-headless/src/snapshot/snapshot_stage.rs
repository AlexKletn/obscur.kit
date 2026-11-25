//! Identifies which pipeline stage a snapshot captures: [`SnapshotStage`].

#[doc = include_str!("../../docs/api/snapshot/snapshot_stage.doc/SnapshotStage.md")]
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
    #[doc = include_str!("../../docs/api/snapshot/snapshot_stage.doc/name.md")]
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

    #[doc = include_str!("../../docs/api/snapshot/snapshot_stage.doc/is_backend_dependent.md")]
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
