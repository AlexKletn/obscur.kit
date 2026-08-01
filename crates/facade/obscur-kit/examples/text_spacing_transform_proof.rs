#![doc = include_str!("text_spacing_transform_proof.doc/_module.md")]
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

#[doc = include_str!("text_spacing_transform_proof.doc/VIEWPORT.md")]
const VIEWPORT: Size = Size::new(360.0, 44.0);

#[doc = include_str!("text_spacing_transform_proof.doc/FONT_SIZE.md")]
const FONT_SIZE: f32 = 20.0;

#[doc = include_str!("text_spacing_transform_proof.doc/Run.md")]
#[derive(Clone, Debug)]
struct Run {
    text: String,
    advance: f32,
}

fn main() {
    println!(
        "== obscur-kit letter-spacing + word-spacing + text-transform proof (headless CPU) ==\n"
    );

    let base = "\n.page { display: flex; padding: 8px; background: #ffffff; }\n.block { padding: 2px; color: #12203f; }";

    // A: baseline `hello`. B: same with letter-spacing: 6px.
    let (a_px, a_runs) = render_block(
        &format!("{base}\n.block {{ font-size: {FONT_SIZE}px; }}"),
        "hello",
    );
    let (b_px, b_runs) = render_block(
        &format!("{base}\n.block {{ font-size: {FONT_SIZE}px; letter-spacing: 6px; }}"),
        "hello",
    );
    // C: baseline `ab cd`. D: same with word-spacing: 12px.
    let (c_px, c_runs) = render_block(
        &format!("{base}\n.block {{ font-size: {FONT_SIZE}px; }}"),
        "ab cd",
    );
    let (d_px, d_runs) = render_block(
        &format!("{base}\n.block {{ font-size: {FONT_SIZE}px; word-spacing: 12px; }}"),
        "ab cd",
    );
    // E: `hello` uppercased via text-transform.
    let (e_px, e_runs) = render_block(
        &format!("{base}\n.block {{ font-size: {FONT_SIZE}px; text-transform: uppercase; }}"),
        "hello",
    );

    verify(&a_runs, &b_runs, &c_runs, &d_runs, &e_runs);

    let combined = stack(&[a_px, b_px, c_px, d_px, e_px]);
    let png = combined.encode_png().expect("encode png");
    let out = output_path("text_spacing_transform_proof.png");
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).expect("create target dir");
    }
    std::fs::write(&out, &png).expect("write png");
    println!(
        "\n[png] wrote {} x {} -> {} ({} bytes)",
        combined.width(),
        combined.height(),
        out.display(),
        png.len(),
    );
}

#[doc = include_str!("text_spacing_transform_proof.doc/first_run.md")]
fn first_run(runs: &[Run]) -> &Run {
    runs.iter()
        .find(|r| !r.text.trim().is_empty())
        .expect("block emits a non-empty text run")
}

#[doc = include_str!("text_spacing_transform_proof.doc/verify.md")]
fn verify(a: &[Run], b: &[Run], c: &[Run], d: &[Run], e: &[Run]) {
    let w0 = first_run(a).advance; // baseline "hello"
    let w1 = first_run(b).advance; // "hello" + letter-spacing: 6px
    // §8.1: +6px per typographic character unit; "hello" = 5 clusters.
    let expected_ls = w0 + 6.0 * 5.0;
    assert!(
        (w1 - expected_ls).abs() < 0.5,
        "letter-spacing: run width {w1:.2} should be baseline {w0:.2} + 6x5 = {expected_ls:.2}"
    );
    assert!(
        w1 > w0 + 1.0,
        "letter-spacing must make the run visibly wider"
    );

    let g0 = first_run(c).advance; // baseline "ab cd"
    let g1 = first_run(d).advance; // "ab cd" + word-spacing: 12px
    // §8.2: +12px once, on the single word separator.
    assert!(
        (g1 - (g0 + 12.0)).abs() < 0.5,
        "word-spacing: run width {g1:.2} should be baseline {g0:.2} + 12 = {:.2}",
        g0 + 12.0
    );

    // §2.1: the shaped/painted text was uppercased BEFORE shaping.
    let upper = first_run(e);
    assert_eq!(
        upper.text, "HELLO",
        "text-transform: uppercase must shape 'HELLO'"
    );

    println!(
        "[verify] letter-spacing: W0={w0:.2} -> W1={w1:.2} (=W0+30)\n[verify] word-spacing:   G0={g0:.2} -> G1={g1:.2} (=G0+12)\n[verify] text-transform: painted run text = {:?}",
        upper.text
    );
    println!(
        "[verify] OK - spacing widens measured+painted runs; transform uppercases before shaping"
    );
}

#[doc = include_str!("text_spacing_transform_proof.doc/render_block.md")]
fn render_block(css: &str, content: &str) -> (tiny_skia::Pixmap, Vec<Run>) {
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), VIEWPORT);
    let mut renderer = CpuRenderer::new(RendererId::from_raw(2));
    if let Some(bytes) = load_system_font() {
        wire_text_stack(&mut runtime, &mut renderer, bytes);
    }

    let author = compile(css, CascadeOrigin::GlobalApp);
    let view = block_view(content);
    let result = runtime.render_now(&view, std::slice::from_ref(&author));
    let snapshot = result.snapshot().clone();

    let runs: Vec<Run> = snapshot
        .display_list()
        .commands()
        .iter()
        .filter_map(|c| match c {
            PaintCommand::DrawText { run, .. } => Some(Run {
                text: run.text().to_string(),
                advance: run.advance(),
            }),
            _ => None,
        })
        .collect();
    println!(
        "[block] {:?} -> {} run(s): {:?}",
        content,
        runs.len(),
        runs.iter()
            .map(|r| (r.text.clone(), (r.advance * 100.0).round() / 100.0))
            .collect::<Vec<_>>()
    );

    let frame = RenderFrame::new(snapshot, VIEWPORT);
    let submission = FrameSubmission::new(frame);
    let target = RenderTarget::bitmap(BitmapTarget::new(VIEWPORT));
    renderer
        .render(&submission, &target)
        .expect("cpu render succeeds");
    (renderer.pixmap().expect("a rasterized frame").clone(), runs)
}

#[doc = include_str!("text_spacing_transform_proof.doc/block_view.md")]
fn block_view(content: &str) -> View {
    let block = ViewNode::element(ComponentId::from_raw(1))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("block".into())))
        .with_children(ViewFragment::from_nodes(vec![ViewNode::text(content)]));
    let page = ViewNode::element(ComponentId::from_raw(2))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("page".into())))
        .with_children(ViewFragment::from_nodes(vec![block]));
    View::single(page)
}

#[doc = include_str!("text_spacing_transform_proof.doc/stack.md")]
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

#[doc = include_str!("text_spacing_transform_proof.doc/compile.md")]
fn compile(css: &str, origin: CascadeOrigin) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new()
        .with_origin(origin)
        .compile(&parsed)
        .matchable
}

#[doc = include_str!("text_spacing_transform_proof.doc/output_path.md")]
fn output_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join("target")
        .join(name)
}

#[doc = include_str!("text_spacing_transform_proof.doc/wire_text_stack.md")]
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

#[doc = include_str!("text_spacing_transform_proof.doc/font_handle.md")]
fn font_handle(id: FontId) -> u32 {
    u32::try_from(id.index()).unwrap_or(0)
}

#[doc = include_str!("text_spacing_transform_proof.doc/load_system_font.md")]
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
