//! **Headless imported-UI-Kit → PNG (faithful GPU render via Vello).**
//!
//! The GPU sibling of `uikit_png.rs`: it renders the *very same* shared [`uikit`]
//! scene and the *very same* per-theme stylesheet, but through **Vello/wgpu** into
//! an offscreen texture that is then **read back** to CPU pixels and encoded to a
//! PNG — with no window on screen. This is the *faithful* render: the GPU-class
//! effects the CPU backend only approximates paint for real here.
//!
//! Concretely, for the gradient/shadow-heavy **Skeuomorphic** theme (the default,
//! index 4):
//!
//! - every `linear-gradient(#light,#dark)` face paints a real smooth vertical ramp
//!   (the CPU PNG shows only the first, light, stop — a flat fill);
//! - every `box-shadow` — the `inset 0 1px 0 rgba(255,255,255,…)` gloss highlights
//!   AND the `0 2px 5px rgba(0,0,0,…)` cast shadows, combined in one comma list —
//!   paints a real Gaussian-blurred shadow (the CPU PNG paints a flat offset rect);
//! - bevel borders, radius, and shaped text render as on the CPU, but at full GPU
//!   fidelity.
//!
//! Select the theme with the `UIKIT_THEME` env var (0-based index into
//! [`uikit::themes`], default 4 = Skeuomorphic); e.g. `UIKIT_THEME=1` renders
//! Material for comparison. The output is `target/uikit_<slug>_vello.png`.
//!
//! # Wiring
//!
//! A thin driver over the shared scene and the existing runtime + Vello APIs: it
//! adds no rendering logic of its own. It acquires a **headless** wgpu device (no
//! surface — `request_adapter` with `compatible_surface: None`), builds a
//! [`VelloDevice`] + [`VelloRenderer`], wires the same host text stack as
//! `uikit_png.rs`, runs one frame through [`UiRuntime`], and calls the adapter's
//! [`VelloRenderer::render_to_image`] read-back helper to obtain `(w, h, rgba8)`,
//! which is encoded with `tiny_skia` (the bytes are premultiplied RGBA8, exactly
//! what `Pixmap::from_vec` expects).
//!
//! # Environment note
//!
//! Needs a usable GPU adapter (a display is available here). In a truly headless
//! environment with no adapter it prints a diagnostic and exits cleanly (no panic);
//! it never fabricates a device.
//!
//! Run it (Skeuomorphic):
//! `cargo run -p obscur-kit --example uikit_vello_png \
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

#[path = "uikit/scene.rs"]
mod uikit;

/// The layout + rasterization viewport: identical to `uikit_png.rs` so the two
/// renders (CPU approximation vs faithful Vello) frame the exact same scene.
const VIEWPORT: Size = Size::new(1280.0, 2080.0);

fn main() {
    println!("== obscur-kit imported UI Kit → PNG (headless Vello, faithful) ==\n");

    let index = theme_index();
    let (name, css) = uikit::themes()[index];
    println!("[theme] active = {name} (index {index})");

    // Supersample factor: `UIKIT_SCALE` (default 2.0, clamped 1.0..=4.0). The
    // gallery is laid out at the LOGICAL `VIEWPORT` while the Vello offscreen
    // texture is `VIEWPORT × scale`; `set_device_scale(scale)` brackets the scene
    // in a root `scale()` transform so the physical texture fills crisply.
    // `UIKIT_SCALE=1` reproduces the historical logical-1:1 output.
    let scale = render_scale();
    let physical = Size::new(VIEWPORT.width * scale, VIEWPORT.height * scale);
    println!(
        "[scale] {scale}× · logical {}×{} → physical {}×{}",
        VIEWPORT.width, VIEWPORT.height, physical.width, physical.height,
    );

    // 1. Compile the active theme through the real CSS frontend.
    let sheet = compile(css);

    // 2. Boot the runtime at the LOGICAL viewport and set the device scale.
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), VIEWPORT);
    runtime.set_device_scale(scale);

    // 3. Acquire a headless GPU device + build the Vello renderer. No adapter → an
    //    honest diagnostic + clean exit (never a fabricated device / panic).
    let Some((device, queue)) = init_headless_gpu() else {
        eprintln!(
            "[gpu] no usable GPU adapter/device in this environment — cannot produce the \
             faithful Vello PNG here. (The CPU approximation is available via `uikit_png`.)"
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

    // 4. Wire the host text stack (font is a host resource; same as uikit_png.rs).
    match load_system_font() {
        Some(bytes) => {
            wire_text_stack(&mut runtime, &mut renderer, bytes);
            println!("[text] system font loaded → glyphs rasterize as real outlines");
        }
        None => println!("[text] no system font found → text renders as box marks (no panic)"),
    }

    // 5. Run one frame of the selected theme's gallery.
    let view = uikit::view(index);
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
    let out = output_path(&format!("uikit_{}_vello.png", theme_slug(name)));
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).expect("create target dir");
    }
    std::fs::write(&out, &png).expect("write png");

    println!(
        "\n[png] wrote {out_w} × {out_h} → {} ({} bytes) — gradients + shadows are REAL",
        out.display(),
        png.len(),
    );
}

/// The theme index to render: the `UIKIT_THEME` env var as a 0-based index into
/// [`uikit::themes`], clamped to a valid slot, defaulting to the **Skeuomorphic**
/// theme (index 4) — the toughest, gradient/shadow-heavy stress test.
fn theme_index() -> usize {
    const SKEUOMORPHIC_INDEX: usize = 4;
    let count = uikit::themes().len();
    std::env::var("UIKIT_THEME")
        .ok()
        .and_then(|raw| raw.trim().parse::<usize>().ok())
        .filter(|index| *index < count)
        .unwrap_or(SKEUOMORPHIC_INDEX)
}

/// The supersample render scale: the `UIKIT_SCALE` env var parsed as a float and
/// clamped to `1.0..=4.0`, defaulting to `2.0` for a crisp PNG. `1.0` reproduces
/// the historical logical-1:1 output.
fn render_scale() -> f32 {
    std::env::var("UIKIT_SCALE")
        .ok()
        .and_then(|raw| raw.trim().parse::<f32>().ok())
        .filter(|s| s.is_finite())
        .map_or(2.0, |s| s.clamp(1.0, 4.0))
}

/// A short filename slug for a theme name: `Skeuomorphic` → `skeuo`, otherwise the
/// lowercased name with spaces folded to underscores (`Dark SaaS` → `dark_saas`).
fn theme_slug(name: &str) -> String {
    match name {
        "Skeuomorphic" => "skeuo".to_owned(),
        other => other.to_lowercase().replace(' ', "_"),
    }
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

/// Compiles authored CSS text into a runtime-matchable [`Stylesheet`] through the
/// real `obscur-css` frontend (identical to `uikit_png.rs`).
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
/// family on both the resolver and the Vello renderer (identical id↔bytes mapping
/// so scene font handles line up with rasterized outlines).
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
