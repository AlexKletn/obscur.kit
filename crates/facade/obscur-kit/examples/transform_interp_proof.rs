#![doc = include_str!("transform_interp_proof.doc/_module.md")]
#![forbid(unsafe_code)]

use std::path::PathBuf;

use obscur_kit::geometry::{AffineTransform, Point, Rect, Size};
use obscur_kit::motion::value::MotionValue;
use obscur_kit::render::frame::{FrameSubmission, RenderFrame};
use obscur_kit::render::target::{BitmapTarget, RenderTarget};
use obscur_kit::render::{Renderer, RendererId};
use obscur_kit::render_cpu::renderer::CpuRenderer;
use obscur_kit::scene::color::Color;
use obscur_kit::scene::command::{DisplayList, DisplayListBuilder, PaintCommand};
use obscur_kit::scene::paint::Brush;
use obscur_kit::scene::path::Path;
use obscur_kit::scene::scene::{SceneGeneration, SceneSnapshot};

#[doc = include_str!("transform_interp_proof.doc/VIEWPORT.md")]
const VIEWPORT: Size = Size::new(360.0, 240.0);
#[doc = include_str!("transform_interp_proof.doc/HALF.md")]
const HALF: f32 = 50.0;
#[doc = include_str!("transform_interp_proof.doc/CENTER.md")]
const CENTER: Point = Point::new(180.0, 120.0);
#[doc = include_str!("transform_interp_proof.doc/INK.md")]
const INK: Color = Color::rgba(0.10, 0.20, 0.75, 1.0);

fn main() {
    println!("== obscur-kit 2D transform interpolation proof (headless CPU) ==\n");

    // The two transition endpoints as Transform motion values.
    let from = MotionValue::transform([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]); // rotate(0°)
    let to = MotionValue::transform([0.0, 1.0, -1.0, 0.0, 0.0, 0.0]); // rotate(90°)

    // Blend at the midpoint through the motion engine (decompose→lerp→recompose).
    let mid = from
        .interpolate_checked(&to, 0.5)
        .and_then(|v| v.as_transform())
        .expect("matching Transform variants interpolate");
    println!("[motion] rotate(0°) ⊕ rotate(90°) @ t=0.5 = {mid:?}");

    // Assert the ENGINE result is rotate(45°) with scale preserved, not the lerp.
    let s = 0.5_f32.sqrt();
    let angle_deg = mid[1].atan2(mid[0]).to_degrees();
    let scale_x = mid[0].hypot(mid[1]);
    println!(
        "[motion] recovered angle = {angle_deg:.3}° · scaleX = {scale_x:.4} (naive lerp ≈ {s:.4})"
    );
    assert!(
        (angle_deg - 45.0).abs() <= 0.05,
        "expected rotate(45°), got {angle_deg}°"
    );
    assert!(
        (scale_x - 1.0).abs() <= 1e-3,
        "scale must be preserved at 1, got {scale_x}"
    );

    // Build the scene: white page, a faint reference outline of the un-rotated
    // box, then the filled square under the interpolated transform bracket.
    let list = build_scene(mid);
    println!(
        "[scene] {} paint command(s) · square rotated 45° about its centre",
        list.commands().len()
    );

    let pixmap = render(list);
    verify_diamond(&pixmap);

    let png = pixmap.encode_png().expect("encode png");
    let out = output_path("transform_interp_proof.png");
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).expect("create target dir");
    }
    std::fs::write(&out, &png).expect("write png");
    println!(
        "\n[png] wrote {} × {} → {} ({} bytes) — box is ROTATED 45° at FULL size",
        pixmap.width(),
        pixmap.height(),
        out.display(),
        png.len(),
    );
}

#[doc = include_str!("transform_interp_proof.doc/build_scene.md")]
fn build_scene(matrix: [f32; 6]) -> DisplayList {
    let mut builder = DisplayListBuilder::new();
    builder.fill_rect(
        Rect::new(Point::new(0.0, 0.0), VIEWPORT),
        Brush::solid(Color::rgba(1.0, 1.0, 1.0, 1.0)),
    );

    // The un-rotated square, centred on CENTER.
    let square = Rect::new(
        Point::new(CENTER.x - HALF, CENTER.y - HALF),
        Size::new(2.0 * HALF, 2.0 * HALF),
    );

    // The paint-path transform: translate(C) · matrix · translate(−C) so the
    // affine rotates about the box centre rather than the origin.
    let rot = AffineTransform::from_matrix(
        matrix[0], matrix[1], matrix[2], matrix[3], matrix[4], matrix[5],
    );
    let bracket = AffineTransform::translate(CENTER.x, CENTER.y)
        .multiply(rot)
        .multiply(AffineTransform::translate(-CENTER.x, -CENTER.y));

    let mut commands = builder.build().commands().to_vec();
    commands.push(PaintCommand::PushTransform(bracket));
    commands.push(PaintCommand::DrawPath {
        path: Path::rect(square),
        paint: obscur_kit::scene::paint::Paint::solid_fill(INK),
    });
    commands.push(PaintCommand::PopTransform);
    DisplayList::from_commands(commands)
}

#[doc = include_str!("transform_interp_proof.doc/render.md")]
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

#[doc = include_str!("transform_interp_proof.doc/verify_diamond.md")]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn verify_diamond(pixmap: &tiny_skia::Pixmap) {
    let is_ink = |x: u32, y: u32| {
        pixmap
            .pixel(x, y)
            .is_some_and(|p| p.alpha() > 40 && p.blue() > 120 && p.red() < 120)
    };

    let cx = CENTER.x as u32;
    let cy = CENTER.y as u32;
    assert!(is_ink(cx, cy), "box centre must be inked");

    // Scan the centre column downward from the top; first ink = diamond tip.
    let tip_y = (0..cy)
        .find(|&y| is_ink(cx, y))
        .expect("diamond top vertex must be painted in the centre column");
    let extent = f32::from(u16::try_from(cy - tip_y).expect("extent fits u16"));
    let expected = HALF * 2.0_f32.sqrt(); // 50·√2 ≈ 70.71
    println!(
        "[verify] centre-column top vertex is {extent}px above centre (expected ≈ {expected:.2}px)"
    );
    assert!(
        (extent - expected).abs() <= 3.0,
        "diamond half-diagonal must be ≈{expected:.2}px (rotated, full scale), got {extent}px",
    );

    // The un-rotated corner is outside the diamond → clear.
    let corner_x = (CENTER.x + HALF) as u32;
    let corner_y = (CENTER.y + HALF) as u32;
    assert!(
        !is_ink(corner_x, corner_y),
        "the un-rotated corner must be OUTSIDE the rotated diamond (empty)",
    );
    println!("[verify] OK — rotate(45°) diamond at full scale (not a naive-lerp shrink)");
}

#[doc = include_str!("transform_interp_proof.doc/output_path.md")]
fn output_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join("target")
        .join(name)
}
