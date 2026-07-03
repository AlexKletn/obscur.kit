//! Identifies which pipeline stage a snapshot captures: [`SnapshotStage`].

/// The pipeline stage a snapshot captures (spec §7.4).
///
/// Headless mode can output every stage of the frame pipeline for inspection
/// and golden testing (spec §7.4): the retained tree, the computed styles, the
/// layout boxes, and the composed scene. This enum names those stages so a
/// snapshot, a diff, or a diagnostic can refer to a stage without string
/// dispatch.
///
/// # Ordering
///
/// The variants are declared in **pipeline order** (tree → style → layout →
/// scene), and the derived [`Ord`]/[`PartialOrd`] follow that declaration order.
/// Comparing two stages therefore tells you which one runs earlier in the frame.
///
/// # Later additions
///
/// The accessibility tree is another backend-independent stage the spec lists
/// (§7.4, §7.14), but it is produced by a later phase and is intentionally not a
/// variant here yet. New stages are appended in pipeline order so existing
/// orderings stay meaningful.
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
        }
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
    }

    #[test]
    fn ordering_follows_pipeline_order() {
        assert!(SnapshotStage::RetainedTree < SnapshotStage::ComputedStyle);
        assert!(SnapshotStage::ComputedStyle < SnapshotStage::Layout);
        assert!(SnapshotStage::Layout < SnapshotStage::Scene);
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
