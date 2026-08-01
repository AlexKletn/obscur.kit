#![doc = include_str!("image_snapshot_stage.doc/_module.md")]

use obscur_geometry::{Point, Size};
use obscur_headless::snapshot::ImageSnapshot;
use obscur_render::frame::{FrameSubmission, RenderFrame};
use obscur_render::target::{BitmapTarget, RenderTarget};
use obscur_render::{Renderer, RendererId};
use obscur_render_cpu::renderer::CpuRenderer;
use obscur_scene::color::Color;
use obscur_scene::command::{DisplayList, PaintCommand};
use obscur_scene::paint::Paint;
use obscur_scene::path::Path;
use obscur_scene::scene::{SceneGeneration, SceneSnapshot};

#[doc = include_str!("image_snapshot_stage.doc/red_rect_submission.md")]
fn red_rect_submission() -> FrameSubmission {
    let mut path = Path::new();
    path.move_to(Point::new(4.0, 4.0))
        .line_to(Point::new(12.0, 4.0))
        .line_to(Point::new(12.0, 12.0))
        .line_to(Point::new(4.0, 12.0))
        .close();

    let list = DisplayList::from_commands(vec![PaintCommand::DrawPath {
        path,
        paint: Paint::solid_fill(Color::rgba(1.0, 0.0, 0.0, 1.0)),
    }]);
    let snapshot = SceneSnapshot::new(list, SceneGeneration::from_raw(1));
    FrameSubmission::new(RenderFrame::new(snapshot, Size::new(16.0, 16.0)))
}

#[doc = include_str!("image_snapshot_stage.doc/render_image.md")]
fn render_image(submission: &FrameSubmission) -> ImageSnapshot {
    let mut renderer = CpuRenderer::new(RendererId::from_raw(1));
    let target = RenderTarget::bitmap(BitmapTarget::new(Size::new(16.0, 16.0)));
    renderer.render(submission, &target).expect("renders");
    let pixmap = renderer.pixmap().expect("a frame was rendered");
    ImageSnapshot::from_rgba(pixmap.width(), pixmap.height(), pixmap.data())
}

#[test]
fn image_stage_is_deterministic_across_two_renders() {
    let submission = red_rect_submission();
    let first = render_image(&submission);
    let second = render_image(&submission);

    // Same scene, same target → byte-for-byte-equal image stage (spec §7.14).
    assert_eq!(first, second);
    assert_eq!(first.width(), 16);
    assert_eq!(first.height(), 16);
    assert!(!first.is_empty());
}

#[test]
fn distinct_scenes_produce_distinct_image_hashes() {
    let red = render_image(&red_rect_submission());

    // An empty scene rasterizes to a fully transparent bitmap: a different hash.
    let empty = FrameSubmission::new(RenderFrame::new(
        SceneSnapshot::new(DisplayList::new(), SceneGeneration::from_raw(1)),
        Size::new(16.0, 16.0),
    ));
    let blank = render_image(&empty);

    assert_ne!(red.pixel_hash(), blank.pixel_hash());
}
