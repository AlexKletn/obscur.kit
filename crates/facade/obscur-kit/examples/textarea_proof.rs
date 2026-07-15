//! **Proof: a multi-line `<textarea>` edits, wraps, and paints a LINE-AWARE caret**
//! (spec §5.10/§5.11; CSS Text 3 §4.1.1 segment breaks).
//!
//! Drives a focused **multi-line** field (`type=textarea`, `white-space: pre-wrap`)
//! through the real runtime editing path: it types `"line one\nline two\nline
//! three"` — the two `\n`s entered as real **Enter** key presses, which the editing
//! keymap inserts as segment breaks (a single-line `input` would submit instead) —
//! renders a CPU `target/textarea_proof.png`, and asserts:
//!
//! - the value laid out onto **three** lines (one `DrawText` run per wrapped line),
//!   each exactly `line-height` (22px) tall;
//! - the caret (at the end of the text) sits on the **third** line — its painted
//!   `y ≈ 2 × line-height` — at an `x` measured **within line 3** (the width of
//!   `"line three"` alone), NOT the summed width of all three lines (the single-line
//!   caret bug this replaces).
//!
//! Run it:
//! `cargo run -p obscur-kit --example textarea_proof \
//!   --features "render-cpu text-harfrust text-unicode"`
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
use obscur_kit::scene::text::PositionedGlyph;
use obscur_kit::style::cascade::CascadeOrigin;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::text::font::{FontAttributes, FontFamily, FontId, GenericFamily};
use obscur_kit::text_harfrust::{FontRegistry, HarfrustShaper};
use obscur_kit::text_unicode::UnicodeTextItemizer;
use obscur_kit::view::view::{
    View, ViewEventBinding, ViewFragment, ViewNode, ViewProperty, ViewValue, CLASS, EVENT_INPUT,
    TYPE,
};

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;

/// The logical extent the frame is laid out and rasterized under.
const VIEWPORT: Size = Size::new(360.0, 200.0);

/// The `on-input` command the field binds (registered, never mirrored here).
const CMD: InternedSymbol = InternedSymbol::from_raw(0x0001);

/// The exact authored `line-height`, in logical pixels — so each wrapped line is a
/// deterministic 22px tall and line 3 sits at `y = 2 × 22`.
const LINE_HEIGHT: f32 = 22.0;

/// A multi-line field: 240px wide (each short line fits, so wrapping is driven only
/// by the typed `\n`s), `white-space: pre-wrap` (preserves segment breaks AND wraps),
/// with an exact 22px line-height. The `.stage` frames it.
const CSS: &str = "\
.stage { padding: 24px 20px; background: #12141c; }\
.field { width: 240px; height: 88px; background: #ffffff; color: #101216; \
white-space: pre-wrap; line-height: 22px; overflow: hidden; \
border: 1px solid #5b6472; padding: 6px; }";

/// The value typed into the textarea — three lines separated by real newlines,
/// each entered through the runtime editing path (`\n` = an Enter key press).
const VALUE: &str = "line one\nline two\nline three";

fn main() {
    println!("== obscur-kit multi-line textarea + line-aware caret proof (headless CPU) ==\n");

    let author = compile(CSS, CascadeOrigin::GlobalApp);

    let (mut runtime, mut renderer) = boot();
    runtime.set_default_stylesheet(author.clone());
    let sheets = std::slice::from_ref(&author);

    let _ = runtime.render_now(&field_view(""), sheets);
    // Focus the field by pressing inside it (near its top-left content).
    runtime.handle_pointer(mouse(PointerEventKind::Down, 40.0, 40.0));
    assert!(runtime.focused().is_some(), "the press focused the textarea");

    // Type the value through the REAL key path: printable chars as character keys,
    // each `\n` as an Enter press (a textarea inserts a newline; an input would not).
    for ch in VALUE.chars() {
        if ch == '\n' {
            runtime.handle_key(named_press(NamedKey::Enter));
        } else {
            runtime.handle_key(char_press(ch));
        }
    }
    assert_eq!(
        runtime.editing_text(),
        Some(VALUE),
        "Enter inserted real newlines: the buffer holds all three lines",
    );
    println!("[type] entered {VALUE:?} via the editing keymap (2 Enter → 2 newlines)");

    // Render the multi-line frame (measured from the session), holding the caret at
    // the end of "line three" (line 3, 0-based index 2).
    let snapshot = runtime.render_now(&field_view(""), sheets).snapshot().clone();
    let text_node = text_leaf(&runtime);
    let text_origin = runtime.layout_boxes().get(&text_node).expect("text box").bounds.origin;

    // One DrawText run per wrapped line: collect them top-to-bottom.
    let mut lines = text_runs(&snapshot);
    lines.sort_by(|a, b| a.0.y.partial_cmp(&b.0.y).unwrap_or(std::cmp::Ordering::Equal));
    let line_count = lines.len();
    println!("[layout] painted {line_count} wrapped line(s):");
    for (i, (origin, width)) in lines.iter().enumerate() {
        println!(
            "  line {}: rel y = {:.1}  left = {:.1}  width = {:.1}",
            i + 1,
            origin.y - text_origin.y,
            origin.x,
            width,
        );
    }
    assert_eq!(line_count, 3, "the value laid out onto exactly three lines");

    // The third line's own left edge + intrinsic width — the caret at the end of the
    // text must sit at ~this right edge, NOT at the summed width of all three lines.
    let (line3_origin, line3_width) = lines[2];
    let summed_width: f32 = lines.iter().map(|(_, w)| *w).sum();

    let caret = caret_rect_of(&snapshot).expect("a collapsed caret is painted");
    let caret_rel_y = caret.origin.y - text_origin.y;
    let caret_rel_x = caret.origin.x - text_origin.x;
    println!(
        "\n[caret] painted at rel (x, y) = ({caret_rel_x:.1}, {caret_rel_y:.1})  \
         [line-3 width = {line3_width:.1}, summed 3-line width = {summed_width:.1}]",
    );

    // (1) The caret is on the THIRD line: y ≈ 2 × line-height.
    assert!(
        (caret_rel_y - 2.0 * LINE_HEIGHT).abs() < 1e-2,
        "caret y {caret_rel_y} is on line 3 (≈ 2 × line-height {})",
        2.0 * LINE_HEIGHT,
    );
    // (2) The caret x is measured WITHIN line 3 — at line 3's own right edge — NOT
    //     the summed width of all three lines (the single-line caret bug).
    let expected_x = line3_origin.x - text_origin.x + line3_width;
    assert!(
        (caret_rel_x - expected_x).abs() < 1.0,
        "caret x {caret_rel_x} is line-3-local (line-3 right edge {expected_x:.1})",
    );
    assert!(
        caret_rel_x < summed_width - line3_width,
        "caret x {caret_rel_x} is NOT the summed 3-line width {summed_width:.1} \
         (the flattened single-line bug)",
    );
    println!("  ✓ caret sits on line 3 at its own line-local x — line-aware geometry holds\n");

    // Rasterize the proof PNG.
    let pixmap = rasterize(&mut renderer, &snapshot);
    let png = pixmap.encode_png().expect("encode png");
    let out = output_path("textarea_proof.png");
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).expect("create target dir");
    }
    std::fs::write(&out, &png).expect("write png");
    println!(
        "[png] wrote {} × {} → {} ({} bytes) — textarea: 3 typed lines, caret on line 3 \
         at rel (x, y) = ({caret_rel_x:.1}, {caret_rel_y:.1})",
        pixmap.width(),
        pixmap.height(),
        out.display(),
        png.len(),
    );
    println!("\n== all textarea assertions held; exiting 0 ==");
}

/// A `.stage` holding one editable MULTI-LINE `.field` (`type=textarea`) carrying
/// `text` and binding on-input.
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

/// The `field`'s text-leaf node id (root → stage → field → text).
fn text_leaf(runtime: &UiRuntime) -> NodeId {
    let root = runtime.tree().root().expect("root after a frame");
    let stage = runtime.tree().get(root).unwrap().children()[0];
    let field = runtime.tree().get(stage).unwrap().children()[0];
    runtime.tree().get(field).unwrap().children()[0]
}

/// The `(origin, advance-width)` of every painted `DrawText` run — one per wrapped
/// line — in paint order.
fn text_runs(snapshot: &SceneSnapshot) -> Vec<(Point, f32)> {
    snapshot
        .display_list()
        .commands()
        .iter()
        .filter_map(|c| match c {
            PaintCommand::DrawText { origin, run, .. } => {
                let width: f32 = run.glyphs().iter().map(PositionedGlyph::advance).sum();
                Some((*origin, width))
            }
            _ => None,
        })
        .collect()
}

/// The painted caret rect: the thin (width-1) fill whose height is one line box
/// (`line-height`), so it is distinguished from the field's 1px-wide, full-height
/// border edges (which are also width-1 fills).
fn caret_rect_of(snapshot: &SceneSnapshot) -> Option<obscur_kit::geometry::Rect> {
    snapshot.display_list().commands().iter().find_map(|c| match c {
        PaintCommand::DrawPath { path, .. } => path.bounds().filter(|r| {
            (r.size.width - 1.0).abs() < 1e-3 && (r.size.height - LINE_HEIGHT).abs() < 1.0
        }),
        _ => None,
    })
}

/// Boots a runtime + CPU renderer with the text stack wired (best-effort font).
fn boot() -> (UiRuntime, CpuRenderer) {
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), VIEWPORT);
    runtime.on_text_input(CMD, |_| {});
    let mut renderer = CpuRenderer::new(RendererId::from_raw(2));
    if let Some(bytes) = load_system_font() {
        wire_text_stack(&mut runtime, &mut renderer, bytes);
    }
    (runtime, renderer)
}

/// Rasterizes a snapshot through the CPU renderer and clones the pixmap.
fn rasterize(renderer: &mut CpuRenderer, snapshot: &SceneSnapshot) -> tiny_skia::Pixmap {
    let frame = RenderFrame::new(snapshot.clone(), VIEWPORT);
    let submission = FrameSubmission::new(frame);
    let target = RenderTarget::bitmap(BitmapTarget::new(VIEWPORT));
    renderer.render(&submission, &target).expect("cpu render succeeds");
    renderer.pixmap().expect("a rasterized frame").clone()
}

/// Compiles CSS text into a runtime-matchable [`Stylesheet`] at `origin`.
fn compile(css: &str, origin: CascadeOrigin) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new()
        .with_origin(origin)
        .compile(&parsed)
        .matchable
}

/// A plain (no-modifier) character key press.
fn char_press(c: char) -> KeyEvent {
    KeyEvent::press(PhysicalKey::KeyA, Some(LogicalKey::character(c)))
}

/// A named (non-character) key press.
fn named_press(named: NamedKey) -> KeyEvent {
    KeyEvent::press(PhysicalKey::KeyA, Some(LogicalKey::named(named)))
}

fn mouse(kind: PointerEventKind, x: f32, y: f32) -> PointerEvent {
    PointerEvent::new(PointerId::MOUSE, PointerDevice::Mouse, kind, Point::new(x, y))
}

/// The workspace-relative output path `target/<name>`.
fn output_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join("target")
        .join(name)
}

/// Wires the text stack and registers `bytes` under `sans-serif` on both the
/// resolver and the CPU renderer so glyph handles line up.
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

/// The scene font handle (`u32`) for a resolved [`FontId`].
fn font_handle(id: FontId) -> u32 {
    u32::try_from(id.index()).unwrap_or(0)
}

/// Loads a system font's raw bytes, scanning well-known locations (best effort).
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
