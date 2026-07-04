//! A single entry in a [`DisplayList`](crate::command::DisplayList).

use crate::clip::Clip;
use crate::layer::SceneLayer;
use crate::paint::Paint;
use crate::path::Path;

/// One drawing operation in a display list (spec §5.1).
///
/// A [`DisplayList`](crate::command::DisplayList) is a flat, ordered stream of
/// these commands. Each command is either a **leaf** ([`PaintCommand::DrawPath`])
/// or one half of a **push/pop pair** that brackets a nested region of the
/// stream:
///
/// - [`PaintCommand::PushClip`] / [`PaintCommand::PopClip`] bracket a region
///   confined to a clip.
/// - [`PaintCommand::PushLayer`] / [`PaintCommand::PopLayer`] bracket a region
///   composited into a layer.
///
/// Push/pop pairs must **balance**: every push has a matching pop, they nest
/// (last pushed is first popped), and the running nesting depth never goes
/// negative. A well-formed list is balanced; [`nesting_delta`] lets callers
/// check this by folding over the stream. [`PaintCommand::DrawPath`] is a leaf
/// and contributes no nesting.
///
/// [`nesting_delta`]: PaintCommand::nesting_delta
#[derive(Debug, Clone, PartialEq)]
pub enum PaintCommand {
    /// Fill or stroke `path`. The [`Paint`] decides which: a fill paint fills
    /// the path's interior, a stroke paint draws its outline. This is a leaf
    /// command and never affects nesting.
    DrawPath {
        /// The vector outline to paint.
        path: Path,
        /// How the outline is painted (fill or stroke).
        paint: Paint,
    },
    /// Draw a run of text. A LEAF command (no nesting effect). The origin is the
    /// run's top-left in absolute scene coordinates; the renderer shapes the
    /// string with its own font and places the baseline. Shaping is deliberately
    /// deferred to the renderer in this first cut (a full pipeline would shape via
    /// obscur-text and carry positioned glyphs).
    DrawText {
        /// Top-left origin of the run, in absolute scene coordinates.
        origin: obscur_geometry::Point,
        /// The run's text content.
        text: Box<str>,
        /// Font size in pixels.
        font_size: f32,
        /// The brush the glyphs are painted with.
        brush: crate::paint::Brush,
    },
    /// Push a clip onto the clip stack. Subsequent drawing is confined to the
    /// clip until the matching [`PaintCommand::PopClip`].
    PushClip(Clip),
    /// Pop the most recently pushed clip, ending its region.
    PopClip,
    /// Begin a composited layer described by the given [`SceneLayer`].
    /// Subsequent drawing composites into the layer until the matching
    /// [`PaintCommand::PopLayer`].
    PushLayer(SceneLayer),
    /// End the most recently pushed layer, compositing it into its parent.
    PopLayer,
}

impl PaintCommand {
    /// Returns whether this command opens a nested region, i.e. it is a
    /// [`PaintCommand::PushClip`] or [`PaintCommand::PushLayer`].
    #[must_use]
    pub const fn is_push(&self) -> bool {
        matches!(self, Self::PushClip(_) | Self::PushLayer(_))
    }

    /// Returns whether this command closes a nested region, i.e. it is a
    /// [`PaintCommand::PopClip`] or [`PaintCommand::PopLayer`].
    #[must_use]
    pub const fn is_pop(&self) -> bool {
        matches!(self, Self::PopClip | Self::PopLayer)
    }

    /// The command's effect on nesting depth: `+1` for a push, `-1` for a pop,
    /// and `0` for the leaf [`PaintCommand::DrawPath`].
    ///
    /// Folding this over a stream and requiring the running total to stay
    /// non-negative and end at zero is exactly the balance check a renderer
    /// relies on.
    #[must_use]
    pub const fn nesting_delta(&self) -> i32 {
        match self {
            Self::PushClip(_) | Self::PushLayer(_) => 1,
            Self::PopClip | Self::PopLayer => -1,
            Self::DrawPath { .. } | Self::DrawText { .. } => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Color;
    use crate::paint::{Brush, Stroke};
    use obscur_geometry::{Point, Rect, Size};

    fn sample_path() -> Path {
        Path::rect(Rect::new(Point::new(0.0, 0.0), Size::new(1.0, 1.0)))
    }

    #[test]
    fn nesting_delta_values() {
        let draw = PaintCommand::DrawPath {
            path: sample_path(),
            paint: Paint::solid_fill(Color::WHITE),
        };
        assert_eq!(draw.nesting_delta(), 0);
        assert_eq!(
            PaintCommand::PushClip(Clip::rect(Rect::default())).nesting_delta(),
            1
        );
        assert_eq!(PaintCommand::PopClip.nesting_delta(), -1);
        assert_eq!(
            PaintCommand::PushLayer(SceneLayer::new()).nesting_delta(),
            1
        );
        assert_eq!(PaintCommand::PopLayer.nesting_delta(), -1);
    }

    #[test]
    fn push_and_pop_predicates() {
        assert!(PaintCommand::PushClip(Clip::rect(Rect::default())).is_push());
        assert!(PaintCommand::PushLayer(SceneLayer::new()).is_push());
        assert!(PaintCommand::PopClip.is_pop());
        assert!(PaintCommand::PopLayer.is_pop());

        let draw = PaintCommand::DrawPath {
            path: sample_path(),
            paint: Paint::stroke(Stroke::new(Brush::solid(Color::BLACK), 1.0)),
        };
        assert!(!draw.is_push());
        assert!(!draw.is_pop());
    }

    #[test]
    fn draw_text_is_a_leaf() {
        let text = PaintCommand::DrawText {
            origin: Point::new(2.0, 3.0),
            text: "hi".into(),
            font_size: 16.0,
            brush: Brush::solid(Color::WHITE),
        };
        assert_eq!(text.nesting_delta(), 0);
        assert!(!text.is_push());
        assert!(!text.is_pop());
    }
}
