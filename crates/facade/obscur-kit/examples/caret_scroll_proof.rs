#![doc = include_str!("caret_scroll_proof.doc/_module.md")]
#![forbid(unsafe_code)]

use std::path::PathBuf;

use obscur_kit::geometry::{Point, Size};
use obscur_kit::identity::{ComponentId, InternedSymbol, NodeId, RuntimeId};
use obscur_kit::interaction::command::{KeyEvent, LogicalKey, NamedKey, PhysicalKey};
use obscur_kit::interaction::pointer::{PointerDevice, PointerEvent, PointerEventKind, PointerId};
use obscur_kit::render::frame::{FrameSubmission, RenderFrame};
use obscur_kit::render::target::{BitmapTarget, RenderTarget};
use obscur_kit::render::{Renderer, RendererId};
use obscur_kit::render_cpu::renderer::CpuRenderer;
use obscur_kit::runtime::UiRuntime;
use obscur_kit::scene::command::PaintCommand;
use obscur_kit::scene::scene::SceneSnapshot;
use obscur_kit::style::cascade::CascadeOrigin;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::text::font::{FontAttributes, FontFamily, FontId, GenericFamily};
use obscur_kit::text_harfrust::{FontRegistry, HarfrustShaper};
use obscur_kit::text_unicode::UnicodeTextItemizer;
use obscur_kit::view::view::{
    CLASS, EVENT_INPUT, View, ViewEventBinding, ViewFragment, ViewNode, ViewProperty, ViewValue,
};

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;

#[doc = include_str!("caret_scroll_proof.doc/VIEWPORT.md")]
const VIEWPORT: Size = Size::new(360.0, 120.0);

#[doc = include_str!("caret_scroll_proof.doc/CMD.md")]
const CMD: InternedSymbol = InternedSymbol::from_raw(0x0001);

#[doc = include_str!("caret_scroll_proof.doc/CSS.md")]
const CSS: &str = "\
.stage { padding: 40px 20px; background: #12141c; }\
.field { width: 100px; height: 28px; background: #ffffff; color: #101216; \
white-space: nowrap; overflow: hidden; border: 1px solid #5b6472; padding: 4px; }";

#[doc = include_str!("caret_scroll_proof.doc/VALUE.md")]
const VALUE: &str = "scrolling text field";

fn main() {
    println!("== obscur-kit single-line caret horizontal-scroll proof (headless CPU) ==\n");

    let author = compile(CSS, CascadeOrigin::GlobalApp);

    // --- Drive the runtime: focus, type a wide value, capture the scrolled state.
    let (mut runtime, mut renderer) = boot();
    runtime.set_default_stylesheet(author.clone());
    let sheets = std::slice::from_ref(&author);

    let _ = runtime.render_now(&field_view(""), sheets);
    // Focus the field by pressing inside it (near its left; y in the field row).
    runtime.handle_pointer(mouse(PointerEventKind::Down, 30.0, 54.0));
    assert!(runtime.focused().is_some(), "the press focused the field");

    for c in VALUE.chars() {
        runtime.handle_key(char_press(c));
    }
    println!(
        "[type] entered {:?} ({} chars)",
        VALUE,
        VALUE.chars().count()
    );

    // The scrolled ("tail") frame.
    let scrolled = runtime
        .render_now(&field_view(""), sheets)
        .snapshot()
        .clone();
    let field = field_node(&runtime);
    let field_box = runtime
        .layout_boxes()
        .get(&field)
        .expect("field box")
        .bounds;
    let box_right = field_box.origin.x + field_box.size.width;
    let caret_x = caret_x_of(&scrolled).expect("a caret in the scrolled frame");
    let run_left = run_left_of(&scrolled).expect("a run in the scrolled frame");

    println!(
        "[scrolled] field box x∈[{:.1}, {:.1}]  caret.x = {:.1}  run.left = {:.1}",
        field_box.origin.x, box_right, caret_x, run_left,
    );
    assert!(
        caret_x <= box_right + 0.5,
        "caret.x {caret_x} must stay within the box right {box_right}",
    );
    assert!(
        run_left < field_box.origin.x - 0.5,
        "the head must be scrolled left of the box (run.left {run_left} < box.left {})",
        field_box.origin.x,
    );
    // The run must be the FULL value, and its TAIL must actually land inside the
    // box — the property a `text-overflow: clip` glyph-truncation would violate. A
    // scrolled field must NOT truncate to the box-width prefix (that prefix is then
    // translated off-screen left, leaving an empty box); it keeps the whole run and
    // relies on the `overflow: hidden` clip-rect. So the run's advance width must
    // EXCEED the field's content width (untruncated), and its right edge must sit
    // inside the box near the right (where the tail + caret show).
    let content_width = field_box.size.width - 8.0; // 4px padding each side
    let (run_width, run_right) = run_extent_of(&scrolled).expect("a run in the scrolled frame");
    println!(
        "[scrolled] run width = {run_width:.1}  run.right = {run_right:.1}  content_w = {content_width:.1}"
    );
    assert!(
        run_width > content_width + 0.5,
        "the run must be the untruncated full value (width {run_width} > content {content_width})",
    );
    assert!(
        run_right > field_box.origin.x && run_right <= box_right + 0.5,
        "the tail must land inside the box (run.right {run_right} in ({:.1}, {:.1}])",
        field_box.origin.x,
        box_right,
    );
    println!(
        "  ✓ caret is inside the box; the head is scrolled out and clipped; the full-run TAIL is visible"
    );

    // --- Home: reset the scroll so the head is visible again.
    runtime.handle_key(named_press(NamedKey::Home));
    let reset = runtime
        .render_now(&field_view(""), sheets)
        .snapshot()
        .clone();
    let head_run_left = run_left_of(&reset).expect("a run in the reset frame");
    println!(
        "[home] run.left = {:.1}  (box x∈[{:.1}, {:.1}])",
        head_run_left, field_box.origin.x, box_right,
    );
    // Home resets scroll_left to 0, so the run's left snaps back from its
    // scrolled-out position to the field's (padding-inset) content left — inside
    // the box again, and well right of where it sat while scrolled.
    assert!(
        head_run_left >= field_box.origin.x - 0.5 && head_run_left < box_right,
        "Home brings the head back inside the box: run.left {head_run_left} in [{:.1}, {:.1}]",
        field_box.origin.x,
        box_right,
    );
    assert!(
        head_run_left > run_left + 0.5,
        "the head moved back rightward after Home (scrolled {run_left:.1} → {head_run_left:.1})",
    );
    println!("  ✓ Home reset scroll_left to 0; the head is visible inside the box again\n");

    // --- Render both states into one PNG.
    let left = rasterize(&mut renderer, &scrolled);
    let right = rasterize(&mut renderer, &reset);
    let pixmap = side_by_side(&left, &right);

    let png = pixmap.encode_png().expect("encode png");
    let out = output_path("caret_scroll_proof.png");
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).expect("create target dir");
    }
    std::fs::write(&out, &png).expect("write png");
    println!(
        "[png] wrote {} × {} → {} ({} bytes) — left: scrolled (tail) | right: Home (head)",
        pixmap.width(),
        pixmap.height(),
        out.display(),
        png.len(),
    );
}

#[doc = include_str!("caret_scroll_proof.doc/field_view.md")]
fn field_view(text: &str) -> View {
    View::from_fragment(ViewFragment::from_nodes(vec![
        ViewNode::element(ComponentId::from_raw(1))
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text("stage".into())))
            .with_children(ViewFragment::from_nodes(vec![
                ViewNode::element(ComponentId::from_raw(2))
                    .with_prop(ViewProperty::new(CLASS, ViewValue::Text("field".into())))
                    .with_event(ViewEventBinding::new(EVENT_INPUT, CMD))
                    .with_children(ViewFragment::from_nodes(vec![ViewNode::text(
                        text.to_string(),
                    )])),
            ])),
    ]))
}

#[doc = include_str!("caret_scroll_proof.doc/field_node.md")]
fn field_node(runtime: &UiRuntime) -> NodeId {
    let root = runtime.tree().root().expect("root after a frame");
    let stage = runtime.tree().get(root).unwrap().child_nodes()[0];
    runtime.tree().get(stage).unwrap().child_nodes()[0]
}

#[doc = include_str!("caret_scroll_proof.doc/caret_x_of.md")]
fn caret_x_of(snapshot: &SceneSnapshot) -> Option<f32> {
    snapshot
        .display_list()
        .commands()
        .iter()
        .find_map(|c| match c {
            PaintCommand::DrawPath { path, .. } => path
                .bounds()
                .filter(|r| (r.size.width - 1.0).abs() < 1e-3)
                .map(|r| r.origin.x),
            _ => None,
        })
}

#[doc = include_str!("caret_scroll_proof.doc/run_left_of.md")]
fn run_left_of(snapshot: &SceneSnapshot) -> Option<f32> {
    snapshot
        .display_list()
        .commands()
        .iter()
        .find_map(|c| match c {
            PaintCommand::DrawText { origin, .. } => Some(origin.x),
            _ => None,
        })
}

#[doc = include_str!("caret_scroll_proof.doc/run_extent_of.md")]
fn run_extent_of(snapshot: &SceneSnapshot) -> Option<(f32, f32)> {
    snapshot
        .display_list()
        .commands()
        .iter()
        .find_map(|c| match c {
            PaintCommand::DrawText { origin, run, .. } => {
                let width: f32 = run.glyphs().iter().map(|g| g.advance()).sum();
                Some((width, origin.x + width))
            }
            _ => None,
        })
}

#[doc = include_str!("caret_scroll_proof.doc/boot.md")]
fn boot() -> (UiRuntime, CpuRenderer) {
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), VIEWPORT);
    runtime.on_text_input(CMD, |_| {});
    let mut renderer = CpuRenderer::new(RendererId::from_raw(2));
    if let Some(bytes) = load_system_font() {
        wire_text_stack(&mut runtime, &mut renderer, bytes);
    }
    (runtime, renderer)
}

#[doc = include_str!("caret_scroll_proof.doc/rasterize.md")]
fn rasterize(renderer: &mut CpuRenderer, snapshot: &SceneSnapshot) -> tiny_skia::Pixmap {
    let frame = RenderFrame::new(snapshot.clone(), VIEWPORT);
    let submission = FrameSubmission::new(frame);
    let target = RenderTarget::bitmap(BitmapTarget::new(VIEWPORT));
    renderer
        .render(&submission, &target)
        .expect("cpu render succeeds");
    renderer.pixmap().expect("a rasterized frame").clone()
}

#[doc = include_str!("caret_scroll_proof.doc/side_by_side.md")]
fn side_by_side(left: &tiny_skia::Pixmap, right: &tiny_skia::Pixmap) -> tiny_skia::Pixmap {
    let w = left.width() + right.width();
    let h = left.height().max(right.height());
    let mut out = tiny_skia::Pixmap::new(w, h).expect("allocate composite pixmap");
    out.draw_pixmap(
        0,
        0,
        left.as_ref(),
        &tiny_skia::PixmapPaint::default(),
        tiny_skia::Transform::identity(),
        None,
    );
    out.draw_pixmap(
        left.width() as i32,
        0,
        right.as_ref(),
        &tiny_skia::PixmapPaint::default(),
        tiny_skia::Transform::identity(),
        None,
    );
    out
}

#[doc = include_str!("caret_scroll_proof.doc/compile.md")]
fn compile(css: &str, origin: CascadeOrigin) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new()
        .with_origin(origin)
        .compile(&parsed)
        .matchable
}

#[doc = include_str!("caret_scroll_proof.doc/char_press.md")]
fn char_press(c: char) -> KeyEvent {
    KeyEvent::press(PhysicalKey::KeyA, Some(LogicalKey::character(c)))
}

#[doc = include_str!("caret_scroll_proof.doc/named_press.md")]
fn named_press(named: NamedKey) -> KeyEvent {
    KeyEvent::press(PhysicalKey::KeyA, Some(LogicalKey::named(named)))
}

fn mouse(kind: PointerEventKind, x: f32, y: f32) -> PointerEvent {
    PointerEvent::new(
        PointerId::MOUSE,
        PointerDevice::Mouse,
        kind,
        Point::new(x, y),
    )
}

#[doc = include_str!("caret_scroll_proof.doc/output_path.md")]
fn output_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join("target")
        .join(name)
}

#[doc = include_str!("caret_scroll_proof.doc/wire_text_stack.md")]
fn wire_text_stack(runtime: &mut UiRuntime, renderer: &mut CpuRenderer, bytes: Vec<u8>) {
    let mut registry = FontRegistry::new();
    let id = registry.register(
        FontFamily::generic(GenericFamily::SansSerif),
        FontAttributes::default(),
        bytes.clone(),
    );
    renderer.register_font(font_handle(id), bytes);
    // The shaper answers line-metric queries by `FontId` alone, so it needs the
    // registry's own per-face metrics; without them the glyphs would be real but
    // every line box sized from idealized ratios.
    let shaper = HarfrustShaper::without_font().with_faces(registry.face_metrics());
    runtime.set_font_resolver(Box::new(registry));
    runtime.set_text_itemizer(Box::new(UnicodeTextItemizer::new()));
    runtime.set_text_shaper(Box::new(shaper));
}

#[doc = include_str!("caret_scroll_proof.doc/font_handle.md")]
fn font_handle(id: FontId) -> u32 {
    u32::try_from(id.index()).unwrap_or(0)
}

#[doc = include_str!("caret_scroll_proof.doc/load_system_font.md")]
fn load_system_font() -> Option<Vec<u8>> {
    const CANDIDATES: &[&str] = &[
        "/usr/share/fonts/TTF/DejaVuSans.ttf",
        "/usr/share/fonts/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/noto/NotoSans-Regular.ttf",
        "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
        "/usr/share/fonts/liberation/LiberationSans-Regular.ttf",
        "/System/Library/Fonts/Helvetica.ttc",
        "C:\\Windows\\Fonts\\arial.ttf",
    ];
    CANDIDATES.iter().find_map(|p| std::fs::read(p).ok())
}
