//! **Proof: CSS-authored motion actually moves.**
//!
//! A `transition` and an `@keyframes` animation, authored in plain CSS, compiled
//! through the real CSS frontend, and driven by the runtime's motion scheduler —
//! rasterized through the software [`CpuRenderer`] as a five-panel filmstrip in
//! `target/css_motion_proof.png`:
//!
//! - **The blue `.bar`** carries `transition: width 1s linear`. Hovering it
//!   changes its computed width `60px -> 300px`, so the transition fires and the
//!   bar *grows across the strip* instead of snapping.
//! - **The red `.ghost`** plays `@keyframes fade` via the `animation` shorthand,
//!   so it fades in `opacity: 0.15 -> 1` over the same second.
//!
//! Each panel is one motion frame at `t = 0, 0.25, 0.5, 0.75, 1.0`. The bar's
//! width is read back from the runtime's own layout boxes and **asserted** at
//! every step, so a regression fails the run rather than printing a still image.
//!
//! The wiring is the kit's, not this example's: [`CssPipeline::install`] hands the
//! sources to the CSS frontend and installs *both* halves of the result on the
//! runtime — the matchable rules and the `transition`/`@keyframes` motion (§4.17).
//! A host that installs only the rules gets a motionless UI, which is exactly why
//! the plumbing lives in the kit.
//!
//! Run it:
//! `cargo run -p obscur-kit --example css_motion_proof --features render-cpu`
#![forbid(unsafe_code)]

use std::path::PathBuf;

use obscur_kit::bootstrap::CssPipeline;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::Size;
use obscur_kit::identity::{ComponentId, NodeId, RuntimeId};
use obscur_kit::render::frame::{FrameSubmission, RenderFrame};
use obscur_kit::render::target::{BitmapTarget, RenderTarget};
use obscur_kit::render::{Renderer, RendererId};
use obscur_kit::render_cpu::renderer::CpuRenderer;
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::style::selector::PseudoState;
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

/// The logical extent each panel is laid out and rasterized under.
const VIEWPORT: Size = Size::new(340.0, 130.0);

/// The authored sheet. Both motion frontends at once: a `transition` whose
/// trigger is a `:hover`-gated width change, and a named `@keyframes` played
/// through the `animation` shorthand.
const CSS: &str = "\
.stage { display: flex; flex-direction: column; align-items: flex-start; gap: 16px; \
padding: 20px; background: #f4f3f0; width: 100%; height: 100%; }\
.bar { width: 60px; height: 36px; background: #3366ff; transition: width 1s linear; }\
.bar:hover { width: 300px; }\
@keyframes fade { from { opacity: 0.15; } to { opacity: 1.0; } }\
.ghost { width: 300px; height: 36px; background: #d9534f; animation: fade 1s; }\
";

/// The motion frames captured, in seconds since the transition fired.
const TIMELINE: &[f32] = &[0.0, 0.25, 0.5, 0.75, 1.0];

/// The bar's authored width endpoints — `.bar` and `.bar:hover`.
const WIDTH_FROM: f32 = 60.0;
const WIDTH_TO: f32 = 300.0;

fn main() {
    println!("== obscur-kit CSS motion proof (headless CPU) ==\n");

    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), VIEWPORT);
    let mut renderer = CpuRenderer::new(RendererId::from_raw(2));

    // The kit's CSS path: parse (cached per source revision) -> compile ->
    // install the matchable rules AND the sheet's motion on the runtime.
    let mut css = CssPipeline::new();
    css.install(&mut runtime, &[CssSource::anonymous(CSS)]);
    assert!(css.errors().is_empty(), "the authored CSS is valid");
    println!(
        "[css] installed {} sheet(s): {} transition rule(s), {} @keyframes rule(s)",
        css.sheets().len(),
        css.motion().transitions().len(),
        css.motion().keyframes().len(),
    );
    assert_eq!(
        css.motion().transitions().len(),
        1,
        "`transition: width` reached the runtime"
    );
    assert_eq!(
        css.motion().keyframes().len(),
        1,
        "`animation: fade` reached the runtime"
    );

    let view = view();
    let sheets: Vec<Stylesheet> = css.sheets().to_vec();

    // Frame 1 establishes the baseline the transition eases *from*: the bar's
    // computed width has not changed yet, so nothing is in flight. The `.ghost`
    // animation, which needs no trigger, starts here.
    let _ = runtime.render_now(&view, &sheets);
    let bar = bar_node(&runtime);
    assert_eq!(
        bar_width(&runtime, bar),
        WIDTH_FROM,
        "the bar starts narrow"
    );
    assert_eq!(
        runtime.active_animations(),
        1,
        "the @keyframes animation is running from the first frame"
    );

    // Hover the bar: its computed width changes 60 -> 300, which is what a
    // `transition` watches for. The scheduler now drives both timelines.
    runtime.set_pseudo_state(bar, PseudoState::HOVER);

    let mut panels = Vec::new();
    let mut elapsed = 0.0f32;
    for (index, &time) in TIMELINE.iter().enumerate() {
        runtime.advance_motion(time - elapsed);
        elapsed = time;

        let result = runtime.render_now(&view, &sheets);
        let width = bar_width(&runtime, bar);

        // Linear easing over one second: the width is pinned at every step, so a
        // motionless (or wrongly eased) pipeline fails here rather than producing
        // a still filmstrip.
        let expected = WIDTH_FROM + (WIDTH_TO - WIDTH_FROM) * time;
        assert!(
            (width - expected).abs() < 0.01,
            "at t={time}s the transitioned width must be {expected}, got {width}"
        );
        if index == 0 {
            assert_eq!(
                runtime.active_animations(),
                2,
                "the transition joined the animation on the scheduler"
            );
        }
        println!("[t={time:>4}s] bar width = {width:>5.1}px");

        panels.push(rasterize(&mut renderer, &result));
    }

    assert_eq!(
        runtime.active_animations(),
        0,
        "both timelines finished and were pruned"
    );
    assert!(
        panels.first().map(|p: &tiny_skia::Pixmap| p.data())
            != panels.last().map(tiny_skia::Pixmap::data),
        "the first and last panels must differ — that difference IS the motion"
    );

    let combined = stitch(&panels.iter().collect::<Vec<_>>());
    let png = combined.encode_png().expect("encode png");
    let out = output_path("css_motion_proof.png");
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).expect("create target dir");
    }
    std::fs::write(&out, &png).expect("write png");
    println!(
        "\n[png] wrote {} × {} → {} ({} bytes) — the bar grows and the ghost fades left to right",
        combined.width(),
        combined.height(),
        out.display(),
        png.len(),
    );
    println!(
        "\n== CSS-authored motion moved across {} frames; exiting 0 ==",
        TIMELINE.len()
    );
}

/// The view: a `.stage` column holding the transitioned `.bar` and the animated
/// `.ghost`.
fn view() -> View {
    let bar = ViewNode::element(ComponentId::from_raw(2))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("bar".into())));
    let ghost = ViewNode::element(ComponentId::from_raw(3))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("ghost".into())));
    let stage = ViewNode::element(ComponentId::from_raw(1))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("stage".into())))
        .with_children(ViewFragment::from_nodes(vec![bar, ghost]));
    View::single(stage)
}

/// The `.bar`'s node: the tree root is synthetic, so the `.stage` is its first
/// child and the `.bar` is the stage's.
fn bar_node(runtime: &UiRuntime) -> NodeId {
    let root = runtime.tree().root().expect("a root after the first frame");
    let stage = runtime.tree().get(root).expect("the root node").children()[0];
    runtime
        .tree()
        .get(stage)
        .expect("the stage node")
        .children()[0]
}

/// The laid-out width of `node`, read back from the runtime's own layout boxes.
fn bar_width(runtime: &UiRuntime, node: NodeId) -> f32 {
    runtime
        .layout_boxes()
        .get(&node)
        .expect("the sized node has a box")
        .size()
        .width
}

/// Rasterizes a rendered frame's scene through the software renderer.
fn rasterize(
    renderer: &mut CpuRenderer,
    result: &obscur_kit::runtime::frame::FrameResult,
) -> tiny_skia::Pixmap {
    let frame = RenderFrame::new(result.snapshot().clone(), VIEWPORT);
    let submission = FrameSubmission::new(frame);
    let target = RenderTarget::bitmap(BitmapTarget::new(VIEWPORT));
    renderer
        .render(&submission, &target)
        .expect("cpu render succeeds");
    renderer.pixmap().expect("a rasterized frame").clone()
}

/// Horizontally stitches `panels` left-to-right with thin dividers between them.
fn stitch(panels: &[&tiny_skia::Pixmap]) -> tiny_skia::Pixmap {
    let gap = 3u32;
    let height = panels.iter().map(|p| p.height()).max().unwrap_or(0);
    let dividers = u32::try_from(panels.len()).unwrap_or(0).saturating_sub(1);
    let width: u32 = panels.iter().map(|p| p.width()).sum::<u32>() + gap * dividers;
    let mut out = tiny_skia::Pixmap::new(width, height).expect("combined pixmap");
    out.fill(tiny_skia::Color::from_rgba8(0x2b, 0x2a, 0x27, 0xff));
    let paint = tiny_skia::PixmapPaint::default();
    let mut x = 0i32;
    for panel in panels {
        out.draw_pixmap(
            x,
            0,
            panel.as_ref(),
            &paint,
            tiny_skia::Transform::identity(),
            None,
        );
        x += i32::try_from(panel.width() + gap).unwrap_or(0);
    }
    out
}

/// The workspace-relative output path `target/<name>`.
fn output_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join("target")
        .join(name)
}
