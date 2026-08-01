#![doc = include_str!("capabilities_png.doc/_module.md")]
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

#[path = "capabilities/scene.rs"]
mod capabilities;

#[doc = include_str!("capabilities_png.doc/VIEWPORT.md")]
const VIEWPORT: Size = Size::new(1440.0, 1076.0);

fn main() {
    println!("== obscur-kit capabilities demo → PNG (headless CPU) ==\n");

    // Supersample: `CAP_SCALE` (default 2.0, clamped 1.0..=4.0). Layout runs at
    // the LOGICAL viewport; the pixmap is `VIEWPORT × scale` and the runtime
    // brackets the scene in a root `scale()` so the physical buffer fills crisply.
    let scale = render_scale();
    let physical = Size::new(VIEWPORT.width * scale, VIEWPORT.height * scale);
    println!(
        "[scale] {scale}× · logical {}×{} → physical {}×{}",
        VIEWPORT.width, VIEWPORT.height, physical.width, physical.height,
    );

    // 1. Compile the authored capabilities stylesheet through the real frontend.
    let sheet = compile(capabilities::stylesheet());

    // 2. Boot the runtime at the logical viewport and set the device scale.
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), VIEWPORT);
    runtime.set_device_scale(scale);

    // 3. Wire the text stack; register the SAME bytes on the CPU renderer so shaped
    //    glyph handles line up with the rasterized outlines. No font → box marks.
    let mut renderer = CpuRenderer::new(RendererId::from_raw(2));
    match load_system_font() {
        Some(bytes) => {
            wire_text_stack(&mut runtime, &mut renderer, bytes);
            println!("[text] system font loaded → glyphs rasterize as real outlines");
        }
        None => println!("[text] no system font found → text renders as box marks (no panic)"),
    }

    // 4. Render one frame. `cpu = true` swaps the gradient/blur captions to the
    //    honest "GPU / Vello" note (the geometry is identical either way).
    let result = runtime.render_now(&capabilities::view(true), std::slice::from_ref(&sheet));
    let snapshot = result.snapshot().clone();
    println!(
        "[scene] composed={} balanced={} · {} paint command(s)",
        result.composed(),
        result.is_balanced(),
        snapshot.display_list().commands().len(),
    );

    // 5. Rasterize the display list into a pixmap at the PHYSICAL extent.
    let frame = RenderFrame::new(snapshot, physical);
    let submission = FrameSubmission::new(frame);
    let target = RenderTarget::bitmap(BitmapTarget::new(physical));
    renderer
        .render(&submission, &target)
        .expect("cpu render succeeds");
    let pixmap = renderer.pixmap().expect("a rasterized frame");

    // 6. Encode the pixmap to PNG and write the artifact.
    let png = pixmap.encode_png().expect("encode png");
    let out = output_path("capabilities_cpu.png");
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).expect("create target dir");
    }
    std::fs::write(&out, &png).expect("write png");

    println!(
        "\n[png] wrote {} × {} → {} ({} bytes) — gradient + blur are the CPU approximation (see the Vello PNG)",
        pixmap.width(),
        pixmap.height(),
        out.display(),
        png.len(),
    );
}

#[doc = include_str!("capabilities_png.doc/render_scale.md")]
fn render_scale() -> f32 {
    std::env::var("CAP_SCALE")
        .ok()
        .and_then(|raw| raw.trim().parse::<f32>().ok())
        .filter(|s| s.is_finite())
        .map_or(2.0, |s| s.clamp(1.0, 4.0))
}

#[doc = include_str!("capabilities_png.doc/compile.md")]
fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

#[doc = include_str!("capabilities_png.doc/output_path.md")]
fn output_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join("target")
        .join(name)
}

#[doc = include_str!("capabilities_png.doc/wire_text_stack.md")]
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

#[doc = include_str!("capabilities_png.doc/font_handle.md")]
fn font_handle(id: FontId) -> u32 {
    u32::try_from(id.index()).unwrap_or(0)
}

#[doc = include_str!("capabilities_png.doc/load_system_font.md")]
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
