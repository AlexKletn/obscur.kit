#![doc = include_str!("frame_output.doc/_module.md")]

use obscur_identity::RuntimeId;
use obscur_scene::scene::{Scene, SceneSnapshot};
use obscur_tree::tree::RetainedTree;

use crate::orchestration::LayoutBoxes;

use super::UiRuntime;

impl UiRuntime {
    #[doc = include_str!("frame_output.doc/tree.md")]
    #[must_use]
    pub fn tree(&self) -> &RetainedTree {
        &self.tree
    }

    #[doc = include_str!("frame_output.doc/commit_version.md")]
    #[must_use]
    pub fn commit_version(&self) -> obscur_identity::VersionStamp {
        self.enactor.version()
    }

    #[doc = include_str!("frame_output.doc/scene.md")]
    #[must_use]
    pub fn scene(&self) -> &Scene {
        &self.scene
    }

    #[doc = include_str!("frame_output.doc/snapshot.md")]
    #[must_use]
    pub fn snapshot(&self) -> SceneSnapshot {
        self.scene.snapshot()
    }

    #[doc = include_str!("frame_output.doc/layout_boxes.md")]
    #[must_use]
    pub fn layout_boxes(&self) -> &LayoutBoxes {
        self.scroll.projection(&self.boxes)
    }

    #[doc = include_str!("frame_output.doc/runtime_id.md")]
    #[must_use]
    pub fn runtime_id(&self) -> RuntimeId {
        self.id
    }

    #[doc = include_str!("frame_output.doc/id.md")]
    #[must_use]
    pub fn id(&self) -> RuntimeId {
        self.id
    }
}
