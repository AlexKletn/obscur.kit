//! A node's `box-shadow` list and the scene shadow each entry paints
//! (css-backgrounds-3 §6.1, spec §5.2).

use obscur_geometry::{CornerRadii, Point, Rect, Size};
use obscur_scene::clip::Clip;
use obscur_scene::command::DisplayListBuilder;
use obscur_scene::path::Path;
use obscur_style::computed::{ComputedStyle, ComputedValue};
use obscur_style::property::{BoxShadow, Length, PropertyId, StyleColor};

use super::paint_color::scene_color_from;

#[doc = include_str!("../../docs/api/orchestration/box_shadow_paint.doc/box_shadows_of.md")]
pub(super) fn box_shadows_of(style: Option<&ComputedStyle>) -> Option<&[BoxShadow]> {
    style
        .and_then(|style| style.get(PropertyId::BoxShadow))
        .and_then(ComputedValue::as_box_shadows)
}

#[doc = include_str!("../../docs/api/orchestration/box_shadow_paint.doc/emit_box_shadow.md")]
pub(super) fn emit_box_shadow(
    builder: &mut DisplayListBuilder,
    bounds: Rect,
    radii: CornerRadii,
    shadow: &BoxShadow,
    current: StyleColor,
) {
    let px = |length: Length| length.as_px().map_or(0.0, |px| px.0);
    let color = scene_color_from(shadow.color().unwrap_or(current));
    let blur = px(shadow.blur());
    let spread = px(shadow.spread());
    let offset = (px(shadow.offset_x()), px(shadow.offset_y()));
    let inset = shadow.is_inset();
    // css-backgrounds-3 §6.1: an OUTER (drop) shadow "is clipped inside the
    // border-box of the element" — it is painted only OUTSIDE the border edge. The
    // element's own (possibly translucent) background then paints over an untinted
    // border box. Without this the shadow bleeds under a translucent background and
    // tints it. The clip is the complement of the border box: an even-odd ring
    // whose outer rectangle comfortably contains the whole shadow (border box grown
    // by spread + offset + a blur allowance) and whose punched-out hole is the
    // border box with its own corner radii. An inset shadow is confined to the box
    // by the renderer and takes no such clip.
    if inset {
        builder.draw_shadow(bounds, radii, color, blur, spread, offset, inset);
        return;
    }
    let grow = spread + blur * 3.0 + offset.0.abs() + offset.1.abs() + 2.0;
    let outer = Rect::new(
        Point::new(bounds.origin.x - grow, bounds.origin.y - grow),
        Size::new(
            bounds.size.width + 2.0 * grow,
            bounds.size.height + 2.0 * grow,
        ),
    );
    let exclusion = Path::rounded_rect_ring(outer, CornerRadii::uniform(0.0), bounds, radii);
    builder.push_clip(Clip::path(exclusion));
    builder.draw_shadow(bounds, radii, color, blur, spread, offset, inset);
    builder.pop_clip();
}
