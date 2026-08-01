#![doc = include_str!("backdrop_proof.doc/_module.md")]
#![forbid(unsafe_code)]

use std::path::PathBuf;

use obscur_kit::geometry::{CornerRadii, Point, Rect, Size};
use obscur_kit::render::frame::{FrameSubmission, RenderFrame};
use obscur_kit::render::renderer::RendererId;
use obscur_kit::render_vello::renderer::{VelloDevice, VelloRenderer};
use obscur_kit::scene::color::Color;
use obscur_kit::scene::command::{DisplayList, PaintCommand};
use obscur_kit::scene::paint::Paint;
use obscur_kit::scene::path::Path;
use obscur_kit::scene::scene::{SceneGeneration, SceneSnapshot};

#[doc = include_str!("backdrop_proof.doc/VIEWPORT.md")]
const VIEWPORT: Size = Size::new(480.0, 320.0);
#[doc = include_str!("backdrop_proof.doc/CELLS.md")]
const CELLS: i32 = 16;
#[doc = include_str!("backdrop_proof.doc/BLUR_RADIUS.md")]
const BLUR_RADIUS: f32 = 10.0;
#[doc = include_str!("backdrop_proof.doc/PANEL_RADIUS.md")]
const PANEL_RADIUS: f32 = 28.0;

fn main() {
    println!("== obscur-kit backdrop-filter: blur() proof (headless Vello) ==\n");

    // Acquire a headless GPU device; no adapter → honest diagnostic + clean exit.
    let Some((device, queue)) = init_headless_gpu() else {
        eprintln!(
            "[gpu] no usable GPU adapter/device in this environment — cannot produce the \
             backdrop-filter proof here."
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

    // Build the display list: a full-viewport checkerboard backdrop, then a
    // rounded translucent panel with `backdrop-filter: blur(10px)`.
    let list = build_scene();
    println!(
        "[scene] {} paint command(s) · full-viewport checkerboard backdrop + \
         backdrop-filter: blur({BLUR_RADIUS}px) rounded panel",
        list.commands().len()
    );

    // Render headless and read back to premultiplied RGBA8.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let width = VIEWPORT.width.round() as u32;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let height = VIEWPORT.height.round() as u32;
    let snapshot = SceneSnapshot::new(list, SceneGeneration::from_raw(1));
    let submission = FrameSubmission::new(RenderFrame::new(snapshot, VIEWPORT));
    let (out_w, out_h, pixels) = renderer
        .render_to_image(&submission, width, height)
        .expect("headless vello render + read-back succeeds");

    // Encode to PNG via tiny-skia (bytes are premultiplied RGBA8, what it expects).
    let size = tiny_skia::IntSize::from_wh(out_w, out_h).expect("valid image extent");
    let pixmap = tiny_skia::Pixmap::from_vec(pixels, size).expect("premultiplied rgba8 pixmap");
    let png = pixmap.encode_png().expect("encode png");
    let out = output_path("backdrop_proof.png");
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).expect("create target dir");
    }
    std::fs::write(&out, &png).expect("write png");

    println!(
        "\n[png] wrote {out_w} × {out_h} → {} ({} bytes) — the checkerboard is FROSTED under \
         the rounded panel, sharp outside it",
        out.display(),
        png.len(),
    );
}

#[doc = include_str!("backdrop_proof.doc/build_scene.md")]
fn build_scene() -> DisplayList {
    let mut commands: Vec<PaintCommand> = Vec::new();

    // A busy full-viewport checkerboard: the backdrop the panel frosts.
    checkerboard(&mut commands, Rect::new(Point::new(0.0, 0.0), VIEWPORT));

    // The panel: a rounded rect over the middle. `backdrop-filter` frosts the
    // checkerboard within it, then the translucent tint draws over the frost.
    let panel = Rect::new(Point::new(90.0, 70.0), Size::new(300.0, 180.0));
    let radii = CornerRadii::uniform(PANEL_RADIUS);

    // 1) Frost the backdrop under the panel (clipped to the rounded corners).
    commands.push(PaintCommand::BackdropFilter {
        bounds: panel,
        radii,
        blur: BLUR_RADIUS,
    });
    // 2) The panel's own translucent (acrylic) tint, over the frosted backdrop.
    commands.push(PaintCommand::DrawPath {
        path: Path::rounded_rect(panel, radii),
        paint: Paint::solid_fill(Color::rgba(1.0, 1.0, 1.0, 0.35)),
    });

    DisplayList::from_commands(commands)
}

#[doc = include_str!("backdrop_proof.doc/checkerboard.md")]
#[allow(clippy::cast_precision_loss)]
fn checkerboard(commands: &mut Vec<PaintCommand>, panel: Rect) {
    let cell_w = panel.size.width / CELLS as f32;
    let cell_h = panel.size.height / CELLS as f32;
    let blue = Color::rgba(0.09, 0.12, 0.45, 1.0);
    let orange = Color::rgba(1.0, 0.55, 0.10, 1.0);
    for row in 0..CELLS {
        for col in 0..CELLS {
            let color = if (row + col) % 2 == 0 { blue } else { orange };
            let rect = Rect::new(
                Point::new(
                    panel.origin.x + col as f32 * cell_w,
                    panel.origin.y + row as f32 * cell_h,
                ),
                Size::new(cell_w, cell_h),
            );
            commands.push(fill(rect, color));
        }
    }
}

#[doc = include_str!("backdrop_proof.doc/fill.md")]
fn fill(rect: Rect, color: Color) -> PaintCommand {
    PaintCommand::DrawPath {
        path: Path::rect(rect),
        paint: Paint::solid_fill(color),
    }
}

#[doc = include_str!("backdrop_proof.doc/init_headless_gpu.md")]
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

#[doc = include_str!("backdrop_proof.doc/output_path.md")]
fn output_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join("target")
        .join(name)
}
