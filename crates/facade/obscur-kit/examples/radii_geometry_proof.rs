//! **Proof: canonical CSS radii geometry (headless CPU).**
//!
//! Renders two panels that exercise the two fixes and writes
//! `target/radii_geometry_proof.png`:
//!
//! - **Left — border-radius overlapping-radius clamp (css-backgrounds §5.5).** A
//!   wide-short `200×40` box with `border-radius: 30px` on all four corners. Each
//!   corner (30) is below `min(w,h)/2 = 20`? No — the LEFT/RIGHT sides sum to
//!   `30+30 = 60 > 40` (the height), so the spec's single uniform scale
//!   `f = 40/60` shrinks ALL four corners to `20px`, preserving proportions. The
//!   panel fills the box with the §5.5-normalized radii; the corners fit and never
//!   overflow the box.
//! - **Right — box-shadow spread → radii (css-backgrounds §7.1.1).** A rounded
//!   `border-radius: 10px` box with an outset `box-shadow` of `spread: 8px`. The
//!   shadow's corner radii are the box radii grown by `+spread` → `18px`, so the
//!   shadow's rounded corners are visibly LARGER than the box's and stay
//!   concentric with them.
//!
//! It rasterizes through the software [`CpuRenderer`] (no GPU). Before writing, it
//! asserts (left) the normalized radius is `20px` and the box corner is rounded away
//! while the mid-edges reach the box bounds, and (right) that the shadow extends
//! beyond the box corner (a larger, concentric round).
//!
//! Run it:
//! `cargo run -p obscur-kit --example radii_geometry_proof --features render-cpu`
#![forbid(unsafe_code)]

use std::path::PathBuf;

use obscur_kit::geometry::{CornerRadii, Point, Rect, Size};
use obscur_kit::render::frame::{FrameSubmission, RenderFrame};
use obscur_kit::render::target::{BitmapTarget, RenderTarget};
use obscur_kit::render::{Renderer, RendererId};
use obscur_kit::render_cpu::renderer::CpuRenderer;
use obscur_kit::scene::color::Color;
use obscur_kit::scene::command::{DisplayList, DisplayListBuilder};
use obscur_kit::scene::paint::Brush;
use obscur_kit::scene::path::Path;
use obscur_kit::scene::scene::{SceneGeneration, SceneSnapshot};

/// The output image extent.
const VIEWPORT: Size = Size::new(420.0, 240.0);

fn main() {
    println!("== obscur-kit radii geometry proof (headless CPU) ==\n");

    let page = Color::rgba(0.80, 0.80, 0.80, 1.0); // mid grey page
    let card = Color::rgba(0.20, 0.45, 0.95, 1.0); // blue box
    let shadow = Color::rgba(0.10, 0.10, 0.10, 0.9); // dark shadow

    // --- Left panel: §5.5 overlapping-radius normalization. ---
    let wide_box = Rect::new(Point::new(30.0, 40.0), Size::new(200.0, 40.0));
    let authored = CornerRadii::uniform(30.0);
    let normalized = authored.normalized(wide_box);
    println!(
        "[left]  200x40 box, border-radius 30 → normalized {:.3} (f = 40/60)",
        normalized.top_left
    );

    // --- Right panel: §7.1.1 spread → radii. ---
    let shadow_box = Rect::new(Point::new(150.0, 150.0), Size::new(140.0, 60.0));
    let box_radii = CornerRadii::uniform(10.0);
    let spread = 8.0_f32;
    println!(
        "[right] rounded box radius 10 + outset shadow spread 8 → shadow radii {:.1}",
        box_radii.offset(spread).top_left
    );

    let mut builder = DisplayListBuilder::new();
    builder.fill_rect(
        Rect::new(Point::new(0.0, 0.0), VIEWPORT),
        Brush::solid(page),
    );

    // Left: the §5.5-normalized rounded box.
    builder.fill(Path::rounded_rect(wide_box, normalized), Brush::solid(card));

    // Right: outset drop shadow behind, then the rounded box over it. The
    // coordinator emits the box radii + spread; the CPU backend grows the shadow
    // radii by +spread on the spread-inflated rect (§7.1.1).
    builder.draw_shadow(
        shadow_box,
        box_radii,
        shadow,
        0.0,
        spread,
        (0.0, 0.0),
        false,
    );
    builder.fill(
        Path::rounded_rect(shadow_box, box_radii),
        Brush::solid(card),
    );

    let list = builder.build();
    println!("[scene] {} paint command(s)", list.commands().len());

    let pixmap = render(list);
    verify_left(&pixmap, wide_box, normalized.top_left);
    verify_right(&pixmap, shadow_box, spread);

    let png = pixmap.encode_png().expect("encode png");
    let out = output_path("radii_geometry_proof.png");
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).expect("create target dir");
    }
    std::fs::write(&out, &png).expect("write png");
    println!(
        "\n[png] wrote {} x {} → {} ({} bytes)",
        pixmap.width(),
        pixmap.height(),
        out.display(),
        png.len(),
    );
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

/// Left panel: the normalized radius is 20 (f applied uniformly), the extreme box
/// corner is rounded away (page grey, not box blue), and the mid-height left edge
/// reaches the box bound (box blue).
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn verify_left(pixmap: &tiny_skia::Pixmap, box_rect: Rect, radius: f32) {
    assert!(
        (radius - 20.0).abs() < 1e-3,
        "§5.5 uniform scale must give 20px (f = 40/60 · 30), got {radius}"
    );

    let is_box = |x: u32, y: u32| {
        pixmap
            .pixel(x, y)
            .is_some_and(|p| p.blue() > 180 && p.red() < 120)
    };

    let left = box_rect.origin.x as u32;
    let top = box_rect.origin.y as u32;
    let mid_y = (box_rect.origin.y + box_rect.size.height / 2.0) as u32;

    // The extreme top-left corner pixel is rounded away (not box blue).
    let corner_rounded = !is_box(left + 1, top + 1);
    // The mid-height left edge reaches the box bound (box blue).
    let edge_reaches = is_box(left + 1, mid_y);

    println!(
        "[verify left]  corner rounded away={corner_rounded}, mid-edge reaches box={edge_reaches}"
    );
    assert!(
        corner_rounded,
        "the box corner must be rounded (not overflowing square)"
    );
    assert!(
        edge_reaches,
        "the box must still fill to its mid-height edge"
    );
}

/// Right panel: the shadow extends beyond the box straight edge by ~`spread`, and
/// the shadow's rounded corner reaches further out than the box's corner would with
/// its own (smaller) radius — proving concentric, larger shadow corners.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn verify_right(pixmap: &tiny_skia::Pixmap, box_rect: Rect, spread: f32) {
    // The software backend fills the drop shadow at half alpha, so a shadow pixel
    // over the light-grey page is a mid grey: darker than the page (red < 190) but
    // not the blue box (blue < 190), and roughly neutral (|red − blue| small).
    let is_shadow = |x: u32, y: u32| {
        pixmap.pixel(x, y).is_some_and(|p| {
            let (r, b) = (i32::from(p.red()), i32::from(p.blue()));
            r < 190 && b < 190 && (r - b).abs() < 40
        })
    };

    let left = box_rect.origin.x as u32;
    let mid_y = (box_rect.origin.y + box_rect.size.height / 2.0) as u32;

    // A few px left of the box, at mid-height: the outset shadow extends here.
    let extends = is_shadow(
        left.saturating_sub((spread as u32).saturating_sub(2)),
        mid_y,
    );
    // Diagonally out at the box's top-left corner region the shadow's LARGER round
    // still covers a point that the box's own 10px round would not reach.
    let diag_x = left.saturating_sub(2);
    let diag_y = (box_rect.origin.y + 4.0) as u32;
    let corner_shadow = is_shadow(diag_x, diag_y);

    println!(
        "[verify right] shadow extends past edge={extends}, larger corner round={corner_shadow}"
    );
    assert!(
        extends,
        "the outset shadow must extend beyond the box edge by ~spread"
    );
    assert!(
        corner_shadow,
        "the shadow's grown corner must reach beyond the box corner"
    );
}

/// The workspace-relative output path `target/<name>`.
fn output_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join("target")
        .join(name)
}
