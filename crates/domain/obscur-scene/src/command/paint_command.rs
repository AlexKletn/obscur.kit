#![doc = include_str!("paint_command.doc/_module.md")]

use obscur_geometry::{AffineTransform, CornerRadii, Rect};

use crate::clip::Clip;
use crate::color::Color;
use crate::layer::SceneLayer;
use crate::paint::{Paint, SceneImage};
use crate::path::Path;

#[doc = include_str!("paint_command.doc/PaintCommand.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum PaintCommand {
    #[doc = include_str!("paint_command.doc/DrawPath.md")]
    DrawPath {
        #[doc = include_str!("paint_command.doc/path.md")]
        path: Path,
        #[doc = include_str!("paint_command.doc/paint.md")]
        paint: Paint,
    },
    #[doc = include_str!("paint_command.doc/DrawText.md")]
    DrawText {
        #[doc = include_str!("paint_command.doc/origin.md")]
        origin: obscur_geometry::Point,
        #[doc = include_str!("paint_command.doc/run.md")]
        run: crate::text::GlyphRun,
        #[doc = include_str!("paint_command.doc/brush.md")]
        brush: crate::paint::Brush,
    },
    #[doc = include_str!("paint_command.doc/DrawShadow.md")]
    DrawShadow {
        #[doc = include_str!("paint_command.doc/rect.md")]
        rect: Rect,
        #[doc = include_str!("paint_command.doc/radii.md")]
        radii: CornerRadii,
        #[doc = include_str!("paint_command.doc/color.md")]
        color: Color,
        #[doc = include_str!("paint_command.doc/blur.md")]
        blur: f32,
        #[doc = include_str!("paint_command.doc/spread.md")]
        spread: f32,
        #[doc = include_str!("paint_command.doc/offset.md")]
        offset: (f32, f32),
        #[doc = include_str!("paint_command.doc/inset.md")]
        inset: bool,
    },
    #[doc = include_str!("paint_command.doc/DrawImage.md")]
    DrawImage {
        #[doc = include_str!("paint_command.doc/image.md")]
        image: SceneImage,
        #[doc = include_str!("paint_command.doc/tile.md")]
        tile: Rect,
        #[doc = include_str!("paint_command.doc/region.md")]
        region: Rect,
        #[doc = include_str!("paint_command.doc/repeat_x.md")]
        repeat_x: bool,
        #[doc = include_str!("paint_command.doc/repeat_y.md")]
        repeat_y: bool,
    },
    #[doc = include_str!("paint_command.doc/BackdropFilter.md")]
    BackdropFilter {
        #[doc = include_str!("paint_command.doc/bounds.md")]
        bounds: Rect,
        #[doc = include_str!("paint_command.doc/radii_2.md")]
        radii: CornerRadii,
        #[doc = include_str!("paint_command.doc/blur_2.md")]
        blur: f32,
    },
    #[doc = include_str!("paint_command.doc/PushTransform.md")]
    PushTransform(AffineTransform),
    #[doc = include_str!("paint_command.doc/PopTransform.md")]
    PopTransform,
    #[doc = include_str!("paint_command.doc/PushClip.md")]
    PushClip(Clip),
    #[doc = include_str!("paint_command.doc/PopClip.md")]
    PopClip,
    #[doc = include_str!("paint_command.doc/PushLayer.md")]
    PushLayer(SceneLayer),
    #[doc = include_str!("paint_command.doc/PopLayer.md")]
    PopLayer,
}

impl PaintCommand {
    #[doc = include_str!("paint_command.doc/is_push.md")]
    #[must_use]
    pub const fn is_push(&self) -> bool {
        matches!(
            self,
            Self::PushClip(_) | Self::PushLayer(_) | Self::PushTransform(_)
        )
    }

    #[doc = include_str!("paint_command.doc/is_pop.md")]
    #[must_use]
    pub const fn is_pop(&self) -> bool {
        matches!(self, Self::PopClip | Self::PopLayer | Self::PopTransform)
    }

    #[doc = include_str!("paint_command.doc/nesting_delta.md")]
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
