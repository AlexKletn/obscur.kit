//! The two render backends must agree about clipping (spec §5.2).
//!
//! §5.2 draws the determinism boundary explicitly. "Must match across backends"
//! names **clipping**, **paint order**, and **scene geometry**; "allowed
//! differences" names **rasterization**, **anti-aliasing**, and backend-specific
//! precision. So the property under test here is the *clip decision* — for a
//! given scene, which points does drawing reach? — and not the pixels each
//! rasterizer produces for it.
//!
//! There was no cross-backend test in this repo at all, which is why `Clip::Mask`
//! was able to mean opposite things on the two backends: the CPU renderer painted
//! masked content, Vello clipped it entirely away, and nothing compared them.
//!
//! # How each backend is asked
//!
//! - **CPU** — the real [`CpuRenderer`] rasterizes the scene into a bitmap and a
//!   probe pixel is read back. This is the production path, end to end.
//! - **Vello** — the real [`ClipConverter`] resolves the clip to its outline and
//!   the probe point is tested against it. Vello's *rasterizer* needs a GPU, but
//!   the clip decision does not: the adapter is deliberately built so that "only
//!   rasterization is device-bound" (`vello_device.rs`), which puts the whole of
//!   §5.2's must-match clip geometry on this GPU-free path.
//!
//! # What this does NOT cover — read before trusting it
//!
//! - **No pixels from Vello.** Vello's clip outline is compared against the CPU's
//!   *rasterized coverage*, so this proves the two backends make the same clip
//!   decision, not that Vello's rasterizer honors it. A true golden-image
//!   comparison needs a GPU; this workspace has no GPU test target at all (every
//!   GPU proof in `obscur-kit` is an `[[example]]` run by hand), so a
//!   `#[test]` doing readback would silently no-op on any CI without an adapter —
//!   a worse lie than this stated gap.
//! - **Nesting and paint order are not compared**, only a single clip bracket.
//! - The **layer-clip** comparison shares the Vello limitation above. It does
//!   drive Vello's real `push_layer`: `a_layer_clip_reaches_vellos_push_layer`
//!   converts through the public `SceneConverter` and reads the clip back out of
//!   `Scene::encoding()`, which is GPU-free. What that cannot show is the
//!   *rasterized* result, which is the same gap as above.
//! - **Mask coverage is not tested because neither backend implements it.** Both
//!   now *ignore* mask coverage identically (the §5.8 fallback). Agreeing to
//!   ignore it is what this test pins; it is not evidence that masks work.
//! - Antialiased clip edges are avoided by probing well inside/outside a clip
//!   rather than on its boundary, since §5.2 lets AA differ.

use obscur_geometry::{CornerRadii, Point, Rect, Size};
use obscur_render::frame::{FrameSubmission, RenderFrame};
use obscur_render::target::{BitmapTarget, RenderTarget};
use obscur_render::{Renderer, RendererId};
use obscur_render_cpu::renderer::CpuRenderer;
use obscur_render_vello::conversion::{ClipConverter, SceneConverter};
use obscur_scene::clip::{Clip, Mask};
use obscur_scene::color::Color;
use obscur_scene::command::{DisplayList, PaintCommand};
use obscur_scene::layer::SceneLayer;
use obscur_scene::paint::Paint;
use obscur_scene::path::Path;
use obscur_scene::scene::{SceneGeneration, SceneSnapshot};
use vello::kurbo::{Point as KurboPoint, Shape};

/// The surface both backends are asked about.
const SURFACE: f32 = 64.0;

/// A closed rectangular outline.
fn rect_path(rect: Rect) -> Path {
    let mut path = Path::new();
    path.move_to(Point::new(rect.origin.x, rect.origin.y))
        .line_to(Point::new(rect.origin.x + rect.size.width, rect.origin.y))
        .line_to(Point::new(
            rect.origin.x + rect.size.width,
            rect.origin.y + rect.size.height,
        ))
        .line_to(Point::new(rect.origin.x, rect.origin.y + rect.size.height))
        .close();
    path
}

/// `clip` bracketing an opaque red fill over the whole surface. Whatever survives
/// the clip is exactly what the clip admits.
fn clipped_fill(clip: Clip) -> FrameSubmission {
    let list = DisplayList::from_commands(vec![
        PaintCommand::PushClip(clip),
        PaintCommand::DrawPath {
            path: rect_path(Rect::new(Point::new(0.0, 0.0), Size::new(SURFACE, SURFACE))),
            paint: Paint::solid_fill(Color::rgba(1.0, 0.0, 0.0, 1.0)),
        },
        PaintCommand::PopClip,
    ]);
    let snapshot = SceneSnapshot::new(list, SceneGeneration::from_raw(1));
    FrameSubmission::new(RenderFrame::new(snapshot, Size::new(SURFACE, SURFACE)))
}

/// Whether the CPU backend's rasterized output is painted at `point` — the real
/// renderer, asked the only way a rasterizer can be asked. Probe points are pixel
/// centers, so the two backends are asked about the same spot without a cast.
fn cpu_paints(clip: Clip, point: (u32, u32)) -> bool {
    let mut renderer = CpuRenderer::new(RendererId::from_raw(1));
    let target = RenderTarget::bitmap(BitmapTarget::new(Size::new(SURFACE, SURFACE)));
    renderer
        .render(&clipped_fill(clip), &target)
        .expect("the CPU backend renders a 64x64 target");
    let pixmap = renderer.pixmap().expect("a frame was rendered");
    let pixel = pixmap
        .pixel(point.0, point.1)
        .expect("probe point is inside the surface");
    pixel.alpha() > 0
}

/// Whether the Vello backend's clip admits `point`.
fn vello_admits(clip: &Clip, point: (u32, u32)) -> bool {
    ClipConverter::default()
        .shape(clip)
        .contains(KurboPoint::new(f64::from(point.0), f64::from(point.1)))
}

/// The same fill, but with the clip riding on a [`SceneLayer`] instead of being
/// bracketed by a `PushClip` — the shape the coordinator actually emits.
///
/// `opacity` is a parameter because the CPU backend takes two different paths: a
/// layer below `1.0` rasterizes its subtree into an offscreen buffer and
/// composites it back, while a trivial layer is a passthrough. A layer clip has to
/// be honored on both.
fn layer_clipped_fill(clip: Clip, opacity: f32) -> FrameSubmission {
    let list = DisplayList::from_commands(vec![
        PaintCommand::PushLayer(SceneLayer::new().with_clip(clip).with_opacity(opacity)),
        PaintCommand::DrawPath {
            path: rect_path(Rect::new(Point::new(0.0, 0.0), Size::new(SURFACE, SURFACE))),
            paint: Paint::solid_fill(Color::rgba(1.0, 0.0, 0.0, 1.0)),
        },
        PaintCommand::PopLayer,
    ]);
    let snapshot = SceneSnapshot::new(list, SceneGeneration::from_raw(1));
    FrameSubmission::new(RenderFrame::new(snapshot, Size::new(SURFACE, SURFACE)))
}

/// Whether the CPU backend paints at `point` when the clip rides on the layer.
fn cpu_paints_layer_clip(clip: Clip, opacity: f32, point: (u32, u32)) -> bool {
    let mut renderer = CpuRenderer::new(RendererId::from_raw(1));
    let target = RenderTarget::bitmap(BitmapTarget::new(Size::new(SURFACE, SURFACE)));
    renderer
        .render(&layer_clipped_fill(clip, opacity), &target)
        .expect("the CPU backend renders a 64x64 target");
    let pixmap = renderer.pixmap().expect("a frame was rendered");
    let pixel = pixmap
        .pixel(point.0, point.1)
        .expect("probe point is inside the surface");
    pixel.alpha() > 0
}

/// The cases both backends must answer identically, each with a probe point and
/// the answer the domain requires.
///
/// `Clip::Mask` is the regression this test was written for. The domain settles
/// which answer is correct: a `Mask` "does not carry the source itself"
/// (`mask.rs`) and `Clip::bounds()` is `None` for a mask because "a mask's extent
/// depends on its source content, which is not known to the clip itself"
/// (`clip.rs`). A mask clip therefore states no extent, and a backend may not
/// invent one — least of all the empty extent, which throws the content away.
fn cases() -> Vec<(&'static str, Clip, (u32, u32), bool)> {
    let inner = Rect::new(Point::new(16.0, 16.0), Size::new(32.0, 32.0));
    vec![
        ("rect: inside", Clip::rect(inner), (32, 32), true),
        ("rect: outside", Clip::rect(inner), (4, 4), false),
        (
            "rounded_rect: center is inside",
            Clip::rounded_rect(inner, CornerRadii::uniform(8.0)),
            (32, 32),
            true,
        ),
        (
            "rounded_rect: cut corner is outside",
            Clip::rounded_rect(inner, CornerRadii::uniform(8.0)),
            (17, 17),
            false,
        ),
        ("path: inside", Clip::path(rect_path(inner)), (32, 32), true),
        ("path: outside", Clip::path(rect_path(inner)), (4, 4), false),
        // A mask states no extent, so it confines nothing and the fill survives.
        (
            "luminance mask admits content",
            Clip::mask(Mask::luminance()),
            (32, 32),
            true,
        ),
        (
            "alpha mask admits content",
            Clip::mask(Mask::alpha()),
            (8, 8),
            true,
        ),
        // The mirror image of the mask case, and the reason the two must not be
        // conflated: an empty outline states an extent, and states that it is
        // empty. Nothing passes. (A mask states no extent at all, so everything
        // passes.) "No bounds known" and "bounds known to be empty" are opposite
        // instructions that both arrive as an absence.
        (
            "empty path clip admits nothing",
            Clip::path(Path::new()),
            (32, 32),
            false,
        ),
        (
            "single-moveto path clip admits nothing",
            Clip::path({
                let mut path = Path::new();
                path.move_to(Point::new(10.0, 10.0));
                path
            }),
            (32, 32),
            false,
        ),
        (
            "zero-size rect clip admits nothing",
            Clip::rect(Rect::new(Point::new(10.0, 10.0), Size::new(0.0, 0.0))),
            (32, 32),
            false,
        ),
    ]
}

/// The test that did not exist: the same scene, both backends, one answer.
#[test]
fn backends_agree_on_every_clip_variant() {
    for (name, clip, point, expected) in cases() {
        let cpu = cpu_paints(clip.clone(), point);
        let vello = vello_admits(&clip, point);

        assert_eq!(
            cpu, vello,
            "{name}: backends disagree at {point:?} — CPU paints={cpu}, Vello admits={vello} \
             (spec §5.2 requires clipping to match across backends)"
        );
        assert_eq!(
            cpu, expected,
            "{name}: both backends agree at {point:?}, and both are wrong \
             (the domain requires admitted={expected})"
        );
    }
}

/// A clip on a [`SceneLayer`] must confine drawing exactly as the same clip
/// bracketed by `PushClip` does, on both backends.
///
/// This is the divergence next door to §5-D, and a larger one: Vello honored
/// `SceneLayer::clip()` (`push_layer`'s bounding shape) and the CPU backend
/// dropped it on the floor, applying only the layer's opacity. It is not an edge
/// case — the coordinator brackets every `filter: blur()` / color-matrix group
/// with `with_clip`, so a real blurred element painted outside its clip on CPU and
/// inside it on Vello.
///
/// Both opacities are exercised because they are different code paths in the CPU
/// backend: a reduced-opacity layer rasterizes into an offscreen buffer, a trivial
/// one does not.
#[test]
fn backends_agree_when_the_clip_rides_on_the_layer() {
    for (name, clip, point, expected) in cases() {
        for opacity in [1.0, 0.5] {
            let cpu = cpu_paints_layer_clip(clip.clone(), opacity, point);
            let vello = vello_admits(&clip, point);

            assert_eq!(
                cpu, vello,
                "{name} @ layer opacity {opacity}: backends disagree at {point:?} — \
                 CPU paints={cpu}, Vello admits={vello} \
                 (spec §5.2 requires clipping to match across backends)"
            );
            assert_eq!(
                cpu, expected,
                "{name} @ layer opacity {opacity}: both backends agree at {point:?}, \
                 and both are wrong (the domain requires admitted={expected})"
            );

            // The clip must not depend on how it was delivered: a layer clip and a
            // clip command naming the same region are the same instruction. Both
            // sides of this are the real CPU renderer, so this half needs no GPU
            // and no stand-in.
            assert_eq!(
                cpu,
                cpu_paints(clip.clone(), point),
                "{name} @ layer opacity {opacity}: the CPU backend answers differently \
                 depending on whether the clip rides on the layer or on a PushClip"
            );
        }
    }
}

/// A layer's clip must stop confining drawing once the layer is popped.
///
/// The other direction of honoring a layer clip, and the one a single-bracket
/// scene cannot see: a clip that is pushed but never popped confines everything
/// drawn afterwards too, which is just as wrong as ignoring it and would let the
/// push half of the fix pass this file alone. The scene draws *outside* the
/// layer's clip after `PopLayer`; that paint must survive.
#[test]
fn a_layer_clip_does_not_outlive_its_layer() {
    let inner = Rect::new(Point::new(16.0, 16.0), Size::new(32.0, 32.0));
    let outside = Rect::new(Point::new(0.0, 0.0), Size::new(8.0, 8.0));
    let list = DisplayList::from_commands(vec![
        PaintCommand::PushLayer(SceneLayer::new().with_clip(Clip::rect(inner))),
        PaintCommand::DrawPath {
            path: rect_path(Rect::new(Point::new(0.0, 0.0), Size::new(SURFACE, SURFACE))),
            paint: Paint::solid_fill(Color::rgba(1.0, 0.0, 0.0, 1.0)),
        },
        PaintCommand::PopLayer,
        // Wholly outside the layer's clip, and drawn after it closed.
        PaintCommand::DrawPath {
            path: rect_path(outside),
            paint: Paint::solid_fill(Color::rgba(0.0, 1.0, 0.0, 1.0)),
        },
    ]);
    let snapshot = SceneSnapshot::new(list, SceneGeneration::from_raw(1));
    let submission = FrameSubmission::new(RenderFrame::new(snapshot, Size::new(SURFACE, SURFACE)));

    let mut renderer = CpuRenderer::new(RendererId::from_raw(1));
    let target = RenderTarget::bitmap(BitmapTarget::new(Size::new(SURFACE, SURFACE)));
    renderer
        .render(&submission, &target)
        .expect("the CPU backend renders a 64x64 target");
    let pixmap = renderer.pixmap().expect("a frame was rendered");

    assert!(
        pixmap.pixel(4, 4).expect("inside the surface").alpha() > 0,
        "the layer's clip outlived its PopLayer and swallowed later drawing"
    );
}

/// A layer's clip reaches Vello's real `push_layer`, carrying the geometry the
/// domain asked for.
///
/// The other layer test asks `ClipConverter` what Vello *would* resolve, which
/// proves the rule and not the plumbing: a `push_layer` that dropped `clip()` on
/// the floor — the CPU backend's exact bug — would pass it. This drives the public
/// `SceneConverter::convert` and reads the clip back out of `Scene::encoding()`,
/// so the assertion is that the converter really encoded it.
///
/// No GPU: `encoding()` is the scene's CPU-side buffer, populated by `push_layer`
/// before anything is submitted. This is the one part of the Vello comparison that
/// does not need an adapter.
#[test]
fn a_layer_clip_reaches_vellos_push_layer() {
    let inner = Rect::new(Point::new(16.0, 16.0), Size::new(32.0, 32.0));
    let converter = SceneConverter::new();

    let layer = converter.convert(&DisplayList::from_commands(vec![
        PaintCommand::PushLayer(SceneLayer::new().with_clip(Clip::rect(inner))),
        PaintCommand::DrawPath {
            path: rect_path(Rect::new(Point::new(0.0, 0.0), Size::new(SURFACE, SURFACE))),
            paint: Paint::solid_fill(Color::rgba(1.0, 0.0, 0.0, 1.0)),
        },
        PaintCommand::PopLayer,
    ]));
    assert!(
        layer.encoding().n_clips > 0,
        "push_layer encoded no clip at all — the layer's clip never reached Vello"
    );

    // Delivery-independence, the same property the CPU half asserts: the same
    // region as a `PushClip` bracket must encode the same geometry.
    let bracketed = converter.convert(&DisplayList::from_commands(vec![
        PaintCommand::PushClip(Clip::rect(inner)),
        PaintCommand::DrawPath {
            path: rect_path(Rect::new(Point::new(0.0, 0.0), Size::new(SURFACE, SURFACE))),
            paint: Paint::solid_fill(Color::rgba(1.0, 0.0, 0.0, 1.0)),
        },
        PaintCommand::PopClip,
    ]));
    assert_eq!(
        layer.encoding().path_data,
        bracketed.encoding().path_data,
        "a layer clip and a PushClip of the same region encoded different geometry"
    );
}

/// The §5-D regression, stated on its own so a failure names the bug.
///
/// Vello resolved a mask clip to an empty `BezPath`, which clips *everything*
/// away, while the CPU backend inherited the enclosing clip and painted. Same
/// scene, opposite pixels.
#[test]
fn a_mask_clip_does_not_erase_content_on_either_backend() {
    for mask in [Mask::luminance(), Mask::alpha()] {
        let clip = Clip::mask(mask);
        assert!(
            cpu_paints(clip.clone(), (32, 32)),
            "{mask:?}: CPU backend erased masked content"
        );
        assert!(
            vello_admits(&clip, (32, 32)),
            "{mask:?}: Vello backend erased masked content"
        );
    }
}
