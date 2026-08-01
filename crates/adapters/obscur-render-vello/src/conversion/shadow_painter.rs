#![doc = include_str!("shadow_painter.doc/_module.md")]

use vello::Scene;
use vello::kurbo::{Affine, Rect as KurboRect};
use vello::peniko;

use super::color_converter::ColorConverter;

#[doc = include_str!("shadow_painter.doc/draw_shadow.md")]
#[allow(clippy::too_many_arguments)]
pub(super) fn draw_shadow(
    scene: &mut Scene,
    rect: &obscur_geometry::Rect,
    radii: &obscur_geometry::CornerRadii,
    color: obscur_scene::color::Color,
    blur: f32,
    spread: f32,
    offset: (f32, f32),
    inset: bool,
    transform: Affine,
) {
    // An inset shadow is an *inner* shadow confined to the box, not an offset
    // rounded rect — it takes its own path so it is clipped to the box interior
    // and painted as a soft frame rather than a filled blob.
    if inset {
        draw_inset_shadow(scene, rect, radii, color, blur, spread, offset, transform);
        return;
    }
    // Drop shadows inflate by `spread`.
    let inflate = f64::from(spread);
    let (ox, oy) = (f64::from(offset.0), f64::from(offset.1));
    let x0 = f64::from(rect.origin.x) + ox - inflate;
    let y0 = f64::from(rect.origin.y) + oy - inflate;
    let x1 = f64::from(rect.origin.x + rect.size.width) + ox + inflate;
    let y1 = f64::from(rect.origin.y + rect.size.height) + oy + inflate;
    if x1 <= x0 || y1 <= y0 {
        return;
    }
    let kurbo_rect = KurboRect::new(x0, y0, x1, y1);
    // Spec §6.1.1: an outset shadow's corner radii are the box radii grown by
    // `+spread` (a sharp corner stays sharp) so the rounded corners stay
    // concentric with the box, eased where a radius is smaller than the spread.
    let radius = blurred_shadow_radius(radii.spread_offset(spread));
    // Map the CSS blur radius to a gaussian standard deviation.
    let std_dev = f64::from(blur.max(0.0)) / 2.0;
    let peniko_color = ColorConverter::new().convert(color);
    scene.draw_blurred_rounded_rect(transform, kurbo_rect, peniko_color, radius, std_dev);
}

#[doc = include_str!("shadow_painter.doc/blurred_shadow_radius.md")]
fn blurred_shadow_radius(radii: obscur_geometry::CornerRadii) -> f64 {
    f64::from(
        radii
            .corners()
            .iter()
            .map(|c| c.circular_approximation())
            .fold(0.0_f32, f32::max)
            .max(0.0),
    )
}

#[doc = include_str!("shadow_painter.doc/draw_inset_shadow.md")]
#[allow(clippy::too_many_arguments)]
fn draw_inset_shadow(
    scene: &mut Scene,
    rect: &obscur_geometry::Rect,
    radii: &obscur_geometry::CornerRadii,
    color: obscur_scene::color::Color,
    blur: f32,
    spread: f32,
    offset: (f32, f32),
    transform: Affine,
) {
    let bx0 = f64::from(rect.origin.x);
    let by0 = f64::from(rect.origin.y);
    let bx1 = f64::from(rect.origin.x + rect.size.width);
    let by1 = f64::from(rect.origin.y + rect.size.height);
    if bx1 <= bx0 || by1 <= by0 {
        return;
    }
    // A single corner radius, matching the drop-shadow path — see
    // [`blurred_shadow_radius`] for why a shadow may collapse the four corners.
    let radius = blurred_shadow_radius(*radii);
    let box_shape = KurboRect::new(bx0, by0, bx1, by1).to_rounded_rect(radius);
    let peniko_color = ColorConverter::new().convert(color);

    // (1) Clip + isolate to the box: the inner shadow can never escape it.
    scene.push_layer(
        peniko::Fill::NonZero,
        peniko::BlendMode::default(),
        1.0,
        transform,
        &box_shape,
    );
    // (2) Paint the shadow color across the whole box interior.
    scene.fill(
        peniko::Fill::NonZero,
        transform,
        peniko_color,
        None,
        &box_shape,
    );

    // (3) Punch out the lit interior (box deflated by `spread`, shifted by
    // `offset`), blurred, leaving only the inner frame.
    let sp = f64::from(spread);
    let ox = f64::from(offset.0);
    let oy = f64::from(offset.1);
    let ix0 = bx0 + ox + sp;
    let iy0 = by0 + oy + sp;
    let ix1 = bx1 + ox - sp;
    let iy1 = by1 + oy - sp;
    if ix1 > ix0 && iy1 > iy0 {
        let inner = KurboRect::new(ix0, iy0, ix1, iy1);
        // Spec §6.1.1, as on the drop path: the hole's radii are the box radii
        // contracted by `spread`, so they take `−spread`. Offsetting the eight
        // radii *before* collapsing to Vello's single scalar (rather than
        // subtracting from the already-collapsed `radius`) is what lets §6.1.1's
        // per-axis easing apply at all, and matches the drop path's ordering.
        let inner_radius = blurred_shadow_radius(radii.spread_offset(-spread));
        let std_dev = f64::from(blur.max(0.0)) / 2.0;
        // Opaque punch: `DestOut` clears the destination by the source alpha.
        let punch = peniko::Color::new([
            peniko_color.components[0],
            peniko_color.components[1],
            peniko_color.components[2],
            1.0,
        ]);
        let dest_out = peniko::BlendMode::new(peniko::Mix::Normal, peniko::Compose::DestOut);
        scene.push_layer(peniko::Fill::NonZero, dest_out, 1.0, transform, &box_shape);
        scene.draw_blurred_rounded_rect(transform, inner, punch, inner_radius, std_dev);
        scene.pop_layer();
    }
    // Close the box clip layer. A collapsed inner rect (huge `spread`) leaves the
    // whole box filled with the shadow color — the correct limit.
    scene.pop_layer();
}
