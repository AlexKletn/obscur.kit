//! Verifies the CSS root radii supplied to native window blur.

use obscur_kit::bootstrap::CssPipeline;
use obscur_kit::geometry::{CornerRadii, CornerRadius, Size};
use obscur_kit::identity::RuntimeId;
use obscur_kit::runtime::{UiRuntime, frame::FrameReason};
use obscur_kit::{AppControl, OkuiScreen};

#[test]
fn window_blur_uses_the_full_viewport_roots_resolved_css_radii() {
    let screen =
        OkuiScreen::compile(r#"<div class="window"><div class="content"/></div>"#).unwrap();
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(400.0, 300.0));
    runtime.set_surface(Size::new(800.0, 600.0), 2.0);
    runtime.set_root_view(move || screen.view());
    let control = AppControl::new();
    let mut css = CssPipeline::new();

    for (root_style, expected) in [
        (
            ".window { width: 100%; height: 100%; border-top-left-radius: 10%; }",
            CornerRadii {
                top_left: CornerRadius::new(40.0, 30.0),
                ..CornerRadii::default()
            },
        ),
        (
            ".window { width: 50%; height: 100%; border-radius: 12px; }",
            CornerRadii::default(),
        ),
        (
            ".window { width: 100%; height: 100%; }",
            CornerRadii::default(),
        ),
    ] {
        control.set_stylesheet(format!("{root_style} .content {{ border-radius: 30px; }}"));
        control.apply(&mut runtime, &mut css);
        runtime.request_frame(FrameReason::UiCommit);
        runtime.run_due_frame_reactive().unwrap();
        assert_eq!(runtime.viewport_corner_radii(), expected);
    }
}

#[test]
fn window_clip_masks_layered_content_in_every_corner_at_fractional_scale() {
    use obscur_render::frame::{FrameSubmission, RenderFrame};
    use obscur_render::target::{BitmapTarget, RenderTarget};
    use obscur_render::{Renderer, RendererId};
    use obscur_render_cpu::renderer::CpuRenderer;

    let screen =
        OkuiScreen::compile(r#"<div class="window"><div class="content"/></div>"#).unwrap();
    let viewport = Size::new(80.0, 60.0);
    let physical = Size::new(100.0, 75.0);
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), viewport);
    runtime.set_surface(physical, 1.25);
    runtime.set_root_view(move || screen.view());
    let control = AppControl::new();
    let mut css = CssPipeline::new();
    let mut renderer = CpuRenderer::new(RendererId::from_raw(1));
    let target = RenderTarget::bitmap(BitmapTarget::new(physical));

    for (radius, expected_alpha) in [("12px 16px 20px 8px", 0), ("0", 255)] {
        control.set_stylesheet(format!(
            ".window {{ width:100%; height:100%; border-radius:{radius}; }}
             .content {{ position:absolute; inset:0; background-color:#ff0000;
                         z-index:2; transform:translateX(1px) scale(1.1); }}"
        ));
        control.apply(&mut runtime, &mut css);
        runtime.request_frame(FrameReason::UiCommit);
        runtime.run_due_frame_reactive().unwrap();
        let frame = FrameSubmission::new(RenderFrame::new(runtime.snapshot(), physical));
        renderer.render(&frame, &target).unwrap();
        let pixels = renderer.pixmap().unwrap();
        assert_eq!(pixels.pixel(50, 37).unwrap().alpha(), 255);
        for (x, y) in [(0, 0), (99, 0), (99, 74), (0, 74)] {
            assert_eq!(
                pixels.pixel(x, y).unwrap().alpha(),
                expected_alpha,
                "corner ({x}, {y}) at radius {radius}"
            );
        }
    }
}
