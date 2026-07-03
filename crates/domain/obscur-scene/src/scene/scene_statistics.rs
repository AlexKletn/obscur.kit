//! Per-scene diagnostic counters for `DevTools` and telemetry.

use crate::command::{DisplayList, PaintCommand};

/// Per-scene counters surfaced to `DevTools` and performance telemetry
/// (spec §7.9).
///
/// These are **diagnostic metrics only**: they describe the shape of a
/// [`DisplayList`] (how many commands, draws, layers, and clips it contains)
/// and never bear on rendering correctness. A renderer must not depend on them;
/// they exist purely for observability.
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
    /// Creates a statistics record from explicit counts.
    ///
    /// The counts are diagnostic only and are stored verbatim; this constructor
    /// does not cross-check them for consistency.
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

    /// The total number of [`PaintCommand`]s in the scene.
    #[must_use]
    pub const fn command_count(self) -> usize {
        self.command_count
    }

    /// The number of [`PaintCommand::DrawPath`] commands in the scene.
    #[must_use]
    pub const fn draw_count(self) -> usize {
        self.draw_count
    }

    /// The number of [`PaintCommand::PushLayer`] commands in the scene.
    #[must_use]
    pub const fn layer_count(self) -> usize {
        self.layer_count
    }

    /// The number of [`PaintCommand::PushClip`] commands in the scene.
    #[must_use]
    pub const fn clip_count(self) -> usize {
        self.clip_count
    }

    /// Computes statistics by folding over a [`DisplayList`], counting each
    /// command category.
    ///
    /// `command_count` is the total command count; `draw_count`, `layer_count`,
    /// and `clip_count` count [`PaintCommand::DrawPath`],
    /// [`PaintCommand::PushLayer`], and [`PaintCommand::PushClip`] respectively.
    /// Pops are counted only towards `command_count`.
    #[must_use]
    pub fn from_display_list(list: &DisplayList) -> Self {
        let mut stats = Self::new(list.len(), 0, 0, 0);
        for command in list {
            match command {
                PaintCommand::DrawPath { .. } => stats.draw_count += 1,
                PaintCommand::PushLayer(_) => stats.layer_count += 1,
                PaintCommand::PushClip(_) => stats.clip_count += 1,
                PaintCommand::PopClip | PaintCommand::PopLayer => {}
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
