//! Software rasterization of outset and inset shadows with rounded geometry,
//! spread, offset and Gaussian blur.

use obscur_geometry::{CornerRadii, Point, Rect, Size};
use obscur_scene::color::Color;
use obscur_scene::path::Path;

use crate::renderer::skia_lowering::{solid_paint, to_sk_fill_rule, to_sk_path};

use super::gaussian_blur::{MAX_SIGMA, blur_layer};

/// Rasterizes a rounded shadow. Positive `blur` is the Gaussian sigma in device
/// pixels. The outset silhouette is blurred before the enclosing clip is applied.
/// The inset shadow fills the box with the complement of a blurred hole. Zero
/// sigma uses the direct fill and ring paths.
/// The scratch surface covers the visible shadow bounds and its kernel halo.
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
    if blur > 0.0
        && draw_blurred_shadow(
            pixmap, rect, radii, color, blur, spread, offset, inset, clip, transform,
        )
    {
        return;
    }
    // An inset shadow is an inner frame confined to the box, not an offset rect.
    if inset {
        draw_inset_shadow_approx(pixmap, rect, radii, color, spread, offset, clip, transform);
        return;
    }
    let Some(sk_path) = shadow_shape(rect, radii, spread, offset) else {
        return;
    };
    let Some(sk_paint) = solid_paint(color) else {
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

/// Constructs the spread-adjusted outline used for an outset shadow or an inset hole.
fn shadow_shape(
    rect: Rect,
    radii: CornerRadii,
    spread: f32,
    (ox, oy): (f32, f32),
) -> Option<tiny_skia::Path> {
    let x = rect.origin.x + ox - spread;
    let y = rect.origin.y + oy - spread;
    let w = 2.0f32.mul_add(spread, rect.size.width);
    let h = 2.0f32.mul_add(spread, rect.size.height);
    if w <= 0.0 || h <= 0.0 {
        return None;
    }
    let bounds = Rect::new(Point::new(x, y), Size::new(w, h));
    to_sk_path(&Path::rounded_rect(bounds, radii.spread_offset(spread)))
}

/// Blurs the rounded silhouette, or the hole punched out of an inset shadow.
#[allow(clippy::too_many_arguments)]
fn draw_blurred_shadow(
    target: &mut tiny_skia::Pixmap,
    rect: Rect,
    radii: CornerRadii,
    color: Color,
    sigma: f32,
    spread: f32,
    offset: (f32, f32),
    inset: bool,
    clip: Option<&tiny_skia::Mask>,
    transform: tiny_skia::Transform,
) -> bool {
    let color_paint = match solid_paint(color) {
        Some(paint) => paint,
        None => return true,
    };
    let silhouette = shadow_shape(rect, radii, if inset { -spread } else { spread }, offset);
    let outer = inset
        .then(|| to_sk_path(&Path::rounded_rect(rect, radii)))
        .flatten();
    let Some(outline) = (if inset {
        outer.as_ref()
    } else {
        silhouette.as_ref()
    }) else {
        return true;
    };
    let Some(device_outline) = outline.clone().transform(transform) else {
        return true;
    };
    let bounds = device_outline.bounds();
    let margin = (3.0 * sigma.min(MAX_SIGMA)).ceil() as i32;
    let spread = if inset { 0 } else { margin };
    let left = (bounds.left().floor() as i32).saturating_sub(spread).max(0);
    let top = (bounds.top().floor() as i32).saturating_sub(spread).max(0);
    let right = (bounds.right().ceil() as i32)
        .saturating_add(spread)
        .min(target.width() as i32);
    let bottom = (bounds.bottom().ceil() as i32)
        .saturating_add(spread)
        .min(target.height() as i32);
    if right <= left || bottom <= top {
        return true;
    }

    // Preserve the filter input just beyond the visible result, including
    // source geometry that lies outside the viewport.
    let x = left - margin;
    let y = top - margin;
    let width = (right - left) as u32 + 2 * margin as u32;
    let height = (bottom - top) as u32 + 2 * margin as u32;
    let Some(mut scratch) = tiny_skia::Pixmap::new(width, height) else {
        return false;
    };
    let local = transform.post_translate(-(x as f32), -(y as f32));
    if let Some(silhouette) = silhouette {
        let mut coverage = tiny_skia::Paint::default();
        coverage.set_color_rgba8(255, 255, 255, 255);
        let paint = if inset { &coverage } else { &color_paint };
        scratch.fill_path(
            &silhouette,
            paint,
            tiny_skia::FillRule::Winding,
            local,
            None,
        );
        blur_layer(&mut scratch, sigma);
    }

    if let Some(outer) = outer {
        let mut mask = tiny_skia::Mask::from_pixmap(scratch.as_ref(), tiny_skia::MaskType::Alpha);
        mask.invert();
        scratch.fill(tiny_skia::Color::TRANSPARENT);
        scratch.fill_path(
            &outer,
            &color_paint,
            tiny_skia::FillRule::Winding,
            local,
            Some(&mask),
        );
    }
    target.draw_pixmap(
        x,
        y,
        scratch.as_ref(),
        &tiny_skia::PixmapPaint::default(),
        tiny_skia::Transform::identity(),
        clip,
    );
    true
}

#[doc = include_str!("../../docs/api/renderer/shadow_approximation.doc/draw_inset_shadow_approx.md")]
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
    fn a_blurred_shadow_softens_its_edge() {
        let identity = tiny_skia::Transform::identity();
        let mut blurred = pixmap();
        draw_shadow_approx(
            &mut blurred,
            box_rect(),
            CornerRadii::uniform(0.0),
            black(),
            2.0,
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
        let faded_alpha = blurred.pixel(4, 10).expect("pixel").alpha();
        let full_alpha = crisp.pixel(10, 10).expect("pixel").alpha();
        assert_eq!(full_alpha, 255, "no blur means no softness to fake");
        assert!(
            faded_alpha < full_alpha,
            "a blurred shadow fades: {faded_alpha} vs {full_alpha}"
        );
    }

    #[test]
    fn a_shadow_outside_the_viewport_can_still_blur_into_it() {
        let mut target = pixmap();
        draw_shadow_approx(
            &mut target,
            box_rect(),
            CornerRadii::uniform(0.0),
            black(),
            2.0,
            0.0,
            (0.0, 0.0),
            false,
            None,
            tiny_skia::Transform::from_translate(-17.0, 0.0),
        );
        assert!(target.pixel(0, 10).unwrap().alpha() > 0);
        assert_eq!(target.pixel(7, 10).unwrap().alpha(), 0);
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
