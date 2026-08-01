#![doc = include_str!("shadow_approximation.doc/_module.md")]

use obscur_geometry::{CornerRadii, Point, Rect, Size};
use obscur_scene::color::Color;
use obscur_scene::path::Path;

use crate::renderer::skia_lowering::{solid_paint, to_sk_fill_rule, to_sk_path};

#[doc = include_str!("shadow_approximation.doc/SHADOW_APPROX_ALPHA.md")]
const SHADOW_APPROX_ALPHA: f32 = 0.5;

#[doc = include_str!("shadow_approximation.doc/draw_shadow_approx.md")]
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_shadow_approx(
    pixmap: &mut tiny_skia::Pixmap,
    rect: Rect,
    radii: CornerRadii,
    color: Color,
    blur: f32,
    spread: f32,
    offset: (f32, f32),
    inset: bool,
    clip: Option<&tiny_skia::Mask>,
    transform: tiny_skia::Transform,
) {
    // An inset shadow is an inner frame confined to the box, not an offset rect.
    if inset {
        draw_inset_shadow_approx(pixmap, rect, radii, color, spread, offset, clip, transform);
        return;
    }
    // Drop shadows inflate by `spread`.
    let inflate = spread;
    let (ox, oy) = offset;
    let x = rect.origin.x + ox - inflate;
    let y = rect.origin.y + oy - inflate;
    let w = 2.0f32.mul_add(inflate, rect.size.width);
    let h = 2.0f32.mul_add(inflate, rect.size.height);
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    let shadow_rect = Rect::new(Point::new(x, y), Size::new(w, h));
    // CSS Backgrounds 3 §6.1.1: an outset shadow's corner radii are the box radii
    // grown by `+spread` (a sharp corner stays sharp), so the rounded corners on
    // the spread-inflated rect stay concentric with the box's corners — with
    // §6.1.1's easing where a radius is smaller than the spread, which is why this
    // is `spread_offset` and not the raw `offset` primitive §4.2 uses.
    let path = Path::rounded_rect(shadow_rect, radii.spread_offset(spread));
    let Some(sk_path) = to_sk_path(&path) else {
        return;
    };
    // The alpha fade fakes the softness a real gaussian blur would give. A shadow
    // with **no blur** (`blur == 0`) has a crisp, fully-opaque edge — there is no
    // softness to fake — so it keeps the color's own alpha, exactly as the inset
    // path does for its exact ring. Only a blurred shadow is faded.
    let alpha = if blur > 0.0 {
        color.a * SHADOW_APPROX_ALPHA
    } else {
        color.a
    };
    let faded = Color::rgba(color.r, color.g, color.b, alpha);
    let Some(sk_paint) = solid_paint(faded) else {
        return;
    };
    pixmap.fill_path(
        &sk_path,
        &sk_paint,
        tiny_skia::FillRule::Winding,
        transform,
        clip,
    );
}

#[doc = include_str!("shadow_approximation.doc/draw_inset_shadow_approx.md")]
#[allow(clippy::too_many_arguments)]
fn draw_inset_shadow_approx(
    pixmap: &mut tiny_skia::Pixmap,
    rect: Rect,
    radii: CornerRadii,
    color: Color,
    spread: f32,
    offset: (f32, f32),
    clip: Option<&tiny_skia::Mask>,
    transform: tiny_skia::Transform,
) {
    if rect.size.width <= 0.0 || rect.size.height <= 0.0 {
        return;
    }
    let (ox, oy) = offset;
    // The lit hole: the box deflated by `spread`, shifted by `offset`.
    let ix = rect.origin.x + ox + spread;
    let iy = rect.origin.y + oy + spread;
    let iw = 2.0f32.mul_add(-spread, rect.size.width);
    let ih = 2.0f32.mul_add(-spread, rect.size.height);
    let Some(sk_paint) = solid_paint(color) else {
        return;
    };
    // A collapsed hole → the whole box interior is shadow (fill it solid).
    let path = if iw <= 0.0 || ih <= 0.0 {
        Path::rounded_rect(rect, radii)
    } else {
        let inner = Rect::new(Point::new(ix, iy), Size::new(iw, ih));
        // CSS Backgrounds 3 §6.1.1: an inset shadow CONTRACTS by `spread`, so its
        // radii take `−spread` (a sharp corner stays sharp) and the lit hole stays
        // concentric with the box's rounded corners. `spread_offset` handles the
        // sign: a positive spread shrinks here and is a plain subtraction, while a
        // negative authored spread grows the hole and takes §6.1.1's easing —
        // the spec's "in the case of an inner shadow, less than the absolute value
        // of a negative spread distance" trigger.
        Path::rounded_rect_ring(rect, radii, inner, radii.spread_offset(-spread))
    };
    let Some(sk_path) = to_sk_path(&path) else {
        return;
    };
    pixmap.fill_path(
        &sk_path,
        &sk_paint,
        to_sk_fill_rule(path.fill_rule()),
        transform,
        clip,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 20x20 transparent pixmap to rasterize a shadow into.
    fn pixmap() -> tiny_skia::Pixmap {
        tiny_skia::Pixmap::new(20, 20).expect("20x20 pixmap allocates")
    }

    fn black() -> Color {
        Color::rgba(0.0, 0.0, 0.0, 1.0)
    }

    fn box_rect() -> Rect {
        Rect::new(Point::new(4.0, 4.0), Size::new(12.0, 12.0))
    }

    #[test]
    fn a_blurred_shadow_is_faded_and_an_unblurred_one_keeps_its_alpha() {
        let identity = tiny_skia::Transform::identity();
        let mut blurred = pixmap();
        draw_shadow_approx(
            &mut blurred,
            box_rect(),
            CornerRadii::uniform(0.0),
            black(),
            4.0,
            0.0,
            (0.0, 0.0),
            false,
            None,
            identity,
        );
        let mut crisp = pixmap();
        draw_shadow_approx(
            &mut crisp,
            box_rect(),
            CornerRadii::uniform(0.0),
            black(),
            0.0,
            0.0,
            (0.0, 0.0),
            false,
            None,
            identity,
        );
        let faded_alpha = blurred.pixel(10, 10).expect("pixel").alpha();
        let full_alpha = crisp.pixel(10, 10).expect("pixel").alpha();
        assert_eq!(full_alpha, 255, "no blur means no softness to fake");
        assert!(
            faded_alpha < full_alpha,
            "a blurred shadow fades: {faded_alpha} vs {full_alpha}"
        );
    }

    #[test]
    fn a_degenerate_box_draws_nothing() {
        let mut target = pixmap();
        draw_shadow_approx(
            &mut target,
            Rect::new(Point::new(4.0, 4.0), Size::new(0.0, 0.0)),
            CornerRadii::uniform(0.0),
            black(),
            0.0,
            0.0,
            (0.0, 0.0),
            false,
            None,
            tiny_skia::Transform::identity(),
        );
        assert!(
            target.pixels().iter().all(|p| p.alpha() == 0),
            "a zero-area box leaves the buffer untouched"
        );
    }

    #[test]
    fn an_inset_shadow_leaves_a_hole_and_never_bleeds_outside_the_box() {
        let mut target = pixmap();
        draw_shadow_approx(
            &mut target,
            box_rect(),
            CornerRadii::uniform(0.0),
            black(),
            0.0,
            2.0,
            (0.0, 0.0),
            true,
            None,
            tiny_skia::Transform::identity(),
        );
        // Inside the ring band (just within the border box) is shadow.
        assert!(
            target.pixel(5, 10).expect("pixel").alpha() > 200,
            "the inner frame is painted"
        );
        // The lit hole at the centre is untouched.
        assert_eq!(
            target.pixel(10, 10).expect("pixel").alpha(),
            0,
            "the hole stays lit"
        );
        // Outside the border box nothing is painted at all.
        assert_eq!(
            target.pixel(1, 1).expect("pixel").alpha(),
            0,
            "an inset shadow never bleeds outside its box"
        );
    }

    #[test]
    fn a_collapsed_hole_fills_the_whole_box_interior() {
        let mut target = pixmap();
        draw_shadow_approx(
            &mut target,
            box_rect(),
            CornerRadii::uniform(0.0),
            black(),
            0.0,
            100.0,
            (0.0, 0.0),
            true,
            None,
            tiny_skia::Transform::identity(),
        );
        assert!(
            target.pixel(10, 10).expect("pixel").alpha() > 200,
            "a huge spread collapses the hole and fills the box"
        );
    }
}
