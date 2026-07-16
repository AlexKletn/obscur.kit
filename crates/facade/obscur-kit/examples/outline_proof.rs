//! **Proof: `outline` + `outline-offset` paints a focus ring OUTSIDE the box.**
//!
//! Builds a display list of three filled boxes, each with a `2px` outline offset
//! `3px` — exactly as the scene coordinator emits it: a single `DrawPath` that
//! STROKES a rect (or rounded rect) positioned at the border box INFLATED by
//! `outline-offset + width/2`, so the ring's inner edge sits `offset` px outside
//! the box and does not touch it. The three variants are a dashed outline
//! (`2px dashed #2b2a27`), a solid outline, and a rounded box with a rounded
//! outline. It rasterizes through the software [`CpuRenderer`] (no GPU) and writes
//! `target/outline_proof.png`.
//!
//! Before writing it machine-checks the dashed box: (1) ink pixels exist OUTSIDE
//! the box's left edge — the ring is outside the box; (2) the column one pixel
//! left of the box is empty (white) — the `outline-offset` gap is visible; and
//! (3) the top ring edge alternates painted dashes and gaps — the outline is
//! dashed, not solid.
//!
//! Run it:
//! `cargo run -p obscur-kit --example outline_proof --features render-cpu`
#![forbid(unsafe_code)]

use std::path::PathBuf;

use obscur_kit::geometry::{CornerRadii, Point, Rect, Size};
use obscur_kit::render::frame::{FrameSubmission, RenderFrame};
use obscur_kit::render::target::{BitmapTarget, RenderTarget};
use obscur_kit::render::{Renderer, RendererId};
use obscur_kit::render_cpu::renderer::CpuRenderer;
use obscur_kit::scene::color::Color;
use obscur_kit::scene::command::{DisplayList, DisplayListBuilder};
use obscur_kit::scene::paint::{Brush, Stroke};
use obscur_kit::scene::path::Path;
use obscur_kit::scene::scene::{SceneGeneration, SceneSnapshot};

/// The output image extent.
const VIEWPORT: Size = Size::new(420.0, 220.0);
/// Outline width in logical px (the CSS `2px`).
const OUTLINE: f32 = 2.0;
/// Outline offset in logical px (the CSS `outline-offset: 3px`).
const OFFSET: f32 = 3.0;

fn main() {
    println!("== obscur-kit outline + outline-offset proof (headless CPU) ==\n");

    let ink = Color::rgba(0.169, 0.165, 0.153, 1.0); // #2b2a27 (focus ring)
    let fill = Color::rgba(0.60, 0.70, 0.90, 1.0); // a light-blue box fill

    // Three border boxes: dashed outline, solid outline, and a rounded variant.
    let dashed_box = Rect::new(Point::new(40.0, 55.0), Size::new(100.0, 100.0));
    let solid_box = Rect::new(Point::new(180.0, 55.0), Size::new(100.0, 100.0));
    let rounded_box = Rect::new(Point::new(310.0, 55.0), Size::new(100.0, 100.0));
    let radius = 16.0;

    let mut builder = DisplayListBuilder::new();
    // Opaque white background so the ring, its offset gap, and the fill all read.
    builder.fill_rect(
        Rect::new(Point::new(0.0, 0.0), VIEWPORT),
        Brush::solid(Color::rgba(1.0, 1.0, 1.0, 1.0)),
    );

    // Each box: fill first, then the outline ring painted OUTSIDE it (as the
    // coordinator emits — the ring is a separate unit, on top and unclipped).
    builder.fill_rect(dashed_box, Brush::solid(fill));
    builder.stroke(
        outline_path(dashed_box, CornerRadii::default(), false),
        outline_stroke(ink, true),
    );

    builder.fill_rect(solid_box, Brush::solid(fill));
    builder.stroke(
        outline_path(solid_box, CornerRadii::default(), false),
        outline_stroke(ink, false),
    );

    builder.fill(
        Path::rounded_rect(rounded_box, CornerRadii::uniform(radius)),
        Brush::solid(fill),
    );
    builder.stroke(
        outline_path(rounded_box, CornerRadii::uniform(radius), true),
        outline_stroke(ink, false),
    );

    let list = builder.build();
    println!(
        "[scene] {} paint command(s) · dashed + solid + rounded, outline: 2px offset 3px",
        list.commands().len()
    );

    // Rasterize on the CPU.
    let pixmap = render(list);

    // Machine checks on the dashed box.
    verify_ring_outside_with_gap(&pixmap, dashed_box);
    // The top ring edge centerline sits at box.top - (offset + width/2) = -4px.
    verify_dashes(
        &pixmap,
        (dashed_box.origin.y - OFFSET - OUTLINE / 2.0) as u32,
        dashed_box,
    );

    let png = pixmap.encode_png().expect("encode png");
    let out = output_path("outline_proof.png");
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).expect("create target dir");
    }
    std::fs::write(&out, &png).expect("write png");
    println!(
        "\n[png] wrote {} × {} → {} ({} bytes) — focus rings sit OUTSIDE the boxes",
        pixmap.width(),
        pixmap.height(),
        out.display(),
        png.len(),
    );
}

/// The outline stroke: solid ink brush, `OUTLINE` wide; dashed `[3·w, 3·w]` when
/// `dashed` — the same pattern the coordinator's `dashed_border_pattern` uses.
fn outline_stroke(ink: Color, dashed: bool) -> Stroke {
    let stroke = Stroke::new(Brush::solid(ink), OUTLINE);
    if dashed {
        let dash = 3.0 * OUTLINE;
        stroke.with_dash([dash, dash], 0.0)
    } else {
        stroke
    }
}

/// Mirrors the coordinator's `outline_paint` geometry: the border box INFLATED by
/// `outline-offset + width/2` (rounded, with radii grown by the same inflation,
/// when `has_radius`), so a centered stroke's inner edge sits `offset` px outside.
fn outline_path(bounds: Rect, radii: CornerRadii, has_radius: bool) -> Path {
    let d = OFFSET + OUTLINE / 2.0;
    let rect = Rect::new(
        Point::new(bounds.origin.x - d, bounds.origin.y - d),
        Size::new(
            (bounds.size.width + 2.0 * d).max(0.0),
            (bounds.size.height + 2.0 * d).max(0.0),
        ),
    );
    if has_radius {
        Path::rounded_rect(rect, radii.offset(d))
    } else {
        Path::rect(rect)
    }
}

/// Renders `list` through the software renderer into a pixmap at [`VIEWPORT`].
fn render(list: DisplayList) -> tiny_skia::Pixmap {
    let snapshot = SceneSnapshot::new(list, SceneGeneration::from_raw(1));
    let frame = RenderFrame::new(snapshot, VIEWPORT);
    let submission = FrameSubmission::new(frame);
    let target = RenderTarget::bitmap(BitmapTarget::new(VIEWPORT));
    let mut renderer = CpuRenderer::new(RendererId::from_raw(1));
    renderer
        .render(&submission, &target)
        .expect("cpu render succeeds");
    renderer.pixmap().expect("a rasterized frame").clone()
}

/// Whether a pixel reads as ring ink (dark and opaque).
fn is_ink(pixmap: &tiny_skia::Pixmap, x: u32, y: u32) -> bool {
    pixmap
        .pixel(x, y)
        .is_some_and(|p| p.alpha() > 40 && p.red() < 120)
}

/// Asserts the ring is painted OUTSIDE the box's left edge with a visible offset
/// gap between the ring and the box.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn verify_ring_outside_with_gap(pixmap: &tiny_skia::Pixmap, box_rect: Rect) {
    let left = box_rect.origin.x as u32;
    let top = box_rect.origin.y as u32;
    let bottom = (box_rect.origin.y + box_rect.size.height) as u32;

    // (1) Ring outside: some ink exists in the band just LEFT of the box (the
    // ring centerline sits at box.left - 4; its band is entirely left of the box).
    let mut ink_outside = 0u32;
    for x in (left - 6)..(left - 1) {
        for y in top..bottom {
            if is_ink(pixmap, x, y) {
                ink_outside += 1;
            }
        }
    }
    assert!(
        ink_outside > 0,
        "no ring ink found outside the box's left edge"
    );

    // (2) Offset gap: the column one pixel left of the box is background (white),
    // not ink and not fill — the `outline-offset` gap between the box and the ring.
    let gap_x = left - 1;
    let mid_y = top + (bottom - top) / 2;
    let gap = pixmap.pixel(gap_x, mid_y).expect("gap pixel in bounds");
    assert!(
        gap.red() > 200 && gap.green() > 200 && gap.blue() > 200,
        "expected a white offset gap at x={gap_x}, got rgba=({},{},{},{})",
        gap.red(),
        gap.green(),
        gap.blue(),
        gap.alpha(),
    );
    println!(
        "[verify] ring outside box: {ink_outside} ink px left of the box; offset gap at x={gap_x} is white"
    );
}

/// Scans row `y` across the box's outline width and asserts the ink alternates
/// between painted dashes and empty gaps (proving the dashed outline).
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn verify_dashes(pixmap: &tiny_skia::Pixmap, y: u32, box_rect: Rect) {
    let d = OFFSET + OUTLINE / 2.0;
    let x0 = (box_rect.origin.x - d) as u32;
    let x1 = (box_rect.origin.x + box_rect.size.width + d) as u32;
    let mut runs: Vec<(bool, u32)> = Vec::new();
    for x in x0..x1 {
        let painted = is_ink(pixmap, x, y);
        match runs.last_mut() {
            Some((ink, len)) if *ink == painted => *len += 1,
            _ => runs.push((painted, 1)),
        }
    }
    let dashes = runs.iter().filter(|(ink, _)| *ink).count();
    let gaps = runs.iter().filter(|(ink, _)| !*ink).count();
    println!("[verify] top outline scanline y={y}: {dashes} painted dash run(s), {gaps} gap(s)");
    assert!(
        dashes >= 2 && gaps >= 1,
        "expected an alternating dash/gap pattern, got runs={runs:?} (outline rendered solid?)",
    );
    println!("[verify] OK — the outline is a dashed ring outside the box with a visible offset");
}

/// The workspace-relative output path `target/<name>`.
fn output_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join("target")
        .join(name)
}
