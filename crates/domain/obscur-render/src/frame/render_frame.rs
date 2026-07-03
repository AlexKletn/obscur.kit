//! A scene snapshot ready to render at a viewport (spec §5.2).

use obscur_geometry::Size;
use obscur_scene::scene::{SceneGeneration, SceneSnapshot};

/// A [`SceneSnapshot`] paired with the viewport it is to be rendered at
/// (spec §5.2).
///
/// The frame carries the backend-agnostic [`SceneSnapshot`] — the
/// determinism-boundary artifact of spec §5.2. Its display list fixes the
/// geometry, paint order, and clipping that every backend must reproduce
/// identically; only rasterization may differ. The [`Self::viewport`] is the
/// target area in physical pixels the snapshot renders into.
///
/// This type is not `Copy`: it owns a [`SceneSnapshot`], which holds a display
/// list.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderFrame {
    scene: SceneSnapshot,
    viewport: Size,
}

impl RenderFrame {
    /// Pairs a [`SceneSnapshot`] with the physical-pixel `viewport` it renders
    /// into.
    #[must_use]
    pub fn new(scene: SceneSnapshot, viewport: Size) -> Self {
        Self { scene, viewport }
    }

    /// The backend-agnostic scene snapshot this frame renders (spec §5.2).
    #[must_use]
    pub const fn scene(&self) -> &SceneSnapshot {
        &self.scene
    }

    /// The target area this frame renders into, in physical pixels.
    #[must_use]
    pub const fn viewport(&self) -> Size {
        self.viewport
    }

    /// The generation of the scene snapshot this frame carries.
    ///
    /// Equal to [`SceneSnapshot::generation`]; used to check whether the frame
    /// that reached the screen is still current (spec §5.2, §3.5).
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
