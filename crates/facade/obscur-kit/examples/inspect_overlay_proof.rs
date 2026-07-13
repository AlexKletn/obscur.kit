//! **In-window inspect overlay → PNG proof (spec §7.6).**
//!
//! Renders a small row of cards, hovers the middle one through the real
//! hit-test path, turns the inspector on, and rasterizes the frame — so the
//! translucent-fill-plus-outline highlight the running window shows on hover is
//! captured as `target/inspect_overlay_proof.png` without a display.
//!
//! Run it:
//! `cargo run -p obscur-kit --example inspect_overlay_proof --features render-cpu`
#![forbid(unsafe_code)]

use std::path::PathBuf;

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::{Point, Size};
use obscur_kit::identity::{ComponentId, RuntimeId};
use obscur_kit::interaction::pointer::{PointerDevice, PointerEvent, PointerEventKind, PointerId};
use obscur_kit::render::frame::{FrameSubmission, RenderFrame};
use obscur_kit::render::target::{BitmapTarget, RenderTarget};
use obscur_kit::render::{Renderer, RendererId};
use obscur_kit::render_cpu::renderer::CpuRenderer;
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{View, ViewFragment, ViewNode, ViewProperty, ViewValue, CLASS};

const VIEWPORT: Size = Size::new(520.0, 200.0);

fn main() {
    let sheet = compile(
        ".row { display: flex; flex-direction: row; gap: 20px; padding: 20px; \
                width: 520px; height: 200px; background-color: #12151f }\n\
         .card { width: 140px; height: 160px; border-radius: 12px }\n\
         .a { background-color: #3552d9 }\n\
         .b { background-color: #22d3a6 }\n\
         .c { background-color: #ffd93d }\n",
    );

    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), VIEWPORT);
    // Lay out first so the hit-test has geometry, then hover the middle card
    // (`.b`, centred around x≈250, y≈100) and enable the inspector.
    runtime.render_now(&view(), std::slice::from_ref(&sheet));
    runtime.handle_pointer(PointerEvent::new(
        PointerId::MOUSE,
        PointerDevice::Mouse,
        PointerEventKind::Move,
        Point::new(250.0, 100.0),
    ));
    runtime.set_inspect_mode(true);
    if let Some(info) = runtime.hovered_inspect() {
        println!("[inspect] highlighting: {info}");
    }

    let frame = runtime.render_now(&view(), std::slice::from_ref(&sheet));
    let snapshot = frame.snapshot().clone();
    println!("[scene] {} paint command(s)", snapshot.display_list().len());

    let mut renderer = CpuRenderer::new(RendererId::from_raw(2));
    let submission = FrameSubmission::new(RenderFrame::new(snapshot, VIEWPORT));
    let target = RenderTarget::bitmap(BitmapTarget::new(VIEWPORT));
    renderer.render(&submission, &target).expect("cpu render");
    let pixmap = renderer.pixmap().expect("a rasterized frame");

    let png = pixmap.encode_png().expect("encode png");
    let out = output_path();
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).expect("create target dir");
    }
    std::fs::write(&out, &png).expect("write png");
    println!("[png] wrote {} → {}", pixmap.width(), out.display());
}

/// A `#row` flex container holding three cards `.a`, `.b`, `.c`.
fn view() -> View {
    let container = ComponentId::from_raw(0x00C0_0001);
    let card = |class: &str| {
        ViewNode::element(container)
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text(format!("card {class}").into())))
    };
    let row = ViewNode::element(container)
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("row".into())))
        .with_children(ViewFragment::from_nodes(vec![card("a"), card("b"), card("c")]));
    View::from_fragment(ViewFragment::from_nodes(vec![row]))
}

fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

fn output_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join("target")
        .join("inspect_overlay_proof.png")
}
