//! **Proof: a real `overflow-y: auto` vertical scroll container — clip, scroll
//! offset, scrollbar, and a scrolled hit-test.**
//!
//! Reimplemented from **CSS Overflow Module Level 3** (§overflow, §scroll-container,
//! §scrollport). A 200×120 `.scroller` with `overflow: auto` holds six 40-px rows
//! (240 px of content), so it is a scroll container. This drives the REAL pipeline
//! through the public facade on a live [`UiRuntime`]: reconcile → style → layout →
//! compose, where the scene coordinator brackets the scroller's descendants with a
//! `PushTransform(translate(0, −scroll_top))` inside its overflow clip and paints a
//! vertical scrollbar. Two panels are rasterized headless through the software
//! [`CpuRenderer`] (no GPU) to `target/scroll_proof.png`:
//!
//! - **Panel 1 — Idle.** `scroll_top = 0`: rows 0–2 fill the scrollport; rows 3–5
//!   are CLIPPED away (they do not paint below the box). The scrollbar thumb sits
//!   at the top.
//! - **Panel 2 — Scrolled.** After [`UiRuntime::scroll_by`]`(scroller, 80)` the
//!   content is translated up 80 px: row 2 now paints at the top and rows 0–1 have
//!   scrolled out. The thumb has moved down.
//!
//! Before writing the image it machine-checks: (a) row 3 does NOT paint below the
//! scrollport (the clip bites); (b) the top-of-scrollport pixel is row 0 idle and
//! row 2 scrolled (the content scrolled up by exactly one 80-px step / two rows);
//! (c) a scrollbar thumb paints in the right gutter and moves down when scrolled;
//! and (d) a pointer at the same screen point hit-tests row 0 idle but row 2
//! scrolled — the pointer mapping composes the container's `scroll_top`.
//!
//! Run it:
//! `cargo run -p obscur-kit --example scroll_proof --features render-cpu`
#![forbid(unsafe_code)]

use std::path::PathBuf;

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::{Point, Size};
use obscur_kit::identity::{ComponentId, NodeId, RuntimeId};
use obscur_kit::interaction::pointer::{PointerDevice, PointerEvent, PointerEventKind, PointerId};
use obscur_kit::render::frame::{FrameSubmission, RenderFrame};
use obscur_kit::render::target::{BitmapTarget, RenderTarget};
use obscur_kit::render::{Renderer, RendererId};
use obscur_kit::render_cpu::renderer::CpuRenderer;
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{View, ViewFragment, ViewNode, ViewProperty, ViewValue, CLASS};

/// The logical extent each panel is laid out and rasterized under.
const VIEWPORT: Size = Size::new(220.0, 150.0);

/// One wheel step, in logical px — two rows (2 × 40).
const STEP: f32 = 80.0;

/// The demo stylesheet: a light page, a 200×120 `overflow: auto` scroller, and six
/// 40-px rows in distinct colors so a pixel unambiguously identifies its row.
const CSS: &str = "\
.page { width: 220px; height: 150px; background-color: #dddddd; }\n\
.scroller { width: 200px; height: 120px; overflow: auto; background-color: #ffffff; }\n\
.row { width: 200px; height: 40px; }\n\
.r0 { background-color: #d93333; }\n\
.r1 { background-color: #33b359; }\n\
.r2 { background-color: #3366d9; }\n\
.r3 { background-color: #e68c26; }\n\
.r4 { background-color: #8c4dbf; }\n\
.r5 { background-color: #26a6a6; }\n\
";

fn main() {
    println!("== obscur-kit overflow scroll-container proof (headless CPU) ==\n");

    let sheet = compile(CSS);
    let sheets = std::slice::from_ref(&sheet);
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), VIEWPORT);
    let mut renderer = CpuRenderer::new(RendererId::from_raw(1));

    // --- Panel 1: idle -----------------------------------------------------
    let _ = runtime.render_now(&scroller_view(), sheets);
    let (scroller, rows) = scroll_nodes(&runtime);
    assert_eq!(rows.len(), 6, "the scroller holds six rows");
    assert_eq!(runtime.scroll_top(scroller), 0.0, "starts unscrolled");

    // The overflow clip is emitted (step 1) and the content overflows.
    let idle_snapshot = runtime.snapshot();
    let idle_list = idle_snapshot.display_list();
    let clips = idle_list
        .commands()
        .iter()
        .filter(|c| matches!(c, obscur_kit::scene::command::PaintCommand::PushClip(_)))
        .count();
    assert!(clips >= 1, "the scroller brackets its subtree with an overflow clip");
    println!("[scene] idle: {} commands, {clips} PushClip", idle_list.commands().len());

    let idle = capture(&mut runtime, &mut renderer);

    // (d) idle hit-test: a pointer at the top of the scrollport lands on row 0.
    let idle_hit = runtime.handle_pointer(mouse(PointerEventKind::Move, 100.0, 20.0)).target();
    assert_eq!(idle_hit, Some(rows[0]), "idle: (100,20) hit-tests row 0");

    // --- Panel 2: scrolled by one step (two rows) --------------------------
    let landed = runtime.scroll_by(scroller, STEP);
    assert_eq!(landed, STEP, "scroll_by lands at 80 (within 0..=120)");
    assert_eq!(runtime.scroll_top(scroller), STEP);
    // Clamp: a huge delta stops at content_height − scrollport_height = 240 − 120.
    let clamped = runtime.scroll_by(scroller, 10_000.0);
    assert_eq!(clamped, 120.0, "scroll_top clamps to the scrollable range");
    runtime.scroll_by(scroller, -40.0); // back to 80 for the panel
    assert_eq!(runtime.scroll_top(scroller), STEP);

    let _ = runtime.render_now(&scroller_view(), sheets);
    // The scrolled content is wrapped in a translate(0, −80) transform.
    let scrolled_snapshot = runtime.snapshot();
    let scrolled_list = scrolled_snapshot.display_list();
    let has_scroll_tf = scrolled_list.commands().iter().any(|c| {
        matches!(
            c,
            obscur_kit::scene::command::PaintCommand::PushTransform(t)
                if (t.coeffs[5] + STEP).abs() < 0.01
        )
    });
    assert!(has_scroll_tf, "a PushTransform(translate(0,-80)) scrolls the content");
    println!("[scene] scrolled: PushTransform(translate(0,-{STEP})) present");

    let scrolled = capture(&mut runtime, &mut renderer);

    // (d) scrolled hit-test: the SAME screen point now lands on row 2.
    let scrolled_hit = runtime.handle_pointer(mouse(PointerEventKind::Move, 100.0, 20.0)).target();
    assert_eq!(scrolled_hit, Some(rows[2]), "scrolled: (100,20) hit-tests row 2");

    // --- Machine-check the pixels -----------------------------------------
    verify(&idle, &scrolled);

    let combined = stitch(&[&idle, &scrolled]);
    let png = combined.encode_png().expect("encode png");
    let out = output_path("scroll_proof.png");
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).expect("create target dir");
    }
    std::fs::write(&out, &png).expect("write png");
    println!(
        "\n[png] wrote {} × {} → {} ({} bytes) — idle | scrolled",
        combined.width(),
        combined.height(),
        out.display(),
        png.len(),
    );
    println!("[verify] OK — clip + scroll offset + scrollbar + scrolled hit-test all hold");
}

/// The view: a `.page` holding a `.scroller` that holds six `.row`s.
fn scroller_view() -> View {
    let row = |n: u64, idx: usize| {
        ViewNode::element(ComponentId::from_raw(n))
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text(format!("row r{idx}").into())))
    };
    let rows: Vec<ViewNode> = (0..6).map(|i| row(200 + i as u64, i)).collect();
    let scroller = ViewNode::element(ComponentId::from_raw(100))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("scroller".into())))
        .with_children(ViewFragment::from_nodes(rows));
    let page = ViewNode::element(ComponentId::from_raw(10))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("page".into())))
        .with_children(ViewFragment::from_nodes(vec![scroller]));
    View::from_fragment(ViewFragment::from_nodes(vec![page]))
}

/// Returns `(scroller node, [row nodes])` after a frame has populated the tree.
fn scroll_nodes(runtime: &UiRuntime) -> (NodeId, Vec<NodeId>) {
    let root = runtime.tree().root().expect("root after a frame");
    let page = runtime.tree().get(root).unwrap().children()[0];
    let scroller = runtime.tree().get(page).unwrap().children()[0];
    let rows = runtime.tree().get(scroller).unwrap().children().to_vec();
    (scroller, rows)
}

fn mouse(kind: PointerEventKind, x: f32, y: f32) -> PointerEvent {
    PointerEvent::new(PointerId::MOUSE, PointerDevice::Mouse, kind, Point::new(x, y))
}

/// Rasterizes the runtime's current committed scene into a fresh pixmap.
fn capture(runtime: &mut UiRuntime, renderer: &mut CpuRenderer) -> tiny_skia::Pixmap {
    let snapshot = runtime.snapshot();
    let frame = RenderFrame::new(snapshot, VIEWPORT);
    let submission = FrameSubmission::new(frame);
    let target = RenderTarget::bitmap(BitmapTarget::new(VIEWPORT));
    renderer.render(&submission, &target).expect("cpu render succeeds");
    renderer.pixmap().expect("a rasterized frame").clone()
}

/// Whether a pixel roughly matches an `(r, g, b)` in 0..=255 (loose, so anti-alias
/// and the translucent scrollbar wash do not break the check).
fn near(px: Option<tiny_skia::PremultipliedColorU8>, r: u8, g: u8, b: u8) -> bool {
    px.is_some_and(|p| {
        let d = |a: u8, b: u8| i32::from(a) - i32::from(b);
        d(p.red(), r).abs() < 40 && d(p.green(), g).abs() < 40 && d(p.blue(), b).abs() < 40
    })
}

/// Asserts the clip, the scroll offset, and the scrollbar thumb movement.
fn verify(idle: &tiny_skia::Pixmap, scrolled: &tiny_skia::Pixmap) {
    // (a) Clip: row 3 (orange #e68c26) is entirely below the 120-px scrollport, so
    //     it must NOT paint at y = 130 (below the box) — the overflow clip bit.
    assert!(
        !near(idle.pixel(10, 130), 0xe6, 0x8c, 0x26),
        "clip failed — row 3 painted below the scrollport at (10,130)"
    );
    // The page grey shows there instead.
    assert!(near(idle.pixel(10, 130), 0xdd, 0xdd, 0xdd), "page grey below the clipped scroller");

    // (b) Scroll offset: the top of the scrollport is row 0 (red) idle, row 2
    //     (blue) after scrolling up two rows.
    assert!(near(idle.pixel(10, 20), 0xd9, 0x33, 0x33), "idle: row 0 (red) at scrollport top");
    assert!(near(scrolled.pixel(10, 20), 0x33, 0x66, 0xd9), "scrolled: row 2 (blue) at scrollport top");
    // A mid-scrollport pixel: row 1 (green) idle → row 3 (orange) scrolled.
    assert!(near(idle.pixel(10, 60), 0x33, 0xb3, 0x59), "idle: row 1 (green) mid-scrollport");
    assert!(near(scrolled.pixel(10, 60), 0xe6, 0x8c, 0x26), "scrolled: row 3 (orange) mid-scrollport");

    // (c) Scrollbar: the right gutter (x≈196) carries a ~35%-black thumb over a
    //     faint track. The thumb DARKENS its row noticeably vs the same row left of
    //     the gutter; the track barely tints it. Detect the thumb by that contrast,
    //     so it is unambiguous over the colored rows. The thumb is at the top idle
    //     and has moved down when scrolled.
    assert!(thumb_at(idle, 8), "idle: scrollbar thumb sits at the top of the gutter");
    assert!(!thumb_at(scrolled, 8), "scrolled: the thumb has left the top of the gutter");
    assert!(thumb_at(scrolled, 70), "scrolled: the thumb moved down the gutter");
    println!("[verify] clip bites row 3; content scrolled row0→row2; thumb moved top→down");
}

/// Whether the scrollbar thumb covers row `y`: the gutter pixel (x≈196, inside the
/// 8-px scrollbar) is markedly darker than the same row left of the gutter (x=100).
/// The ~35%-black thumb drops the channel sum by ~a third; the ~6% track barely
/// changes it, so a 40-unit margin cleanly tells thumb from track.
fn thumb_at(pixmap: &tiny_skia::Pixmap, y: u32) -> bool {
    let sum = |x: u32| {
        pixmap
            .pixel(x, y)
            .map_or(0i32, |p| i32::from(p.red()) + i32::from(p.green()) + i32::from(p.blue()))
    };
    sum(196) + 40 < sum(100)
}

/// Horizontally stitches `panels` with a thin divider between them.
fn stitch(panels: &[&tiny_skia::Pixmap]) -> tiny_skia::Pixmap {
    let gap = 3u32;
    let height = panels.iter().map(|p| p.height()).max().unwrap_or(0);
    let width: u32 =
        panels.iter().map(|p| p.width()).sum::<u32>() + gap * (panels.len().max(1) as u32 - 1);
    let mut out = tiny_skia::Pixmap::new(width, height).expect("combined pixmap");
    out.fill(tiny_skia::Color::from_rgba8(0x2b, 0x2a, 0x27, 0xff));
    let paint = tiny_skia::PixmapPaint::default();
    let mut x = 0i32;
    for panel in panels {
        out.draw_pixmap(x, 0, panel.as_ref(), &paint, tiny_skia::Transform::identity(), None);
        x += panel.width() as i32 + gap as i32;
    }
    out
}

/// Compiles CSS text into a runtime-matchable [`Stylesheet`] (author origin).
fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

/// The workspace-relative output path `target/<name>`.
fn output_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join("target")
        .join(name)
}
