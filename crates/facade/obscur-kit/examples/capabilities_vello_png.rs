//! **Headless capabilities demo → PNG (faithful GPU render via Vello).**
//!
//! The GPU sibling of `capabilities_png.rs`: it renders the *very same* shared
//! [`capabilities`] scene through **Vello/wgpu** into an offscreen texture that
//! is read back to CPU pixels and encoded to `target/capabilities_vello.png` —
//! with no window on screen. This is the *faithful* render: the GPU-class effects
//! the CPU backend only approximates paint for real here.
//!
//! Concretely:
//!
//! - the **Oklab gradient** card (§2) paints a real `linear-gradient(90deg,
//!   #f00, #0f0)` interpolated in Oklab — a vivid yellow-green midpoint, not the
//!   muddy sRGB blend and not the CPU's flat first stop;
//! - the **linearRGB blur** card (§3) paints a real Gaussian blur convolved in
//!   linear light (the CPU PNG shows only a coarse approximation);
//! - everything else (layout, color parsing, radius/spread, stacking, flex/grid,
//!   margins, text, the interpolated `rotate(45°)` square) renders as on the CPU,
//!   but at full GPU fidelity.
//!
//! # Environment note
//!
//! Needs a usable GPU adapter (a display is available here). In a truly headless
//! environment with no adapter it prints a diagnostic and exits cleanly (no
//! panic); it never fabricates a device.
//!
//! Run it:
//! `cargo run -p obscur-kit --example capabilities_vello_png \
//!   --features "render-vello text-harfrust text-unicode"`
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

#[path = "capabilities/scene.rs"]
mod capabilities;

/// The logical layout viewport: identical to `capabilities_png.rs` so the two
/// renders (CPU approximation vs faithful Vello) frame the exact same scene.
const VIEWPORT: Size = Size::new(1440.0, 1076.0);

fn main() {
    println!("== obscur-kit capabilities demo → PNG (headless Vello, faithful) ==\n");

    // Supersample: `CAP_SCALE` (default 2.0, clamped 1.0..=4.0). Layout runs at
    // the LOGICAL viewport; the offscreen texture is `VIEWPORT × scale`.
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

    // 3. Acquire a headless GPU device + build the Vello renderer. No adapter → an
    //    honest diagnostic + clean exit (never a fabricated device / panic).
    let Some((device, queue)) = init_headless_gpu() else {
        eprintln!(
            "[gpu] no usable GPU adapter/device in this environment — cannot produce the \
             faithful Vello PNG here. (The CPU approximation is available via `capabilities_png`.)"
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

    // 4. Wire the host text stack (font is a host resource; same as the CPU one).
    match load_system_font() {
        Some(bytes) => {
            wire_text_stack(&mut runtime, &mut renderer, bytes);
            println!("[text] system font loaded → glyphs rasterize as real outlines");
        }
        None => println!("[text] no system font found → text renders as box marks (no panic)"),
    }

    // 5. Render one frame. `cpu = false` keeps the faithful gradient/blur captions.
    let result = runtime.render_now(&capabilities::view(false), std::slice::from_ref(&sheet));
    let snapshot = result.snapshot().clone();
    println!(
        "[scene] composed={} balanced={} · {} paint command(s)",
        result.composed(),
        result.is_balanced(),
        snapshot.display_list().commands().len(),
    );

    // 6. Render offscreen on the GPU at the PHYSICAL extent and READ BACK to pixels.
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
    let out = output_path("capabilities_vello.png");
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).expect("create target dir");
    }
    std::fs::write(&out, &png).expect("write png");

    println!(
        "\n[png] wrote {out_w} × {out_h} → {} ({} bytes) — Oklab gradient + linearRGB blur are REAL",
        out.display(),
        png.len(),
    );
}

/// The render scale: `CAP_SCALE` clamped to `1.0..=4.0`, **default 2.0**.
///
/// The Vello filter pass composites each filtered subtree in *device* space, and
/// the scene converter now maps the filter bounds + gaussian radius through the
/// root `device_scale` transform — so the `filter: blur()` card (§3) paints for
/// real at any HiDPI scale (it previously dropped out under `device_scale > 1`).
/// Supersampling at 2.0 renders everything crisply and also exercises that
/// device-scale filter path. `CAP_SCALE=1` renders 1:1.
fn render_scale() -> f32 {
    std::env::var("CAP_SCALE")
        .ok()
        .and_then(|raw| raw.trim().parse::<f32>().ok())
        .filter(|s| s.is_finite())
        .map_or(2.0, |s| s.clamp(1.0, 4.0))
}

/// Acquires a **headless** wgpu device + queue (no surface). Returns [`None`] when
/// no compatible adapter/device is available, so the caller exits cleanly.
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

/// Wires the full text stack and registers `bytes` under `sans-serif` on both the
/// resolver and the Vello renderer (identical id↔bytes mapping).
fn wire_text_stack(runtime: &mut UiRuntime, renderer: &mut VelloRenderer, bytes: Vec<u8>) {
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
