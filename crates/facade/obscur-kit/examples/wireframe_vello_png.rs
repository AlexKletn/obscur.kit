//! **Headless Browser-Elements-Wireframe-Kit → PNG (faithful GPU render via Vello).**
//!
//! The GPU sibling of `wireframe_png.rs`: it renders the *very same* shared
//! [`wireframe`] scene and the *very same* wireframe user-agent stylesheet, but
//! through **Vello/wgpu** into an offscreen texture that is then **read back** to
//! CPU pixels and encoded to a PNG — with no window on screen. This is the
//! *faithful* render: the GPU-class effects the CPU backend only approximates paint
//! for real here.
//!
//! Concretely, the sheet's `repeating-linear-gradient()` layers paint as REAL tiled
//! stripes on the GPU (the CPU PNG shows only the flat first stop, by policy):
//!
//! - the **graph-paper page grid** — two crossing `repeating-linear-gradient()`s in
//!   one `background-image` comma list — draws its real 24px `#e2e0d9` grid;
//! - the **indeterminate progress** hatch and the **meter** stripe fills draw their
//!   real 45°/90° stripes;
//! - the **colour-swatch** checker draws its real hatch.
//!
//! Everything else (real dashed disabled borders, real `outline` focus rings, the
//! CSS border-triangle arrows, the wrapping textarea, the inset error ring, and the
//! `overflow: hidden` corner-clipping) renders identically on both backends.
//!
//! The output is `target/wireframe_kit_vello.png`.
//!
//! # Wiring
//!
//! A thin driver over the shared scene and the existing runtime + Vello APIs (a
//! faithful copy of `uikit_vello_png.rs`'s harness, retargeted at the wireframe
//! scene). It acquires a **headless** wgpu device (no surface), builds a
//! [`VelloDevice`] + [`VelloRenderer`], wires the same host text stack as
//! `wireframe_png.rs`, runs one frame through [`UiRuntime`], and calls
//! [`VelloRenderer::render_to_image`] to obtain `(w, h, rgba8)`, encoded with
//! `tiny_skia`.
//!
//! # Environment note
//!
//! Needs a usable GPU adapter. In a truly headless environment with no adapter it
//! prints a diagnostic and exits cleanly (no panic); it never fabricates a device.
//! The CPU approximation is always available via `wireframe_png`.
//!
//! Run it:
//! `cargo run -p obscur-kit --example wireframe_vello_png \
//!   --features "render-vello resource-fs resource-decode text-harfrust text-unicode"`
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

/// The layout + rasterization viewport: identical to `wireframe_png.rs` so the two
/// renders (CPU approximation vs faithful Vello) frame the exact same sheet.
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

/// The largest offscreen texture dimension this harness will ask the GPU for. The
/// hard `wgpu` limit is 8192, but the tall wireframe sheet (4160px logical × 2× =
/// 8320) loses the device near that ceiling on some drivers, so we keep a safe
/// working margin: `VIEWPORT × scale` is capped to this on the longest axis.
const MAX_TEXTURE_DIM: f32 = 7000.0;

/// The supersample render scale: the `UIKIT_SCALE` env var parsed as a float and
/// clamped to `1.0..=4.0` (default `2.0`), then further capped so `VIEWPORT × scale`
/// fits [`MAX_TEXTURE_DIM`] on both axes (the sheet is tall enough that a full 2×
/// overflows the GPU texture ceiling).
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

/// Acquires a **headless** wgpu device + queue (no surface), driving the async
/// `request_adapter` / `request_device` to completion with `pollster`. Returns
/// [`None`] when no compatible adapter or device is available (a headless box with
/// no GPU), so the caller can exit cleanly rather than panic.
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

/// Wires the full text stack and registers `bytes` under the `sans-serif` generic
/// family on both the resolver and the Vello renderer.
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
