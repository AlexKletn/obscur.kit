//! **Proof: the CSS border-triangle trick renders (headless CPU).**
//!
//! The classic trick — a `width:0;height:0` box with three transparent border
//! sides and one colored side — draws a solid TRIANGLE (an arrow/marker glyph:
//! select ▼, disclosure ▸▾, stepper ▲▼). It works because the scene coordinator
//! now paints each SOLID border side as a **mitered trapezoid quad** (outer edge
//! full, inner edge inset by the two adjacent border widths) instead of an
//! axis-aligned rect: when a box collapses, the colored side's trapezoid folds
//! into a triangle, and transparent sides paint nothing.
//!
//! This example mirrors the coordinator's `border_fills` geometry to emit the
//! four cardinal arrows (▽ ▲ ▷ ◁) plus a normal 4-colour bordered box (whose
//! corners are now 45°-mitered), rasterizes them through the software
//! [`CpuRenderer`] (no GPU), verifies each triangle is filled at its centroid yet
//! empty at an outside corner, and writes `target/border_triangle_proof.png`.
//!
//! Run it:
//! `cargo run -p obscur-kit --example border_triangle_proof --features render-cpu`
#![forbid(unsafe_code)]

use std::path::PathBuf;

use obscur_kit::geometry::{Point, Rect, Size};
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
const VIEWPORT: Size = Size::new(480.0, 260.0);
/// The arrow ink (#333).
const INK: Color = Color::rgba(0.2, 0.2, 0.2, 1.0);

fn main() {
    println!("== obscur-kit border-triangle trick proof (headless CPU) ==\n");

    let clear = None; // a transparent border side paints nothing
    let ink = Some(INK);

    let mut builder = DisplayListBuilder::new();
    // Opaque white background so the triangles read clearly.
    builder.fill_rect(
        Rect::new(Point::new(0.0, 0.0), VIEWPORT),
        Brush::solid(Color::rgba(1.0, 1.0, 1.0, 1.0)),
    );

    // The four cardinal arrows. Each is a 0×0-content box whose BORDER BOX is
    // exactly its border widths: (left+right) × (top+bottom). One side is inked,
    // the other three transparent → a triangle. Widths are [top, right, bottom, left].
    // ▽ down — border-top inked (box 60×40).
    let down = Rect::new(Point::new(40.0, 40.0), Size::new(60.0, 40.0));
    emit_border(&mut builder, down, [40.0, 30.0, 0.0, 30.0], [ink, clear, clear, clear]);
    // ▲ up — border-bottom inked (box 60×40).
    let up = Rect::new(Point::new(140.0, 40.0), Size::new(60.0, 40.0));
    emit_border(&mut builder, up, [0.0, 30.0, 40.0, 30.0], [clear, clear, ink, clear]);
    // ▷ right — border-left inked (box 40×60).
    let right = Rect::new(Point::new(250.0, 30.0), Size::new(40.0, 60.0));
    emit_border(&mut builder, right, [30.0, 0.0, 30.0, 40.0], [clear, clear, clear, ink]);
    // ◁ left — border-right inked (box 40×60).
    let left = Rect::new(Point::new(340.0, 30.0), Size::new(40.0, 60.0));
    emit_border(&mut builder, left, [30.0, 40.0, 30.0, 0.0], [clear, ink, clear, clear]);

    // A normal 4-colour bordered box: an 8px border with a distinct colour per
    // side, so the 45° mitered corners are visible. Light fill behind it.
    let box_rect = Rect::new(Point::new(70.0, 150.0), Size::new(340.0, 80.0));
    builder.fill_rect(box_rect, Brush::solid(Color::rgba(0.93, 0.93, 0.95, 1.0)));
    let red = Some(Color::rgba(0.85, 0.20, 0.20, 1.0));
    let green = Some(Color::rgba(0.20, 0.60, 0.25, 1.0));
    let blue = Some(Color::rgba(0.20, 0.40, 0.80, 1.0));
    let orange = Some(Color::rgba(0.90, 0.55, 0.10, 1.0));
    emit_border(&mut builder, box_rect, [8.0, 8.0, 8.0, 8.0], [red, green, blue, orange]);

    let list = builder.build();
    println!(
        "[scene] {} paint command(s) · four arrows (▽ ▲ ▷ ◁) + one 4-colour bordered box",
        list.commands().len()
    );

    let pixmap = render(list);

    // Machine check: each arrow is FILLED at its centroid but EMPTY at an outside
    // corner of its bounding box (so it is a triangle, not a rect).
    verify_triangle(&pixmap, "▽ down", (70, 53), (43, 77));
    verify_triangle(&pixmap, "▲ up", (170, 67), (143, 43));
    verify_triangle(&pixmap, "▷ right", (263, 60), (287, 33));
    verify_triangle(&pixmap, "◁ left", (357, 60), (343, 33));

    let png = pixmap.encode_png().expect("encode png");
    let out = output_path("border_triangle_proof.png");
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).expect("create target dir");
    }
    std::fs::write(&out, &png).expect("write png");
    println!(
        "\n[png] wrote {} × {} → {} ({} bytes) — four triangles + a mitered bordered box",
        pixmap.width(),
        pixmap.height(),
        out.display(),
        png.len(),
    );
}

/// Emits a node's SOLID border as mitered trapezoid quads — a faithful copy of the
/// scene coordinator's `border_fills`. `widths`/`colors` are in top/right/bottom/
/// left order; a zero-width or `None` (transparent) side paints nothing.
fn emit_border(
    builder: &mut DisplayListBuilder,
    bounds: Rect,
    widths: [f32; 4],
    colors: [Option<Color>; 4],
) {
    let [top, right, bottom, left] = widths;
    let x = bounds.origin.x;
    let y = bounds.origin.y;
    let w = bounds.size.width;
    let h = bounds.size.height;
    // Outer corners (border box) and inner corners (inset by adjacent widths).
    let outer_tl = Point::new(x, y);
    let outer_tr = Point::new(x + w, y);
    let outer_br = Point::new(x + w, y + h);
    let outer_bl = Point::new(x, y + h);
    let inner_tl = Point::new(x + left, y + top);
    let inner_tr = Point::new(x + w - right, y + top);
    let inner_br = Point::new(x + w - right, y + h - bottom);
    let inner_bl = Point::new(x + left, y + h - bottom);

    let sides = [
        (top, colors[0], [outer_tl, outer_tr, inner_tr, inner_tl]),
        (right, colors[1], [outer_tr, outer_br, inner_br, inner_tr]),
        (bottom, colors[2], [outer_br, outer_bl, inner_bl, inner_br]),
        (left, colors[3], [outer_bl, outer_tl, inner_tl, inner_bl]),
    ];
    for (width, color, [a, b, c, d]) in sides {
        if width > 0.0 {
            if let Some(color) = color {
                let mut path = Path::new();
                path.move_to(a).line_to(b).line_to(c).line_to(d).close();
                builder.fill(path, Brush::solid(color));
            }
        }
    }
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

/// Asserts the arrow is INKED at `inside` (its centroid) yet EMPTY (white) at
/// `outside` (a corner of its bounding box the triangle does not cover).
fn verify_triangle(pixmap: &tiny_skia::Pixmap, name: &str, inside: (u32, u32), outside: (u32, u32)) {
    let inked = |(x, y): (u32, u32)| {
        pixmap
            .pixel(x, y)
            .is_some_and(|p| p.alpha() > 40 && p.red() < 120)
    };
    let inside_inked = inked(inside);
    let outside_inked = inked(outside);
    println!(
        "[verify] {name}: centroid {inside:?} inked={inside_inked}, corner {outside:?} inked={outside_inked}"
    );
    assert!(inside_inked, "{name}: expected the triangle centroid to be filled");
    assert!(
        !outside_inked,
        "{name}: expected the outside corner to be empty (a triangle, not a rect)"
    );
}

/// The workspace-relative output path `target/<name>`.
fn output_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join("target")
        .join(name)
}
