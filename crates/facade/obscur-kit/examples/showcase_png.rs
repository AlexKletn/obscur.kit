//! **Headless capability showcase → PNG.**
//!
//! Renders the shared [`showcase`] scene (grid + flex layout, gradients, borders,
//! radius, shadows, opacity, transforms, a `background-image` and an `<img>`
//! primitive, real shaped text, and the [`Button`]/[`Checkbox`]/[`Input`]
//! widgets) through the **software** [`CpuRenderer`] — no display and no GPU — and
//! writes the rasterized frame to `target/showcase_cpu.png`, a real viewable
//! artifact.
//!
//! GPU-class effects (gradients, shadows, blur) degrade to the documented CPU
//! approximation per the project's effect-split policy: a gradient paints its
//! first stop, a shadow paints a flat offset rect. Layout, text, images, borders,
//! radius, opacity, and transform geometry render faithfully. Sub-step 2's
//! windowed Vello example shows the effects at full fidelity from the *same*
//! [`showcase`] module.
//!
//! Text is a host resource: this example loads a system font off disk (best
//! effort) and wires the full itemize → resolve-by-coverage → shape stack, then
//! hands the **same** font bytes to the CPU renderer so glyphs rasterize as real
//! outlines. With no system font present it still runs — text falls back to the
//! backend's box-coverage marks (never a panic).
//!
//! Run it:
//! `cargo run -p obscur-kit --example showcase_png \
//!   --features "render-cpu resource-fs resource-decode text-harfrust text-unicode"`
#![forbid(unsafe_code)]

use std::path::PathBuf;

use obscur_kit::bootstrap::KitBootstrap;
use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::Size;
use obscur_kit::identity::RuntimeId;
use obscur_kit::render::frame::{FrameSubmission, RenderFrame};
use obscur_kit::render::{Renderer, RendererId};
use obscur_kit::render::target::{BitmapTarget, RenderTarget};
use obscur_kit::render_cpu::renderer::CpuRenderer;
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::text::font::{FontAttributes, FontFamily, FontId, GenericFamily};
use obscur_kit::text_harfrust::{FontRegistry, HarfrustShaper};
use obscur_kit::text_unicode::UnicodeTextItemizer;

#[path = "showcase/scene.rs"]
mod showcase;

/// The fixed viewport the showcase is laid out and rasterized under.
const VIEWPORT: Size = Size::new(1000.0, 700.0);

fn main() {
    println!("== obscur-kit capability showcase → PNG (headless CPU) ==\n");

    // 1. Compile the authored showcase stylesheet through the real CSS frontend.
    let sheet = compile(showcase::stylesheet());

    // 2. Stage the image asset in a sandbox dir the resource loader is rooted at,
    //    so both `background-image: url("gallery.png")` and the `<img>` primitive
    //    resolve to real decoded pixels.
    let asset_dir = stage_image_asset();

    // 3. Boot the runtime at the fixed viewport and inject the real filesystem
    //    image loader + decoder as domain ports (facade builds the adapters).
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), VIEWPORT);
    KitBootstrap::new().install_image_resources(&mut runtime, &asset_dir);

    // 4. Wire the text stack. Fonts are a host resource, so we load one off disk
    //    (best effort) and register the SAME bytes on the runtime's resolver and
    //    the CPU renderer, so shaped glyph handles line up with the rasterized
    //    outlines. No font → text degrades to box marks (still no panic).
    let mut renderer = CpuRenderer::new(RendererId::from_raw(2));
    match load_system_font() {
        Some(bytes) => {
            wire_text_stack(&mut runtime, &mut renderer, bytes);
            println!("[text] system font loaded → glyphs rasterize as real outlines");
        }
        None => println!("[text] no system font found → text renders as box marks (no panic)"),
    }

    // 5. Render one frame of the default showcase state.
    let state = showcase::ShowcaseState::default();
    let result = runtime.render_now(&showcase::view(&state), std::slice::from_ref(&sheet));
    let snapshot = result.snapshot().clone();
    let commands = snapshot.display_list().commands().len();
    println!(
        "[scene] composed={} balanced={} · {commands} paint command(s)",
        result.composed(),
        result.is_balanced(),
    );

    // 6. Rasterize the display list into a CPU pixmap sized to the viewport.
    let frame = RenderFrame::new(snapshot, VIEWPORT);
    let submission = FrameSubmission::new(frame);
    let target = RenderTarget::bitmap(BitmapTarget::new(VIEWPORT));
    renderer
        .render(&submission, &target)
        .expect("cpu render succeeds");
    let pixmap = renderer.pixmap().expect("a rasterized frame");

    // 7. Encode the pixmap to PNG and write the artifact.
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

/// Compiles authored CSS text into a runtime-matchable [`Stylesheet`].
fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

/// Writes the embedded showcase PNG into a fresh temp directory and returns it
/// (the resource loader's sandbox root). A `url()` / `src` path resolves relative
/// to this directory.
fn stage_image_asset() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("obscur-kit-showcase-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create asset dir");
    std::fs::write(dir.join(showcase::IMAGE_ASSET_NAME), showcase::image_asset_bytes())
        .expect("write image asset");
    dir
}

/// The workspace-relative output path `target/showcase_cpu.png`, resolved from the
/// crate manifest dir up to the workspace root so it is independent of the cwd.
fn output_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join("target")
        .join("showcase_cpu.png")
}

/// Wires the full text stack: itemize → resolve-by-coverage → shape, registering
/// `bytes` under the `sans-serif` generic family (id `0`) on the runtime's
/// resolver and the SAME id↔bytes on the CPU renderer so shaped run handles select
/// the rasterized outlines.
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

/// The scene font handle (`u32`) for a resolved [`FontId`]: the renderer and the
/// scene both key fonts by this value, so the two must agree.
fn font_handle(id: FontId) -> u32 {
    u32::try_from(id.index()).unwrap_or(0)
}

/// Loads a system font's raw bytes for the text path, scanning well-known
/// Linux/macOS/Windows locations and returning the first that exists, or `None`
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
