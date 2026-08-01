#![doc = include_str!("scene_statistics.doc/_module.md")]

use crate::command::{DisplayList, PaintCommand};

#[doc = include_str!("scene_statistics.doc/SceneStatistics.md")]
// The shared `_count` postfix is intentional: every field is a diagnostic
// tally, and the postfix documents that uniformly (spec §7.9).
#[allow(clippy::struct_field_names)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SceneStatistics {
    command_count: usize,
    draw_count: usize,
    layer_count: usize,
    clip_count: usize,
}

impl SceneStatistics {
    #[doc = include_str!("scene_statistics.doc/new.md")]
    #[must_use]
    pub const fn new(
        command_count: usize,
        draw_count: usize,
        layer_count: usize,
        clip_count: usize,
    ) -> Self {
        Self {
            command_count,
            draw_count,
            layer_count,
            clip_count,
        }
    }

    #[doc = include_str!("scene_statistics.doc/command_count.md")]
    #[must_use]
    pub const fn command_count(self) -> usize {
        self.command_count
    }

    #[doc = include_str!("scene_statistics.doc/draw_count.md")]
    #[must_use]
    pub const fn draw_count(self) -> usize {
        self.draw_count
    }

    #[doc = include_str!("scene_statistics.doc/layer_count.md")]
    #[must_use]
    pub const fn layer_count(self) -> usize {
        self.layer_count
    }

    #[doc = include_str!("scene_statistics.doc/clip_count.md")]
    #[must_use]
    pub const fn clip_count(self) -> usize {
        self.clip_count
    }

    #[doc = include_str!("scene_statistics.doc/from_display_list.md")]
    #[must_use]
    pub fn from_display_list(list: &DisplayList) -> Self {
        let mut stats = Self::new(list.len(), 0, 0, 0);
        for command in list {
            match command {
                PaintCommand::DrawPath { .. }
                | PaintCommand::DrawText { .. }
                | PaintCommand::DrawShadow { .. }
                | PaintCommand::DrawImage { .. }
                | PaintCommand::BackdropFilter { .. } => {
                    stats.draw_count += 1;
                }
                PaintCommand::PushLayer(_) => stats.layer_count += 1,
                PaintCommand::PushClip(_) => stats.clip_count += 1,
                // Transform brackets are counted only towards `command_count`
                // (they are neither a draw, a layer, nor a clip).
                PaintCommand::PopClip
                | PaintCommand::PopLayer
                | PaintCommand::PushTransform(_)
                | PaintCommand::PopTransform => {}
            }
        }
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clip::Clip;
    use crate::color::Color;
    use crate::layer::SceneLayer;
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
    fn accessors_return_constructed_values() {
        let stats = SceneStatistics::new(10, 4, 2, 3);
        assert_eq!(stats.command_count(), 10);
        assert_eq!(stats.draw_count(), 4);
        assert_eq!(stats.layer_count(), 2);
        assert_eq!(stats.clip_count(), 3);
    }

    #[test]
    fn from_display_list_counts_each_category() {
        let list = DisplayList::from_commands(vec![
            PaintCommand::PushLayer(SceneLayer::new()),
            PaintCommand::PushClip(Clip::rect(Rect::default())),
            draw(),
            draw(),
            PaintCommand::PopClip,
            PaintCommand::PopLayer,
        ]);
        let stats = SceneStatistics::from_display_list(&list);
        assert_eq!(stats.command_count(), 6);
        assert_eq!(stats.draw_count(), 2);
        assert_eq!(stats.layer_count(), 1);
        assert_eq!(stats.clip_count(), 1);
    }

    #[test]
    fn empty_list_has_zero_counts() {
        let stats = SceneStatistics::from_display_list(&DisplayList::new());
        assert_eq!(stats, SceneStatistics::new(0, 0, 0, 0));
    }
}
