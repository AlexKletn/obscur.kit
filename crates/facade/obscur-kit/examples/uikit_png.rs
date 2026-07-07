//! **Headless imported-UI-Kit → PNG.**
//!
//! Renders the shared [`uikit`] scene — the theme-switcher bar over the Material 3
//! component gallery — through the **software** [`CpuRenderer`] (no display, no
//! GPU) and writes the frame to `target/uikit_material.png`, a real viewable
//! artifact.
//!
//! The scene is *theme-agnostic*: [`uikit::view`] uses only semantic classes and
//! [`uikit::themes`] supplies the appearance. This example selects the **Material**
//! entry ([`uikit::MATERIAL_INDEX`]); swapping to another theme later is a one-line
//! index change once that theme's CSS is authored.
//!
//! GPU-class effects degrade to the documented CPU approximation per the project's
//! effect-split policy (a two-layer `box-shadow` paints as flat offset rects; a
//! gradient paints its first stop). Layout, text, borders, radius, and color all
//! render faithfully.
//!
//! Run it:
//! `cargo run -p obscur-kit --example uikit_png \
//!   --features "render-cpu resource-fs resource-decode text-harfrust text-unicode"`
#![forbid(unsafe_code)]

use std::path::PathBuf;

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

#[path = "uikit/scene.rs"]
mod uikit;

/// The viewport the gallery is laid out and rasterized under: wide enough for the
/// two-column card grid and tall enough for all nine cards plus the switcher.
const VIEWPORT: Size = Size::new(1280.0, 2080.0);

fn main() {
    println!("== obscur-kit imported UI Kit → PNG (headless CPU, Material 3) ==\n");

    let (name, css) = uikit::themes()[uikit::MATERIAL_INDEX];
    println!("[theme] active = {name} (index {})", uikit::MATERIAL_INDEX);

    // 1. Compile the active theme's stylesheet through the real CSS frontend.
    let sheet = compile(css);

    // 2. Boot the runtime at the fixed viewport (no image resources needed).
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), VIEWPORT);

    // 3. Wire the text stack: load a host font (best effort) and register the SAME
    //    bytes on the resolver and the CPU renderer so glyph handles line up. No
    //    font → text degrades to box marks (still no panic).
    let mut renderer = CpuRenderer::new(RendererId::from_raw(2));
    match load_system_font() {
        Some(bytes) => {
            wire_text_stack(&mut runtime, &mut renderer, bytes);
            println!("[text] system font loaded → glyphs rasterize as real outlines");
        }
        None => println!("[text] no system font found → text renders as box marks (no panic)"),
    }

    // 4. Render one frame of the Material gallery.
    let view = uikit::view(uikit::MATERIAL_INDEX);
    let result = runtime.render_now(&view, std::slice::from_ref(&sheet));
    let snapshot = result.snapshot().clone();
    let commands = snapshot.display_list().commands().len();
    println!(
        "[scene] composed={} balanced={} · {commands} paint command(s)",
        result.composed(),
        result.is_balanced(),
    );

    // 5. Rasterize the display list into a CPU pixmap sized to the viewport.
    let frame = RenderFrame::new(snapshot, VIEWPORT);
    let submission = FrameSubmission::new(frame);
    let target = RenderTarget::bitmap(BitmapTarget::new(VIEWPORT));
    renderer.render(&submission, &target).expect("cpu render succeeds");
    let pixmap = renderer.pixmap().expect("a rasterized frame");

    // 6. Encode the pixmap to PNG and write the artifact.
    let png = pixmap.encode_png().expect("encode png");
    let out = output_path("uikit_material.png");
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

/// Compiles authored CSS text into a runtime-matchable [`Stylesheet`].
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

/// Wires the full text stack and registers `bytes` under the `sans-serif` generic
/// family on both the resolver and the CPU renderer.
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
