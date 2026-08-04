#![doc = include_str!("canvas_region.doc/_module.md")]

use obscur_geometry::{AffineTransform, Point, Rect, Size};

use super::DisplayListBuilder;
use crate::clip::Clip;
use crate::paint::{BufferFit, ExternalBuffer, SceneImage};
use crate::path::Path;

#[doc = include_str!("canvas_region.doc/CanvasRegion.md")]
#[derive(Debug)]
pub struct CanvasRegion<'a> {
    #[doc = include_str!("canvas_region.doc/builder.md")]
    builder: &'a mut DisplayListBuilder,
    #[doc = include_str!("canvas_region.doc/bounds.md")]
    bounds: Rect,
    #[doc = include_str!("canvas_region.doc/entry_depth.md")]
    entry_depth: u32,
    #[doc = include_str!("canvas_region.doc/hit.md")]
    hit: Option<Path>,
}

impl<'a> CanvasRegion<'a> {
    #[doc = include_str!("canvas_region.doc/open.md")]
    pub fn open(builder: &'a mut DisplayListBuilder, bounds: Rect) -> Self {
        let entry_depth = builder.depth();
        // Two brackets, in this order, and the order is not arbitrary: the clip
        // is stated in the region's OWN coordinates, so it has to sit inside
        // the translation that establishes them.
        builder.push_transform(AffineTransform::translate(bounds.origin.x, bounds.origin.y));
        builder.push_clip(Clip::rect(Rect::new(Point::new(0.0, 0.0), bounds.size)));
        Self {
            builder,
            bounds,
            entry_depth,
            hit: None,
        }
    }

    #[doc = include_str!("canvas_region.doc/size.md")]
    #[must_use]
    pub const fn size(&self) -> Size {
        self.bounds.size
    }

    #[doc = include_str!("canvas_region.doc/at.md")]
    #[must_use]
    pub fn at(&self, x: f32, y: f32) -> Point {
        Point::new(x * self.bounds.size.width, y * self.bounds.size.height)
    }

    #[doc = include_str!("canvas_region.doc/painter.md")]
    pub fn painter(&mut self) -> &mut DisplayListBuilder {
        self.builder
    }

    #[doc = include_str!("canvas_region.doc/draw_buffer.md")]
    pub fn draw_buffer(&mut self, buffer: &ExternalBuffer<'_>, fit: BufferFit) -> bool {
        if buffer.is_empty() {
            // A camera that has not produced a frame yet is a real state, and
            // drawing a nothing-sized image would put a leaf command in the list
            // for every frame it takes to arrive.
            return false;
        }
        #[allow(clippy::cast_precision_loss)]
        let source = Size::new(buffer.width() as f32, buffer.height() as f32);
        self.draw_image_into(buffer.to_scene_image(), fit.place(source, self.bounds.size))
    }

    #[doc = include_str!("canvas_region.doc/draw_image_into.md")]
    pub fn draw_image_into(&mut self, image: SceneImage, dest: Rect) -> bool {
        if dest.size.width <= 0.0 || dest.size.height <= 0.0 {
            return false;
        }
        // `tile` is the one copy, `region` is the clip — and the region here is
        // the canvas's own box in its OWN coordinates, so a `Cover` frame that
        // hangs off both edges is trimmed by the same clip that already brackets
        // every other thing the painter draws.
        self.builder.draw_image(
            image,
            dest,
            Rect::new(Point::new(0.0, 0.0), self.bounds.size),
            false,
            false,
        );
        true
    }

    #[doc = include_str!("canvas_region.doc/hit_region.md")]
    pub fn hit_region(&mut self, path: Path) {
        self.hit = Some(path);
    }

    #[doc = include_str!("canvas_region.doc/take_hit_region.md")]
    pub fn take_hit_region(&mut self) -> Option<Path> {
        self.hit.take()
    }

    #[doc = include_str!("canvas_region.doc/close.md")]
    #[must_use]
    pub fn close(self) -> bool {
        // What the application left open, this closes — innermost first, which
        // is the only order that can close anything at all. A painter is
        // ordinary application code running inside a shared display list, and an
        // unbalanced one would not corrupt itself: it would swallow every node
        // painted after it into a clip or a transform that was never meant for
        // them. So the region is responsible for its own depth, and reports
        // whether it had to do the closing.
        let balanced = self.builder.depth() == self.entry_depth + 2;
        while self.builder.depth() > self.entry_depth {
            // `pop_clip` and `pop_transform` both emit their own command, and
            // the builder tracks one depth across all three bracket kinds, so
            // unwinding needs to close the right KIND at each level rather than
            // guessing. `pop_any` is what knows which.
            self.builder.pop_any();
        }
        balanced
    }
}

#[cfg(test)]
mod tests {
    use super::CanvasRegion;
    use crate::color::{AlphaMode, Color};
    use crate::command::{DisplayListBuilder, PaintCommand};
    use crate::paint::{Brush, BufferFit, BufferFormat, ExternalBuffer};
    use obscur_geometry::{Point, Rect, Size};

    fn bounds() -> Rect {
        Rect::new(Point::new(30.0, 50.0), Size::new(200.0, 100.0))
    }

    fn red() -> Brush {
        Brush::solid(Color::rgba(1.0, 0.0, 0.0, 1.0))
    }

    #[test]
    fn a_region_brackets_itself_with_a_translation_and_a_clip() {
        let mut builder = DisplayListBuilder::new();
        let region = CanvasRegion::open(&mut builder, bounds());
        assert!(region.close());
        let list = builder.build();

        let kinds: Vec<&str> = list
            .commands()
            .iter()
            .map(|c| match c {
                PaintCommand::PushTransform(_) => "push_transform",
                PaintCommand::PushClip(_) => "push_clip",
                PaintCommand::PopClip => "pop_clip",
                PaintCommand::PopTransform => "pop_transform",
                _ => "other",
            })
            .collect();
        assert_eq!(
            kinds,
            ["push_transform", "push_clip", "pop_clip", "pop_transform"],
            "the clip sits inside the translation, because it is stated in the \
             region's own coordinates"
        );
    }

    #[test]
    fn the_painter_draws_in_the_regions_own_coordinates() {
        // The point of the translation: an application draws at (0, 0) and lands
        // at the canvas's top-left, wherever layout put it. Nothing it draws
        // needs to know where on the screen it is.
        let mut builder = DisplayListBuilder::new();
        let mut region = CanvasRegion::open(&mut builder, bounds());
        region.painter().fill_rect(
            Rect::new(Point::new(0.0, 0.0), Size::new(10.0, 10.0)),
            red(),
        );
        assert!(region.close());

        let list = builder.build();
        let transform = list.commands().iter().find_map(|c| match c {
            PaintCommand::PushTransform(t) => Some(*t),
            _ => None,
        });
        assert_eq!(
            transform.map(|t| t.apply(Point::new(0.0, 0.0))),
            Some(Point::new(30.0, 50.0)),
            "the region's origin is the laid-out box's origin"
        );
    }

    #[test]
    fn size_is_the_laid_out_box_and_at_reads_normalized() {
        // §4.5: "normalized coordinates are available for Canvas, gestures and
        // shaders." A painter that wants to draw a ring at 50% of its width
        // should not have to be told what its width is.
        let mut builder = DisplayListBuilder::new();
        let region = CanvasRegion::open(&mut builder, bounds());
        assert_eq!(region.size(), Size::new(200.0, 100.0));
        assert_eq!(region.at(0.5, 0.5), Point::new(100.0, 50.0));
        assert_eq!(region.at(0.0, 0.0), Point::new(0.0, 0.0));
        assert_eq!(region.at(1.0, 1.0), Point::new(200.0, 100.0));
        assert!(region.close());
    }

    #[test]
    fn a_painter_that_leaves_a_bracket_open_is_closed_for_it_and_reported() {
        // The failure this exists to contain. A painter is application code
        // running inside a display list every other node also paints into: one
        // that pushes a clip and forgets to pop would not damage itself, it
        // would swallow every node painted after it.
        let mut builder = DisplayListBuilder::new();
        let mut region = CanvasRegion::open(&mut builder, bounds());
        region
            .painter()
            .push_clip(crate::clip::Clip::rect(bounds()));
        region
            .painter()
            .push_transform(obscur_geometry::AffineTransform::translate(5.0, 5.0));
        assert!(
            !region.close(),
            "the region reports that it had to do the closing"
        );

        let list = builder.build();
        assert!(
            list.is_balanced(),
            "and the list it leaves behind is balanced regardless"
        );
    }

    #[test]
    fn a_painter_that_draws_nothing_still_leaves_a_balanced_list() {
        let mut builder = DisplayListBuilder::new();
        let region = CanvasRegion::open(&mut builder, bounds());
        assert!(region.close());
        assert!(builder.build().is_balanced());
    }

    #[test]
    fn a_zero_sized_region_is_openable_and_normalizes_to_its_origin() {
        // Layout can give a canvas no room — a collapsed flex item, a hidden
        // branch — and a painter must not have to guard against it.
        let mut builder = DisplayListBuilder::new();
        let region = CanvasRegion::open(
            &mut builder,
            Rect::new(Point::new(10.0, 10.0), Size::new(0.0, 0.0)),
        );
        assert_eq!(region.size(), Size::new(0.0, 0.0));
        assert_eq!(region.at(0.5, 0.5), Point::new(0.0, 0.0));
        assert!(region.close());
    }

    /// A 2×1 BGRA frame with a padded row — what a real capture looks like.
    const FRAME: [u8; 12] = [30, 20, 10, 255, 60, 50, 40, 255, 0xDE, 0xAD, 0xBE, 0xEF];

    fn frame() -> ExternalBuffer<'static> {
        ExternalBuffer::new(&FRAME, 2, 1, 12, BufferFormat::Bgra8, AlphaMode::Straight)
            .expect("a valid frame")
    }

    #[test]
    fn a_foreign_buffer_is_drawn_in_the_regions_own_coordinates() {
        // The whole point: the painter never learns where its box sits on
        // screen. The translation the region already opened puts it there.
        let mut builder = DisplayListBuilder::new();
        let mut region = CanvasRegion::open(&mut builder, bounds());
        assert!(region.draw_buffer(&frame(), BufferFit::Stretch));
        assert!(region.close());

        let list = builder.build();
        let drawn = list
            .commands()
            .iter()
            .find_map(|c| match c {
                PaintCommand::DrawImage {
                    image,
                    tile,
                    region,
                    repeat_x,
                    repeat_y,
                } => Some((image, *tile, *region, *repeat_x, *repeat_y)),
                _ => None,
            })
            .expect("the frame reached the list");
        let (image, tile, clip, repeat_x, repeat_y) = drawn;
        assert_eq!(
            tile.origin,
            Point::new(0.0, 0.0),
            "region-local, not absolute"
        );
        assert_eq!(tile.size, bounds().size, "stretched to fill");
        assert_eq!(clip.size, bounds().size);
        assert!(!repeat_x && !repeat_y, "one copy, not a tiling");
        assert_eq!(
            image.pixels(),
            &[10, 20, 30, 255, 40, 50, 60, 255],
            "channels in order and no padding byte in the picture"
        );
    }

    #[test]
    fn a_covering_frame_hangs_off_the_edges_and_the_region_clips_it() {
        let mut builder = DisplayListBuilder::new();
        let mut region = CanvasRegion::open(&mut builder, bounds());
        assert!(region.draw_buffer(&frame(), BufferFit::Cover));
        assert!(region.close());

        let list = builder.build();
        let PaintCommand::DrawImage { tile, .. } = list
            .commands()
            .iter()
            .find(|c| matches!(c, PaintCommand::DrawImage { .. }))
            .expect("the frame reached the list")
        else {
            unreachable!()
        };
        // 2:1 source into a 200×100 box covers exactly; a taller box would
        // overflow, and either way the region's own clip trims it.
        assert!(tile.size.width >= bounds().size.width);
        assert!(tile.size.height >= bounds().size.height);
    }

    #[test]
    fn a_camera_that_has_not_woken_up_yet_draws_nothing() {
        // A real state, not an error — and it must not put a leaf command in
        // the list for every frame it takes to arrive.
        let mut builder = DisplayListBuilder::new();
        let mut region = CanvasRegion::open(&mut builder, bounds());
        let empty = ExternalBuffer::packed(&[], 0, 0, BufferFormat::Bgra8, AlphaMode::Straight)
            .expect("an empty buffer is a buffer");
        assert!(!region.draw_buffer(&empty, BufferFit::Contain));
        assert!(region.close());
        assert!(
            !builder
                .build()
                .commands()
                .iter()
                .any(|c| matches!(c, PaintCommand::DrawImage { .. }))
        );
    }
}
