//! **Proof: multi-line auto-wrapping text + `line-height` in the coordinator.**
//!
//! Renders three text blocks through the REAL runtime + software [`CpuRenderer`]
//! (no GPU), stitched top-to-bottom into `target/multiline_text_proof.png`:
//!
//! - **Block A — a long paragraph in a ~200px box.** The text auto-wraps into
//!   several visual lines at the box's content width; the lines stack by the
//!   font's natural (metric) line height so they do not overlap.
//! - **Block B — an explicit `\n` two-line string** in a wide box: even though
//!   nothing wraps, the mandatory newline forces two lines.
//! - **Block C — the same long paragraph with `line-height: 2`** (inherited from
//!   the container): it wraps to the same width but its lines are spaced twice as
//!   far apart (2 · font-size), proving the CSS `line-height` property feeds the
//!   line box height.
//!
//! Before writing the PNG it machine-checks the composed display list: Block A
//! emits ≥ 2 `DrawText` runs at strictly increasing y (wrapped, non-overlapping);
//! Block B emits exactly 2 runs (`\n` break); Block C's per-line spacing equals
//! `2 · font-size` and is strictly wider than Block A's metric spacing.
//!
//! Run it:
//! `cargo run -p obscur-kit --example multiline_text_proof \
//!   --features "render-cpu text-harfrust text-unicode"`
#![forbid(unsafe_code)]

use std::path::PathBuf;

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::Size;
use obscur_kit::identity::{ComponentId, RuntimeId};
use obscur_kit::render::frame::{FrameSubmission, RenderFrame};
use obscur_kit::render::target::{BitmapTarget, RenderTarget};
use obscur_kit::render::{Renderer, RendererId};
use obscur_kit::render_cpu::renderer::CpuRenderer;
use obscur_kit::runtime::UiRuntime;
use obscur_kit::scene::command::PaintCommand;
use obscur_kit::style::cascade::CascadeOrigin;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::text::font::{FontAttributes, FontFamily, FontId, GenericFamily};
use obscur_kit::text_harfrust::{FontRegistry, HarfrustShaper};
use obscur_kit::text_unicode::UnicodeTextItemizer;
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

/// The logical extent each block is laid out and rasterized under.
const VIEWPORT: Size = Size::new(340.0, 150.0);

/// The font size the blocks author (px). `line-height: 2` therefore spaces C's
/// lines by exactly `2 · FONT_SIZE`.
const FONT_SIZE: f32 = 15.0;

/// A long paragraph that overflows a ~200px box and must wrap.
const PARAGRAPH: &str =
    "The quick brown fox jumps over the lazy dog while the sun sets behind the hills.";

fn main() {
    println!("== obscur-kit multi-line wrapping + line-height proof (headless CPU) ==\n");

    // Shared harness: a white flex page. Each block sets its own width + colours.
    let base = "\
.page { display: flex; flex-direction: column; padding: 12px; background: #ffffff; }";

    // Block A: a ~200px box; the long paragraph wraps at the content width and its
    // lines stack by the font's natural (metric) line height.
    let (a_px, a_ys) = render_block(
        &format!("{base}\n.block {{ width: 200px; padding: 6px; background: #eef2ff; color: #12203f; font-size: {FONT_SIZE}px; }}"),
        PARAGRAPH,
    );
    // Block B: a wide box; the `\n` forces two lines with no wrapping. Under the
    // initial `white-space: normal` a segment break collapses to a space (CSS Text
    // 3 §4.1.1), so this fixture opts into `pre-line` to PRESERVE the newline as a
    // forced break — the behaviour this block proves.
    let (b_px, b_ys) = render_block(
        &format!("{base}\n.block {{ width: 300px; padding: 6px; background: #fff3e6; color: #3f2410; font-size: {FONT_SIZE}px; white-space: pre-line; }}"),
        "First line before the newline\nSecond line after the newline",
    );
    // Block C: the same ~200px box, but `line-height: 2` (inherited to the text)
    // spaces the wrapped lines twice as far apart.
    let (c_px, c_ys) = render_block(
        &format!("{base}\n.block {{ width: 200px; padding: 6px; background: #e9f7ee; color: #103018; font-size: {FONT_SIZE}px; line-height: 2; }}"),
        PARAGRAPH,
    );

    verify(&a_ys, &b_ys, &c_ys);

    let combined = stack(&[a_px, b_px, c_px]);
    let png = combined.encode_png().expect("encode png");
    let out = output_path("multiline_text_proof.png");
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).expect("create target dir");
    }
    std::fs::write(&out, &png).expect("write png");
    println!(
        "\n[png] wrote {} × {} → {} ({} bytes) — A wraps, B breaks on \\n, C uses line-height:2",
        combined.width(),
        combined.height(),
        out.display(),
        png.len(),
    );
}

/// Renders one text `block` (a `.block` container holding a single text node)
/// through the real runtime and returns its rasterized pixmap plus the DISTINCT,
/// ascending y-origins of its `DrawText` runs (one per wrapped/visual line).
fn render_block(css: &str, content: &str) -> (tiny_skia::Pixmap, Vec<f32>) {
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), VIEWPORT);
    let mut renderer = CpuRenderer::new(RendererId::from_raw(2));
    if let Some(bytes) = load_system_font() {
        wire_text_stack(&mut runtime, &mut renderer, bytes);
    }

    let author = compile(css, CascadeOrigin::GlobalApp);
    let view = block_view(content);
    let result = runtime.render_now(&view, std::slice::from_ref(&author));
    let snapshot = result.snapshot().clone();

    // Collect the DISTINCT y-origins of the DrawText runs, ascending. A multi-font
    // line would emit several runs at the SAME y; deduping (to the nearest 0.1px)
    // leaves one y per visual line.
    let mut ys: Vec<f32> = snapshot
        .display_list()
        .commands()
        .iter()
        .filter_map(|c| match c {
            PaintCommand::DrawText { origin, .. } => Some(origin.y),
            _ => None,
        })
        .collect();
    ys.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    ys.dedup_by(|a, b| (*a - *b).abs() < 0.1);
    println!(
        "[block] {} DrawText line(s) at y = {:?}",
        ys.len(),
        ys.iter().map(|y| (y * 10.0).round() / 10.0).collect::<Vec<_>>()
    );

    let frame = RenderFrame::new(snapshot, VIEWPORT);
    let submission = FrameSubmission::new(frame);
    let target = RenderTarget::bitmap(BitmapTarget::new(VIEWPORT));
    renderer.render(&submission, &target).expect("cpu render succeeds");
    (renderer.pixmap().expect("a rasterized frame").clone(), ys)
}

/// Machine-checks the three blocks' per-line y-origins.
fn verify(a: &[f32], b: &[f32], c: &[f32]) {
    // Block A: the long paragraph WRAPPED into ≥ 2 visual lines, strictly stacked.
    assert!(a.len() >= 2, "block A must wrap into ≥ 2 lines, got {}", a.len());
    assert!(strictly_increasing(a), "block A lines must not overlap: {a:?}");
    let a_gap = a[1] - a[0];
    assert!(a_gap > 0.0, "block A line spacing must be positive");

    // Block B: exactly two lines from the explicit `\n`.
    assert_eq!(b.len(), 2, "block B (`\\n`) must be two lines, got {}", b.len());
    assert!(b[1] > b[0], "block B's second line must sit below the first");

    // Block C: the SAME paragraph wrapped, but spaced by line-height:2 = 2·15 = 30px.
    assert!(c.len() >= 2, "block C must wrap into ≥ 2 lines, got {}", c.len());
    assert!(strictly_increasing(c), "block C lines must not overlap: {c:?}");
    let c_gap = c[1] - c[0];
    let expected = 2.0 * FONT_SIZE;
    assert!(
        (c_gap - expected).abs() < 0.5,
        "block C line spacing should be 2·font-size ({expected}px), got {c_gap}"
    );
    // The line-height:2 spacing is strictly wider than A's natural metric spacing.
    assert!(
        c_gap > a_gap + 1.0,
        "line-height:2 spacing ({c_gap}) must exceed the default metric spacing ({a_gap})"
    );

    println!(
        "\n[verify] A wraps to {} lines (gap {:.1}px); B is {} lines (\\n); C wraps to {} lines (gap {:.1}px ≈ 2·{FONT_SIZE})",
        a.len(),
        a_gap,
        b.len(),
        c.len(),
        c_gap,
    );
    println!("[verify] OK — text wraps at the box width, stacks without overlap, and line-height widens the spacing");
}

/// Whether `ys` is strictly increasing (each line strictly below the previous).
fn strictly_increasing(ys: &[f32]) -> bool {
    ys.windows(2).all(|w| w[1] > w[0])
}

/// The view: a single `.block` element wrapping one text node.
fn block_view(content: &str) -> View {
    let block = ViewNode::element(ComponentId::from_raw(1))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("block".into())))
        .with_children(ViewFragment::from_nodes(vec![ViewNode::text(content)]));
    let page = ViewNode::element(ComponentId::from_raw(2))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("page".into())))
        .with_children(ViewFragment::from_nodes(vec![block]));
    View::single(page)
}

/// Vertically stacks `pixmaps` into one image with thin gray dividers.
fn stack(pixmaps: &[tiny_skia::Pixmap]) -> tiny_skia::Pixmap {
    let gap = 2u32;
    let width = pixmaps.iter().map(tiny_skia::Pixmap::width).max().unwrap_or(1);
    let height: u32 = pixmaps.iter().map(|p| p.height() + gap).sum::<u32>().saturating_sub(gap);
    let mut out = tiny_skia::Pixmap::new(width, height.max(1)).expect("combined pixmap");
    out.fill(tiny_skia::Color::from_rgba8(0xd0, 0xcd, 0xc7, 0xff));
    let paint = tiny_skia::PixmapPaint::default();
    let mut y = 0i32;
    for px in pixmaps {
        out.draw_pixmap(0, y, px.as_ref(), &paint, tiny_skia::Transform::identity(), None);
        y += (px.height() + gap) as i32;
    }
    out
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
    runtime.set_font_resolver(Box::new(registry));
    runtime.set_text_itemizer(Box::new(UnicodeTextItemizer::new()));
    runtime.set_text_shaper(Box::new(HarfrustShaper::without_font()));
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
