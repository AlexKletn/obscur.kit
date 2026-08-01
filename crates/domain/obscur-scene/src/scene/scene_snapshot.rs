#![doc = include_str!("scene_snapshot.doc/_module.md")]

use crate::command::DisplayList;
use crate::scene::{Scene, SceneGeneration, SceneStatistics};

#[doc = include_str!("scene_snapshot.doc/SceneSnapshot.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct SceneSnapshot {
    display_list: DisplayList,
    generation: SceneGeneration,
    statistics: SceneStatistics,
}

impl SceneSnapshot {
    #[doc = include_str!("scene_snapshot.doc/new.md")]
    #[must_use]
    pub fn new(display_list: DisplayList, generation: SceneGeneration) -> Self {
        let statistics = SceneStatistics::from_display_list(&display_list);
        Self {
            display_list,
            generation,
            statistics,
        }
    }

    #[doc = include_str!("scene_snapshot.doc/from_scene.md")]
    #[must_use]
    pub fn from_scene(scene: &Scene) -> Self {
        Self {
            display_list: scene.root().clone(),
            generation: scene.generation(),
            statistics: scene.statistics(),
        }
    }

    #[doc = include_str!("scene_snapshot.doc/display_list.md")]
    #[must_use]
    pub fn display_list(&self) -> &DisplayList {
        &self.display_list
    }

    #[doc = include_str!("scene_snapshot.doc/generation.md")]
    #[must_use]
    pub fn generation(&self) -> SceneGeneration {
        self.generation
    }

    #[doc = include_str!("scene_snapshot.doc/statistics.md")]
    #[must_use]
    pub fn statistics(&self) -> SceneStatistics {
        self.statistics
    }

    #[doc = include_str!("scene_snapshot.doc/is_stale_against.md")]
    #[must_use]
    pub fn is_stale_against(&self, current: SceneGeneration) -> bool {
        current > self.generation
    }

    #[doc = include_str!("scene_snapshot.doc/is_balanced.md")]
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
