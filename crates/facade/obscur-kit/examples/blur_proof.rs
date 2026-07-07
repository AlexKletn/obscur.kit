//! **Proof: CSS `filter: blur()` as a REAL gaussian via our own wgpu pass.**
//!
//! Renders two identical hard-edged checkerboards side by side through headless
//! Vello/wgpu: the LEFT one is drawn straight (sharp), the RIGHT one is wrapped in
//! a blur layer — `PushLayer(SceneLayer::with_clip(bounds).add_effect(blur(6)))` …
//! `PopLayer` — exactly the command bracket the scene coordinator emits for
//! `filter: blur(6px)`. The renderer's owned separable-gaussian pass blurs the
//! right panel; the read-back PNG (`target/blur_proof.png`) shows the right
//! checkerboard visibly softened while the left stays crisp.
//!
//! This drives the whole renderer half of the slice (sub-scene extraction in the
//! converter → H+V gaussian pipelines → premultiplied composite). The CSS lowering
//! and the coordinator's blur bracket are covered by unit tests; this hand-built
//! display list is byte-for-byte the structure those produce.
//!
//! # Environment note
//!
//! Needs a usable GPU adapter. With none it prints a diagnostic and exits cleanly
//! (no panic); it never fabricates a device.
//!
//! Run it:
//! `cargo run -p obscur-kit --example blur_proof --features render-vello`
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

/// The output image extent.
const VIEWPORT: Size = Size::new(480.0, 260.0);
/// Each panel is a `PANEL × PANEL` checkerboard.
const PANEL: f32 = 200.0;
/// Number of checker cells per axis (a fine grid so the blur is obvious).
const CELLS: i32 = 10;
/// The blur radius applied to the right panel (logical px).
const BLUR_RADIUS: f32 = 6.0;

fn main() {
    println!("== obscur-kit filter: blur() proof (headless Vello) ==\n");

    // Acquire a headless GPU device; no adapter → honest diagnostic + clean exit.
    let Some((device, queue)) = init_headless_gpu() else {
        eprintln!(
            "[gpu] no usable GPU adapter/device in this environment — cannot produce the \
             blur proof here."
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

    // Build the display list: white backdrop, sharp left checkerboard, and a blur
    // layer wrapping the right checkerboard (the coordinator's `filter: blur` shape).
    let list = build_scene();
    println!(
        "[scene] {} paint command(s) · left = sharp, right = filter: blur({BLUR_RADIUS}px)",
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
    let out = output_path("blur_proof.png");
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).expect("create target dir");
    }
    std::fs::write(&out, &png).expect("write png");

    println!(
        "\n[png] wrote {out_w} × {out_h} → {} ({} bytes) — the RIGHT checkerboard is REAL-blurred",
        out.display(),
        png.len(),
    );
}

/// Builds the two-panel display list: a white backdrop + sharp left checkerboard in
/// the main scene, and the right checkerboard inside a blur group layer.
fn build_scene() -> DisplayList {
    let mut commands: Vec<PaintCommand> = Vec::new();

    // Opaque white backdrop so transparency around the blur reads clearly.
    commands.push(fill(
        Rect::new(Point::new(0.0, 0.0), VIEWPORT),
        Color::rgba(1.0, 1.0, 1.0, 1.0),
    ));

    // Left panel: a sharp checkerboard drawn straight into the main scene.
    let left = Rect::new(Point::new(20.0, 30.0), Size::new(PANEL, PANEL));
    checkerboard(&mut commands, left);

    // Right panel: the SAME checkerboard, bracketed by a blur layer clipped to its
    // border box — exactly what the coordinator emits for `filter: blur(6px)`.
    let right = Rect::new(Point::new(260.0, 30.0), Size::new(PANEL, PANEL));
    let mut layer = SceneLayer::new().with_clip(Clip::rect(right));
    layer.add_effect(LayerEffect::blur(BLUR_RADIUS));
    commands.push(PaintCommand::PushLayer(layer));
    checkerboard(&mut commands, right);
    commands.push(PaintCommand::PopLayer);

    DisplayList::from_commands(commands)
}

/// Appends a hard-edged `CELLS × CELLS` checkerboard filling `panel`: alternating
/// deep-blue and orange cells, a strong pattern whose internal edges smear visibly
/// under blur.
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

/// A solid-fill `DrawPath` command over `rect`.
fn fill(rect: Rect, color: Color) -> PaintCommand {
    PaintCommand::DrawPath {
        path: Path::rect(rect),
        paint: Paint::solid_fill(color),
    }
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
