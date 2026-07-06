//! A single entry in a [`DisplayList`](crate::command::DisplayList).

use obscur_geometry::{AffineTransform, CornerRadii, Rect};

use crate::clip::Clip;
use crate::color::Color;
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
    /// Draw a run of pre-shaped text. A LEAF command (no nesting effect). The
    /// `origin` is the run's top-left in absolute scene coordinates; `run` carries
    /// the positioned glyphs produced **once** by the `obscur-text` domain, so
    /// every renderer consumes an identical run and only rasterization differs
    /// (spec §5.10, §5.2). Renderers map each glyph to an outline and draw it at
    /// its run-relative position — they never re-shape.
    DrawText {
        /// Top-left origin of the run, in absolute scene coordinates.
        origin: obscur_geometry::Point,
        /// The shaped, positioned glyphs (with the font size and source text).
        run: crate::text::GlyphRun,
        /// The brush the glyphs are painted with.
        brush: crate::paint::Brush,
    },
    /// Paint a CSS `box-shadow` (spec §5.2). A LEAF command (no nesting effect).
    ///
    /// The shadow is a gaussian-blurred rounded rectangle derived from the box's
    /// border box `rect` and corner `radii`: it is translated by `offset`,
    /// inflated (a **drop** shadow, `inset = false`) or deflated (an **inset**
    /// shadow, `inset = true`) by `spread`, and blurred by `blur` (the CSS blur
    /// *radius*, which each renderer maps to a gaussian standard deviation).
    ///
    /// The coordinator emits a drop shadow **before** (behind) the box's
    /// background and an inset shadow **after** it; the renderer decides the
    /// geometry from these fields so both backends consume identical parameters
    /// and only rasterization differs. `color` is already resolved (the CSS
    /// `currentColor` default having been substituted by the coordinator).
    DrawShadow {
        /// The box's border-box rectangle, before offset / spread are applied.
        rect: Rect,
        /// The border box's per-corner radii (all zero for a square box).
        radii: CornerRadii,
        /// The (already resolved) shadow color.
        color: Color,
        /// The CSS blur *radius* in logical pixels (`0` = a hard edge).
        blur: f32,
        /// The spread distance in logical pixels: a drop shadow grows by this,
        /// an inset shadow shrinks by it.
        spread: f32,
        /// The shadow's `(x, y)` offset from the box, in logical pixels.
        offset: (f32, f32),
        /// Whether this is an **inset** shadow (painted inside the box) rather
        /// than a drop shadow (painted behind it).
        inset: bool,
    },
    /// Push an affine transform onto the transform stack (CSS 2D `transform`,
    /// spec §5.2). Subsequent drawing — up to the matching
    /// [`PaintCommand::PopTransform`] — is mapped by the composition of every
    /// transform on the stack, applied to absolute scene coordinates. The affine
    /// is already resolved **about its transform origin** by the coordinator
    /// (`translate(origin) · matrix · translate(-origin)`), so a renderer just
    /// multiplies it into its current transform. This half of the pair opens the
    /// nested region; a well-formed list balances it with a `PopTransform`.
    PushTransform(AffineTransform),
    /// Pop the most recently pushed transform, ending its region.
    PopTransform,
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
        matches!(
            self,
            Self::PushClip(_) | Self::PushLayer(_) | Self::PushTransform(_)
        )
    }

    /// Returns whether this command closes a nested region, i.e. it is a
    /// [`PaintCommand::PopClip`] or [`PaintCommand::PopLayer`].
    #[must_use]
    pub const fn is_pop(&self) -> bool {
        matches!(self, Self::PopClip | Self::PopLayer | Self::PopTransform)
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
            Self::PushClip(_) | Self::PushLayer(_) | Self::PushTransform(_) => 1,
            Self::PopClip | Self::PopLayer | Self::PopTransform => -1,
            Self::DrawPath { .. } | Self::DrawText { .. } | Self::DrawShadow { .. } => 0,
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
        use crate::text::{GlyphRun, PositionedGlyph};
        let run = GlyphRun::new(
            vec![PositionedGlyph::new(u32::from('h'), 0, 0.0, 16.0, 9.6)],
            16.0,
            "hi",
        );
        let text = PaintCommand::DrawText {
            origin: Point::new(2.0, 3.0),
            run,
            brush: Brush::solid(Color::WHITE),
        };
        assert_eq!(text.nesting_delta(), 0);
        assert!(!text.is_push());
        assert!(!text.is_pop());
    }

    #[test]
    fn push_and_pop_transform_nest() {
        let push = PaintCommand::PushTransform(AffineTransform::translate(10.0, 0.0));
        assert_eq!(push.nesting_delta(), 1);
        assert!(push.is_push());
        assert!(!push.is_pop());
        assert_eq!(PaintCommand::PopTransform.nesting_delta(), -1);
        assert!(PaintCommand::PopTransform.is_pop());
        assert!(!PaintCommand::PopTransform.is_push());
    }

    #[test]
    fn draw_shadow_is_a_leaf() {
        let shadow = PaintCommand::DrawShadow {
            rect: Rect::new(Point::new(1.0, 2.0), Size::new(10.0, 20.0)),
            radii: CornerRadii::uniform(4.0),
            color: Color::BLACK,
            blur: 8.0,
            spread: 0.0,
            offset: (4.0, 4.0),
            inset: false,
        };
        assert_eq!(shadow.nesting_delta(), 0);
        assert!(!shadow.is_push());
        assert!(!shadow.is_pop());
    }
}
