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
