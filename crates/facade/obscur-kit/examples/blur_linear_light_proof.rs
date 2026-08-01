#![doc = include_str!("blur_linear_light_proof.doc/_module.md")]
#![forbid(unsafe_code)]

use std::path::PathBuf;

use obscur_kit::geometry::{Point, Rect, Size};
use obscur_kit::render::frame::{FrameSubmission, RenderFrame};
use obscur_kit::render::renderer::RendererId;
use obscur_kit::render_vello::renderer::{VelloDevice, VelloRenderer};
use obscur_kit::scene::clip::Clip;
use obscur_kit::scene::color::Color;
use obscur_kit::scene::command::{DisplayList, PaintCommand};
use obscur_kit::scene::layer::{LayerEffect, SceneLayer};
use obscur_kit::scene::paint::Paint;
use obscur_kit::scene::path::Path;
use obscur_kit::scene::scene::{SceneGeneration, SceneSnapshot};

#[doc = include_str!("blur_linear_light_proof.doc/VIEWPORT.md")]
const VIEWPORT: Size = Size::new(240.0, 240.0);
#[doc = include_str!("blur_linear_light_proof.doc/PANEL.md")]
const PANEL: Rect = Rect::new(Point::new(20.0, 20.0), Size::new(200.0, 200.0));
#[doc = include_str!("blur_linear_light_proof.doc/GREEN_BOX.md")]
const GREEN_BOX: Rect = Rect::new(Point::new(70.0, 70.0), Size::new(100.0, 100.0));
#[doc = include_str!("blur_linear_light_proof.doc/BLUR_RADIUS.md")]
const BLUR_RADIUS: f32 = 8.0;
#[doc = include_str!("blur_linear_light_proof.doc/SCAN_Y.md")]
const SCAN_Y: u32 = 120;

fn main() {
    println!("== obscur-kit linear-light blur proof (headless Vello) ==\n");

    // Acquire a headless GPU device; no adapter → honest diagnostic + clean exit.
    let Some((device, queue)) = init_headless_gpu() else {
        eprintln!(
            "[gpu] no usable GPU adapter/device in this environment — cannot produce the \
             linear-light blur proof here."
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

    let list = build_scene();
    println!(
        "[scene] {} paint command(s) · green #00ff00 box on red #ff0000, filter: blur({BLUR_RADIUS}px)",
        list.commands().len()
    );

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let width = VIEWPORT.width.round() as u32;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let height = VIEWPORT.height.round() as u32;
    let snapshot = SceneSnapshot::new(list, SceneGeneration::from_raw(1));
    let submission = FrameSubmission::new(RenderFrame::new(snapshot, VIEWPORT));
    let (out_w, out_h, pixels) = renderer
        .render_to_image(&submission, width, height)
        .expect("headless vello render + read-back succeeds");

    // --- Self-assert linear light at the green↔red crossover ---------------------
    // Along SCAN_Y, walk left→right across the box's LEFT edge (red→green). The
    // crossover is the pixel where the blurred R and G channels are closest to
    // equal — the geometric 50/50 boundary. A gamma-space blur lands that value near
    // 128 (average of 255 and 0); linear light lands it near sRGB-encode(0.5) ≈ 182.
    let px = |x: u32, y: u32| -> [u8; 4] {
        let i = ((y * out_w + x) * 4) as usize;
        [pixels[i], pixels[i + 1], pixels[i + 2], pixels[i + 3]]
    };
    let mut best_x = 0u32;
    let mut best_diff = i32::MAX;
    for x in 40..100 {
        let p = px(x, SCAN_Y);
        let diff = (i32::from(p[0]) - i32::from(p[1])).abs();
        // Only consider the actual transition band (both channels partly on).
        if p[0] > 20 && p[1] > 20 && diff < best_diff {
            best_diff = diff;
            best_x = x;
        }
    }
    let crossover = px(best_x, SCAN_Y);
    let cross_val = f32::from(crossover[0].min(crossover[1]));
    let gamma_avg = 128.0_f32; // naive gamma blur of {255,0}
    let linear_expected = 182.0_f32; // 255 * sRGB_encode(0.5)
    println!(
        "\n[crossover] ({best_x}, {SCAN_Y}) rgba = {crossover:?}  |R-G| = {best_diff}\n  gamma-blur \
         would give ~{gamma_avg}, linear-light ~{linear_expected}"
    );
    assert!(
        cross_val > 150.0,
        "linear-light blur regressed: crossover channel {cross_val} is not above the gamma \
         average {gamma_avg} (expected ~{linear_expected})"
    );

    // Flat-region round-trip: deep interior of the green box is ~#00ff00, deep red
    // field is ~#ff0000 (blur of a constant is that constant; the transfer round
    // trips). Sample points well inside each flat area.
    let green_deep = px(120, 120);
    let red_deep = px(35, 120);
    println!("[flat] deep green = {green_deep:?} (≈ #00ff00), deep red = {red_deep:?} (≈ #ff0000)");
    assert!(
        green_deep[1] > 250 && green_deep[0] < 8,
        "flat green interior drifted under blur: {green_deep:?}"
    );
    // Red sits nearer the panel's soft outer border (which blurs toward the dark
    // backdrop), so a few counts of edge-tail bleed is expected and not a color-space
    // effect — the deep-green [0,255,0] above is the clean transfer round-trip witness.
    assert!(
        red_deep[0] > 240 && red_deep[1] < 8,
        "flat red interior drifted under blur: {red_deep:?}"
    );

    // --- Encode the PNG ----------------------------------------------------------
    let size = tiny_skia::IntSize::from_wh(out_w, out_h).expect("valid image extent");
    let pixmap = tiny_skia::Pixmap::from_vec(pixels, size).expect("premultiplied rgba8 pixmap");
    let png = pixmap.encode_png().expect("encode png");
    let out = output_path("blur_linear_light_proof.png");
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).expect("create target dir");
    }
    std::fs::write(&out, &png).expect("write png");

    println!(
        "\n[png] wrote {out_w} × {out_h} → {} ({} bytes)\n[proof] crossover channel {cross_val} > \
         gamma avg {gamma_avg} → blur convolved in LINEAR light",
        out.display(),
        png.len(),
    );
}

#[doc = include_str!("blur_linear_light_proof.doc/build_scene.md")]
fn build_scene() -> DisplayList {
    let mut commands: Vec<PaintCommand> = Vec::new();

    // Neutral dark backdrop (the panel interior is opaque, so it only frames the
    // soft blurred edge at the panel's outer border).
    commands.push(fill(
        Rect::new(Point::new(0.0, 0.0), VIEWPORT),
        Color::rgba(0.1, 0.1, 0.1, 1.0),
    ));

    let mut layer = SceneLayer::new().with_clip(Clip::rect(PANEL));
    layer.add_effect(LayerEffect::blur(BLUR_RADIUS));
    commands.push(PaintCommand::PushLayer(layer));
    // Red field, then the saturated green box on top — both fully opaque, so the
    // green↔red edge is a pure color transition (alpha is 1 throughout the interior).
    commands.push(fill(PANEL, Color::rgba(1.0, 0.0, 0.0, 1.0)));
    commands.push(fill(GREEN_BOX, Color::rgba(0.0, 1.0, 0.0, 1.0)));
    commands.push(PaintCommand::PopLayer);

    DisplayList::from_commands(commands)
}

#[doc = include_str!("blur_linear_light_proof.doc/fill.md")]
fn fill(rect: Rect, color: Color) -> PaintCommand {
    PaintCommand::DrawPath {
        path: Path::rect(rect),
        paint: Paint::solid_fill(color),
    }
}

#[doc = include_str!("blur_linear_light_proof.doc/init_headless_gpu.md")]
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

#[doc = include_str!("blur_linear_light_proof.doc/output_path.md")]
fn output_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join("target")
        .join(name)
}
