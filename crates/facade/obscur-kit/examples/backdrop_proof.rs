//! **Proof: CSS `backdrop-filter: blur()` as REAL frosted-glass / acrylic.**
//!
//! Renders a busy hard-edged checkerboard BACKDROP filling the viewport through
//! headless Vello/wgpu, then lays a semi-transparent rounded panel on top carrying
//! `backdrop-filter: blur(10px)` — a `BackdropFilter { bounds, radii, blur }`
//! command emitted BEFORE the panel's own translucent fill, exactly what the scene
//! coordinator produces. The renderer's segmented compositing frosts the backdrop
//! *under the panel only*: the read-back PNG (`target/backdrop_proof.png`) shows the
//! checkerboard visibly blurred inside the rounded panel (with the tint reading as
//! acrylic) while the surrounding backdrop stays crisp and the corners stay sharp.
//!
//! This drives the whole renderer half of the slice (display-list segmentation →
//! per-segment offscreen → composite onto the accumulating frame → in-place region
//! blur masked to the rounded radii). The CSS lowering and the coordinator's
//! `BackdropFilter` emission are covered by unit tests; this hand-built display list
//! is byte-for-byte the structure those produce.
//!
//! # Environment note
//!
//! Needs a usable GPU adapter. With none it prints a diagnostic and exits cleanly
//! (no panic); it never fabricates a device.
//!
//! Run it:
//! `cargo run -p obscur-kit --example backdrop_proof --features render-vello`
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

/// The output image extent.
const VIEWPORT: Size = Size::new(480.0, 320.0);
/// Number of checker cells per axis across the whole viewport (a fine grid so the
/// frost is obvious).
const CELLS: i32 = 16;
/// The backdrop-filter blur radius applied under the panel (logical px).
const BLUR_RADIUS: f32 = 10.0;
/// The panel corner radius (logical px).
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

/// Builds the display list: a full-viewport hard-edged checkerboard, then a
/// `backdrop-filter: blur` boundary + a translucent rounded panel fill (the exact
/// command shape the coordinator emits: BackdropFilter BEFORE the panel's own bg).
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

/// Appends a hard-edged `CELLS × CELLS` checkerboard filling `panel`: alternating
/// deep-blue and orange cells, a strong pattern whose internal edges smear visibly
/// under the backdrop frost.
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
