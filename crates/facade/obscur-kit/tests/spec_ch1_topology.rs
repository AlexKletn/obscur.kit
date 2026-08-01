#![doc = include_str!("spec_ch1_topology.doc/_module.md")]
#![forbid(unsafe_code)]

#[path = "spec_ch1_support/mod.rs"]
mod support;

use obscur_kit::geometry::Size;
use obscur_kit::identity::RuntimeId;
use obscur_kit::render::frame::{FrameSubmission, RenderFrame};
use obscur_kit::render::target::{BitmapTarget, RenderTarget};
use obscur_kit::render::{Renderer, RendererId};
use obscur_kit::render_cpu::renderer::CpuRenderer;
use obscur_kit::scene::command::PaintCommand;

use support::{
    Collected, approx, at, compile, el, geom, install_missing_resources, runtime_with_id, view,
};

#[doc = include_str!("spec_ch1_topology.doc/VP.md")]
const VP: f32 = 1000.0;

#[doc = include_str!("spec_ch1_topology.doc/arch_04_two_runtimes_hold_independent_trees.md")]
#[test]
fn arch_04_two_runtimes_hold_independent_trees() {
    let mut a = runtime_with_id(1, VP, VP);
    let mut b = runtime_with_id(2, VP, VP);

    let box_view = view(vec![el(10, "box", vec![])]);
    let sheet_a = compile(".box { width: 100px; height: 10px }");
    let sheet_b = compile(".box { width: 300px; height: 10px }");

    a.render_now(&box_view, std::slice::from_ref(&sheet_a));
    b.render_now(&box_view, std::slice::from_ref(&sheet_b));

    // Two distinct runtime handles, alive at once.
    assert_ne!(a.id(), b.id(), "two distinct runtimes were constructed");

    approx(geom(&a, at(&a, &[0])).2, 100.0, "runtime A's box width");
    approx(geom(&b, at(&b, &[0])).2, 300.0, "runtime B's box width");
}

#[doc = include_str!("spec_ch1_topology.doc/arch_05_dropping_one_runtime_leaves_the_other_usable.md")]
#[test]
fn arch_05_dropping_one_runtime_leaves_the_other_usable() {
    let mut a = runtime_with_id(1, VP, VP);
    let mut b = runtime_with_id(2, VP, VP);

    let box_view = view(vec![el(10, "box", vec![])]);
    let sheet_a = compile(".box { width: 100px; height: 10px }");
    let sheet_b = compile(".box { width: 300px; height: 10px }");

    a.render_now(&box_view, std::slice::from_ref(&sheet_a));
    b.render_now(&box_view, std::slice::from_ref(&sheet_b));

    let node_before = at(&b, &[0]);

    drop(a);

    let diagnostics = Collected::install(&mut b);
    let sheet_b2 = compile(".box { width: 250px; height: 10px }");
    b.render_now(&box_view, std::slice::from_ref(&sheet_b2));

    let node_after = at(&b, &[0]);
    assert_eq!(
        node_before, node_after,
        "B's node IDs from before the drop still resolve to the same nodes"
    );
    approx(geom(&b, node_after).2, 250.0, "B's box after A was dropped");

    let errors: Vec<_> = diagnostics
        .all()
        .into_iter()
        .filter(|d| d.severity >= obscur_kit::diagnostics::DiagnosticSeverity::Error)
        .collect();
    assert_eq!(
        errors.len(),
        0,
        "A's drop must produce zero error diagnostics in B, got {errors:?}"
    );
}

#[doc = include_str!("spec_ch1_topology.doc/arch_06_resource_hub_is_not_shared_by_default.md")]
#[test]
fn arch_06_resource_hub_is_not_shared_by_default() {
    let mut a = runtime_with_id(1, VP, VP);
    let mut b = runtime_with_id(2, VP, VP);

    // A registers `logo` through its own hub; B is constructed the ordinary way
    // and never told about it.
    install_missing_resources(&mut a);
    let sheet = compile(".img { width: 10px; height: 10px; background-image: url(logo) }");
    let img = view(vec![el(10, "img", vec![])]);
    a.render_now(&img, std::slice::from_ref(&sheet));

    assert_eq!(
        a.resource_count(),
        1,
        "A's hub reports the one resource A requested"
    );
    assert_eq!(
        b.resource_count(),
        0,
        "B's hub must report 0 registered resources: sharing is opt-in"
    );
}

#[doc = include_str!("spec_ch1_topology.doc/arch_08_headless_produces_layout_scene_and_a_rendered_image.md")]
#[test]
fn arch_08_headless_produces_layout_scene_and_a_rendered_image() {
    // No GPU adapter and no window system: the runtime is constructed with
    // nothing but a viewport.
    let mut rt = runtime_with_id(1, 200.0, 200.0);
    let diagnostics = Collected::install(&mut rt);

    let sheet = compile(".box { width: 120px; height: 40px; background-color: #ff0000 }");
    let result = rt.render_now(
        &view(vec![el(10, "box", vec![])]),
        std::slice::from_ref(&sheet),
    );

    // Layout.
    let (_, _, w, h) = geom(&rt, at(&rt, &[0]));
    approx(w, 120.0, "headless box width");
    approx(h, 40.0, "headless box height");

    // Scene: at least one draw command for that box.
    let snapshot = result.snapshot().clone();
    let draws = snapshot
        .display_list()
        .commands()
        .iter()
        .filter(|c| {
            matches!(
                c,
                PaintCommand::DrawPath { .. }
                    | PaintCommand::DrawImage { .. }
                    | PaintCommand::DrawText { .. }
            )
        })
        .count();
    assert!(
        draws >= 1,
        "headless mode must produce a scene with at least one draw command"
    );

    // Rendered image, at the expected surface size, with no GPU device.
    let surface = Size::new(200.0, 200.0);
    let mut renderer = CpuRenderer::new(RendererId::from_raw(1));
    let submission = FrameSubmission::new(RenderFrame::new(snapshot, surface));
    renderer
        .render(
            &submission,
            &RenderTarget::bitmap(BitmapTarget::new(surface)),
        )
        .expect("headless mode renders an image without a GPU");
    let pixmap = renderer.pixmap().expect("a rasterized frame");
    assert_eq!(pixmap.width(), 200, "the rendered image's width");
    assert_eq!(pixmap.height(), 200, "the rendered image's height");

    // No diagnostic may complain about a missing adapter or display.
    for d in diagnostics.all() {
        let m = d.message.to_lowercase();
        assert!(
            !(m.contains("adapter") || m.contains("display")),
            "headless mode must not report a missing adapter/display: {d:?}"
        );
    }
}

#[doc = include_str!("spec_ch1_topology.doc/arch_04b_construction_never_returns_the_other_runtime.md")]
#[test]
fn arch_04b_construction_never_returns_the_other_runtime() {
    let a = obscur_kit::runtime::UiRuntime::new(RuntimeId::from_raw(7), Size::new(VP, VP));
    let b = obscur_kit::runtime::UiRuntime::new(RuntimeId::from_raw(9), Size::new(VP, VP));
    assert_eq!(a.runtime_id(), RuntimeId::from_raw(7));
    assert_eq!(b.runtime_id(), RuntimeId::from_raw(9));
}
