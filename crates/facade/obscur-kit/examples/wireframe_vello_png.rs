#![doc = include_str!("wireframe_vello_png.doc/_module.md")]
#![forbid(unsafe_code)]

use std::path::PathBuf;

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::Size;
use obscur_kit::identity::RuntimeId;
use obscur_kit::render::frame::{FrameSubmission, RenderFrame};
use obscur_kit::render::renderer::RendererId;
use obscur_kit::render_vello::renderer::{VelloDevice, VelloRenderer};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::text::font::{FontAttributes, FontFamily, FontId, GenericFamily};
use obscur_kit::text_harfrust::{FontRegistry, HarfrustShaper};
use obscur_kit::text_unicode::UnicodeTextItemizer;

#[path = "wireframe/scene.rs"]
mod wireframe;

#[doc = include_str!("wireframe_vello_png.doc/VIEWPORT.md")]
const VIEWPORT: Size = Size::new(1200.0, 4160.0);

fn main() {
    println!("== obscur-kit Browser Elements Wireframe Kit → PNG (headless Vello, faithful) ==\n");

    // Supersample factor: `UIKIT_SCALE` (default 2.0, clamped 1.0..=4.0). The sheet
    // is laid out at the LOGICAL `VIEWPORT` while the Vello offscreen texture is
    // `VIEWPORT × scale`; `set_device_scale(scale)` brackets the scene in a root
    // `scale()` transform so the physical texture fills crisply.
    let scale = render_scale();
    let physical = Size::new(VIEWPORT.width * scale, VIEWPORT.height * scale);
    println!(
        "[scale] {scale}× · logical {}×{} → physical {}×{}",
        VIEWPORT.width, VIEWPORT.height, physical.width, physical.height,
    );

    // 1. Compile the wireframe stylesheet through the real CSS frontend.
    let sheet = compile(wireframe::stylesheet());

    // 2. Boot the runtime at the LOGICAL viewport and set the device scale.
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), VIEWPORT);
    runtime.set_device_scale(scale);

    // 3. Acquire a headless GPU device + build the Vello renderer. No adapter → an
    //    honest diagnostic + clean exit (never a fabricated device / panic).
    let Some((device, queue)) = init_headless_gpu() else {
        eprintln!(
            "[gpu] no usable GPU adapter/device in this environment — cannot produce the \
             faithful Vello PNG here. (The CPU approximation is available via `wireframe_png`.)"
        );
        return;
    };
    let vello_device = match VelloDevice::new(device, queue) {
        Ok(vello_device) => vello_device,
        Err(error) => {
            eprintln!("[gpu] failed to build the Vello device: {error}");
            return;
        }
    };
    let mut renderer = VelloRenderer::new(RendererId::from_raw(2)).with_device(vello_device);
    println!("[gpu] headless wgpu device acquired · Vello renderer ready");

    // 4. Wire the host text stack (font is a host resource; same as wireframe_png.rs).
    match load_system_font() {
        Some(bytes) => {
            wire_text_stack(&mut runtime, &mut renderer, bytes);
            println!("[text] system font loaded → glyphs rasterize as real outlines");
        }
        None => println!("[text] no system font found → text renders as box marks (no panic)"),
    }

    // 5. Run one frame of the wireframe sheet.
    let view = wireframe::view();
    let result = runtime.render_now(&view, std::slice::from_ref(&sheet));
    let snapshot = result.snapshot().clone();
    let commands = snapshot.display_list().commands().len();
    println!(
        "[scene] composed={} balanced={} · {commands} paint command(s)",
        result.composed(),
        result.is_balanced(),
    );

    // 6. Render offscreen on the GPU at the PHYSICAL extent (logical × scale) and
    //    READ BACK to CPU pixels (the faithful path).
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let width = physical.width.round() as u32;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let height = physical.height.round() as u32;
    let frame = RenderFrame::new(snapshot, physical);
    let submission = FrameSubmission::new(frame);
    let (out_w, out_h, pixels) = renderer
        .render_to_image(&submission, width, height)
        .expect("headless vello render + read-back succeeds");

    // 7. Encode the premultiplied-RGBA8 read-back to PNG via tiny-skia.
    let size = tiny_skia::IntSize::from_wh(out_w, out_h).expect("valid image extent");
    let pixmap = tiny_skia::Pixmap::from_vec(pixels, size).expect("premultiplied rgba8 pixmap");
    let png = pixmap.encode_png().expect("encode png");
    let out = output_path("wireframe_kit_vello.png");
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).expect("create target dir");
    }
    std::fs::write(&out, &png).expect("write png");

    println!(
        "\n[png] wrote {out_w} × {out_h} → {} ({} bytes) — grid + hatches are REAL",
        out.display(),
        png.len(),
    );
}

#[doc = include_str!("wireframe_vello_png.doc/MAX_TEXTURE_DIM.md")]
const MAX_TEXTURE_DIM: f32 = 7000.0;

#[doc = include_str!("wireframe_vello_png.doc/render_scale.md")]
fn render_scale() -> f32 {
    let requested = std::env::var("UIKIT_SCALE")
        .ok()
        .and_then(|raw| raw.trim().parse::<f32>().ok())
        .filter(|s| s.is_finite())
        .map_or(2.0, |s| s.clamp(1.0, 4.0));
    let longest = VIEWPORT.width.max(VIEWPORT.height);
    let cap = MAX_TEXTURE_DIM / longest;
    requested.min(cap).max(1.0)
}

#[doc = include_str!("wireframe_vello_png.doc/init_headless_gpu.md")]
fn init_headless_gpu() -> Option<(wgpu::Device, wgpu::Queue)> {
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        force_fallback_adapter: false,
        compatible_surface: None,
    }))
    .ok()?;
    pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).ok()
}

#[doc = include_str!("wireframe_vello_png.doc/compile.md")]
fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

#[doc = include_str!("wireframe_vello_png.doc/output_path.md")]
fn output_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join("target")
        .join(name)
}

#[doc = include_str!("wireframe_vello_png.doc/wire_text_stack.md")]
fn wire_text_stack(runtime: &mut UiRuntime, renderer: &mut VelloRenderer, bytes: Vec<u8>) {
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

#[doc = include_str!("wireframe_vello_png.doc/font_handle.md")]
fn font_handle(id: FontId) -> u32 {
    u32::try_from(id.index()).unwrap_or(0)
}

#[doc = include_str!("wireframe_vello_png.doc/load_system_font.md")]
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
