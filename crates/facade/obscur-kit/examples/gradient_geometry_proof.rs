//! **Proof: canonical CSS gradient GEOMETRY (corner direction, radial extent
//! keywords, ellipse) via Vello.**
//!
//! Drives three stacked WIDE (non-square, 480 × 120) boxes end-to-end through the
//! real `obscur-css` frontend + [`UiRuntime`] scene coordinator, then renders them
//! through headless Vello/wgpu into `target/gradient_geometry_proof.png`:
//!
//! 1. `background-image: linear-gradient(to top right, #000, #fff)` — the gradient
//!    line runs to the box's actual top-right corner. On the wide box its direction
//!    is `normalize(h, −w) = normalize(120, −480)` (spec §3.1) — a *shallow* angle
//!    (≈14° above horizontal), NOT a fixed 45°. The dark→light ramp therefore runs
//!    nearly horizontally, tilting slightly up-right.
//! 2. `radial-gradient(closest-side circle, …)` — a true circle whose radius is the
//!    distance to the closest side (60px, the half-height), so a tight centered
//!    disc that fades out well before the box's left/right edges.
//! 3. `radial-gradient(ellipse farthest-corner, …)` — an ellipse reaching the
//!    farthest corner, radii `(dx_far·√2, dy_far·√2) = (240√2, 60√2) ≈ (339, 85)`:
//!    a horizontally STRETCHED oval (rx ≫ ry), not a circle.
//!
//! On the CPU backend every gradient flattens to its first stop per the GPU-class
//! effect policy; the faithful geometry is the Vello proof.
//!
//! # Environment note
//!
//! Needs a usable GPU adapter. With none it prints a diagnostic and exits cleanly.
//!
//! Run it:
//! `cargo run -p obscur-kit --example gradient_geometry_proof --features render-vello`
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

/// The output image extent — deliberately WIDE so corner directions and ellipse
/// radii are visibly aspect-ratio-dependent.
const VIEWPORT: Size = Size::new(480.0, 360.0);

/// The authored stylesheet: three stacked 480 × 120 boxes, one per geometry case.
const CSS: &str = r"
.app {
    display: block;
    width: 480px;
    height: 360px;
}
.corner {
    display: block;
    width: 480px;
    height: 120px;
    background-image: linear-gradient(to top right, #000000, #ffffff);
}
.circle {
    display: block;
    width: 480px;
    height: 120px;
    background-image: radial-gradient(closest-side circle, #ffffff, #0066cc);
}
.ellipse {
    display: block;
    width: 480px;
    height: 120px;
    background-image: radial-gradient(ellipse farthest-corner, #ffffff, #dd2200);
}
";

fn main() {
    println!("== obscur-kit gradient GEOMETRY proof (headless Vello) ==\n");

    let Some((device, queue)) = init_headless_gpu() else {
        eprintln!(
            "[gpu] no usable GPU adapter/device in this environment — cannot produce the \
             gradient-geometry proof here."
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

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let width = VIEWPORT.width.round() as u32;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let height = VIEWPORT.height.round() as u32;
    let submission = FrameSubmission::new(RenderFrame::new(snapshot, VIEWPORT));
    let (out_w, out_h, pixels) = renderer
        .render_to_image(&submission, width, height)
        .expect("headless vello render + read-back succeeds");

    let size = tiny_skia::IntSize::from_wh(out_w, out_h).expect("valid image extent");
    let pixmap = tiny_skia::Pixmap::from_vec(pixels, size).expect("premultiplied rgba8 pixmap");
    let png = pixmap.encode_png().expect("encode png");
    let out = output_path("gradient_geometry_proof.png");
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).expect("create target dir");
    }
    std::fs::write(&out, &png).expect("write png");

    println!(
        "\n[png] wrote {out_w} × {out_h} → {} ({} bytes)",
        out.display(),
        png.len(),
    );
    println!(
        "      top band: to-top-right corner ramp (shallow, ≈14° — not 45°)\n\
         \x20     mid band: closest-side CIRCLE (tight centered disc, r=60)\n\
         \x20     bottom band: farthest-corner ELLIPSE (stretched oval, rx≫ry)"
    );
}

/// Builds the three-band `.app` view: a corner-linear band, a circle-radial band,
/// and an ellipse-radial band.
fn build_view() -> View {
    let band = |class: &str| {
        Container::new()
            .into_view_node()
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
    };
    let app = Container::new()
        .with_children(vec![band("corner"), band("circle"), band("ellipse")])
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
