//! A single entry in a [`DisplayList`](crate::command::DisplayList).

use obscur_geometry::{AffineTransform, CornerRadii, Rect};

use crate::clip::Clip;
use crate::color::Color;
use crate::layer::SceneLayer;
use crate::paint::{Paint, SceneImage};
use crate::path::Path;

use super::command_scope::CommandScope;

#[doc = include_str!("../../docs/api/command/paint_command.doc/PaintCommand.md")]
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
    #[doc = include_str!("../../docs/api/command/paint_command.doc/DrawText.md")]
    DrawText {
        /// Top-left origin of the run, in absolute scene coordinates.
        origin: obscur_geometry::Point,
        /// The shaped, positioned glyphs (with the font size and source text).
        run: crate::text::GlyphRun,
        /// The brush the glyphs are painted with.
        brush: crate::paint::Brush,
    },
    #[doc = include_str!("../../docs/api/command/paint_command.doc/DrawShadow.md")]
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
    #[doc = include_str!("../../docs/api/command/paint_command.doc/DrawImage.md")]
    DrawImage {
        /// The decoded RGBA8 image to draw.
        image: SceneImage,
        /// The rectangle of one image copy (position + size), in absolute scene
        /// coordinates. The image is scaled from its intrinsic size to this rect.
        tile: Rect,
        /// The background paint area (border box) the image(s) are clipped to.
        region: Rect,
        /// Whether to tile the image horizontally to fill `region`.
        repeat_x: bool,
        /// Whether to tile the image vertically to fill `region`.
        repeat_y: bool,
    },
    #[doc = include_str!("../../docs/api/command/paint_command.doc/BackdropFilter.md")]
    BackdropFilter {
        /// The element's border-box rectangle: the backdrop region to frost.
        bounds: Rect,
        /// The border box's per-corner radii (all zero for a square panel); the
        /// frosted region is clipped to this rounded shape.
        radii: CornerRadii,
        /// The CSS blur *radius* in logical pixels (`> 0`; `0` = no effect).
        blur: f32,
    },
    #[doc = include_str!("../../docs/api/command/paint_command.doc/PushTransform.md")]
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
    /// Returns whether this command opens a transform, clip or layer.
    #[must_use]
    pub const fn is_push(&self) -> bool {
        CommandScope::from_push(self).is_some()
    }

    /// Returns whether this command closes a transform, clip or layer.
    #[must_use]
    pub const fn is_pop(&self) -> bool {
        matches!(self, Self::PopClip | Self::PopLayer | Self::PopTransform)
    }

    /// Returns the closing command for a transform, clip or layer push.
    #[must_use]
    pub const fn scope_close(&self) -> Option<Self> {
        match CommandScope::from_push(self) {
            Some(scope) => Some(scope.close()),
            None => None,
        }
    }

    /// Checks whether this command closes the scope opened by `opening`.
    #[must_use]
    pub fn closes_scope(&self, opening: &Self) -> bool {
        opening.scope_close().is_some_and(|expected| {
            core::mem::discriminant(self) == core::mem::discriminant(&expected)
        })
    }

    #[doc = include_str!("../../docs/api/command/paint_command.doc/nesting_delta.md")]
    #[must_use]
    pub const fn nesting_delta(&self) -> i32 {
        match self {
            Self::PushClip(_) | Self::PushLayer(_) | Self::PushTransform(_) => 1,
            Self::PopClip | Self::PopLayer | Self::PopTransform => -1,
            Self::DrawPath { .. }
            | Self::DrawText { .. }
            | Self::DrawShadow { .. }
            | Self::DrawImage { .. }
            | Self::BackdropFilter { .. } => 0,
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
    fn draw_image_is_a_leaf() {
        use crate::color::AlphaMode;
        use crate::paint::SceneImage;
        use std::sync::Arc;

        let image = SceneImage::new(1, 1, Arc::from(vec![255u8, 0, 0, 255]), AlphaMode::Straight)
            .expect("1x1 image");
        let rect = Rect::new(Point::new(0.0, 0.0), Size::new(4.0, 4.0));
        let cmd = PaintCommand::DrawImage {
            image,
            tile: rect,
            region: rect,
            repeat_x: false,
            repeat_y: false,
        };
        assert_eq!(cmd.nesting_delta(), 0);
        assert!(!cmd.is_push());
        assert!(!cmd.is_pop());
    }

    #[test]
    fn backdrop_filter_is_a_leaf() {
        let backdrop = PaintCommand::BackdropFilter {
            bounds: Rect::new(Point::new(4.0, 6.0), Size::new(40.0, 24.0)),
            radii: CornerRadii::uniform(12.0),
            blur: 10.0,
        };
        assert_eq!(backdrop.nesting_delta(), 0);
        assert!(!backdrop.is_push());
        assert!(!backdrop.is_pop());
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
