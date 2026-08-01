#![doc = include_str!("dashed_border_proof.doc/_module.md")]
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

#[doc = include_str!("dashed_border_proof.doc/VIEWPORT.md")]
const VIEWPORT: Size = Size::new(360.0, 200.0);
#[doc = include_str!("dashed_border_proof.doc/BORDER.md")]
const BORDER: f32 = 2.0;

fn main() {
    println!("== obscur-kit border-style: dashed proof (headless CPU) ==\n");

    let ink = Color::rgba(0.169, 0.165, 0.153, 1.0); // #2b2a27

    // Two border boxes: a square one on the left, a rounded one on the right.
    let square = Rect::new(Point::new(30.0, 40.0), Size::new(120.0, 120.0));
    let rounded = Rect::new(Point::new(210.0, 40.0), Size::new(120.0, 120.0));

    let mut builder = DisplayListBuilder::new();
    // Opaque white background so the dashes and their gaps read clearly.
    builder.fill_rect(
        Rect::new(Point::new(0.0, 0.0), VIEWPORT),
        Brush::solid(Color::rgba(1.0, 1.0, 1.0, 1.0)),
    );
    // Each border: stroke the centerline (border box inset by width/2) dashed.
    builder.stroke(
        dashed_border_path(square, CornerRadii::default(), BORDER, false),
        border_stroke(ink),
    );
    builder.stroke(
        dashed_border_path(rounded, CornerRadii::uniform(24.0), BORDER, true),
        border_stroke(ink),
    );
    let list = builder.build();
    println!(
        "[scene] {} paint command(s) · square + rounded, border: 2px dashed #2b2a27",
        list.commands().len()
    );

    // Rasterize on the CPU.
    let pixmap = render(list);

    // Machine check: the square's top border scanline must alternate painted runs
    // and gaps. y just inside the top edge (border band is y∈[40,42)).
    verify_dashes(&pixmap, 41, square);

    let png = pixmap.encode_png().expect("encode png");
    let out = output_path("dashed_border_proof.png");
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).expect("create target dir");
    }
    std::fs::write(&out, &png).expect("write png");
    println!(
        "\n[png] wrote {} × {} → {} ({} bytes) — borders are VISIBLY DASHED",
        pixmap.width(),
        pixmap.height(),
        out.display(),
        png.len(),
    );
}

#[doc = include_str!("dashed_border_proof.doc/border_stroke.md")]
fn border_stroke(ink: Color) -> Stroke {
    let dash = 3.0 * BORDER;
    Stroke::new(Brush::solid(ink), BORDER).with_dash([dash, dash], 0.0)
}

#[doc = include_str!("dashed_border_proof.doc/dashed_border_path.md")]
fn dashed_border_path(bounds: Rect, radii: CornerRadii, width: f32, has_radius: bool) -> Path {
    let half = width / 2.0;
    let center = Rect::new(
        Point::new(bounds.origin.x + half, bounds.origin.y + half),
        Size::new(
            (bounds.size.width - width).max(0.0),
            (bounds.size.height - width).max(0.0),
        ),
    );
    if has_radius {
        Path::rounded_rect(center, radii.offset(-half))
    } else {
        Path::rect(center)
    }
}

#[doc = include_str!("dashed_border_proof.doc/render.md")]
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

#[doc = include_str!("dashed_border_proof.doc/verify_dashes.md")]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn verify_dashes(pixmap: &tiny_skia::Pixmap, y: u32, box_rect: Rect) {
    let x0 = box_rect.origin.x as u32;
    let x1 = (box_rect.origin.x + box_rect.size.width) as u32;
    let mut runs: Vec<(bool, u32)> = Vec::new();
    for x in x0..x1 {
        let painted = pixmap
            .pixel(x, y)
            .is_some_and(|p| p.alpha() > 40 && p.red() < 120);
        match runs.last_mut() {
            Some((ink, len)) if *ink == painted => *len += 1,
            _ => runs.push((painted, 1)),
        }
    }
    let dashes = runs.iter().filter(|(ink, _)| *ink).count();
    let gaps = runs.iter().filter(|(ink, _)| !*ink).count();
    println!("[verify] top border scanline y={y}: {dashes} painted dash run(s), {gaps} gap(s)");
    assert!(
        dashes >= 2 && gaps >= 1,
        "expected an alternating dash/gap pattern, got runs={runs:?} (border rendered solid?)",
    );
    println!("[verify] OK — the border is dashed, not solid");
}

#[doc = include_str!("dashed_border_proof.doc/output_path.md")]
fn output_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join("target")
        .join(name)
}
