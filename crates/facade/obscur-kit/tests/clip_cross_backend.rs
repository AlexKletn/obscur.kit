#![doc = include_str!("clip_cross_backend.doc/_module.md")]

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

#[doc = include_str!("clip_cross_backend.doc/SURFACE.md")]
const SURFACE: f32 = 64.0;

#[doc = include_str!("clip_cross_backend.doc/ELLIPSE_BOX.md")]
const ELLIPSE_BOX: Rect = Rect::new(Point::new(0.0, 0.0), Size::new(64.0, 32.0));

#[doc = include_str!("clip_cross_backend.doc/rect_path.md")]
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

#[doc = include_str!("clip_cross_backend.doc/clipped_fill.md")]
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

#[doc = include_str!("clip_cross_backend.doc/cpu_paints.md")]
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

#[doc = include_str!("clip_cross_backend.doc/vello_admits.md")]
fn vello_admits(clip: &Clip, point: (u32, u32)) -> bool {
    ClipConverter::default()
        .shape(clip)
        .contains(KurboPoint::new(f64::from(point.0), f64::from(point.1)))
}

#[doc = include_str!("clip_cross_backend.doc/layer_clipped_fill.md")]
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

#[doc = include_str!("clip_cross_backend.doc/cpu_paints_layer_clip.md")]
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

#[doc = include_str!("clip_cross_backend.doc/cases.md")]
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
        // `border-radius: 50%` on a NON-SQUARE box — the circle idiom (audit 4-A).
        // A 64×32 box with radii (32, 16) per corner is a true ellipse, and it is a
        // shape no single scalar radius can express, so the two backends can only
        // agree here by both honoring the two axes. They route through the same
        // `Path::rounded_rect`, which is exactly why they do; this pins that they
        // keep doing so, since §5.2 puts scene geometry under "must match".
        //
        // Probes sit well clear of the outline (AA is an allowed difference): the
        // centre is deep inside, and (3, 3) is outside the ellipse — but INSIDE the
        // 64×32 bounding box, so it is only excluded by the curve itself.
        (
            "50% ellipse: center is inside",
            Clip::rounded_rect(ELLIPSE_BOX, CornerRadii::elliptical(32.0, 16.0)),
            (32, 16),
            true,
        ),
        (
            "50% ellipse: corner of the box is outside the curve",
            Clip::rounded_rect(ELLIPSE_BOX, CornerRadii::elliptical(32.0, 16.0)),
            (3, 3),
            false,
        ),
        // The scalar discriminators, and the reason these two points and not a
        // prettier pair. EVERY plausible scalar implementation of `50%` on this box
        // collapses to the SAME shape — a stadium with r = 16:
        //
        //   * circular `50%` → uniform(32) → §4.5 gives f = 32/64 → uniform(16);
        //   * both axes against the WIDTH → (32, 32) → f = 1/2 → uniform(16);
        //   * both axes against the HEIGHT → (16, 16) → f = 1 → uniform(16).
        //
        // The true ellipse is strictly *contained* in that stadium, so no point can
        // discriminate by being admitted. These two are the other direction: inside
        // the stadium, outside the ellipse. A circular backend admits them; the
        // ellipse must reject them. Both sit >3px clear of the ellipse's outline,
        // so no rasterizer's AA can decide the verdict (§5.2 lets AA differ).
        (
            "50% ellipse: rejects a point a scalar radius would admit (near-corner)",
            Clip::rounded_rect(ELLIPSE_BOX, CornerRadii::elliptical(32.0, 16.0)),
            (11, 1),
            false,
        ),
        (
            "50% ellipse: rejects a point a scalar radius would admit (diagonal)",
            Clip::rounded_rect(ELLIPSE_BOX, CornerRadii::elliptical(32.0, 16.0)),
            (7, 3),
            false,
        ),
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

#[doc = include_str!("clip_cross_backend.doc/backends_agree_on_every_clip_variant.md")]
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

#[doc = include_str!("clip_cross_backend.doc/backends_agree_when_the_clip_rides_on_the_layer.md")]
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

#[doc = include_str!("clip_cross_backend.doc/a_layer_clip_does_not_outlive_its_layer.md")]
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

#[doc = include_str!("clip_cross_backend.doc/a_layer_clip_reaches_vellos_push_layer.md")]
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

#[doc = include_str!("clip_cross_backend.doc/a_mask_clip_does_not_erase_content_on_either_backend.md")]
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
