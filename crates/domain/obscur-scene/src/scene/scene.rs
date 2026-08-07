#![doc = include_str!("scene.doc/_module.md")]

use std::sync::Arc;

use crate::command::DisplayList;
use crate::scene::{SceneGeneration, SceneSnapshot, SceneStatistics};

#[doc = include_str!("scene.doc/Scene.md")]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Scene {
    // Shared rather than owned outright so that `snapshot` — called at least once
    // a frame, and once more per frame that composed nothing — is a refcount bump
    // instead of a deep copy of every command in the frame.
    root: Arc<DisplayList>,
    generation: SceneGeneration,
}

impl Scene {
    #[doc = include_str!("scene.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self {
            root: Arc::new(DisplayList::new()),
            generation: SceneGeneration::FIRST,
        }
    }

    #[doc = include_str!("scene.doc/root.md")]
    #[must_use]
    pub fn root(&self) -> &DisplayList {
        &self.root
    }

    #[doc = include_str!("scene.doc/shared_root.md")]
    #[must_use]
    pub fn shared_root(&self) -> &Arc<DisplayList> {
        &self.root
    }

    #[doc = include_str!("scene.doc/set_root.md")]
    pub fn set_root(&mut self, list: DisplayList) {
        self.root = Arc::new(list);
        self.generation = self.generation.next();
    }

    #[doc = include_str!("scene.doc/generation.md")]
    #[must_use]
    pub fn generation(&self) -> SceneGeneration {
        self.generation
    }

    #[doc = include_str!("scene.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.root.is_empty()
    }

    #[doc = include_str!("scene.doc/statistics.md")]
    #[must_use]
    pub fn statistics(&self) -> SceneStatistics {
        SceneStatistics::from_display_list(&self.root)
    }

    #[doc = include_str!("scene.doc/snapshot.md")]
    #[must_use]
    pub fn snapshot(&self) -> SceneSnapshot {
        SceneSnapshot::from_scene(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Color;
    use crate::command::PaintCommand;
    use crate::paint::Paint;
    use crate::path::Path;
    use obscur_geometry::Rect;

    fn draw_list() -> DisplayList {
        DisplayList::from_commands(vec![PaintCommand::DrawPath {
            path: Path::rect(Rect::default()),
            paint: Paint::solid_fill(Color::WHITE),
        }])
    }

    #[test]
    fn new_and_default_are_empty_at_first_generation() {
        let scene = Scene::new();
        assert!(scene.is_empty());
        assert_eq!(scene.generation(), SceneGeneration::FIRST);
        assert_eq!(scene, Scene::default());
    }

    #[test]
    fn set_root_replaces_content_and_advances_generation() {
        let mut scene = Scene::new();
        let before = scene.generation();
        scene.set_root(draw_list());
        assert!(!scene.is_empty());
        assert!(scene.generation() > before);
        assert_eq!(scene.generation(), before.next());
        assert_eq!(scene.statistics().draw_count(), 1);
    }

    #[test]
    fn snapshot_captures_state_and_is_unaffected_by_later_mutation() {
        let mut scene = Scene::new();
        scene.set_root(draw_list());
        let snapshot = scene.snapshot();
        let captured_generation = snapshot.generation();
        assert_eq!(captured_generation, scene.generation());
        assert_eq!(snapshot.statistics().draw_count(), 1);

        // Mutating the scene must not affect the already-taken snapshot.
        scene.set_root(DisplayList::new());
        assert_eq!(snapshot.generation(), captured_generation);
        assert_eq!(snapshot.statistics().draw_count(), 1);
        assert!(scene.generation() > snapshot.generation());
    }
}
