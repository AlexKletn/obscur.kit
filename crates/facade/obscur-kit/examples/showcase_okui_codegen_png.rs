#![doc = include_str!("showcase_okui_codegen_png.doc/_module.md")]
#![forbid(unsafe_code)]

use std::path::PathBuf;

use obscur_kit::bootstrap::KitBootstrap;
use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::Size;
use obscur_kit::identity::RuntimeId;
use obscur_kit::render::frame::{FrameSubmission, RenderFrame};
use obscur_kit::render::target::{BitmapTarget, RenderTarget};
use obscur_kit::render::{Renderer, RendererId};
use obscur_kit::render_cpu::renderer::CpuRenderer;
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::text::font::{FontAttributes, FontFamily, FontId, GenericFamily};
use obscur_kit::text_harfrust::{FontRegistry, HarfrustShaper};
use obscur_kit::text_unicode::UnicodeTextItemizer;

// Shared only for the embedded image asset the `background-image: url(...)` card
// resolves against; the scene itself is authored in `.okui` and compiled by
// `build.rs`.
#[path = "showcase/scene.rs"]
mod showcase;

// The build-time generated view: `pub fn showcase_view() -> ::obscur_view::view::View`.
// `::obscur_view` / `::obscur_identity` resolve because `obscur-kit` depends on
// both directly.
include!(concat!(env!("OUT_DIR"), "/showcase_okui_view.rs"));

#[doc = include_str!("showcase_okui_codegen_png.doc/VIEWPORT.md")]
const VIEWPORT: Size = Size::new(1000.0, 620.0);

fn main() {
    println!("== obscur-kit .okui → generated Rust → View → PNG (headless CPU) ==\n");

    // The View is ordinary code now — just call the generated function.
    let view = showcase_view();
    println!(
        "[codegen] {} root view node(s) from the build-time generated view",
        view.root().len()
    );

    let sheet = compile_css(stylesheet());
    let asset_dir = stage_image_asset();

    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), VIEWPORT);
    KitBootstrap::new().install_image_resources(&mut runtime, &asset_dir);

    let mut renderer = CpuRenderer::new(RendererId::from_raw(2));
    match load_system_font() {
        Some(bytes) => {
            wire_text_stack(&mut runtime, &mut renderer, bytes);
            println!("[text] system font loaded → glyphs rasterize as real outlines");
        }
        None => println!("[text] no system font found → text renders as box marks (no panic)"),
    }

    let sheets = std::slice::from_ref(&sheet);
    let render = runtime.render_now(&view, sheets);
    let snapshot = render.snapshot().clone();
    let commands = snapshot.display_list().commands().len();
    println!(
        "[scene] composed={} balanced={} · {commands} paint command(s)",
        render.composed(),
        render.is_balanced(),
    );

    let frame = RenderFrame::new(snapshot, VIEWPORT);
    let submission = FrameSubmission::new(frame);
    let target = RenderTarget::bitmap(BitmapTarget::new(VIEWPORT));
    renderer
        .render(&submission, &target)
        .expect("cpu render succeeds");
    let pixmap = renderer.pixmap().expect("a rasterized frame");

    let png = pixmap.encode_png().expect("encode png");
    let out = output_path();
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).expect("create target dir");
    }
    std::fs::write(&out, &png).expect("write png");

    println!(
        "\n[png] wrote {} × {} → {} ({} bytes)",
        pixmap.width(),
        pixmap.height(),
        out.display(),
        png.len(),
    );
}

#[doc = include_str!("showcase_okui_codegen_png.doc/stylesheet.md")]
fn stylesheet() -> &'static str {
    "\
#app { display: flex; flex-direction: column; gap: 14px; padding: 20px; \
       width: 100%; height: 100%; background-color: #12151f; }\n\
.header { display: flex; flex-direction: column; gap: 4px; width: 960px; height: 62px; }\n\
.title { color: #f5f7fa; font-size: 30px; font-weight: 700; height: 36px; }\n\
.subtitle { color: #8b95ad; font-size: 15px; font-style: italic; height: 20px; }\n\
\
.gallery { display: grid; grid-template-columns: 1fr 1fr 1fr 1fr; \
           grid-template-rows: 143px 143px 143px; gap: 12px; \
           width: 960px; height: 454px; }\n\
[type=card] { border-radius: 10px; padding: 12px; background-color: #1c2130; }\n\
.card-label { color: #e6ebf5; font-size: 14px; font-weight: 600; height: 18px; }\n\
.card-note { color: #aab4c8; font-size: 12px; height: 15px; }\n\
\
.solid { background-color: #3552d9; }\n\
.linear { background-image: linear-gradient(90deg, #ff6b6b, #ffd93d); }\n\
.radial { background-image: radial-gradient(circle, #22d3a6, #0f5f8a); }\n\
.conic { background-image: conic-gradient(from 0deg, #f857a6, #ff5858, #f857a6); }\n\
.bordered { border: 5px solid #ffd93d; border-radius: 14px; background-color: #23283a; }\n\
.shadowed { background-color: #2a3550; box-shadow: 6px 8px 18px 0 rgba(0,0,0,0.55); }\n\
.dropshadow { background-color: #4b3a6b; filter: drop-shadow(4px 6px 8px rgba(0,0,0,0.6)); }\n\
.faded { background-color: #e63946; opacity: 0.5; }\n\
.rotated { background-color: #2a9d8f; transform: rotate(8deg) scale(0.92); }\n\
.bgimage { background-image: url(\"gallery.png\"); background-size: contain; \
           background-repeat: no-repeat; background-position: center; \
           background-color: #0f1220; }\n\
\
.toolbar { display: flex; flex-direction: row; gap: 10px; align-items: center; \
           width: 960px; height: 52px; padding: 8px; border-radius: 10px; \
           background-color: #1a1f2e; }\n\
.chip { border-radius: 8px; padding: 6px; width: 96px; height: 28px; }\n\
.chip-text { color: #10131c; font-size: 13px; font-weight: 700; height: 16px; }\n\
.chip-a { background-color: #ffd93d; }\n\
.chip-b { background-color: #22d3a6; }\n\
.chip-c { background-color: #6ea8ff; }\n\
.counter { color: #ffd93d; font-size: 18px; font-weight: 700; width: 200px; height: 24px; }\n\
"
}

#[doc = include_str!("showcase_okui_codegen_png.doc/compile_css.md")]
fn compile_css(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

#[doc = include_str!("showcase_okui_codegen_png.doc/stage_image_asset.md")]
fn stage_image_asset() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("obscur-kit-okui-codegen-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create asset dir");
    std::fs::write(
        dir.join(showcase::IMAGE_ASSET_NAME),
        showcase::image_asset_bytes(),
    )
    .expect("write image asset");
    dir
}

#[doc = include_str!("showcase_okui_codegen_png.doc/output_path.md")]
fn output_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join("target")
        .join("showcase_okui_codegen.png")
}

#[doc = include_str!("showcase_okui_codegen_png.doc/wire_text_stack.md")]
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

#[doc = include_str!("showcase_okui_codegen_png.doc/font_handle.md")]
fn font_handle(id: FontId) -> u32 {
    u32::try_from(id.index()).unwrap_or(0)
}

#[doc = include_str!("showcase_okui_codegen_png.doc/load_system_font.md")]
fn load_system_font() -> Option<Vec<u8>> {
    const CANDIDATES: &[&str] = &[
        "/usr/share/fonts/TTF/DejaVuSans.ttf",
        "/usr/share/fonts/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/TTF/Hack-Regular.ttf",
        "/usr/share/fonts/noto/NotoSans-Regular.ttf",
        "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
        "/usr/share/fonts/liberation/LiberationSans-Regular.ttf",
        "/usr/share/fonts/TTF/Arimo-Regular.ttf",
        "/System/Library/Fonts/Helvetica.ttc",
        "C:\\Windows\\Fonts\\arial.ttf",
    ];
    CANDIDATES.iter().find_map(|p| std::fs::read(p).ok())
}
