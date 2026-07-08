//! **Proof: CSS color-matrix `filter` functions as REAL per-pixel transforms via
//! our own wgpu color-matrix pass over Vello.**
//!
//! Renders one colourful reference element (eight vertical colour bars) beside six
//! copies, each wrapped in a `ColorMatrix` layer —
//! `PushLayer(SceneLayer::with_clip(bounds).add_effect(color_matrix(M)))` …
//! `PopLayer` — exactly the command bracket the scene coordinator emits when it
//! folds an element's color-op `filter` chain into one 4×5 matrix. The renderer's
//! owned color-matrix wgpu pass applies each `M`; the read-back PNG
//! (`target/color_filter_proof.png`) shows each transform:
//!
//! - `grayscale(1)`  — the bars collapse to their luminance grays,
//! - `brightness(1.5)` — everything lightens,
//! - `saturate(2)`   — the colours intensify,
//! - `invert(1)`     — every colour negates (red→cyan, white→black, …),
//! - `hue-rotate(120deg)` — hues rotate a third of the wheel (red→green→blue),
//! - `sepia(1)`      — the image warms to brown tones.
//!
//! Vello 0.9 has no color filters, so this drives the renderer half of the slice
//! end to end (sub-scene extraction → the owned color-matrix fragment pass →
//! composite). The matrices here are the spec-exact folds the coordinator produces
//! (its folding is separately unit-tested); this hand-built list is byte-for-byte
//! the structure it emits for these single-function filters.
//!
//! # Environment note
//!
//! Needs a usable GPU adapter. With none it prints a diagnostic and exits cleanly
//! (no panic); it never fabricates a device.
//!
//! Run it:
//! `cargo run -p obscur-kit --example color_filter_proof --features render-vello`
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

/// A single tile's size (one filtered element).
const TILE: Size = Size::new(220.0, 120.0);
/// Gap between tiles.
const GAP: f32 = 16.0;
/// Tiles per row in the grid.
const COLS: usize = 4;

fn main() {
    println!("== obscur-kit color-matrix filter proof (headless Vello) ==\n");

    let Some((device, queue)) = init_headless_gpu() else {
        eprintln!(
            "[gpu] no usable GPU adapter/device in this environment — cannot produce the \
             color-filter proof here."
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

    // The tiles: a reference (no filter) plus one per color-matrix function.
    let tiles: Vec<(&str, Option<[f32; 20]>)> = vec![
        ("reference", None),
        ("grayscale(1)", Some(grayscale(1.0))),
        ("brightness(1.5)", Some(brightness(1.5))),
        ("saturate(2)", Some(saturate(2.0))),
        ("invert(1)", Some(invert(1.0))),
        ("hue-rotate(120deg)", Some(hue_rotate(120.0_f32.to_radians()))),
        ("sepia(1)", Some(sepia(1.0))),
    ];

    let rows = tiles.len().div_ceil(COLS);
    #[allow(clippy::cast_precision_loss)]
    let viewport = Size::new(
        GAP + COLS as f32 * (TILE.width + GAP),
        GAP + rows as f32 * (TILE.height + GAP),
    );

    let list = build_scene(&tiles, viewport);
    println!(
        "[scene] {} paint command(s) · {} tiles ({} filtered)",
        list.commands().len(),
        tiles.len(),
        tiles.iter().filter(|(_, m)| m.is_some()).count(),
    );

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let width = viewport.width.round() as u32;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let height = viewport.height.round() as u32;
    let snapshot = SceneSnapshot::new(list, SceneGeneration::from_raw(1));
    let submission = FrameSubmission::new(RenderFrame::new(snapshot, viewport));
    let (out_w, out_h, pixels) = renderer
        .render_to_image(&submission, width, height)
        .expect("headless vello render + read-back succeeds");

    let size = tiny_skia::IntSize::from_wh(out_w, out_h).expect("valid image extent");
    let pixmap = tiny_skia::Pixmap::from_vec(pixels, size).expect("premultiplied rgba8 pixmap");
    let png = pixmap.encode_png().expect("encode png");
    let out = output_path("color_filter_proof.png");
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).expect("create target dir");
    }
    std::fs::write(&out, &png).expect("write png");

    for (label, _) in &tiles {
        println!("  · {label}");
    }
    println!(
        "\n[png] wrote {out_w} × {out_h} → {} ({} bytes) — each tile is a REAL color-matrix pass",
        out.display(),
        png.len(),
    );
}

/// Builds the grid: a dark backdrop plus each tile's colour bars, the filtered
/// ones wrapped in a `ColorMatrix` layer clipped to the tile (the coordinator's
/// folded-color-op-chain shape).
#[allow(clippy::cast_precision_loss)]
fn build_scene(tiles: &[(&str, Option<[f32; 20]>)], viewport: Size) -> DisplayList {
    let mut commands: Vec<PaintCommand> = Vec::new();

    // A neutral mid-gray backdrop so both bright and inverted tiles read clearly.
    commands.push(fill(
        Rect::new(Point::new(0.0, 0.0), viewport),
        Color::rgba(0.5, 0.5, 0.5, 1.0),
    ));

    for (index, (_, matrix)) in tiles.iter().enumerate() {
        let col = index % COLS;
        let row = index / COLS;
        let origin = Point::new(
            GAP + col as f32 * (TILE.width + GAP),
            GAP + row as f32 * (TILE.height + GAP),
        );
        let bounds = Rect::new(origin, TILE);

        match matrix {
            None => color_bars(&mut commands, bounds),
            Some(matrix) => {
                // Exactly what the coordinator emits for a color-op `filter`: the
                // subtree bracketed by a layer carrying the folded 4×5 matrix.
                let mut layer = SceneLayer::new().with_clip(Clip::rect(bounds));
                layer.add_effect(LayerEffect::color_matrix(*matrix));
                commands.push(PaintCommand::PushLayer(layer));
                color_bars(&mut commands, bounds);
                commands.push(PaintCommand::PopLayer);
            }
        }
    }

    DisplayList::from_commands(commands)
}

/// Fills `panel` with eight vertical colour bars (red, orange, yellow, green,
/// cyan, blue, magenta, white) — a saturated spectrum whose transform under each
/// filter is unmistakable.
#[allow(clippy::cast_precision_loss)]
fn color_bars(commands: &mut Vec<PaintCommand>, panel: Rect) {
    let bars = [
        Color::rgba(1.0, 0.0, 0.0, 1.0), // red
        Color::rgba(1.0, 0.5, 0.0, 1.0), // orange
        Color::rgba(1.0, 1.0, 0.0, 1.0), // yellow
        Color::rgba(0.0, 0.8, 0.0, 1.0), // green
        Color::rgba(0.0, 0.8, 0.8, 1.0), // cyan
        Color::rgba(0.0, 0.0, 1.0, 1.0), // blue
        Color::rgba(0.8, 0.0, 0.8, 1.0), // magenta
        Color::rgba(1.0, 1.0, 1.0, 1.0), // white
    ];
    let bar_w = panel.size.width / bars.len() as f32;
    for (i, color) in bars.iter().enumerate() {
        let rect = Rect::new(
            Point::new(panel.origin.x + i as f32 * bar_w, panel.origin.y),
            Size::new(bar_w, panel.size.height),
        );
        commands.push(fill(rect, *color));
    }
}

/// A solid-fill `DrawPath` command over `rect`.
fn fill(rect: Rect, color: Color) -> PaintCommand {
    PaintCommand::DrawPath {
        path: Path::rect(rect),
        paint: Paint::solid_fill(color),
    }
}

// --- The spec-exact folded 4×5 matrices (row-major 4×4 + 4-entry bias) ---------
// These mirror the coordinator's fold for a single color function (CSS Filter
// Effects §, https://drafts.fxtf.org/filter-effects/).

/// Builds a `[f32; 20]` from an RGB 3×3 (alpha row identity) and an RGB bias.
fn mat(rows: [[f32; 3]; 3], bias: [f32; 3]) -> [f32; 20] {
    let mut m = [0.0_f32; 20];
    for (r, row) in rows.iter().enumerate() {
        m[r * 4] = row[0];
        m[r * 4 + 1] = row[1];
        m[r * 4 + 2] = row[2];
    }
    m[15] = 1.0; // alpha ← alpha
    m[16] = bias[0];
    m[17] = bias[1];
    m[18] = bias[2];
    m
}

fn brightness(b: f32) -> [f32; 20] {
    mat([[b, 0.0, 0.0], [0.0, b, 0.0], [0.0, 0.0, b]], [0.0; 3])
}

fn invert(i: f32) -> [f32; 20] {
    let k = 1.0 - 2.0 * i;
    mat([[k, 0.0, 0.0], [0.0, k, 0.0], [0.0, 0.0, k]], [i, i, i])
}

fn grayscale(a: f32) -> [f32; 20] {
    let t = 1.0 - a;
    mat(
        [
            [0.2126 + 0.7874 * t, 0.7152 - 0.7152 * t, 0.0722 - 0.0722 * t],
            [0.2126 - 0.2126 * t, 0.7152 + 0.2848 * t, 0.0722 - 0.0722 * t],
            [0.2126 - 0.2126 * t, 0.7152 - 0.7152 * t, 0.0722 + 0.9278 * t],
        ],
        [0.0; 3],
    )
}

fn saturate(s: f32) -> [f32; 20] {
    mat(
        [
            [0.213 + 0.787 * s, 0.715 - 0.715 * s, 0.072 - 0.072 * s],
            [0.213 - 0.213 * s, 0.715 + 0.285 * s, 0.072 - 0.072 * s],
            [0.213 - 0.213 * s, 0.715 - 0.715 * s, 0.072 + 0.928 * s],
        ],
        [0.0; 3],
    )
}

fn sepia(a: f32) -> [f32; 20] {
    let t = 1.0 - a;
    mat(
        [
            [0.393 + 0.607 * t, 0.769 - 0.769 * t, 0.189 - 0.189 * t],
            [0.349 - 0.349 * t, 0.686 + 0.314 * t, 0.168 - 0.168 * t],
            [0.272 - 0.272 * t, 0.534 - 0.534 * t, 0.131 + 0.869 * t],
        ],
        [0.0; 3],
    )
}

fn hue_rotate(theta: f32) -> [f32; 20] {
    let (s, c) = theta.sin_cos();
    mat(
        [
            [
                0.213 + c * 0.787 - s * 0.213,
                0.715 - c * 0.715 - s * 0.715,
                0.072 - c * 0.072 + s * 0.928,
            ],
            [
                0.213 - c * 0.213 + s * 0.143,
                0.715 + c * 0.285 + s * 0.140,
                0.072 - c * 0.072 - s * 0.283,
            ],
            [
                0.213 - c * 0.213 - s * 0.787,
                0.715 - c * 0.715 + s * 0.715,
                0.072 + c * 0.928 + s * 0.072,
            ],
        ],
        [0.0; 3],
    )
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
