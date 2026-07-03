//! The immutable, versioned snapshot handed off to a renderer.

use crate::command::DisplayList;
use crate::scene::{Scene, SceneGeneration, SceneStatistics};

/// An **immutable, versioned** snapshot of a [`Scene`], the hand-off a renderer
/// backend consumes (spec §1.6, §5.2).
///
/// A snapshot is **immutable after publication**: it exposes only read-only
/// accessors and no mutators. The runtime may retain it and later discard it as
/// stale once the live scene's generation advances past it — see
/// [`SceneSnapshot::is_stale_against`] (spec §1.6).
///
/// The snapshot is the exact contract a renderer honours: its
/// [`DisplayList`] fixes the geometry, paint order, and clipping that every
/// backend must reproduce. Rasterization may differ between backends, but the
/// scene geometry and command order must not (spec §5.2).
#[derive(Debug, Clone, PartialEq)]
pub struct SceneSnapshot {
    display_list: DisplayList,
    generation: SceneGeneration,
    statistics: SceneStatistics,
}

impl SceneSnapshot {
    /// Builds a snapshot directly from a display list at a given generation,
    /// computing the [`SceneStatistics`] from the list.
    #[must_use]
    pub fn new(display_list: DisplayList, generation: SceneGeneration) -> Self {
        let statistics = SceneStatistics::from_display_list(&display_list);
        Self {
            display_list,
            generation,
            statistics,
        }
    }

    /// Captures an immutable snapshot of `scene`'s current state.
    ///
    /// Clones the scene's root display list and captures its generation and
    /// statistics. This is what [`Scene::snapshot`] calls.
    #[must_use]
    pub fn from_scene(scene: &Scene) -> Self {
        Self {
            display_list: scene.root().clone(),
            generation: scene.generation(),
            statistics: scene.statistics(),
        }
    }

    /// The display list a renderer consumes: the frozen geometry, paint order,
    /// and clipping contract (spec §5.2).
    #[must_use]
    pub fn display_list(&self) -> &DisplayList {
        &self.display_list
    }

    /// The generation of the scene at the moment this snapshot was captured.
    #[must_use]
    pub fn generation(&self) -> SceneGeneration {
        self.generation
    }

    /// The diagnostic statistics captured with this snapshot.
    #[must_use]
    pub fn statistics(&self) -> SceneStatistics {
        self.statistics
    }

    /// Whether this snapshot is stale relative to a `current` scene generation.
    ///
    /// Returns `true` when `current` is strictly newer than this snapshot's
    /// generation, i.e. the live scene advanced its content after the snapshot
    /// was taken, so the snapshot may be discarded (spec §1.6).
    #[must_use]
    pub fn is_stale_against(&self, current: SceneGeneration) -> bool {
        current > self.generation
    }

    /// Whether the snapshot's display list is balanced, i.e. it is a well-formed
    /// stream a renderer can consume.
    ///
    /// Delegates to [`DisplayList::is_balanced`].
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
    fn new_computes_statistics_from_list() {
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
