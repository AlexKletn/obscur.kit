#![doc = include_str!("../../../docs/api/runtime/ui_runtime/frame_output.doc/_module.md")]

use obscur_geometry::CornerRadii;
use obscur_identity::RuntimeId;
use obscur_scene::scene::{Scene, SceneSnapshot};
use obscur_tree::tree::RetainedTree;

use crate::orchestration::LayoutBoxes;

use super::UiRuntime;

impl UiRuntime {
    /// Returns the retained tree carried across frames.
    #[must_use]
    pub fn tree(&self) -> &RetainedTree {
        &self.tree
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/frame_output.doc/commit_version.md")]
    #[must_use]
    pub fn commit_version(&self) -> obscur_identity::VersionStamp {
        self.enactor.version()
    }

    /// Returns the mutable scene's current state (as a shared reference).
    #[must_use]
    pub fn scene(&self) -> &Scene {
        &self.scene
    }

    /// Returns an immutable snapshot of the current scene — the hand-off a
    /// renderer consumes.
    #[must_use]
    pub fn snapshot(&self) -> SceneSnapshot {
        self.scene.snapshot()
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/frame_output.doc/layout_boxes.md")]
    #[must_use]
    pub fn layout_boxes(&self) -> &LayoutBoxes {
        self.scroll.projection(&self.boxes)
    }

    /// Resolved CSS corners of a root that covers the window viewport.
    #[must_use]
    pub fn viewport_corner_radii(&self) -> CornerRadii {
        let Some(document) = self.tree.root().and_then(|root| self.tree.get(root)) else {
            return CornerRadii::default();
        };
        let &[root] = document.child_nodes() else {
            return CornerRadii::default();
        };
        let Some(layout) = self.boxes.get(&root) else {
            return CornerRadii::default();
        };
        let bounds = layout.bounds;
        let viewport = self.viewport();
        if bounds.origin.x.abs() > 0.01
            || bounds.origin.y.abs() > 0.01
            || (bounds.size.width - viewport.width).abs() > 0.01
            || (bounds.size.height - viewport.height).abs() > 0.01
        {
            return CornerRadii::default();
        }
        crate::orchestration::corner_radii_of(self.computed.get(&root), Some(bounds))
            .normalized(bounds)
    }

    /// Returns this runtime's identity.
    #[must_use]
    pub fn runtime_id(&self) -> RuntimeId {
        self.id
    }

    /// Returns this runtime's identity (alias of [`runtime_id`](Self::runtime_id)).
    #[must_use]
    pub fn id(&self) -> RuntimeId {
        self.id
    }
}
