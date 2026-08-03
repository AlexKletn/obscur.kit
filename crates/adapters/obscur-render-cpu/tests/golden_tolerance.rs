#![doc = include_str!("golden_tolerance.doc/_module.md")]

use obscur_geometry::{CornerRadii, Point, Rect, Size};
use obscur_headless::snapshot::{GoldenMismatch, ImageSnapshot, ImageTolerance};
use obscur_render::frame::{FrameSubmission, RenderFrame};
use obscur_render::target::{BitmapTarget, RenderTarget};
use obscur_render::{Renderer, RendererId};
use obscur_render_cpu::renderer::CpuRenderer;
use obscur_scene::color::Color;
use obscur_scene::command::{DisplayList, PaintCommand};
use obscur_scene::paint::Paint;
use obscur_scene::path::Path;
use obscur_scene::scene::{SceneGeneration, SceneSnapshot};

#[doc = include_str!("golden_tolerance.doc/SURFACE.md")]
const SURFACE: f32 = 64.0;

#[doc = include_str!("golden_tolerance.doc/GOLDEN_RADIUS.md")]
const GOLDEN_RADIUS: f32 = 14.0;

#[doc = include_str!("golden_tolerance.doc/tolerance.md")]
fn tolerance() -> ImageTolerance {
    ImageTolerance::new(4, 0.03)
}

#[doc = include_str!("golden_tolerance.doc/red.md")]
fn red() -> Color {
    Color::rgba(1.0, 0.0, 0.0, 1.0)
}

#[doc = include_str!("golden_tolerance.doc/base_rect.md")]
fn base_rect() -> Rect {
    Rect::new(Point::new(8.0, 8.0), Size::new(48.0, 48.0))
}

#[doc = include_str!("golden_tolerance.doc/scene.md")]
fn scene(rect: Rect, radius: f32, color: Color) -> FrameSubmission {
    let path = Path::rounded_rect(rect, CornerRadii::uniform(radius));
    let list = DisplayList::from_commands(vec![PaintCommand::DrawPath {
        path,
        paint: Paint::solid_fill(color),
    }]);
    let snapshot = SceneSnapshot::new(list, SceneGeneration::from_raw(1));
    FrameSubmission::new(RenderFrame::new(snapshot, Size::new(SURFACE, SURFACE)))
}

#[doc = include_str!("golden_tolerance.doc/render.md")]
fn render(submission: &FrameSubmission) -> ImageSnapshot {
    let mut renderer = CpuRenderer::new(RendererId::from_raw(1));
    let target = RenderTarget::bitmap(BitmapTarget::new(Size::new(SURFACE, SURFACE)));
    renderer.render(submission, &target).expect("renders");
    let pixmap = renderer.pixmap().expect("a frame was rendered");
    ImageSnapshot::from_rgba(pixmap.width(), pixmap.height(), pixmap.data())
}

#[doc = include_str!("golden_tolerance.doc/golden.md")]
fn golden() -> ImageSnapshot {
    render(&scene(base_rect(), GOLDEN_RADIUS, red()))
}

#[doc = include_str!("golden_tolerance.doc/aa_band.md")]
fn aa_band(img: &ImageSnapshot) -> Vec<usize> {
    img.pixels()
        .expect("a captured render retains its raster")
        .chunks_exact(4)
        .enumerate()
        .filter(|(_, px)| px[3] != 0 && px[3] != 255)
        .map(|(index, _)| index)
        .collect()
}

#[doc = include_str!("golden_tolerance.doc/worst_case_aa_variation.md")]
fn worst_case_aa_variation(img: &ImageSnapshot) -> ImageSnapshot {
    let mut bytes = img.pixels().expect("retained raster").to_vec();
    for (nth, &index) in aa_band(img).iter().enumerate() {
        let extreme: [u8; 4] = if nth % 2 == 0 {
            [255, 0, 0, 255]
        } else {
            [0, 0, 0, 0]
        };
        bytes[index * 4..index * 4 + 4].copy_from_slice(&extreme);
    }
    ImageSnapshot::from_rgba(img.width(), img.height(), &bytes)
}

#[test]
fn the_scene_really_is_anti_aliased_and_the_band_is_a_small_minority() {
    // The premise every case below rests on. If this scene had no AA band, the
    // tolerance tests would be vacuous — they would be tolerating nothing.
    let golden = golden();
    let band = aa_band(&golden);
    assert_eq!(band.len(), 80, "the four corner arcs, measured");
    let fraction = band.len() as f64 / 4096.0;
    assert!(
        (0.019..0.020).contains(&fraction),
        "AA band is {fraction:.4} of the image"
    );
}

// What the tolerance must ACCEPT — one test per knob, and they are disjoint.

#[test]
fn worst_case_anti_aliasing_variation_is_accepted() {
    // §5.2 lists anti-aliasing under *allowed differences*. This is the knob-2
    // (differing-fraction) half: a large delta confined to a small fraction.
    //
    // Load-bearing: set `tolerance()`'s fraction to 0.0 and this fails (1.95% > 0).
    // Setting its channel delta to 0 leaves it passing — the band pixels differ
    // by 240 either way — which is what makes this test knob-2-specific.
    let golden = golden();
    let varied = worst_case_aa_variation(&golden);

    assert_ne!(
        golden.pixel_hash(),
        varied.pixel_hash(),
        "the hash facility rejects this permitted difference — the whole point"
    );

    let diff = golden
        .compare_within(&varied, tolerance())
        .expect("AA variation is a difference §5.2 permits");
    assert_eq!(diff.differing_pixels(), 80, "confined to the band");
    assert_eq!(diff.max_channel_delta(), 240, "and violent on it");
}

#[test]
fn a_backend_precision_shift_inside_the_noise_floor_is_accepted() {
    // §5.2 lists backend-specific precision under *allowed differences*. This is
    // the knob-1 (channel-delta) half: a small delta across every pixel.
    //
    // Load-bearing: set `tolerance()`'s channel delta to 0 and this fails — the same
    // 4/255 shift then moves 52% of the image past the floor. Setting its
    // fraction to 0.0 leaves it passing, since nothing exceeds the floor at all.
    let golden = golden();
    let shifted = render(&scene(base_rect(), GOLDEN_RADIUS, off_red(4)));

    assert_ne!(
        golden.pixel_hash(),
        shifted.pixel_hash(),
        "the hash facility rejects this permitted difference too"
    );

    let diff = golden
        .compare_within(&shifted, tolerance())
        .expect("a 4/255 rounding difference is backend precision, not a regression");
    assert_eq!(
        diff.max_channel_delta(),
        4,
        "exactly at the floor, not under it"
    );
    assert_eq!(diff.differing_pixels(), 0);
}

// What the tolerance must REJECT — a loose tolerance is worse than a hash.

#[doc = include_str!("golden_tolerance.doc/off_red.md")]
fn off_red(delta: u8) -> Color {
    Color::rgba(1.0 - f32::from(delta) / 255.0, 0.0, 0.0, 1.0)
}

#[test]
fn a_whole_fill_colour_regression_is_rejected() {
    // Many pixels × small delta: the shape a lone differing-pixel cap cannot
    // catch, because "differs at all" cannot tell a rounding wobble from a
    // wrong colour. Only the noise floor separates 8/255 from 4/255.
    let golden = golden();
    let wrong = render(&scene(base_rect(), GOLDEN_RADIUS, off_red(8)));

    let Err(GoldenMismatch::TooManyDifferingPixels { diff, .. }) =
        golden.compare_within(&wrong, tolerance())
    else {
        panic!("an 8/255 fill regression must not pass a delta-4 noise floor");
    };
    assert!(
        diff.differing_fraction() > 0.5,
        "the whole fill moved: {:.4}",
        diff.differing_fraction()
    );
    assert_eq!(diff.max_channel_delta(), 8);
}

#[test]
fn a_one_pixel_geometry_shift_is_rejected() {
    // Few pixels × large delta: the shape that looks *most* like AA variation,
    // and the hardest thing this tolerance has to discriminate. §5.2 puts scene
    // geometry on the *must match* side, so a 1px translation is a regression
    // however small its footprint.
    //
    // This is the test that proves the 3% cap is not merely a rubber stamp: the
    // permitted AA band is 1.95% and this is 4.30%, so the cap sits in the gap.
    let golden = golden();
    let moved = render(&scene(
        Rect::new(Point::new(9.0, 8.0), Size::new(48.0, 48.0)),
        GOLDEN_RADIUS,
        red(),
    ));

    let Err(GoldenMismatch::TooManyDifferingPixels { diff, .. }) =
        golden.compare_within(&moved, tolerance())
    else {
        panic!("a 1px translation is a geometry change §5.2 requires to match");
    };
    assert!(
        diff.differing_fraction() > 0.04,
        "measured 4.30%: {:.4}",
        diff.differing_fraction()
    );
}

#[test]
fn a_corner_radius_regression_is_rejected() {
    // A shape change confined to the same corners the AA band occupies — it
    // cannot be caught by "ignore the corners", only by counting.
    let golden = golden();
    let rounder = render(&scene(base_rect(), 20.0, red()));

    assert!(
        matches!(
            golden.compare_within(&rounder, tolerance()),
            Err(GoldenMismatch::TooManyDifferingPixels { .. })
        ),
        "a 6px corner-radius change must not pass"
    );
}

#[test]
fn a_wholly_blank_render_is_rejected() {
    // The whole scene gone — 52.9% of the surface differing. This was named
    // `a_dropped_element_is_rejected`, which claimed far more than it proves: it
    // drops EVERYTHING, not an element. A single 10x10 element dropped from this
    // 64x64 surface moves 2.44% of the pixels and the tolerance ACCEPTS it (see
    // `ImageTolerance`'s "what this therefore does not catch"). The fraction cap
    // catches a diffuse regression; this test pins only that.
    let golden = golden();
    let blank = render(&FrameSubmission::new(RenderFrame::new(
        SceneSnapshot::new(DisplayList::new(), SceneGeneration::from_raw(1)),
        Size::new(SURFACE, SURFACE),
    )));

    assert!(matches!(
        golden.compare_within(&blank, tolerance()),
        Err(GoldenMismatch::TooManyDifferingPixels { .. })
    ));
}

#[test]
fn the_cap_does_not_catch_a_sub_pixel_radius_drift() {
    // Honesty, pinned as a test rather than left as prose: a 3% cap that accepts
    // a 1.95% AA band cannot also reject a 2.64% change. This drift PASSES.
    //
    // If a future change tightens the cap below 2.64%, this test fails and
    // forces the AA headroom to be re-measured rather than silently spent.
    let golden = golden();
    let drifted = render(&scene(base_rect(), 15.0, red()));

    let diff = golden
        .compare_within(&drifted, tolerance())
        .expect("a 1px radius drift is inside this cap — a real cost of tolerating AA");
    assert!(
        (0.026..0.027).contains(&diff.differing_fraction()),
        "measured 2.64%: {:.4}",
        diff.differing_fraction()
    );
}

#[test]
fn the_same_backend_twice_is_still_bit_exact() {
    // The tolerance is a permission for *cross*-backend variation, not a licence
    // for one backend to wobble. Same renderer, same scene → same bytes, which
    // is what the existing exact `PartialEq` in `image_snapshot_stage.rs` pins.
    let golden = golden();
    let diff = golden
        .compare_within(&golden.clone(), ImageTolerance::exact())
        .expect("a deterministic backend needs no tolerance at all");
    assert_eq!(diff.differing_pixels(), 0);
    assert_eq!(diff.max_channel_delta(), 0);
}

#[test]
fn a_golden_that_lost_its_pixels_fails_rather_than_passes() {
    // The facility's own footgun, closed: a snapshot rebuilt from a bare hash
    // cannot be compared under a tolerance, and reports that instead of
    // returning a verdict it never computed.
    let golden = golden();
    let hash_only = ImageSnapshot::from_parts(golden.width(), golden.height(), golden.pixel_hash());

    assert_eq!(
        hash_only.compare_within(&golden, tolerance()),
        Err(GoldenMismatch::PixelsUnavailable),
        "no pixels, no verdict"
    );
    // Exact equality still holds — which is exactly why the tolerant path must
    // not quietly borrow that answer.
    assert_eq!(hash_only, golden);
}
