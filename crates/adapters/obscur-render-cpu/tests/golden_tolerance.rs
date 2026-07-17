//! §7.14's raster golden tolerance, exercised against real CPU renders.
//!
//! §7.14: *"Raster golden tests are allowed with explicit tolerances for
//! anti-aliasing and font/backend variation."* §5.2 repeats it — *"Golden tests
//! use tolerances"* — under an **allowed differences** list naming
//! rasterization, anti-aliasing, font rendering details, and backend-specific
//! precision, while **scene geometry**, **clipping** and **paint order** are on
//! the *must match* side.
//!
//! Neither sentence names a metric, so [`ImageTolerance`] derives one from the
//! two variation *sources* the spec does name, and this file is where the
//! resulting numbers are justified. Every constant below was **measured** on the
//! renders this file produces, not chosen because it sounded safe.
//!
//! # The measurements this file's constants come from
//!
//! Scene: a 48×48 rounded rect (radius 14) filled red on a 64×64 transparent
//! surface — 4096 pixels, of which exactly **80 (1.95%)** have partial alpha,
//! i.e. lie on an anti-aliased corner arc. The straight edges are integer-
//! aligned and produce no AA at all.
//!
//! | change to the render                        | pixels past a delta-4 floor |
//! |---------------------------------------------|-----------------------------|
//! | same scene rendered twice                   | 0.00% (max delta 0)         |
//! | fill colour shifted by 4/255                | 0.00% (max delta exactly 4) |
//! | **worst-case AA disagreement (see below)**  | **1.95%** (max delta 240)   |
//! | corner radius 14 → 15                       | 2.64%                       |
//! | corner radius 14 → 16                       | 3.61%                       |
//! | **translated 1px in x**                     | **4.30%** (max delta 255)   |
//! | corner radius 14 → 20                       | 6.45%                       |
//! | fill colour shifted by 8/255                | 52.03% (max delta 8)        |
//!
//! Hence [`tolerance`]: a **delta-4 noise floor** (accepts the 4/255 colour
//! shift, rejects the 8/255 one) and a **3% differing cap** (accepts the 1.95%
//! AA band, rejects the 4.30% translation).
//!
//! # What this does NOT cover — read before trusting it
//!
//! - **The AA variation is a model, not a second rasterizer.** There is no
//!   second rasterization of these pixels to compare against: the CPU backend
//!   hardcodes `anti_alias: true` (`cpu_renderer.rs`), and Vello's rasterizer
//!   needs a GPU, which this workspace has no test target for (every GPU proof
//!   in the repo is a hand-run `[[example]]`). So a *true* cross-backend golden
//!   comparison is out of reach here, exactly as `clip_cross_backend.rs` says.
//!   What [`worst_case_aa_variation`] does instead is take the real render and
//!   replace **every partial-coverage pixel** with the most extreme value any
//!   rasterizer could hold about it — fully covered or fully empty, alternating
//!   both ways. That is *strictly worse* than a real AA disagreement (it is a
//!   240-delta swing on every band pixel at once), so accepting it implies
//!   accepting any milder one; it is not evidence that two named backends agree.
//! - **No golden ships on disk.** The comparison is render-vs-perturbed-render.
//!   Storing a reference PNG and diffing a fresh render against it across font
//!   versions — §7.14's motivating workload — is served *in principle* by this
//!   facility and is not exercised by any test in this repo.
//! - **The margin is real but modest.** Worst-case AA sits at 1.95% and the
//!   tightest regression rejected here is 4.30%, a factor of ~2.2. A corner
//!   radius drifting 14 → 15 lands at 2.64% and is **accepted** by this cap —
//!   see [`the_cap_does_not_catch_a_sub_pixel_radius_drift`], which pins that
//!   limitation rather than letting it be discovered later. A tolerance is a
//!   permission to differ, and this one really does grant it.
//! - **A tolerance is not an equivalence.** `compare_within` is not transitive:
//!   three renders can pairwise pass while the first and last differ wholesale.
//!   It answers "is this render still its golden", not "are these the same".

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

/// The surface both renders are made on.
const SURFACE: f32 = 64.0;

/// The golden's corner radius, in pixels.
const GOLDEN_RADIUS: f32 = 14.0;

/// The §7.14 tolerance under test, with both knobs measured above.
///
/// Both knobs are load-bearing, and *independently* so. Tightening each to
/// nothing fails a **disjoint** set of tests here — measured, not assumed:
///
/// - `max_differing_fraction` → `0.0` fails exactly the two AA-band cases
///   (`worst_case_anti_aliasing_variation_is_accepted` and
///   `the_cap_does_not_catch_a_sub_pixel_radius_drift`); the precision case
///   still passes, because nothing there exceeds the floor to be counted.
/// - `max_channel_delta` → `0` fails exactly
///   `a_backend_precision_shift_inside_the_noise_floor_is_accepted`; the AA case
///   still passes, because its band pixels differ by 240 either way.
///
/// The reverse direction does **not** separate, and no such claim is made:
/// loosening *either* knob alone (`255`, or `1.0`) waves through all four
/// rejection cases, because both knobs feed the one verdict — the floor decides
/// which pixels count, the cap decides how many may. Two permissions, one gate.
fn tolerance() -> ImageTolerance {
    ImageTolerance::new(4, 0.03)
}

/// The golden's fill.
fn red() -> Color {
    Color::rgba(1.0, 0.0, 0.0, 1.0)
}

/// The golden's rect.
fn base_rect() -> Rect {
    Rect::new(Point::new(8.0, 8.0), Size::new(48.0, 48.0))
}

/// A one-rounded-rect scene.
fn scene(rect: Rect, radius: f32, color: Color) -> FrameSubmission {
    let path = Path::rounded_rect(rect, CornerRadii::uniform(radius));
    let list = DisplayList::from_commands(vec![PaintCommand::DrawPath {
        path,
        paint: Paint::solid_fill(color),
    }]);
    let snapshot = SceneSnapshot::new(list, SceneGeneration::from_raw(1));
    FrameSubmission::new(RenderFrame::new(snapshot, Size::new(SURFACE, SURFACE)))
}

/// Rasterizes `submission` through the real software backend and captures it.
fn render(submission: &FrameSubmission) -> ImageSnapshot {
    let mut renderer = CpuRenderer::new(RendererId::from_raw(1));
    let target = RenderTarget::bitmap(BitmapTarget::new(Size::new(SURFACE, SURFACE)));
    renderer.render(submission, &target).expect("renders");
    let pixmap = renderer.pixmap().expect("a frame was rendered");
    ImageSnapshot::from_rgba(pixmap.width(), pixmap.height(), pixmap.data())
}

/// The golden: the render every case below is compared against.
fn golden() -> ImageSnapshot {
    render(&scene(base_rect(), GOLDEN_RADIUS, red()))
}

/// The indices of `img`'s partial-coverage pixels — the anti-aliased band.
///
/// Coverage is read off the alpha channel: the scene draws one opaque shape onto
/// a transparent surface, so alpha is 0 outside, 255 inside, and strictly
/// between exactly where the rasterizer anti-aliased an edge. This is the
/// operational definition of "where AA implementations are allowed to disagree".
fn aa_band(img: &ImageSnapshot) -> Vec<usize> {
    img.pixels()
        .expect("a captured render retains its raster")
        .chunks_exact(4)
        .enumerate()
        .filter(|(_, px)| px[3] != 0 && px[3] != 255)
        .map(|(index, _)| index)
        .collect()
}

/// `img` with every partial-coverage pixel replaced by the most extreme opinion
/// a rasterizer could hold about it, alternating fully-covered and fully-empty.
///
/// See this file's header: strictly worse than any real AA disagreement, and
/// confined to the band by construction — no full-coverage pixel is touched,
/// which is precisely what separates AA variation from a geometry change.
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

// ---------------------------------------------------------------------------
// What the tolerance must ACCEPT — one test per knob, and they are disjoint.
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// What the tolerance must REJECT — a loose tolerance is worse than a hash.
// ---------------------------------------------------------------------------

/// Red with the red channel pulled down by `delta`/255.
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

// ---------------------------------------------------------------------------
// Disclosed limits.
// ---------------------------------------------------------------------------

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
