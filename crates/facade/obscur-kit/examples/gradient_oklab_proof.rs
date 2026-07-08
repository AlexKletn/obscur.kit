//! **Proof: CSS gradients interpolate in Oklab (CSS Images 4 default) via Vello.**
//!
//! Per **CSS Images Module Level 4 §3.4**, a gradient with no authored
//! `in <color-space>` interpolates its color stops in **Oklab** — the default
//! rectangular interpolation space (CSS Color 4 §12). Today's fix sets
//! `Gradient::interpolation_cs = Oklab` on the peniko builders in
//! `brush_converter.rs`; this example proves Vello actually *honors* that field
//! during rasterization.
//!
//! It drives two horizontal 256 × 64 bands end-to-end through the real
//! `obscur-css` frontend + [`UiRuntime`], then renders them headless through
//! Vello/wgpu into `target/gradient_oklab_proof.png`:
//!
//! 1. `linear-gradient(to right, red, green)` — the naive sRGB-gamma midpoint is
//!    a dark muddy olive (~`(128, 64, 0)`-ish once premultiplied over sRGB). The
//!    **Oklab** midpoint is a distinctly *lighter*, more vivid yellow-green: its
//!    green channel and overall luminance are meaningfully higher. The example
//!    **self-asserts** the sampled midpoint green channel clears a threshold that
//!    the sRGB-gamma mix could never reach — the machine-checked proof that Vello
//!    interpolated in Oklab.
//! 2. `linear-gradient(to right, blue, yellow)` — rendered for visual comparison.
//!
//! On the CPU backend every gradient flattens to its first stop per the GPU-class
//! effect policy; the faithful interpolation is the Vello proof.
//!
//! # Environment note
//!
//! Needs a usable GPU adapter. With none it prints a diagnostic and exits cleanly
//! (never a fabricated device / panic).
//!
//! Run it:
//! `cargo run -p obscur-kit --example gradient_oklab_proof --features render-vello`
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

/// The output image extent: two stacked 256-wide bands.
const VIEWPORT: Size = Size::new(256.0, 128.0);

/// The authored stylesheet: two 256 × 64 horizontal gradient bands.
const CSS: &str = r"
.app {
    display: block;
    width: 256px;
    height: 128px;
}
.red-green {
    display: block;
    width: 256px;
    height: 64px;
    background-image: linear-gradient(to right, #ff0000, #008000);
}
.blue-yellow {
    display: block;
    width: 256px;
    height: 64px;
    background-image: linear-gradient(to right, #0000ff, #ffff00);
}
";

fn main() {
    println!("== obscur-kit gradient OKLAB-interpolation proof (headless Vello) ==\n");

    let Some((device, queue)) = init_headless_gpu() else {
        eprintln!(
            "[gpu] no usable GPU adapter/device in this environment — cannot produce the \
             Oklab-interpolation proof here. (The unit test in brush_converter asserts \
             interpolation_cs == Oklab regardless.)"
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

    // Sample the midpoint of the red→green band (top band, y = 32; x = 128).
    let mid = sample(&pixels, out_w, 128, 32);
    println!(
        "\n[sample] red→green midpoint pixel (128, 32) = rgba{:?}",
        mid
    );

    // The sRGB-gamma (peniko default) midpoint of red→green is a dark muddy olive:
    // red is (255,0,0), green is (0,128,0); the naive premultiplied-sRGB mix has a
    // green channel around ~64. The Oklab midpoint is a much lighter yellow-green
    // whose green channel is well above 100. Assert we cleared the sRGB ceiling.
    const OKLAB_GREEN_FLOOR: u8 = 100;
    assert!(
        mid[1] >= OKLAB_GREEN_FLOOR,
        "midpoint green channel {} must clear the sRGB-gamma ceiling ({}) — Vello did NOT \
         interpolate in Oklab",
        mid[1],
        OKLAB_GREEN_FLOOR,
    );
    println!(
        "[assert] midpoint green {} >= {} → Vello rasterized the interpolation in Oklab (vivid, \
         not muddy) ✓",
        mid[1], OKLAB_GREEN_FLOOR,
    );

    let size = tiny_skia::IntSize::from_wh(out_w, out_h).expect("valid image extent");
    let pixmap = tiny_skia::Pixmap::from_vec(pixels, size).expect("premultiplied rgba8 pixmap");
    let png = pixmap.encode_png().expect("encode png");
    let out = output_path("gradient_oklab_proof.png");
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
        "      top band: linear-gradient(red, green) — Oklab midpoint is a LIGHT yellow-green\n\
         \x20     bottom band: linear-gradient(blue, yellow) for visual comparison"
    );
}

/// Samples the un-premultiplied-looking RGBA8 bytes at `(x, y)`. The read-back is
/// premultiplied RGBA8, but the bands are fully opaque (alpha 255) so the stored
/// channels equal the display color.
fn sample(pixels: &[u8], width: u32, x: u32, y: u32) -> [u8; 4] {
    let idx = ((y * width + x) * 4) as usize;
    [
        pixels[idx],
        pixels[idx + 1],
        pixels[idx + 2],
        pixels[idx + 3],
    ]
}

/// Builds the two-band `.app` view: a red→green band and a blue→yellow band.
fn build_view() -> View {
    let band = |class: &str| {
        Container::new()
            .into_view_node()
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
    };
    let app = Container::new()
        .with_children(vec![band("red-green"), band("blue-yellow")])
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
