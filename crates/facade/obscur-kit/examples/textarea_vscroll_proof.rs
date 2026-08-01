#![doc = include_str!("textarea_vscroll_proof.doc/_module.md")]
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
    CLASS, EVENT_INPUT, TYPE, View, ViewEventBinding, ViewFragment, ViewNode, ViewProperty,
    ViewValue,
};

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;

#[doc = include_str!("textarea_vscroll_proof.doc/VIEWPORT.md")]
const VIEWPORT: Size = Size::new(280.0, 200.0);

#[doc = include_str!("textarea_vscroll_proof.doc/CMD.md")]
const CMD: InternedSymbol = InternedSymbol::from_raw(0x0001);

#[doc = include_str!("textarea_vscroll_proof.doc/CSS.md")]
const CSS: &str = "\
.stage { padding: 40px; background: #12141c; }\
.field { width: 200px; height: 80px; background: #ffffff; color: #101216; \
white-space: pre-wrap; overflow: auto; line-height: 20px; \
border: 1px solid #5b6472; }";

#[doc = include_str!("textarea_vscroll_proof.doc/LINES.md")]
const LINES: &[&str] = &[
    "line one",
    "line two",
    "line three",
    "line four",
    "line five",
    "line six",
    "line seven",
    "line eight",
];

#[doc = include_str!("textarea_vscroll_proof.doc/SCROLLBAR_WIDTH.md")]
const SCROLLBAR_WIDTH: f32 = 8.0;

fn main() {
    println!("== obscur-kit textarea VERTICAL caret-scroll proof (headless CPU) ==\n");

    let author = compile(CSS, CascadeOrigin::GlobalApp);

    // --- Drive the runtime: focus the textarea, type more lines than fit.
    let (mut runtime, mut renderer) = boot();
    runtime.set_default_stylesheet(author.clone());
    let sheets = std::slice::from_ref(&author);

    let _ = runtime.render_now(&field_view(""), sheets);
    // Focus the textarea by pressing near its top-left.
    runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 50.0));
    assert!(
        runtime.focused().is_some(),
        "the press focused the textarea"
    );

    for (i, line) in LINES.iter().enumerate() {
        for c in line.chars() {
            runtime.handle_key(char_press(c));
        }
        if i + 1 < LINES.len() {
            runtime.handle_key(named_press(NamedKey::Enter));
        }
    }
    println!(
        "[type] entered {} lines (caret ends on the last)",
        LINES.len()
    );

    // The scrolled frame (the box laid out at its grown extent + scroll offset).
    let snapshot = runtime
        .render_now(&field_view(""), sheets)
        .snapshot()
        .clone();
    let field = field_node(&runtime);
    let field_box = runtime
        .layout_boxes()
        .get(&field)
        .expect("field box")
        .bounds;
    let box_top = field_box.origin.y;
    let box_bottom = field_box.origin.y + field_box.size.height;
    let box_right = field_box.origin.x + field_box.size.width;

    // 1. The overflow container scrolled: the caret at the bottom pushed scroll_top
    //    past 0 (the SAME value the wheel + scrollbar drive).
    let scroll_top = runtime.scroll_top(field);
    println!(
        "[scroll] scroll_top = {:.1}  (box y∈[{:.1}, {:.1}])",
        scroll_top, box_top, box_bottom,
    );
    assert!(
        scroll_top > 0.0,
        "typing past the box scrolled it (scroll_top {scroll_top})"
    );

    // 2. The painted caret's line, offset by the scroll transform, is INSIDE the box.
    let (caret_top, caret_bottom) = caret_y_of(&snapshot).expect("a caret is painted");
    let eff_top = caret_top - scroll_top;
    let eff_bottom = caret_bottom - scroll_top;
    println!(
        "[caret] unscrolled y∈[{:.1}, {:.1}]  →  effective y∈[{:.1}, {:.1}]",
        caret_top, caret_bottom, eff_top, eff_bottom,
    );
    assert!(
        eff_top >= box_top - 0.5 && eff_bottom <= box_bottom + 0.5,
        "the caret's line is visible: effective y∈[{eff_top:.1}, {eff_bottom:.1}] within \
         the box [{box_top:.1}, {box_bottom:.1}]",
    );
    println!("  ✓ the caret's line is inside the box (you see the line you type on)");

    // 3. The TOP lines are scrolled out above the box and clipped: the topmost text
    //    run's unscrolled y, minus scroll_top, sits above the box top.
    let top_run_y = min_text_run_y(&snapshot).expect("a text run");
    let eff_top_run = top_run_y - scroll_top;
    println!("[clip] topmost run effective y = {eff_top_run:.1}  (box top {box_top:.1})");
    assert!(
        eff_top_run < box_top - 0.5,
        "the first lines are scrolled out above the box (effective y {eff_top_run:.1} < \
         box top {box_top:.1})",
    );
    println!("  ✓ the top lines are scrolled out + clipped");

    // 4. A scrollbar thumb (an 8-px-wide fill at the box right edge) paints.
    let has_thumb = has_scrollbar_thumb(&snapshot, box_right);
    assert!(
        has_thumb,
        "a scrollbar thumb paints at the box's right edge"
    );
    println!(
        "  ✓ a scrollbar thumb paints at x ≈ {:.1}\n",
        box_right - SCROLLBAR_WIDTH
    );

    // --- Render the scrolled state into a PNG.
    let pixmap = rasterize(&mut renderer, &snapshot);
    let png = pixmap.encode_png().expect("encode png");
    let out = output_path("textarea_vscroll_proof.png");
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).expect("create target dir");
    }
    std::fs::write(&out, &png).expect("write png");
    println!(
        "[png] wrote {} × {} → {} ({} bytes) — a scrolled textarea; caret line in view, \
         top lines clipped, scrollbar thumb",
        pixmap.width(),
        pixmap.height(),
        out.display(),
        png.len(),
    );
}

#[doc = include_str!("textarea_vscroll_proof.doc/field_view.md")]
fn field_view(text: &str) -> View {
    View::from_fragment(ViewFragment::from_nodes(vec![
        ViewNode::element(ComponentId::from_raw(1))
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text("stage".into())))
            .with_children(ViewFragment::from_nodes(vec![
                ViewNode::element(ComponentId::from_raw(2))
                    .with_prop(ViewProperty::new(CLASS, ViewValue::Text("field".into())))
                    .with_prop(ViewProperty::new(TYPE, ViewValue::from("textarea")))
                    .with_event(ViewEventBinding::new(EVENT_INPUT, CMD))
                    .with_children(ViewFragment::from_nodes(vec![ViewNode::text(
                        text.to_string(),
                    )])),
            ])),
    ]))
}

#[doc = include_str!("textarea_vscroll_proof.doc/field_node.md")]
fn field_node(runtime: &UiRuntime) -> NodeId {
    let root = runtime.tree().root().expect("root after a frame");
    let stage = runtime.tree().get(root).unwrap().child_nodes()[0];
    runtime.tree().get(stage).unwrap().child_nodes()[0]
}

#[doc = include_str!("textarea_vscroll_proof.doc/caret_y_of.md")]
fn caret_y_of(snapshot: &SceneSnapshot) -> Option<(f32, f32)> {
    snapshot
        .display_list()
        .commands()
        .iter()
        .find_map(|c| match c {
            PaintCommand::DrawPath { path, .. } => path
                .bounds()
                .filter(|r| (r.size.width - 1.0).abs() < 1e-3 && (r.size.height - 20.0).abs() < 4.0)
                .map(|r| (r.origin.y, r.origin.y + r.size.height)),
            _ => None,
        })
}

#[doc = include_str!("textarea_vscroll_proof.doc/min_text_run_y.md")]
fn min_text_run_y(snapshot: &SceneSnapshot) -> Option<f32> {
    snapshot
        .display_list()
        .commands()
        .iter()
        .filter_map(|c| match c {
            PaintCommand::DrawText { origin, .. } => Some(origin.y),
            _ => None,
        })
        .fold(None, |acc, y| Some(acc.map_or(y, |a: f32| a.min(y))))
}

#[doc = include_str!("textarea_vscroll_proof.doc/has_scrollbar_thumb.md")]
fn has_scrollbar_thumb(snapshot: &SceneSnapshot, box_right: f32) -> bool {
    snapshot.display_list().commands().iter().any(|c| match c {
        PaintCommand::DrawPath { path, .. } => path.bounds().is_some_and(|r| {
            (r.size.width - SCROLLBAR_WIDTH).abs() < 0.5
                && (r.origin.x + r.size.width - box_right).abs() < 1.0
        }),
        _ => false,
    })
}

#[doc = include_str!("textarea_vscroll_proof.doc/boot.md")]
fn boot() -> (UiRuntime, CpuRenderer) {
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), VIEWPORT);
    runtime.on_text_input(CMD, |_| {});
    let mut renderer = CpuRenderer::new(RendererId::from_raw(2));
    if let Some(bytes) = load_system_font() {
        wire_text_stack(&mut runtime, &mut renderer, bytes);
    }
    (runtime, renderer)
}

#[doc = include_str!("textarea_vscroll_proof.doc/rasterize.md")]
fn rasterize(renderer: &mut CpuRenderer, snapshot: &SceneSnapshot) -> tiny_skia::Pixmap {
    let frame = RenderFrame::new(snapshot.clone(), VIEWPORT);
    let submission = FrameSubmission::new(frame);
    let target = RenderTarget::bitmap(BitmapTarget::new(VIEWPORT));
    renderer
        .render(&submission, &target)
        .expect("cpu render succeeds");
    renderer.pixmap().expect("a rasterized frame").clone()
}

#[doc = include_str!("textarea_vscroll_proof.doc/compile.md")]
fn compile(css: &str, origin: CascadeOrigin) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new()
        .with_origin(origin)
        .compile(&parsed)
        .matchable
}

#[doc = include_str!("textarea_vscroll_proof.doc/char_press.md")]
fn char_press(c: char) -> KeyEvent {
    KeyEvent::press(PhysicalKey::KeyA, Some(LogicalKey::character(c)))
}

#[doc = include_str!("textarea_vscroll_proof.doc/named_press.md")]
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

#[doc = include_str!("textarea_vscroll_proof.doc/output_path.md")]
fn output_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join("target")
        .join(name)
}

#[doc = include_str!("textarea_vscroll_proof.doc/wire_text_stack.md")]
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

#[doc = include_str!("textarea_vscroll_proof.doc/font_handle.md")]
fn font_handle(id: FontId) -> u32 {
    u32::try_from(id.index()).unwrap_or(0)
}

#[doc = include_str!("textarea_vscroll_proof.doc/load_system_font.md")]
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
