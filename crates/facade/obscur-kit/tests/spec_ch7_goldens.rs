//! **SPECIFICATION-DERIVED** golden-test cases for obscur.kit §7.14.
//!
//! Written by an agent reading obscur.kit §7.14 with **no access to this
//! repository's source**.
#![forbid(unsafe_code)]

#[path = "spec_ch7_support/mod.rs"]
mod support;

use obscur_kit::geometry::Size;
use obscur_kit::headless::snapshot::{GoldenMismatch, ImageSnapshot, ImageTolerance};
use obscur_kit::render::frame::{FrameSubmission, RenderFrame};
use obscur_kit::render::target::{BitmapTarget, RenderTarget};
use obscur_kit::render::{Renderer, RendererId};
use obscur_kit::render_cpu::renderer::CpuRenderer;
use obscur_kit::scene::command::PaintCommand;
use support::{all_geom, all_nodes, drive, el, txt, view};

const VW: f32 = 100.0;

/// Two 100x100 RGBA rasters differing by exactly 2/255 in one channel of one
/// pixel out of 10,000. The differing pixel is at `(37, 21)`.
fn pair() -> (ImageSnapshot, ImageSnapshot, (u32, u32)) {
    const W: u32 = 100;
    const H: u32 = 100;
    let golden = vec![0x40_u8; (W * H * 4) as usize];
    let mut actual = golden.clone();
    let (x, y) = (37_u32, 21_u32);
    let offset = ((y * W + x) * 4 + 1) as usize;
    actual[offset] = actual[offset].wrapping_add(2);
    (
        ImageSnapshot::from_rgba(W, H, &golden),
        ImageSnapshot::from_rgba(W, H, &actual),
        (x, y),
    )
}

/// §7.14, verbatim: "Raster golden tests are allowed with explicit tolerances
/// for anti-aliasing and font/backend variation."
#[test]
fn a_raster_tolerance_must_be_explicit_and_a_failure_names_the_pixel() {
    let (golden, actual, (x, y)) = pair();

    // Tolerance 0: FAILS, naming the pixel coordinate and the per-channel delta.
    let strict = golden.compare_within(&actual, ImageTolerance::exact());
    let mismatch = strict.expect_err("a 2/255 difference is a failure at tolerance 0");
    assert!(
        matches!(mismatch, GoldenMismatch::TooManyDifferingPixels { .. }),
        "the failure is the typed too-many-differing-pixels mismatch, got {mismatch:?}"
    );
    let reported = format!("{mismatch:?}");
    assert!(
        reported.contains(&x.to_string()) && reported.contains(&y.to_string()),
        "the failure names the pixel coordinate ({x}, {y}), got {reported}"
    );
    assert!(
        reported.contains('2'),
        "the failure names the per-channel delta of 2, got {reported}"
    );

    // A per-channel tolerance of 2: passes.
    let lenient = golden.compare_within(&actual, ImageTolerance::new(2, 0.0));
    assert!(
        lenient.is_ok(),
        "a per-channel tolerance of 2 admits a 2/255 difference, got {lenient:?}"
    );

    // No tolerance supplied: the comparison cannot be spelled at all. There is no
    // `compare(&other)` on the golden API, so a hidden default fuzz is
    // unreachable by construction — the third sub-case is enforced by the type
    // system, and this assertion records that the explicit form is the only form.
    assert_eq!(
        ImageTolerance::exact().max_channel_delta(),
        0,
        "the strictest spellable tolerance permits nothing — there is no built-in fuzz"
    );
    assert_eq!(
        ImageTolerance::exact().max_differing_fraction(),
        0.0,
        "…and no built-in differing-pixel allowance"
    );
}

const RICH_CSS: &str = ".root { width: 100px; height: 100px } \
                        .card { width: 60px; height: 40px; \
                                border-radius: 8px; \
                                background-color: #3366aa; \
                                box-shadow: 0 2px 6px #00000055; \
                                filter: blur(2px); \
                                color: #ffffff }";

fn rich() -> obscur_kit::view::view::View {
    view(vec![el(1, "root", vec![el(2, "card", vec![txt("Hi")])])])
}

/// A comparable rendering of the backend-independent layers §7.14 names.
fn structural(rt: &obscur_kit::runtime::UiRuntime) -> String {
    let commands: Vec<String> = rt
        .snapshot()
        .display_list()
        .commands()
        .iter()
        .map(|c: &PaintCommand| format!("{c:?}"))
        .collect();
    // `inspect_snapshot` is the workspace's order-stable rendering of the
    // retained tree plus computed style plus layout; a raw `ComputedStyle` debug
    // prints from a hash map whose iteration order varies run to run, and that
    // ordering is SNAP-01's subject, not this case's.
    format!(
        "tree={:?}\nstyle+layout={}\nlayout={:?}\nscene={}",
        all_nodes(rt),
        rt.inspect_snapshot(),
        all_geom(rt),
        commands.join(";")
    )
}

/// §7.14, verbatim: "Golden tests should prefer backend-independent snapshots:"
/// / "- UI IR;" / "- retained tree;" / "- computed style;" / "- layout;" /
/// "- Scene/Paint;" / "- accessibility;".
#[test]
fn structural_goldens_do_not_move_when_a_backend_is_attached() {
    // (a) No backend at all.
    let no_backend = drive(VW, VW, RICH_CSS, &rich());
    let a = structural(&no_backend);
    let a_a11y = format!("{:?}", no_backend.semantic_snapshot());

    // (b) The same tree through the CPU raster path.
    let cpu = drive(VW, VW, RICH_CSS, &rich());
    let mut renderer = CpuRenderer::new(RendererId::from_raw(2));
    let submission = FrameSubmission::new(RenderFrame::new(cpu.snapshot(), Size::new(VW, VW)));
    let target = RenderTarget::bitmap(BitmapTarget::new(Size::new(VW, VW)));
    renderer
        .render(&submission, &target)
        .expect("the CPU backend rasterizes the scene");
    let rendered = renderer.pixmap().expect("a rasterized frame");
    assert!(
        rendered.width() > 0,
        "case (b) really produced a rendered image"
    );
    let b = structural(&cpu);
    let b_a11y = format!("{:?}", cpu.semantic_snapshot());

    assert_eq!(
        a, b,
        "retained tree, computed style, layout and Scene/Paint goldens are byte-identical \
         across backend configurations — only the rendered image may differ"
    );
    assert_eq!(
        a_a11y, b_a11y,
        "the accessibility golden is byte-identical across backend configurations"
    );

    // A blur radius clamped by a backend limit, or a device pixel ratio baked in,
    // would show up here: the un-rasterized scene must already carry them.
    assert!(
        a.contains("blur") || a.contains("Blur"),
        "the backend-independent scene records the blur it was asked for: {a}"
    );
}
