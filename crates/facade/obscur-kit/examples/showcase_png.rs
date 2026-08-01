#![doc = include_str!("showcase_png.doc/_module.md")]
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

#[path = "showcase/scene.rs"]
mod showcase;

#[doc = include_str!("showcase_png.doc/VIEWPORT.md")]
const VIEWPORT: Size = Size::new(1440.0, 900.0);

fn main() {
    println!("== obscur-kit capability showcase → PNG (headless CPU) ==\n");

    // Optional HiDPI proof: `SHOWCASE_DEVICE_SCALE=2` lays out at a *logical*
    // 720×500 viewport and renders into a *physical* 1440×1000 buffer at
    // `device_scale = 2.0` — the same device-scale path the windowed example uses
    // for HiDPI. Unset (or `1`) keeps the historical byte-identical 1440×900
    // `showcase_cpu.png` at scale 1.0 (`set_device_scale(1.0)` is a no-op).
    let device_scale = std::env::var("SHOWCASE_DEVICE_SCALE")
        .ok()
        .and_then(|v| v.parse::<f32>().ok())
        .filter(|s| s.is_finite() && *s > 0.0)
        .unwrap_or(2.0);
    let hidpi = (device_scale - 1.0).abs() > f32::EPSILON;
    let (logical_viewport, physical_size, out_name) = if hidpi {
        let logical = Size::new(720.0, 500.0);
        let physical = Size::new(logical.width * device_scale, logical.height * device_scale);
        (logical, physical, "showcase_hidpi.png")
    } else {
        (VIEWPORT, VIEWPORT, "showcase_cpu.png")
    };
    if hidpi {
        println!(
            "[hidpi] device_scale={device_scale} · logical {}×{} → physical {}×{}\n",
            logical_viewport.width,
            logical_viewport.height,
            physical_size.width,
            physical_size.height,
        );
    }

    // 1. Compile the authored showcase stylesheet through the real CSS frontend.
    let sheet = compile(showcase::stylesheet());

    // 2. Stage the image asset in a sandbox dir the resource loader is rooted at,
    //    so both `background-image: url("gallery.png")` and the `<img>` primitive
    //    resolve to real decoded pixels.
    let asset_dir = stage_image_asset();

    // 3. Boot the runtime at the fixed viewport and inject the real filesystem
    //    image loader + decoder as domain ports (facade builds the adapters).
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), logical_viewport);
    runtime.set_device_scale(device_scale);
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

    // 6. Rasterize the display list into a CPU pixmap sized to the PHYSICAL
    //    extent (logical × device_scale). The root scale transform the runtime
    //    bracketed the scene with fills the whole physical buffer crisply.
    let frame = RenderFrame::new(snapshot, physical_size);
    let submission = FrameSubmission::new(frame);
    let target = RenderTarget::bitmap(BitmapTarget::new(physical_size));
    renderer
        .render(&submission, &target)
        .expect("cpu render succeeds");
    let pixmap = renderer.pixmap().expect("a rasterized frame");

    // 7. Encode the pixmap to PNG and write the artifact.
    let png = pixmap.encode_png().expect("encode png");
    let out = output_path(out_name);
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

#[doc = include_str!("showcase_png.doc/compile.md")]
fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

#[doc = include_str!("showcase_png.doc/stage_image_asset.md")]
fn stage_image_asset() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("obscur-kit-showcase-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create asset dir");
    std::fs::write(
        dir.join(showcase::IMAGE_ASSET_NAME),
        showcase::image_asset_bytes(),
    )
    .expect("write image asset");
    dir
}

#[doc = include_str!("showcase_png.doc/output_path.md")]
fn output_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join("target")
        .join(name)
}

#[doc = include_str!("showcase_png.doc/wire_text_stack.md")]
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

#[doc = include_str!("showcase_png.doc/font_handle.md")]
fn font_handle(id: FontId) -> u32 {
    u32::try_from(id.index()).unwrap_or(0)
}

#[doc = include_str!("showcase_png.doc/load_system_font.md")]
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
