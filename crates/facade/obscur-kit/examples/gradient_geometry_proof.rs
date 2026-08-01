#![doc = include_str!("gradient_geometry_proof.doc/_module.md")]
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
use obscur_kit::view::view::{CLASS, View, ViewProperty, ViewValue};

#[doc = include_str!("gradient_geometry_proof.doc/VIEWPORT.md")]
const VIEWPORT: Size = Size::new(480.0, 360.0);

#[doc = include_str!("gradient_geometry_proof.doc/CSS.md")]
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

#[doc = include_str!("gradient_geometry_proof.doc/build_view.md")]
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

#[doc = include_str!("gradient_geometry_proof.doc/compile.md")]
fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

#[doc = include_str!("gradient_geometry_proof.doc/init_headless_gpu.md")]
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

#[doc = include_str!("gradient_geometry_proof.doc/output_path.md")]
fn output_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join("target")
        .join(name)
}
