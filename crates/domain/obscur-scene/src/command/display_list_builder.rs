#![doc = include_str!("display_list_builder.doc/_module.md")]

use obscur_geometry::{AffineTransform, CornerRadii, Point, Rect};

use crate::clip::Clip;
use crate::color::Color;
use crate::layer::SceneLayer;
use crate::paint::{Brush, Paint, SceneImage, Stroke};
use crate::path::Path;
use crate::text::GlyphRun;

use super::{DisplayList, PaintCommand};

#[doc = include_str!("display_list_builder.doc/DisplayListBuilder.md")]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DisplayListBuilder {
    commands: Vec<PaintCommand>,
    depth: u32,
}

impl DisplayListBuilder {
    #[doc = include_str!("display_list_builder.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            depth: 0,
        }
    }

    #[doc = include_str!("display_list_builder.doc/draw.md")]
    pub fn draw(&mut self, path: Path, paint: Paint) -> &mut Self {
        self.commands.push(PaintCommand::DrawPath { path, paint });
        self
    }

    #[doc = include_str!("display_list_builder.doc/fill.md")]
    pub fn fill(&mut self, path: Path, brush: Brush) -> &mut Self {
        self.draw(path, Paint::fill(brush))
    }

    #[doc = include_str!("display_list_builder.doc/fill_rect.md")]
    pub fn fill_rect(&mut self, rect: Rect, brush: Brush) -> &mut Self {
        self.fill(Path::rect(rect), brush)
    }

    #[doc = include_str!("display_list_builder.doc/draw_text.md")]
    pub fn draw_text(&mut self, origin: Point, run: GlyphRun, brush: Brush) -> &mut Self {
        self.commands
            .push(PaintCommand::DrawText { origin, run, brush });
        self
    }

    #[doc = include_str!("display_list_builder.doc/draw_image.md")]
    pub fn draw_image(
        &mut self,
        image: SceneImage,
        tile: Rect,
        region: Rect,
        repeat_x: bool,
        repeat_y: bool,
    ) -> &mut Self {
        self.commands.push(PaintCommand::DrawImage {
            image,
            tile,
            region,
            repeat_x,
            repeat_y,
        });
        self
    }

    #[doc = include_str!("display_list_builder.doc/draw_shadow.md")]
    #[allow(clippy::too_many_arguments)]
    pub fn draw_shadow(
        &mut self,
        rect: Rect,
        radii: CornerRadii,
        color: Color,
        blur: f32,
        spread: f32,
        offset: (f32, f32),
        inset: bool,
    ) -> &mut Self {
        self.commands.push(PaintCommand::DrawShadow {
            rect,
            radii,
            color,
            blur,
            spread,
            offset,
            inset,
        });
        self
    }

    #[doc = include_str!("display_list_builder.doc/backdrop_filter.md")]
    pub fn backdrop_filter(&mut self, bounds: Rect, radii: CornerRadii, blur: f32) -> &mut Self {
        self.commands.push(PaintCommand::BackdropFilter {
            bounds,
            radii,
            blur,
        });
        self
    }

    #[doc = include_str!("display_list_builder.doc/stroke.md")]
    pub fn stroke(&mut self, path: Path, stroke: Stroke) -> &mut Self {
        self.draw(path, Paint::stroke(stroke))
    }

    #[doc = include_str!("display_list_builder.doc/push_transform.md")]
    pub fn push_transform(&mut self, transform: AffineTransform) -> &mut Self {
        self.commands.push(PaintCommand::PushTransform(transform));
        self.depth = self.depth.saturating_add(1);
        self
    }

    #[doc = include_str!("display_list_builder.doc/pop_transform.md")]
    pub fn pop_transform(&mut self) -> &mut Self {
        self.commands.push(PaintCommand::PopTransform);
        self.depth = self.depth.saturating_sub(1);
        self
    }

    #[doc = include_str!("display_list_builder.doc/push_clip.md")]
    pub fn push_clip(&mut self, clip: Clip) -> &mut Self {
        self.commands.push(PaintCommand::PushClip(clip));
        self.depth = self.depth.saturating_add(1);
        self
    }

    #[doc = include_str!("display_list_builder.doc/pop_clip.md")]
    pub fn pop_clip(&mut self) -> &mut Self {
        self.commands.push(PaintCommand::PopClip);
        self.depth = self.depth.saturating_sub(1);
        self
    }

    #[doc = include_str!("display_list_builder.doc/push_layer.md")]
    pub fn push_layer(&mut self, layer: SceneLayer) -> &mut Self {
        self.commands.push(PaintCommand::PushLayer(layer));
        self.depth = self.depth.saturating_add(1);
        self
    }

    #[doc = include_str!("display_list_builder.doc/pop_layer.md")]
    pub fn pop_layer(&mut self) -> &mut Self {
        self.commands.push(PaintCommand::PopLayer);
        self.depth = self.depth.saturating_sub(1);
        self
    }

    #[doc = include_str!("display_list_builder.doc/depth.md")]
    #[must_use]
    pub fn depth(&self) -> u32 {
        self.depth
    }

    #[doc = include_str!("display_list_builder.doc/is_balanced.md")]
    #[must_use]
    pub fn is_balanced(&self) -> bool {
        self.depth == 0
    }

    #[doc = include_str!("display_list_builder.doc/build.md")]
    #[must_use]
    pub fn build(self) -> DisplayList {
        DisplayList::from_commands(self.commands)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Color;
    use obscur_geometry::{Point, Size};

    fn rect() -> Rect {
        Rect::new(Point::new(0.0, 0.0), Size::new(4.0, 4.0))
    }

    #[test]
    fn balanced_sequence_preserves_order_and_returns_to_zero() {
        let mut builder = DisplayListBuilder::new();
        builder
            .push_layer(SceneLayer::new())
            .push_clip(Clip::rect(rect()))
            .fill_rect(rect(), Brush::solid(Color::WHITE))
            .pop_clip()
            .pop_layer();

        assert_eq!(builder.depth(), 0);
        assert!(builder.is_balanced());

        let list = builder.build();
        let commands = list.commands();
        assert_eq!(commands.len(), 5);
        assert!(matches!(commands[0], PaintCommand::PushLayer(_)));
        assert!(matches!(commands[1], PaintCommand::PushClip(_)));
        assert!(matches!(commands[2], PaintCommand::DrawPath { .. }));
        assert_eq!(commands[3], PaintCommand::PopClip);
        assert_eq!(commands[4], PaintCommand::PopLayer);
        assert!(list.is_balanced());
    }

    #[test]
    fn missing_pop_builds_unbalanced_list() {
        let mut builder = DisplayListBuilder::new();
        builder
            .push_clip(Clip::rect(rect()))
            .fill_rect(rect(), Brush::solid(Color::BLACK));

        assert_eq!(builder.depth(), 1);
        assert!(!builder.is_balanced());

        let list = builder.build();
        assert!(!list.is_balanced());
    }

    #[test]
    fn stray_pop_on_empty_builder_stays_at_zero() {
        let mut builder = DisplayListBuilder::new();
        builder.pop_clip();

        // Depth is clamped: no underflow.
        assert_eq!(builder.depth(), 0);
        // The command is still recorded verbatim.
        let list = builder.build();
        assert_eq!(list.len(), 1);
        assert_eq!(list.commands()[0], PaintCommand::PopClip);
        // A stray pop makes the list unbalanced.
        assert!(!list.is_balanced());
    }

    #[test]
    fn fill_and_stroke_produce_expected_paints() {
        let mut builder = DisplayListBuilder::new();
        builder
            .fill(Path::rect(rect()), Brush::solid(Color::WHITE))
            .stroke(
                Path::rect(rect()),
                Stroke::new(Brush::solid(Color::BLACK), 2.0),
            )
            .fill_rect(rect(), Brush::solid(Color::WHITE));

        let list = builder.build();
        match &list.commands()[0] {
            PaintCommand::DrawPath { paint, .. } => assert!(paint.is_fill()),
            other => panic!("expected DrawPath, got {other:?}"),
        }
        match &list.commands()[1] {
            PaintCommand::DrawPath { paint, .. } => assert!(paint.is_stroke()),
            other => panic!("expected DrawPath, got {other:?}"),
        }
        match &list.commands()[2] {
            PaintCommand::DrawPath { path, paint } => {
                assert!(paint.is_fill());
                assert_eq!(path, &Path::rect(rect()));
            }
            other => panic!("expected DrawPath, got {other:?}"),
        }
    }

    #[test]
    fn draw_text_appends_a_leaf_text_command() {
        use crate::text::PositionedGlyph;
        let run = GlyphRun::new(
            vec![PositionedGlyph::new(u32::from('h'), 0, 0.0, 18.0, 10.8)],
            18.0,
            "hello",
        );
        let mut builder = DisplayListBuilder::new();
        builder.draw_text(
            Point::new(3.0, 7.0),
            run.clone(),
            Brush::solid(Color::WHITE),
        );

        assert!(builder.is_balanced());
        let list = builder.build();
        assert_eq!(list.len(), 1);
        assert_eq!(
            list.commands()[0],
            PaintCommand::DrawText {
                origin: Point::new(3.0, 7.0),
                run,
                brush: Brush::solid(Color::WHITE),
            }
        );
        // A list containing DrawText is still balanced (it is a leaf).
        assert!(list.is_balanced());
    }

    #[test]
    fn draw_image_appends_a_leaf_image_command_with_tile_region_and_repeat() {
        use crate::color::AlphaMode;
        use crate::paint::SceneImage;
        use std::sync::Arc;

        let image = SceneImage::new(2, 2, Arc::from(vec![0u8; 16]), AlphaMode::Straight)
            .expect("2x2 image");
        let tile = Rect::new(Point::new(1.0, 2.0), Size::new(2.0, 2.0));
        let region = Rect::new(Point::new(1.0, 2.0), Size::new(8.0, 6.0));
        let mut builder = DisplayListBuilder::new();
        // Ordering: a fill precedes the image, so the image is command index 1.
        builder
            .fill_rect(rect(), Brush::solid(Color::WHITE))
            .draw_image(image.clone(), tile, region, true, false);

        assert!(builder.is_balanced());
        let list = builder.build();
        assert_eq!(list.len(), 2);
        assert!(matches!(list.commands()[0], PaintCommand::DrawPath { .. }));
        assert_eq!(
            list.commands()[1],
            PaintCommand::DrawImage {
                image,
                tile,
                region,
                repeat_x: true,
                repeat_y: false,
            }
        );
        // A DrawImage is a leaf, so the list stays balanced.
        assert!(list.is_balanced());
    }

    #[test]
    fn backdrop_filter_appends_a_balanced_leaf() {
        let mut builder = DisplayListBuilder::new();
        builder.backdrop_filter(rect(), CornerRadii::uniform(8.0), 10.0);
        assert!(builder.is_balanced());
        let list = builder.build();
        assert_eq!(list.len(), 1);
        assert_eq!(
            list.commands()[0],
            PaintCommand::BackdropFilter {
                bounds: rect(),
                radii: CornerRadii::uniform(8.0),
                blur: 10.0,
            }
        );
        // A BackdropFilter is a leaf, so the list stays balanced.
        assert!(list.is_balanced());
    }

    #[test]
    fn push_and_pop_transform_balance() {
        let mut builder = DisplayListBuilder::new();
        builder
            .push_transform(AffineTransform::translate(10.0, 20.0))
            .fill_rect(rect(), Brush::solid(Color::WHITE))
            .pop_transform();
        assert_eq!(builder.depth(), 0);
        assert!(builder.is_balanced());
        let list = builder.build();
        let commands = list.commands();
        assert_eq!(commands.len(), 3);
        assert!(matches!(
            commands[0],
            PaintCommand::PushTransform(t) if t == AffineTransform::translate(10.0, 20.0)
        ));
        assert_eq!(commands[2], PaintCommand::PopTransform);
        assert!(list.is_balanced());
    }

    #[test]
    fn draw_shadow_appends_a_balanced_leaf_shadow() {
        let mut builder = DisplayListBuilder::new();
        builder.draw_shadow(
            rect(),
            CornerRadii::uniform(3.0),
            Color::BLACK,
            6.0,
            1.0,
            (2.0, 4.0),
            false,
        );
        assert!(builder.is_balanced());
        let list = builder.build();
        assert_eq!(list.len(), 1);
        assert_eq!(
            list.commands()[0],
            PaintCommand::DrawShadow {
                rect: rect(),
                radii: CornerRadii::uniform(3.0),
                color: Color::BLACK,
                blur: 6.0,
                spread: 1.0,
                offset: (2.0, 4.0),
                inset: false,
            }
        );
        // A DrawShadow is a leaf, so the list stays balanced.
        assert!(list.is_balanced());
    }
}
