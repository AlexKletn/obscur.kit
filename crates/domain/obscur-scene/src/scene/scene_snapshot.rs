//! The immutable, versioned snapshot handed off to a renderer.

use std::sync::Arc;

use crate::command::DisplayList;
use crate::scene::{Scene, SceneGeneration, SceneStatistics};

#[doc = include_str!("../../docs/api/scene/scene_snapshot.doc/SceneSnapshot.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct SceneSnapshot {
    display_list: Arc<DisplayList>,
    generation: SceneGeneration,
}

impl SceneSnapshot {
    #[doc = include_str!("../../docs/api/scene/scene_snapshot.doc/new.md")]
    #[must_use]
    pub fn new(display_list: DisplayList, generation: SceneGeneration) -> Self {
        Self {
            display_list: Arc::new(display_list),
            generation,
        }
    }

    #[doc = include_str!("../../docs/api/scene/scene_snapshot.doc/from_scene.md")]
    #[must_use]
    pub fn from_scene(scene: &Scene) -> Self {
        Self {
            // The whole point of the shared root: a snapshot of a composed frame
            // costs a refcount bump, not a copy of every command in it.
            display_list: Arc::clone(scene.shared_root()),
            generation: scene.generation(),
        }
    }

    /// The display list a renderer consumes: the frozen geometry, paint order,
    /// and clipping contract (spec §5.2).
    #[must_use]
    pub fn display_list(&self) -> &DisplayList {
        &self.display_list
    }

    #[doc = include_str!("../../docs/api/scene/scene_snapshot.doc/shared_display_list.md")]
    #[must_use]
    pub fn shared_display_list(&self) -> &Arc<DisplayList> {
        &self.display_list
    }

    /// The generation of the scene at the moment this snapshot was captured.
    #[must_use]
    pub fn generation(&self) -> SceneGeneration {
        self.generation
    }

    #[doc = include_str!("../../docs/api/scene/scene_snapshot.doc/statistics.md")]
    #[must_use]
    pub fn statistics(&self) -> SceneStatistics {
        SceneStatistics::from_display_list(&self.display_list)
    }

    #[doc = include_str!("../../docs/api/scene/scene_snapshot.doc/is_stale_against.md")]
    #[must_use]
    pub fn is_stale_against(&self, current: SceneGeneration) -> bool {
        current > self.generation
    }

    #[doc = include_str!("../../docs/api/scene/scene_snapshot.doc/is_balanced.md")]
    #[must_use]
    pub fn is_balanced(&self) -> bool {
        self.display_list.is_balanced()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clip::Clip;
    use crate::color::Color;
    use crate::command::PaintCommand;
    use crate::paint::Paint;
    use crate::path::Path;
    use obscur_geometry::Rect;

    fn draw() -> PaintCommand {
        PaintCommand::DrawPath {
            path: Path::rect(Rect::default()),
            paint: Paint::solid_fill(Color::WHITE),
        }
    }

    #[test]
    fn statistics_are_counted_from_the_list_it_holds() {
        let list = DisplayList::from_commands(vec![draw(), draw()]);
        let snapshot = SceneSnapshot::new(list, SceneGeneration::from_raw(7));
        assert_eq!(snapshot.generation().raw(), 7);
        assert_eq!(snapshot.statistics().draw_count(), 2);
        assert_eq!(snapshot.statistics().command_count(), 2);
    }

    #[test]
    fn is_stale_against_when_generation_advances() {
        let snapshot = SceneSnapshot::new(DisplayList::new(), SceneGeneration::from_raw(3));
        // Same or older generation is not stale.
        assert!(!snapshot.is_stale_against(SceneGeneration::from_raw(3)));
        assert!(!snapshot.is_stale_against(SceneGeneration::from_raw(2)));
        // A strictly newer generation makes it stale.
        assert!(snapshot.is_stale_against(SceneGeneration::from_raw(4)));
    }

    #[test]
    fn stale_once_scene_generation_advances_past_it() {
        let mut scene = Scene::new();
        scene.set_root(DisplayList::from_commands(vec![draw()]));
        let snapshot = scene.snapshot();
        assert!(!snapshot.is_stale_against(scene.generation()));

        scene.set_root(DisplayList::from_commands(vec![draw()]));
        assert!(snapshot.is_stale_against(scene.generation()));
    }

    #[test]
    fn a_snapshot_shares_the_scenes_list_rather_than_copying_it() {
        // The measurable half of "a snapshot is cheap". `from_scene` used to read
        // `scene.root().clone()` — a deep copy of every command in the frame, taken
        // at least once a frame and once more for a frame that composed nothing at
        // all, since the pipeline snapshots outside its `if plan.compose()`. The
        // list is immutable once composed, so the copy bought nothing.
        //
        // Pointer identity is the proof: two `Arc`s that are the same allocation
        // cannot be two copies. Content equality could not tell the two apart.
        let mut scene = Scene::new();
        scene.set_root(DisplayList::from_commands(vec![draw(), draw()]));
        let snapshot = scene.snapshot();
        assert!(
            Arc::ptr_eq(scene.shared_root(), snapshot.shared_display_list()),
            "a snapshot must share the scene's display list, not copy it"
        );
        assert!(
            core::ptr::eq(
                scene.root().commands().as_ptr(),
                snapshot.display_list().commands().as_ptr()
            ),
            "and therefore hand a renderer the very buffer that was composed"
        );
    }

    #[test]
    fn a_shared_list_is_still_frozen_at_the_generation_it_was_taken() {
        // The property the copy was there to protect, kept without it: sharing is
        // safe only because a composed list is never mutated in place — `set_root`
        // replaces the handle, so an already-published snapshot keeps pointing at
        // the frame it captured while the scene moves on.
        let mut scene = Scene::new();
        scene.set_root(DisplayList::from_commands(vec![draw(), draw()]));
        let snapshot = scene.snapshot();

        scene.set_root(DisplayList::new());
        assert_eq!(snapshot.display_list().len(), 2);
        assert_eq!(snapshot.statistics().draw_count(), 2);
        assert!(
            !Arc::ptr_eq(scene.shared_root(), snapshot.shared_display_list()),
            "a new root is a new list, so the old snapshot is left holding the old one"
        );
        assert!(scene.root().is_empty());
    }

    #[test]
    fn is_balanced_delegates_to_display_list() {
        let balanced = SceneSnapshot::new(
            DisplayList::from_commands(vec![
                PaintCommand::PushClip(Clip::rect(Rect::default())),
                draw(),
                PaintCommand::PopClip,
            ]),
            SceneGeneration::FIRST,
        );
        assert!(balanced.is_balanced());

        let unbalanced = SceneSnapshot::new(
            DisplayList::from_commands(vec![PaintCommand::PopClip]),
            SceneGeneration::FIRST,
        );
        assert!(!unbalanced.is_balanced());
    }
}
