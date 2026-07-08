//! **Proof: multiple `background-image` layers (a comma list) via Vello.**
//!
//! Drives the *whole* slice end-to-end: authors one CSS rule whose
//! `background-image` is a **two-layer** comma list — two crossing
//! `repeating-linear-gradient()`s — over a solid `background-color`, compiles it
//! through the real `obscur-css` frontend (which splits the top-level comma into
//! two layers), runs one frame through [`UiRuntime`] (so the scene coordinator
//! paints both layers back-to-front over the color fill), and renders it through
//! headless Vello/wgpu into `target/multi_bg_proof.png`.
//!
//! The first layer (default `to bottom`) tiles 1px lines every 24px along the
//! vertical axis → HORIZONTAL grid lines; the second (`90deg`) tiles the same 1px
//! lines along the horizontal axis → VERTICAL grid lines. Because the lines sit on
//! `transparent` gaps, the bottom layer shows through the top layer's gaps, so the
//! read-back PNG shows a **crossing graph-paper grid** (both line sets), not just
//! one — the multi-layer proof. On the CPU backend each repeating gradient would
//! flatten to its first stop per the GPU-class-effect policy; the faithful grid is
//! the Vello proof.
//!
//! # Environment note
//!
//! Needs a usable GPU adapter. With none it prints a diagnostic and exits cleanly
//! (no panic); it never fabricates a device.
//!
//! Run it:
//! `cargo run -p obscur-kit --example multi_bg_proof --features render-vello`
#![forbid(unsafe_code)]

use std::path::PathBuf;

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::Size;
use obscur_kit::identity::RuntimeId;
use obscur_kit::primitives::Container;
use obscur_kit::render::frame::{FrameSubmission, RenderFrame};
use obscur_kit::render::renderer::RendererId;
use obscur_kit::render_vello::renderer::{VelloDevice, VelloRenderer};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{View, ViewProperty, ViewValue, CLASS};

/// The output image extent (and the box fills it).
const VIEWPORT: Size = Size::new(360.0, 360.0);

/// The authored stylesheet: a full-viewport box whose `background-image` is a
/// TWO-LAYER comma list — two crossing repeating gradients (24px pitch, 1px
/// lines) — painted over a dark `background-color`. The two layers form a
/// graph-paper grid.
const CSS: &str = r"
.app {
    display: block;
    width: 360px;
    height: 360px;
    background-color: #1e1c19;
    background-image:
        repeating-linear-gradient(#e2e0d9 0 1px, transparent 1px 24px),
        repeating-linear-gradient(90deg, #e2e0d9 0 1px, transparent 1px 24px);
}
";

fn main() {
    println!("== obscur-kit multiple background-image layers proof (headless Vello) ==\n");

    // Acquire a headless GPU device; no adapter → honest diagnostic + clean exit.
    let Some((device, queue)) = init_headless_gpu() else {
        eprintln!(
            "[gpu] no usable GPU adapter/device in this environment — cannot produce the \
             multi-background proof here."
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
    let mut renderer = VelloRenderer::new(RendererId::from_raw(1)).with_device(vello_device);
    println!("[gpu] headless wgpu device acquired · Vello renderer ready");

    // Compile the CSS and build the single-box view through the real frontend.
    let sheet = compile(CSS);
    let view = build_view();

    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), VIEWPORT);
    let result = runtime.render_now(&view, std::slice::from_ref(&sheet));
    let snapshot = result.snapshot().clone();
    println!(
        "[scene] composed={} balanced={} · {} paint command(s)",
        result.composed(),
        result.is_balanced(),
        snapshot.display_list().commands().len(),
    );

    // Render headless and read back to premultiplied RGBA8.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let width = VIEWPORT.width.round() as u32;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let height = VIEWPORT.height.round() as u32;
    let submission = FrameSubmission::new(RenderFrame::new(snapshot, VIEWPORT));
    let (out_w, out_h, pixels) = renderer
        .render_to_image(&submission, width, height)
        .expect("headless vello render + read-back succeeds");

    // Encode to PNG via tiny-skia (bytes are premultiplied RGBA8, what it expects).
    let size = tiny_skia::IntSize::from_wh(out_w, out_h).expect("valid image extent");
    let pixmap = tiny_skia::Pixmap::from_vec(pixels, size).expect("premultiplied rgba8 pixmap");
    let png = pixmap.encode_png().expect("encode png");
    let out = output_path("multi_bg_proof.png");
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).expect("create target dir");
    }
    std::fs::write(&out, &png).expect("write png");

    println!(
        "\n[png] wrote {out_w} × {out_h} → {} ({} bytes) — a CROSSING grid (two layers)",
        out.display(),
        png.len(),
    );
}

/// Builds the single `#app` box view carrying the two-layer `background-image`.
fn build_view() -> View {
    let app = Container::new()
        .into_view_node()
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("app".into())));
    View::single(app)
}

/// Compiles authored CSS text into a runtime-matchable [`Stylesheet`].
fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

/// Acquires a headless wgpu device + queue (no surface), or [`None`] when no
/// adapter/device is available so the caller exits cleanly instead of panicking.
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

/// The workspace-relative output path `target/<name>`.
fn output_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join("target")
        .join(name)
}
