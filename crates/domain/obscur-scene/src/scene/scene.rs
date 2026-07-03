//! The mutable scene the runtime builds up each frame.

use crate::command::DisplayList;
use crate::scene::{SceneGeneration, SceneSnapshot, SceneStatistics};

/// The **mutable** Scene/Paint model: a root [`DisplayList`] paired with the
/// [`SceneGeneration`] stamping its content (spec §1.6, §5.2).
///
/// The runtime owns a `Scene` and mutates it while building a frame — for
/// example by replacing the root with [`Scene::set_root`], which advances the
/// generation to record that the content changed. When the frame is ready, the
/// runtime publishes an immutable [`SceneSnapshot`] via [`Scene::snapshot`] and
/// hands it to a renderer. The renderer only ever sees the snapshot, never this
/// mutable scene.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Scene {
    root: DisplayList,
    generation: SceneGeneration,
}

impl Scene {
    /// Creates an empty scene: an empty root display list at
    /// [`SceneGeneration::FIRST`].
    #[must_use]
    pub fn new() -> Self {
        Self {
            root: DisplayList::new(),
            generation: SceneGeneration::FIRST,
        }
    }

    /// The root display list the scene is currently composed of.
    #[must_use]
    pub fn root(&self) -> &DisplayList {
        &self.root
    }

    /// Replaces the root display list and **advances the generation**.
    ///
    /// Replacing the root is a content change, so the generation moves to
    /// [`SceneGeneration::next`]; any [`SceneSnapshot`] taken before this call
    /// now reports stale against the new generation.
    pub fn set_root(&mut self, list: DisplayList) {
        self.root = list;
        self.generation = self.generation.next();
    }

    /// The current generation stamping the scene's content.
    #[must_use]
    pub fn generation(&self) -> SceneGeneration {
        self.generation
    }

    /// Whether the scene has no content, i.e. its root display list is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.root.is_empty()
    }

    /// Computes the diagnostic [`SceneStatistics`] for the current root.
    #[must_use]
    pub fn statistics(&self) -> SceneStatistics {
        SceneStatistics::from_display_list(&self.root)
    }

    /// Produces an immutable [`SceneSnapshot`] of the current state.
    ///
    /// The snapshot clones the root display list, captures the current
    /// generation, and computes the statistics; it is thereafter independent of
    /// this scene and unaffected by later mutation.
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
