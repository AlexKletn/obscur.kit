//! **Proof: CSS Text 3 `white-space` processing + `tab-size` in the coordinator.**
//!
//! Renders text blocks through the REAL runtime + software [`CpuRenderer`] (no
//! GPU), stitched top-to-bottom into `target/white_space_proof.png`, proving the
//! four behaviours of the white-space model (CSS Text 3 §3, §4.1.1, §4.3):
//!
//! - **A — `white-space: normal`** on `"a    b\nc"` (four spaces + a newline) in a
//!   wide box: the run of spaces collapses to ONE space and the segment break
//!   collapses to a space, so it is a single line of width `W_normal`.
//! - **B — `white-space: pre`** on the SAME `"a    b\nc"`: the four spaces are
//!   PRESERVED (so the first line is wider than A's collapsed one) and the `\n`
//!   forces a SECOND line.
//! - **C — `white-space: nowrap`** on a long phrase in a NARROW box: it never
//!   soft-wraps, staying on ONE line whose width exceeds the box (overflow).
//! - **D — `white-space: pre; tab-size: 4`** on `"a\tb"`: the preserved tab
//!   advances the pen to the next tab stop (a multiple of `4 × space-advance`),
//!   proven against a plain single-space `"a b"` (narrower) and a `tab-size: 8`
//!   render (wider), with the tab landing exactly on the stop.
//!
//! Before writing the PNG it machine-checks the composed display list:
//! `W_normal < W_pre` (collapsing is narrower than preserving); B is two lines;
//! C is one line wider than its box; and D's pen after the tab lands on
//! `4 × space-advance`, with `tab-size: 8` landing on `8 × space-advance`.
//!
//! Run it:
//! `cargo run -p obscur-kit --example white_space_proof \
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
const VIEWPORT: Size = Size::new(360.0, 90.0);

/// One wrapped visual line: its y-origin and total width (summed glyph advances
/// of every `DrawText` run sharing that y), plus the per-glyph advances of the
/// first run on the line (enough to inspect a tab's advance).
struct Line {
    y: f32,
    width: f32,
    advances: Vec<f32>,
}

fn main() {
    println!("== obscur-kit white-space processing + tab-size proof (headless CPU) ==\n");

    let base = "\
.page { display: flex; flex-direction: column; padding: 10px; background: #ffffff; }
.block { padding: 4px; font-size: 18px; color: #101828; background: #eef2ff; }";

    // A: `normal` collapses the 4-space run AND the `\n` to a single space → one
    // line. B: `pre` preserves the spaces (wider first line) and the `\n` (2 lines).
    let (a_px, a_lines) = render_block(&css(base, "normal", "white-space: normal;"), "a    b\nc");
    let (b_px, b_lines) = render_block(&css(base, "pre", "white-space: pre;"), "a    b\nc");
    // Clean width comparison on the SAME `"a    b"` (no `\n`): `normal` collapses
    // the 4 spaces to one (narrow), `pre` preserves all four (wide).
    let (_, aw) = render_block(&css(base, "normal_w", "white-space: normal;"), "a    b");
    let (_, bw) = render_block(&css(base, "pre_w", "white-space: pre;"), "a    b");
    // C: `nowrap` keeps a long phrase on ONE line in a narrow box (it overflows,
    // and the initial `text-overflow: clip` trims the painted glyphs to the box) —
    // whereas the SAME phrase under `normal` in the SAME box WRAPS to many lines.
    const PHRASE: &str = "keep this whole phrase on one single line";
    let (c_px, c_lines) = render_block(
        &css(base, "nowrap", "white-space: nowrap; width: 40px;"),
        PHRASE,
    );
    let (_, c_normal) = render_block(
        &css(base, "normal_wrap", "white-space: normal; width: 40px;"),
        PHRASE,
    );
    // D: `pre` + tab-size, three renders to prove the tab lands on the stop.
    let (d_px, d_lines) = render_block(
        &css(base, "pre_tab4", "white-space: pre; tab-size: 4;"),
        "a\tb",
    );
    let (_, d_tab8) = render_block(
        &css(base, "pre_tab8", "white-space: pre; tab-size: 8;"),
        "a\tb",
    );
    let (_, d_space) = render_block(&css(base, "pre_space", "white-space: pre;"), "a b");
    // The advance of one space (its own line under `pre`): the tab-stop unit.
    let (_, sp_line) = render_block(&css(base, "space", "white-space: pre;"), " ");
    let space_adv = sp_line[0].width;

    verify(
        &a_lines, &b_lines, &aw, &bw, &c_lines, &c_normal, &d_lines, &d_tab8, &d_space, space_adv,
    );

    let combined = stack(&[a_px, b_px, c_px, d_px]);
    let png = combined.encode_png().expect("encode png");
    let out = output_path("white_space_proof.png");
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).expect("create target dir");
    }
    std::fs::write(&out, &png).expect("write png");
    println!(
        "\n[png] wrote {} × {} → {} ({} bytes) — A normal-collapse, B pre, C nowrap, D pre+tab",
        combined.width(),
        combined.height(),
        out.display(),
        png.len(),
    );
}

/// Machine-checks the four blocks against the CSS Text 3 white-space model.
#[allow(clippy::too_many_arguments)]
fn verify(
    a: &[Line],
    b: &[Line],
    aw: &[Line],
    bw: &[Line],
    c: &[Line],
    c_normal: &[Line],
    d_tab4: &[Line],
    d_tab8: &[Line],
    d_space: &[Line],
    space_adv: f32,
) {
    // A `normal`: one line (the space run AND the `\n` collapsed to spaces).
    // B `pre`: two lines (the `\n` preserved as a forced break).
    assert_eq!(
        a.len(),
        1,
        "A(normal): `\\n` + space run collapse to one line, got {}",
        a.len()
    );
    assert_eq!(
        b.len(),
        2,
        "B(pre): the `\\n` forces two lines, got {}",
        b.len()
    );
    // Clean width comparison on `"a    b"`: collapsed (one space) < preserved (four
    // spaces) by ~3 space advances.
    let w_normal = aw[0].width;
    let w_pre = bw[0].width;
    assert!(
        w_pre > w_normal + 2.0 * space_adv,
        "collapsed `a    b` ({w_normal:.1}) must be ~3 spaces narrower than preserved ({w_pre:.1})"
    );

    // C `nowrap`: exactly ONE line in the narrow box, whereas the SAME phrase under
    // `normal` in the SAME box WRAPS to several lines — proving nowrap disables the
    // soft wrap.
    assert_eq!(
        c.len(),
        1,
        "C(nowrap): the long phrase stays on ONE line, got {}",
        c.len()
    );
    assert!(
        c_normal.len() >= 2,
        "C(control): `normal` must WRAP the same phrase to ≥2 lines, got {}",
        c_normal.len()
    );

    // D `pre` + tab: the glyph stream is `a`, `\t`, `b`; the pen just after the tab
    // (advance of `a` + advance of the tab) must land on the tab stop = 4·space.
    assert!(
        d_tab4[0].advances.len() >= 2,
        "D: expected a, tab, b glyphs"
    );
    let pen_after_tab_4 = d_tab4[0].advances[0] + d_tab4[0].advances[1];
    let stop_4 = 4.0 * space_adv;
    assert!(
        (pen_after_tab_4 - stop_4).abs() < 0.5,
        "D(tab-size:4): pen after tab ({pen_after_tab_4:.1}) must land on 4·space ({stop_4:.1})"
    );
    // tab-size:8 lands on the farther stop, so its line is wider than tab-size:4,
    // which is wider than a plain single space.
    let pen_after_tab_8 = d_tab8[0].advances[0] + d_tab8[0].advances[1];
    let stop_8 = 8.0 * space_adv;
    assert!(
        (pen_after_tab_8 - stop_8).abs() < 0.5,
        "D(tab-size:8): pen after tab ({pen_after_tab_8:.1}) must land on 8·space ({stop_8:.1})"
    );
    assert!(
        d_tab8[0].width > d_tab4[0].width && d_tab4[0].width > d_space[0].width,
        "D: tab-size:8 ({:.1}) > tab-size:4 ({:.1}) > single space ({:.1})",
        d_tab8[0].width,
        d_tab4[0].width,
        d_space[0].width
    );

    println!(
        "\n[verify] A(normal)=1 line w={w_normal:.1}; B(pre)=2 lines w={w_pre:.1} (>{w_normal:.1}); \
         C nowrap=1 line vs normal={} lines; D tab-size:4 pen→{pen_after_tab_4:.1}=4·{space_adv:.1}, \
         tab-size:8 pen→{pen_after_tab_8:.1}=8·{space_adv:.1}",
        c_normal.len(),
    );
    println!(
        "[verify] OK — collapse < preserve, `\\n` forced under pre, nowrap stays one line, tab lands on the stop"
    );
}

/// Builds one block's CSS: the shared `base` plus a per-block `.block` rule (given
/// the extra `decls`). The `name` is only for readability.
fn css(base: &str, name: &str, decls: &str) -> String {
    format!("{base}\n/* {name} */ .block {{ {decls} }}")
}

/// Renders one `.block` (a container holding a single text node) through the real
/// runtime and returns its rasterized pixmap plus its visual [`Line`]s.
fn render_block(css: &str, content: &str) -> (tiny_skia::Pixmap, Vec<Line>) {
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), VIEWPORT);
    let mut renderer = CpuRenderer::new(RendererId::from_raw(2));
    if let Some(bytes) = load_system_font() {
        wire_text_stack(&mut runtime, &mut renderer, bytes);
    }

    let author = compile(css, CascadeOrigin::GlobalApp);
    let view = block_view(content);
    let result = runtime.render_now(&view, std::slice::from_ref(&author));
    let snapshot = result.snapshot().clone();

    // Group the DrawText runs by y-origin (to the nearest 0.1px) — one group per
    // visual line — summing widths and keeping the first run's per-glyph advances.
    let mut lines: Vec<Line> = Vec::new();
    for command in snapshot.display_list().commands() {
        if let PaintCommand::DrawText { origin, run, .. } = command {
            let advances: Vec<f32> = run.glyphs().iter().map(|g| g.advance()).collect();
            let width = run.advance();
            if let Some(line) = lines.iter_mut().find(|l| (l.y - origin.y).abs() < 0.1) {
                line.width += width;
            } else {
                lines.push(Line {
                    y: origin.y,
                    width,
                    advances,
                });
            }
        }
    }
    lines.sort_by(|a, b| a.y.partial_cmp(&b.y).unwrap_or(std::cmp::Ordering::Equal));
    println!(
        "[block] {} line(s): {:?}",
        lines.len(),
        lines
            .iter()
            .map(|l| (l.width * 10.0).round() / 10.0)
            .collect::<Vec<_>>()
    );

    let frame = RenderFrame::new(snapshot, VIEWPORT);
    let submission = FrameSubmission::new(frame);
    let target = RenderTarget::bitmap(BitmapTarget::new(VIEWPORT));
    renderer
        .render(&submission, &target)
        .expect("cpu render succeeds");
    (
        renderer.pixmap().expect("a rasterized frame").clone(),
        lines,
    )
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
    let width = pixmaps
        .iter()
        .map(tiny_skia::Pixmap::width)
        .max()
        .unwrap_or(1);
    let height: u32 = pixmaps
        .iter()
        .map(|p| p.height() + gap)
        .sum::<u32>()
        .saturating_sub(gap);
    let mut out = tiny_skia::Pixmap::new(width, height.max(1)).expect("combined pixmap");
    out.fill(tiny_skia::Color::from_rgba8(0xd0, 0xcd, 0xc7, 0xff));
    let paint = tiny_skia::PixmapPaint::default();
    let mut y = 0i32;
    for px in pixmaps {
        out.draw_pixmap(
            0,
            y,
            px.as_ref(),
            &paint,
            tiny_skia::Transform::identity(),
            None,
        );
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
