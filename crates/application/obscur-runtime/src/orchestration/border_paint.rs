#![doc = include_str!("border_paint.doc/_module.md")]

use obscur_geometry::{CornerRadii, CornerRadius, Point, Rect, Size};
use obscur_identity::NodeId;
use obscur_scene::command::PaintCommand;
use obscur_scene::paint::{Brush, Paint, Stroke};
use obscur_scene::path::Path;
use obscur_style::computed::ComputedValue;
use obscur_style::property::{Length, PropertyId, PropertyValue, StyleColor, layout_keyword};

use super::ComputedStyles;
use super::paint_color::{current_color_of, scene_color_from};

#[doc = include_str!("border_paint.doc/border_fills.md")]
pub(super) fn border_fills(
    computed: &ComputedStyles,
    node: NodeId,
    bounds: Rect,
) -> Vec<(Path, StyleColor)> {
    let top = border_width_of(computed, node, PropertyId::BorderTopWidth);
    let right = border_width_of(computed, node, PropertyId::BorderRightWidth);
    let bottom = border_width_of(computed, node, PropertyId::BorderBottomWidth);
    let left = border_width_of(computed, node, PropertyId::BorderLeftWidth);
    // Fast path: no positive border width ⇒ nothing to paint (the common case).
    if top <= 0.0 && right <= 0.0 && bottom <= 0.0 && left <= 0.0 {
        return Vec::new();
    }
    let current = current_color_of(computed, node);
    let x = bounds.origin.x;
    let y = bounds.origin.y;
    let w = bounds.size.width;
    let h = bounds.size.height;
    // Outer corners (the border box) and inner corners (each inset by its two
    // adjacent border widths). A side's miter runs from an outer corner to the
    // matching inner corner.
    let outer_tl = Point::new(x, y);
    let outer_tr = Point::new(x + w, y);
    let outer_br = Point::new(x + w, y + h);
    let outer_bl = Point::new(x, y + h);
    let inner_tl = Point::new(x + left, y + top);
    let inner_tr = Point::new(x + w - right, y + top);
    let inner_br = Point::new(x + w - right, y + h - bottom);
    let inner_bl = Point::new(x + left, y + h - bottom);
    let quad = |a: Point, b: Point, c: Point, d: Point| -> Path {
        let mut path = Path::new();
        path.move_to(a).line_to(b).line_to(c).line_to(d).close();
        path
    };
    let mut out = Vec::new();
    // Each side: outer full edge, then the two inner corners (inset by adjacent
    // widths). Emitted only when solid, positive-width, and non-transparent.
    let mut push_side =
        |width: f32, style_id: PropertyId, color_id: PropertyId, corners: [Point; 4]| {
            if width > 0.0 && border_is_solid(computed, node, style_id) {
                let color = border_color_of(computed, node, color_id, current);
                // A fully transparent side paints nothing (the border-triangle trick).
                if color.a > 0.0 {
                    out.push((quad(corners[0], corners[1], corners[2], corners[3]), color));
                }
            }
        };
    push_side(
        top,
        PropertyId::BorderTopStyle,
        PropertyId::BorderTopColor,
        [outer_tl, outer_tr, inner_tr, inner_tl],
    );
    push_side(
        right,
        PropertyId::BorderRightStyle,
        PropertyId::BorderRightColor,
        [outer_tr, outer_br, inner_br, inner_tr],
    );
    push_side(
        bottom,
        PropertyId::BorderBottomStyle,
        PropertyId::BorderBottomColor,
        [outer_br, outer_bl, inner_bl, inner_br],
    );
    push_side(
        left,
        PropertyId::BorderLeftStyle,
        PropertyId::BorderLeftColor,
        [outer_bl, outer_tl, inner_tl, inner_bl],
    );
    out
}

#[doc = include_str!("border_paint.doc/uniform_border.md")]
pub(super) fn uniform_border(computed: &ComputedStyles, node: NodeId) -> Option<(f32, StyleColor)> {
    let top = border_width_of(computed, node, PropertyId::BorderTopWidth);
    let right = border_width_of(computed, node, PropertyId::BorderRightWidth);
    let bottom = border_width_of(computed, node, PropertyId::BorderBottomWidth);
    let left = border_width_of(computed, node, PropertyId::BorderLeftWidth);
    if !(top > 0.0 && top == right && top == bottom && top == left) {
        return None;
    }
    if !all_sides_solid(computed, node) {
        return None;
    }
    // All four colors equal (resolving `currentColor` the same way for each side).
    let current = current_color_of(computed, node);
    let ct = border_color_of(computed, node, PropertyId::BorderTopColor, current);
    let cr = border_color_of(computed, node, PropertyId::BorderRightColor, current);
    let cb = border_color_of(computed, node, PropertyId::BorderBottomColor, current);
    let cl = border_color_of(computed, node, PropertyId::BorderLeftColor, current);
    if ct == cr && ct == cb && ct == cl {
        Some((top, ct))
    } else {
        None
    }
}

#[doc = include_str!("border_paint.doc/uniform_border_ring.md")]
pub(super) fn uniform_border_ring(bounds: Rect, radii: CornerRadii, width: f32) -> Path {
    let inner = Rect::new(
        Point::new(bounds.origin.x + width, bounds.origin.y + width),
        Size::new(
            (bounds.size.width - 2.0 * width).max(0.0),
            (bounds.size.height - 2.0 * width).max(0.0),
        ),
    );
    Path::rounded_rect_ring(bounds, radii, inner, radii.offset(-width))
}

#[doc = include_str!("border_paint.doc/border_width_of.md")]
pub(super) fn border_width_of(computed: &ComputedStyles, node: NodeId, id: PropertyId) -> f32 {
    computed
        .get(&node)
        .and_then(|style| style.get(id))
        .and_then(ComputedValue::as_length)
        .and_then(Length::as_px)
        .map_or(0.0, |px| px.0)
}

#[doc = include_str!("border_paint.doc/corner_radius_of.md")]
fn corner_radius_of(
    computed: &ComputedStyles,
    node: NodeId,
    id: PropertyId,
    bounds: Option<Rect>,
) -> CornerRadius {
    let length = computed
        .get(&node)
        .and_then(|style| style.get(id))
        .and_then(ComputedValue::as_length);
    match length {
        Some(Length::Px(px)) => CornerRadius::circular(px.0),
        Some(Length::Percent(pct)) => bounds.map_or_else(CornerRadius::default, |b| {
            CornerRadius::new(pct / 100.0 * b.size.width, pct / 100.0 * b.size.height)
        }),
        _ => CornerRadius::default(),
    }
}

#[doc = include_str!("border_paint.doc/corner_radii_of.md")]
pub(super) fn corner_radii_of(
    computed: &ComputedStyles,
    node: NodeId,
    bounds: Option<Rect>,
) -> CornerRadii {
    CornerRadii {
        top_left: corner_radius_of(computed, node, PropertyId::BorderTopLeftRadius, bounds),
        top_right: corner_radius_of(computed, node, PropertyId::BorderTopRightRadius, bounds),
        bottom_right: corner_radius_of(computed, node, PropertyId::BorderBottomRightRadius, bounds),
        bottom_left: corner_radius_of(computed, node, PropertyId::BorderBottomLeftRadius, bounds),
    }
}

#[doc = include_str!("border_paint.doc/border_is_solid.md")]
fn border_is_solid(computed: &ComputedStyles, node: NodeId, id: PropertyId) -> bool {
    border_style_is(computed, node, id, layout_keyword::BORDER_STYLE_SOLID)
}

fn all_sides_solid(computed: &ComputedStyles, node: NodeId) -> bool {
    border_is_solid(computed, node, PropertyId::BorderTopStyle)
        && border_is_solid(computed, node, PropertyId::BorderRightStyle)
        && border_is_solid(computed, node, PropertyId::BorderBottomStyle)
        && border_is_solid(computed, node, PropertyId::BorderLeftStyle)
}

#[doc = include_str!("border_paint.doc/border_is_dashed.md")]
fn border_is_dashed(computed: &ComputedStyles, node: NodeId, id: PropertyId) -> bool {
    border_style_is(computed, node, id, layout_keyword::BORDER_STYLE_DASHED)
}

#[doc = include_str!("border_paint.doc/border_style_is.md")]
fn border_style_is(
    computed: &ComputedStyles,
    node: NodeId,
    id: PropertyId,
    expected: obscur_identity::InternedSymbol,
) -> bool {
    matches!(
        computed
            .get(&node)
            .and_then(|style| style.get(id))
            .map(ComputedValue::value),
        Some(&PropertyValue::Keyword(sym)) if sym == expected
    )
}

#[doc = include_str!("border_paint.doc/uniform_dashed_border.md")]
pub(super) fn uniform_dashed_border(
    computed: &ComputedStyles,
    node: NodeId,
) -> Option<(f32, StyleColor)> {
    let top = border_width_of(computed, node, PropertyId::BorderTopWidth);
    let right = border_width_of(computed, node, PropertyId::BorderRightWidth);
    let bottom = border_width_of(computed, node, PropertyId::BorderBottomWidth);
    let left = border_width_of(computed, node, PropertyId::BorderLeftWidth);
    if !(top > 0.0 && top == right && top == bottom && top == left) {
        return None;
    }
    if !(border_is_dashed(computed, node, PropertyId::BorderTopStyle)
        && border_is_dashed(computed, node, PropertyId::BorderRightStyle)
        && border_is_dashed(computed, node, PropertyId::BorderBottomStyle)
        && border_is_dashed(computed, node, PropertyId::BorderLeftStyle))
    {
        return None;
    }
    let current = current_color_of(computed, node);
    let ct = border_color_of(computed, node, PropertyId::BorderTopColor, current);
    let cr = border_color_of(computed, node, PropertyId::BorderRightColor, current);
    let cb = border_color_of(computed, node, PropertyId::BorderBottomColor, current);
    let cl = border_color_of(computed, node, PropertyId::BorderLeftColor, current);
    if ct == cr && ct == cb && ct == cl {
        Some((top, ct))
    } else {
        None
    }
}

#[doc = include_str!("border_paint.doc/dashed_border_pattern.md")]
pub(super) fn dashed_border_pattern(width: f32) -> [f32; 2] {
    let dash = (3.0 * width).max(1.0);
    [dash, dash]
}

#[doc = include_str!("border_paint.doc/dashed_border_path.md")]
pub(super) fn dashed_border_path(
    bounds: Rect,
    radii: CornerRadii,
    width: f32,
    has_radius: bool,
) -> Path {
    let half = width / 2.0;
    let center = Rect::new(
        Point::new(bounds.origin.x + half, bounds.origin.y + half),
        Size::new(
            (bounds.size.width - width).max(0.0),
            (bounds.size.height - width).max(0.0),
        ),
    );
    if has_radius {
        // The centerline runs half a border-width inside the border box, so its
        // radii are the box radii pulled in by `half` on each axis — the §4.2
        // shaping rule applied at the stroke's centre rather than its inner edge.
        Path::rounded_rect(center, radii.offset(-half))
    } else {
        Path::rect(center)
    }
}

#[doc = include_str!("border_paint.doc/outline_paint.md")]
pub(super) fn outline_paint(
    computed: &ComputedStyles,
    node: NodeId,
    bounds: Rect,
    radii: CornerRadii,
    has_radius: bool,
) -> Option<PaintCommand> {
    let width = border_width_of(computed, node, PropertyId::OutlineWidth);
    if width <= 0.0 {
        return None;
    }
    let dashed = border_is_dashed(computed, node, PropertyId::OutlineStyle);
    let solid = border_is_solid(computed, node, PropertyId::OutlineStyle);
    // Only a `solid` or `dashed` (incl. folded `dotted`) outline paints; `none` or
    // an absent style paints nothing.
    if !dashed && !solid {
        return None;
    }
    let offset = border_width_of(computed, node, PropertyId::OutlineOffset);
    // The stroke centerline runs `offset + width/2` outside the border box, so the
    // outline's inner edge sits `offset` px out and the band is `width` thick.
    let d = offset + width / 2.0;
    let rect = Rect::new(
        Point::new(bounds.origin.x - d, bounds.origin.y - d),
        Size::new(
            (bounds.size.width + 2.0 * d).max(0.0),
            (bounds.size.height + 2.0 * d).max(0.0),
        ),
    );
    let path = if has_radius {
        // Concentric outer rounding: each corner grows by `d` on each axis, so the
        // outline stays parallel to the box's own curve. A corner the box leaves
        // square stays square — growing it would round an outline around a sharp
        // corner (see [`CornerRadii::offset`]).
        Path::rounded_rect(rect, radii.offset(d))
    } else {
        Path::rect(rect)
    };
    let current = current_color_of(computed, node);
    let color = border_color_of(computed, node, PropertyId::OutlineColor, current);
    let mut stroke = Stroke::new(Brush::solid(scene_color_from(color)), width);
    if dashed {
        // Reuse the border dash convention — no duplicate dash logic.
        stroke = stroke.with_dash(dashed_border_pattern(width), 0.0);
    }
    Some(PaintCommand::DrawPath {
        path,
        paint: Paint::stroke(stroke),
    })
}

#[doc = include_str!("border_paint.doc/border_color_of.md")]
fn border_color_of(
    computed: &ComputedStyles,
    node: NodeId,
    id: PropertyId,
    current: StyleColor,
) -> StyleColor {
    computed
        .get(&node)
        .and_then(|style| style.get(id))
        .and_then(ComputedValue::as_color)
        .unwrap_or(current)
}
