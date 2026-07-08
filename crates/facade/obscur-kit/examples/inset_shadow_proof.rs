//! **Proof: an `inset` `box-shadow` renders INSIDE the box (headless CPU).**
//!
//! Builds a display list of two light boxes over a mid-grey page, each carrying an
//! inset `box-shadow` exactly as the scene coordinator emits it — a background fill
//! (the light box) followed by an inset `DrawShadow` painted OVER the background:
//!
//! - Left:  `box-shadow: 0 0 0 3px #2b2a27 inset` — a crisp inner RING hugging the
//!   box edge (blur 0, spread 3).
//! - Right: `box-shadow: inset 0 4px 8px rgba(0,0,0,.4)` — a soft inner shadow along
//!   the TOP edge (offset down, blur 8; the software backend approximates the blur
//!   as a hard frame, thicker on the side opposite the offset).
//!
//! It rasterizes through the software [`CpuRenderer`] (no GPU) and writes
//! `target/inset_shadow_proof.png`. Before writing, it asserts for the ring box that
//! the box EDGE is inked, the box CENTER is clear (a ring, not a filled blob), and a
//! pixel just OUTSIDE the box is the untouched page grey — i.e. the inset shadow is
//! confined to the box and never bleeds out.
//!
//! Run it:
//! `cargo run -p obscur-kit --example inset_shadow_proof --features render-cpu`
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
use obscur_kit::scene::scene::{SceneGeneration, SceneSnapshot};

/// The output image extent.
const VIEWPORT: Size = Size::new(360.0, 200.0);

fn main() {
    println!("== obscur-kit inset box-shadow proof (headless CPU) ==\n");

    let ink = Color::rgba(0.169, 0.165, 0.153, 1.0); // #2b2a27
    let page = Color::rgba(0.80, 0.80, 0.80, 1.0); // mid grey page
    let card = Color::rgba(0.98, 0.98, 0.97, 1.0); // near-white box

    let ring_box = Rect::new(Point::new(30.0, 40.0), Size::new(120.0, 120.0));
    let soft_box = Rect::new(Point::new(210.0, 40.0), Size::new(120.0, 120.0));

    let mut builder = DisplayListBuilder::new();
    // Page background so any bleed outside a box would be visible.
    builder.fill_rect(Rect::new(Point::new(0.0, 0.0), VIEWPORT), Brush::solid(page));

    // Left box: background fill, then a crisp inner RING (spread 3, no blur).
    builder.fill_rect(ring_box, Brush::solid(card));
    builder.draw_shadow(ring_box, CornerRadii::default(), ink, 0.0, 3.0, (0.0, 0.0), true);

    // Right box (rounded): background fill, then a soft inner shadow along the TOP.
    let radii = CornerRadii::uniform(16.0);
    builder.fill(
        obscur_kit::scene::path::Path::rounded_rect(soft_box, radii),
        Brush::solid(card),
    );
    builder.draw_shadow(
        soft_box,
        radii,
        Color::rgba(0.0, 0.0, 0.0, 0.4),
        8.0,
        0.0,
        (0.0, 4.0),
        true,
    );

    let list = builder.build();
    println!(
        "[scene] {} paint command(s) · ring box (0 0 0 3px inset) + soft box (inset 0 4px 8px)",
        list.commands().len()
    );

    let pixmap = render(list);
    verify_ring(&pixmap, ring_box);

    let png = pixmap.encode_png().expect("encode png");
    let out = output_path("inset_shadow_proof.png");
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).expect("create target dir");
    }
    std::fs::write(&out, &png).expect("write png");
    println!(
        "\n[png] wrote {} × {} → {} ({} bytes) — inset shadows sit INSIDE the box",
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
    renderer.render(&submission, &target).expect("cpu render succeeds");
    renderer.pixmap().expect("a rasterized frame").clone()
}

/// Asserts the ring box shows an inner ring confined to the box: inked at the edge,
/// clear at the center, and the untouched page grey just outside it.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn verify_ring(pixmap: &tiny_skia::Pixmap, box_rect: Rect) {
    let is_ink = |x: u32, y: u32| {
        pixmap
            .pixel(x, y)
            .is_some_and(|p| p.alpha() > 200 && p.red() < 90)
    };
    let is_greyish = |x: u32, y: u32, lo: u8, hi: u8| {
        pixmap
            .pixel(x, y)
            .is_some_and(|p| p.red() >= lo && p.red() <= hi)
    };

    let left = box_rect.origin.x as u32;
    let top = box_rect.origin.y as u32;
    let mid_y = (box_rect.origin.y + box_rect.size.height / 2.0) as u32;
    let mid_x = (box_rect.origin.x + box_rect.size.width / 2.0) as u32;

    // Edge (2px in) is inked; center is the light card; just-outside is page grey.
    let edge = is_ink(left + 1, mid_y);
    let center = is_greyish(mid_x, mid_y, 230, 255); // near-white card
    let outside = is_greyish(left.saturating_sub(3), mid_y, 190, 215); // page grey

    println!(
        "[verify] ring box: edge inked={edge}, center clear={center}, outside untouched={outside}"
    );
    assert!(edge, "the box edge must show the inset ring");
    assert!(center, "the box center must stay clear (a ring, not a filled blob)");
    assert!(outside, "the inset shadow must not bleed outside the box");
    let _ = top;
    println!("[verify] OK — the inset shadow is a ring INSIDE the box, no bleed");
}

/// The workspace-relative output path `target/<name>`.
fn output_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join("target")
        .join(name)
}
