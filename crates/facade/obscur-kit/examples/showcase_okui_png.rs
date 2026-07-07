//! **`.okui` DSL → View bridge → headless CPU → PNG.**
//!
//! This example drives the *real* `.okui` frontend pipeline end-to-end and
//! rasterizes the result with the software backend — no display, no GPU:
//!
//! 1. **Parse** an authored [`showcase.okui`](./showcase.okui) document with the
//!    lossless [`Parser`](obscur_kit::okui_syntax::parser::Parser).
//! 2. **Compile** it through the [`Compiler`](obscur_kit::okui_compiler::compile::Compiler)
//!    (name resolution → validation against the built-in contracts → lowering to
//!    UI IR). Diagnostics are printed; the produced IR is still rendered.
//! 3. **Bridge** the compiled `UiIrDocument` into a runtime desired
//!    [`View`](obscur_kit::view::view::View) with
//!    [`UiIrViewBridge::document_to_view`](obscur_kit::okui_compiler::bridge::UiIrViewBridge).
//! 4. **Render** that `View` + an authored stylesheet + a staged image asset
//!    through the [`UiRuntime`] and the [`CpuRenderer`], writing
//!    `target/showcase_okui.png`.
//!
//! # What renders *from `.okui`* today
//!
//! The bridge maps each element's `class` / `id` / `type` attributes onto the
//! reserved CSS selector symbols, so **every CSS-driven visual matches on the
//! `.okui`-authored classes/types**: solid + gradient (linear / radial / conic)
//! backgrounds, border + radius, box-shadow, `filter: drop-shadow`, `opacity`,
//! `transform`, and `background-image: url(...)`. Structure/nesting (grid + flex
//! layout) and shaped **text** flow through too. GPU-class effects degrade to the
//! documented CPU approximation (a gradient paints its first stop; a shadow a flat
//! offset rect), exactly as in `showcase_png.rs`.
//!
//! # The honest bridge gaps (NOT rendered from `.okui` yet)
//!
//! - **Real widgets.** `<Button>` / `<Input>` / `<Checkbox>` lower to a
//!   placeholder `ComponentId` (a hash of the tag), not the real widget
//!   `COMPONENT_ID`, so they render as bare classed boxes, not the widgets.
//! - **`<img src>`.** The bridge hashes arbitrary attributes, so an image
//!   element's `src` never reaches the reserved `SRC` symbol the runtime resolves
//!   to pixels. A `background-image: url(...)` card (pure CSS) IS shown instead.
//!
//! Run it:
//! `cargo run -p obscur-kit --example showcase_okui_png \
//!   --features "render-cpu resource-fs resource-decode text-harfrust text-unicode"`
#![forbid(unsafe_code)]

use std::path::PathBuf;

use obscur_kit::bootstrap::KitBootstrap;
use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::Size;
use obscur_kit::identity::RuntimeId;
use obscur_kit::okui_compiler::bridge::UiIrViewBridge;
use obscur_kit::okui_compiler::compile::Compiler;
use obscur_kit::okui_syntax::ast::Document;
use obscur_kit::okui_syntax::parser::Parser;
use obscur_kit::render::frame::{FrameSubmission, RenderFrame};
use obscur_kit::render::target::{BitmapTarget, RenderTarget};
use obscur_kit::render::{Renderer, RendererId};
use obscur_kit::render_cpu::renderer::CpuRenderer;
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::text::font::{FontAttributes, FontFamily, FontId, GenericFamily};
use obscur_kit::text_harfrust::{FontRegistry, HarfrustShaper};
use obscur_kit::text_unicode::UnicodeTextItemizer;
use obscur_kit::view::view::View;

// Shared only for the embedded image asset (bytes + file name) that the
// `background-image: url("gallery.png")` card resolves against; the scene itself
// is authored in `.okui`, not from this module.
#[path = "showcase/scene.rs"]
mod showcase;

/// The authored `.okui` document — the real DSL input, compiled at runtime.
const SOURCE: &str = include_str!("showcase.okui");

/// The fixed viewport the showcase is laid out and rasterized under.
const VIEWPORT: Size = Size::new(1000.0, 620.0);

fn main() {
    println!("== obscur-kit .okui DSL → View bridge → PNG (headless CPU) ==\n");

    // 1. Parse the `.okui` source into a lossless syntax tree, then cast the root
    //    to a typed `Document`. Parser is total: it never panics.
    let parsed = Parser::new().parse(SOURCE);
    let parse_errors = parsed.errors().len();
    println!("[parse] {} node(s) · {parse_errors} parse error(s)", SOURCE.lines().count());
    for error in parsed.errors() {
        println!("  parse: {error:?}");
    }
    let document = Document::cast(parsed.tree().root().clone()).expect("root casts to a Document");

    // 2. Compile through the real `.okui` pipeline (resolution → validation
    //    against the built-in contracts → lowering to UI IR). Every diagnostic is
    //    printed; the lowered document is still rendered.
    let result = Compiler::new("showcase.okui").compile(&document);
    println!("[compile] {} diagnostic(s)", result.diagnostics().len());
    for diagnostic in result.diagnostics() {
        println!("  diag: {diagnostic}");
    }
    let ir = result.document().expect("a lowered UI IR document");

    // 3. Bridge the compiled UI IR into a runtime desired `View`. Reachable via
    //    the facade path `obscur_kit::okui_compiler::bridge::UiIrViewBridge`.
    let view: View = UiIrViewBridge::document_to_view(ir);
    println!("[bridge] {} root view node(s)", view.root().len());

    // 4. Compile the authored stylesheet keyed to the `.okui` classes/types.
    let sheet = compile_css(stylesheet());

    // 5. Stage the image asset in a sandbox dir the resource loader is rooted at,
    //    so `background-image: url("gallery.png")` resolves to real decoded pixels.
    let asset_dir = stage_image_asset();

    // 6. Boot the runtime, install the filesystem image loader + decoder.
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), VIEWPORT);
    KitBootstrap::new().install_image_resources(&mut runtime, &asset_dir);

    // 7. Wire the text stack (host font, best effort). No font → box marks.
    let mut renderer = CpuRenderer::new(RendererId::from_raw(2));
    match load_system_font() {
        Some(bytes) => {
            wire_text_stack(&mut runtime, &mut renderer, bytes);
            println!("[text] system font loaded → glyphs rasterize as real outlines");
        }
        None => println!("[text] no system font found → text renders as box marks (no panic)"),
    }

    // 8. Render one frame of the bridged view.
    let sheets = std::slice::from_ref(&sheet);
    let render = runtime.render_now(&view, sheets);
    let snapshot = render.snapshot().clone();
    let commands = snapshot.display_list().commands().len();
    println!(
        "[scene] composed={} balanced={} · {commands} paint command(s)",
        render.composed(),
        render.is_balanced(),
    );

    // 9. Rasterize into a CPU pixmap and encode to PNG.
    let frame = RenderFrame::new(snapshot, VIEWPORT);
    let submission = FrameSubmission::new(frame);
    let target = RenderTarget::bitmap(BitmapTarget::new(VIEWPORT));
    renderer.render(&submission, &target).expect("cpu render succeeds");
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

/// The authored stylesheet, keyed to the classes/types the `.okui` declares.
/// `#app` is an id selector, `card` is a **type** selector (matching every
/// `type="card"` element), and the rest are class selectors — all three selector
/// kinds are driven by attributes the `.okui` bridge carries through.
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
card { border-radius: 10px; padding: 12px; background-color: #1c2130; }\n\
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

/// Compiles authored CSS text into a runtime-matchable [`Stylesheet`].
fn compile_css(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

/// Writes the embedded showcase PNG into a fresh temp dir (the loader's sandbox
/// root) so `url("gallery.png")` resolves; returns that directory.
fn stage_image_asset() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("obscur-kit-okui-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create asset dir");
    std::fs::write(dir.join(showcase::IMAGE_ASSET_NAME), showcase::image_asset_bytes())
        .expect("write image asset");
    dir
}

/// The workspace-relative output path `target/showcase_okui.png`.
fn output_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join("target")
        .join("showcase_okui.png")
}

/// Wires the full text stack and registers the SAME font bytes on the runtime's
/// resolver and the CPU renderer so shaped run handles select rasterized outlines.
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

/// Loads a system font's raw bytes for the text path (best effort), or `None`
/// (text then renders as box marks — still no panic).
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
