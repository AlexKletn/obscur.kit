#![doc = include_str!("render_frame.doc/_module.md")]

use obscur_geometry::Size;
use obscur_scene::scene::{SceneGeneration, SceneSnapshot};

#[doc = include_str!("render_frame.doc/RenderFrame.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct RenderFrame {
    scene: SceneSnapshot,
    viewport: Size,
}

impl RenderFrame {
    #[doc = include_str!("render_frame.doc/new.md")]
    #[must_use]
    pub fn new(scene: SceneSnapshot, viewport: Size) -> Self {
        Self { scene, viewport }
    }

    #[doc = include_str!("render_frame.doc/scene.md")]
    #[must_use]
    pub const fn scene(&self) -> &SceneSnapshot {
        &self.scene
    }

    #[doc = include_str!("render_frame.doc/viewport.md")]
    #[must_use]
    pub const fn viewport(&self) -> Size {
        self.viewport
    }

    #[doc = include_str!("render_frame.doc/generation.md")]
    #[must_use]
    pub fn generation(&self) -> SceneGeneration {
        self.scene.generation()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_scene::command::DisplayList;

    fn snapshot(generation: u64) -> SceneSnapshot {
        SceneSnapshot::new(DisplayList::new(), SceneGeneration::from_raw(generation))
    }

    #[test]
    fn accessors_return_constructed_values() {
        let frame = RenderFrame::new(snapshot(5), Size::new(800.0, 600.0));
        assert_eq!(frame.viewport(), Size::new(800.0, 600.0));
        assert_eq!(frame.scene().generation(), SceneGeneration::from_raw(5));
    }

    #[test]
    fn generation_passes_through_from_snapshot() {
        let frame = RenderFrame::new(snapshot(9), Size::new(1.0, 1.0));
        assert_eq!(frame.generation(), SceneGeneration::from_raw(9));
        assert_eq!(frame.generation(), frame.scene().generation());
    }
}
