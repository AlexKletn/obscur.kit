//! **Proof: `text-overflow: ellipsis` end-to-end through the real runtime.**
//!
//! Renders text through the REAL runtime + software [`CpuRenderer`] (no GPU),
//! stitched top-to-bottom into `target/text_ellipsis_proof.png`, proving the
//! already-built `text-overflow: ellipsis` path (CSS UI 3 §5.2): a box narrower
//! than its (`white-space: nowrap`) text keeps the largest fitting prefix and
//! appends the ellipsis glyph `…` (U+2026), never exceeding the box.
//!
//! Blocks (all `font-size: 18px`, `width: 60px`, no padding so the content box is
//! exactly 60px):
//!
//! - **A — `text-overflow: ellipsis`** on a long `nowrap` phrase: the one
//!   overflowing line is truncated to a prefix + the ellipsis glyph; its last
//!   painted glyph is the ellipsis and the run width ≤ 60px.
//! - **B — `text-overflow: clip`** (control) on the SAME phrase: the line is
//!   truncated to a prefix too, but the last glyph is a NORMAL glyph, not the
//!   ellipsis — proving A actually appended the ellipsis.
//!
//! Before writing the PNG it machine-checks the composed display list against a
//! standalone `…` probe render (to learn the font's ellipsis glyph index, which
//! is `0x2026` under the reference mock and a real font index under HarfRust) and
//! against a wide `nowrap` render of the phrase (to prove the box really is
//! narrower than the text).
//!
//! Run it:
//! `cargo run -p obscur-kit --example text_ellipsis_proof \
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

/// The Unicode scalar for the horizontal ellipsis `…` — the glyph id the
/// reference mock shaper emits (a real font emits its own glyph index instead).
const ELLIPSIS_CP: u32 = 0x2026;

/// The logical extent each block is laid out and rasterized under.
const VIEWPORT: Size = Size::new(240.0, 40.0);

/// The definite content-box width (px) of the truncating blocks (no padding).
const BOX_WIDTH: f32 = 60.0;

/// The long phrase the narrow box cannot hold on one `nowrap` line.
const PHRASE: &str = "ellipsis truncation demonstration";

/// One painted glyph flattened out of the composed display list.
#[derive(Clone, Copy)]
struct Glyph {
    id: u32,
    advance: f32,
    /// Pen x within the run plus the run origin x, so glyphs from every run on a
    /// line sort into a single left-to-right paint order.
    x: f32,
}

fn main() {
    println!("== obscur-kit text-overflow: ellipsis proof (headless CPU) ==\n");

    let base = "\
.page { display: flex; flex-direction: column; padding: 0; background: #ffffff; }
.block { font-size: 18px; color: #101828; background: #eef2ff; \
         width: 60px; padding: 0; white-space: nowrap; overflow: hidden; }";

    // Probe: the ellipsis glyph alone in a wide box → its font glyph index.
    let (_, probe) = render_block(&css(base, "probe", "text-overflow: clip; width: 200px;"), "\u{2026}");
    let ellipsis_id = probe.last().expect("ellipsis probe has a glyph").id;

    // Natural width of the phrase in a WIDE nowrap box: proves it exceeds the box.
    let (_, natural) = render_block(&css(base, "natural", "text-overflow: clip; width: 400px;"), PHRASE);
    let natural_width: f32 = natural.iter().map(|g| g.advance).sum();

    // A: ellipsis truncation in the narrow box. B: clip control on the same phrase.
    let (a_px, a_glyphs) = render_block(&css(base, "ellipsis", "text-overflow: ellipsis;"), PHRASE);
    let (b_px, b_glyphs) = render_block(&css(base, "clip", "text-overflow: clip;"), PHRASE);

    verify(ellipsis_id, natural_width, &a_glyphs, &b_glyphs);

    let combined = stack(&[a_px, b_px]);
    let png = combined.encode_png().expect("encode png");
    let out = output_path("text_ellipsis_proof.png");
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).expect("create target dir");
    }
    std::fs::write(&out, &png).expect("write png");
    println!(
        "\n[png] wrote {} × {} → {} ({} bytes) — A ellipsis (…), B clip control",
        combined.width(),
        combined.height(),
        out.display(),
        png.len(),
    );
}

/// Machine-checks the ellipsis path: the box is narrower than the text; the
/// ellipsis line ends with the ellipsis glyph and fits the box; the clip control
/// fits the box but does NOT end with the ellipsis.
fn verify(ellipsis_id: u32, natural_width: f32, ellipsis: &[Glyph], clip: &[Glyph]) {
    // Under the reference mock the glyph index IS the codepoint, so the probe
    // pins the actual U+2026; a real font uses its own index (still the ellipsis).
    let is_mock = ellipsis_id == ELLIPSIS_CP;
    println!(
        "[probe] ellipsis glyph index = {ellipsis_id}{}",
        if is_mock { " (== U+2026, reference mock)" } else { " (real font glyph index for …)" }
    );

    // The box (60px) really is narrower than the phrase's natural width.
    assert!(
        natural_width > BOX_WIDTH,
        "phrase natural width ({natural_width:.1}) must exceed the box ({BOX_WIDTH}) to force truncation"
    );

    // A: the truncated ellipsis line ends with the ellipsis glyph and fits the box.
    let a_last = ellipsis.last().expect("ellipsis line has glyphs");
    let a_width: f32 = ellipsis.iter().map(|g| g.advance).sum();
    assert_eq!(
        a_last.id, ellipsis_id,
        "ellipsis: last painted glyph must be the ellipsis (…, index {ellipsis_id}), got {}",
        a_last.id
    );
    assert!(
        a_width <= BOX_WIDTH + 0.5,
        "ellipsis: run width ({a_width:.1}) must be ≤ box width ({BOX_WIDTH})"
    );

    // B (control): clip truncates too and fits the box, but ends with a NORMAL
    // glyph — so A's trailing ellipsis is genuinely the appended glyph, not a
    // coincidence of the source text.
    let b_last = clip.last().expect("clip line has glyphs");
    let b_width: f32 = clip.iter().map(|g| g.advance).sum();
    assert_ne!(
        b_last.id, ellipsis_id,
        "clip control: last glyph must NOT be the ellipsis (it appends nothing)"
    );
    assert!(
        b_width <= BOX_WIDTH + 0.5,
        "clip: run width ({b_width:.1}) must be ≤ box width ({BOX_WIDTH})"
    );

    println!(
        "\n[verify] natural={natural_width:.1} > box={BOX_WIDTH}; \
         ellipsis line width={a_width:.1} (≤{BOX_WIDTH}) ends with … (index {ellipsis_id}); \
         clip line width={b_width:.1} (≤{BOX_WIDTH}) ends with normal glyph {}",
        b_last.id
    );
    println!(
        "[verify] OK — measure and compose share the same shaped ellipsis glyph (one cache), so the \
         sized box and the painted run agree; the box is narrower than its text and the last painted \
         glyph is the ellipsis."
    );
}

/// Builds one block's CSS: the shared `base` plus a per-block `.block` rule.
fn css(base: &str, name: &str, decls: &str) -> String {
    format!("{base}\n/* {name} */ .block {{ {decls} }}")
}

/// Renders one `.block` (holding a single text node) through the real runtime and
/// returns its rasterized pixmap plus the composed line's glyphs, in paint order.
fn render_block(css: &str, content: &str) -> (tiny_skia::Pixmap, Vec<Glyph>) {
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), VIEWPORT);
    let mut renderer = CpuRenderer::new(RendererId::from_raw(2));
    if let Some(bytes) = load_system_font() {
        wire_text_stack(&mut runtime, &mut renderer, bytes);
    }

    let author = compile(css, CascadeOrigin::GlobalApp);
    let view = block_view(content);
    let result = runtime.render_now(&view, std::slice::from_ref(&author));
    let snapshot = result.snapshot().clone();

    // Flatten every DrawText run's glyphs into one list, tagging each with its
    // absolute pen x (run origin + within-run x) so a multi-run line sorts into a
    // single left-to-right paint order. This proof uses single-line LTR text.
    let mut glyphs: Vec<Glyph> = Vec::new();
    for command in snapshot.display_list().commands() {
        if let PaintCommand::DrawText { origin, run, .. } = command {
            for glyph in run.glyphs() {
                glyphs.push(Glyph {
                    id: glyph.glyph_id(),
                    advance: glyph.advance(),
                    x: origin.x + glyph.x(),
                });
            }
        }
    }
    glyphs.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));
    println!(
        "[block] {} glyph(s), last id = {}",
        glyphs.len(),
        glyphs.last().map_or(0, |g| g.id)
    );

    let frame = RenderFrame::new(snapshot, VIEWPORT);
    let submission = FrameSubmission::new(frame);
    let target = RenderTarget::bitmap(BitmapTarget::new(VIEWPORT));
    renderer.render(&submission, &target).expect("cpu render succeeds");
    (renderer.pixmap().expect("a rasterized frame").clone(), glyphs)
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
