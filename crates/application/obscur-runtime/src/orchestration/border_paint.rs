//! The stroked edges of a box: its CSS borders (mitered quads, the uniform
//! rounded ring, and the dashed stroke), the corner radii every consumer of the
//! box shares, and the `outline` ring painted outside the border box.

use obscur_geometry::{CornerRadii, CornerRadius, Point, Rect, Size};
use obscur_scene::command::PaintCommand;
use obscur_scene::paint::{Brush, Paint, Stroke};
use obscur_scene::path::Path;
use obscur_style::computed::{ComputedStyle, ComputedValue};
use obscur_style::property::{Length, PropertyId, PropertyValue, StyleColor, layout_keyword};

use super::paint_color::{current_color_of, scene_color_from};

#[doc = include_str!("../../docs/api/orchestration/border_paint.doc/BorderWidths.md")]
pub(super) type BorderWidths = [f32; 4];

#[doc = include_str!("../../docs/api/orchestration/border_paint.doc/border_widths_of.md")]
pub(super) fn border_widths_of(style: Option<&ComputedStyle>) -> BorderWidths {
    [
        border_width_of(style, PropertyId::BorderTopWidth),
        border_width_of(style, PropertyId::BorderRightWidth),
        border_width_of(style, PropertyId::BorderBottomWidth),
        border_width_of(style, PropertyId::BorderLeftWidth),
    ]
}

/// Returns the padding box used by overflow clipping and scrolling.
pub(crate) fn overflow_padding_box(style: Option<&ComputedStyle>, bounds: Rect) -> Rect {
    let [top, right, bottom, left] = border_widths_of(style).map(|width| width.max(0.0));
    Rect::new(
        Point::new(bounds.origin.x + left, bounds.origin.y + top),
        Size::new(
            (bounds.size.width - left - right).max(0.0),
            (bounds.size.height - top - bottom).max(0.0),
        ),
    )
}

/// Resolves the inner corner radii at the padding edge.
pub(crate) fn overflow_padding_radii(
    style: Option<&ComputedStyle>,
    bounds: Rect,
    padding: Rect,
) -> CornerRadii {
    let [top, right, bottom, left] = border_widths_of(style).map(|width| width.max(0.0));
    let outer = corner_radii_of(style, Some(bounds)).normalized(bounds);
    let inset = |radius: CornerRadius, x: f32, y: f32| {
        CornerRadius::new((radius.x - x).max(0.0), (radius.y - y).max(0.0))
    };
    CornerRadii {
        top_left: inset(outer.top_left, left, top),
        top_right: inset(outer.top_right, right, top),
        bottom_right: inset(outer.bottom_right, right, bottom),
        bottom_left: inset(outer.bottom_left, left, bottom),
    }
    .normalized(padding)
}

#[doc = include_str!("../../docs/api/orchestration/border_paint.doc/any_border_width.md")]
pub(super) fn any_border_width(widths: BorderWidths) -> bool {
    widths.iter().any(|w| *w > 0.0)
}

#[doc = include_str!("../../docs/api/orchestration/border_paint.doc/border_fills.md")]
pub(super) fn border_fills(
    style: Option<&ComputedStyle>,
    bounds: Rect,
    widths: BorderWidths,
) -> Vec<(Path, StyleColor)> {
    let [top, right, bottom, left] = widths;
    // Fast path: no positive border width ⇒ nothing to paint (the common case).
    if top <= 0.0 && right <= 0.0 && bottom <= 0.0 && left <= 0.0 {
        return Vec::new();
    }
    let current = current_color_of(style);
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
            if width > 0.0 && border_is_solid(style, style_id) {
                let color = border_color_of(style, color_id, current);
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

#[doc = include_str!("../../docs/api/orchestration/border_paint.doc/uniform_border.md")]
pub(super) fn uniform_border(
    style: Option<&ComputedStyle>,
    widths: BorderWidths,
) -> Option<(f32, StyleColor)> {
    let [top, right, bottom, left] = widths;
    if !(top > 0.0 && top == right && top == bottom && top == left) {
        return None;
    }
    if !all_sides_solid(style) {
        return None;
    }
    // All four colors equal (resolving `currentColor` the same way for each side).
    let current = current_color_of(style);
    let ct = border_color_of(style, PropertyId::BorderTopColor, current);
    let cr = border_color_of(style, PropertyId::BorderRightColor, current);
    let cb = border_color_of(style, PropertyId::BorderBottomColor, current);
    let cl = border_color_of(style, PropertyId::BorderLeftColor, current);
    if ct == cr && ct == cb && ct == cl {
        Some((top, ct))
    } else {
        None
    }
}

#[doc = include_str!("../../docs/api/orchestration/border_paint.doc/uniform_border_ring.md")]
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

#[doc = include_str!("../../docs/api/orchestration/border_paint.doc/border_width_of.md")]
pub(super) fn border_width_of(style: Option<&ComputedStyle>, id: PropertyId) -> f32 {
    style
        .and_then(|style| style.get(id))
        .and_then(ComputedValue::as_length)
        .and_then(Length::as_px)
        .map_or(0.0, |px| px.0)
}

#[doc = include_str!("../../docs/api/orchestration/border_paint.doc/corner_radius_of.md")]
fn corner_radius_of(
    style: Option<&ComputedStyle>,
    id: PropertyId,
    bounds: Option<Rect>,
) -> CornerRadius {
    match style
        .and_then(|style| style.get(id))
        .map(ComputedValue::value)
    {
        Some(PropertyValue::Length(Length::Px(px))) => CornerRadius::circular(px.0),
        Some(PropertyValue::Length(Length::Percent(pct))) => bounds
            .map_or_else(CornerRadius::default, |b| {
                CornerRadius::new(pct / 100.0 * b.size.width, pct / 100.0 * b.size.height)
            }),
        Some(PropertyValue::Calc(expr)) => bounds.map_or_else(CornerRadius::default, |b| {
            let radius = |basis| {
                expr.resolve_with_percentage_basis(basis)
                    .unwrap_or_default()
                    .max(0.0)
            };
            CornerRadius::new(radius(b.size.width), radius(b.size.height))
        }),
        _ => CornerRadius::default(),
    }
}

#[doc = include_str!("../../docs/api/orchestration/border_paint.doc/corner_radii_of.md")]
pub(crate) fn corner_radii_of(style: Option<&ComputedStyle>, bounds: Option<Rect>) -> CornerRadii {
    CornerRadii {
        top_left: corner_radius_of(style, PropertyId::BorderTopLeftRadius, bounds),
        top_right: corner_radius_of(style, PropertyId::BorderTopRightRadius, bounds),
        bottom_right: corner_radius_of(style, PropertyId::BorderBottomRightRadius, bounds),
        bottom_left: corner_radius_of(style, PropertyId::BorderBottomLeftRadius, bounds),
    }
}

/// Whether a node's computed `border-*-style` (`id`) is the reserved `solid`
/// handle. An absent style (the deferred `none` initial) or any other value is
/// treated as not solid, so no edge is painted.
fn border_is_solid(style: Option<&ComputedStyle>, id: PropertyId) -> bool {
    border_style_is(style, id, layout_keyword::BORDER_STYLE_SOLID)
}

fn all_sides_solid(style: Option<&ComputedStyle>) -> bool {
    border_is_solid(style, PropertyId::BorderTopStyle)
        && border_is_solid(style, PropertyId::BorderRightStyle)
        && border_is_solid(style, PropertyId::BorderBottomStyle)
        && border_is_solid(style, PropertyId::BorderLeftStyle)
}

/// Whether a node's computed `border-*-style` (`id`) is the reserved `dashed`
/// handle (`dotted` folds here too). Any other value — including `solid`, `none`,
/// or an absent style — is not dashed.
fn border_is_dashed(style: Option<&ComputedStyle>, id: PropertyId) -> bool {
    border_style_is(style, id, layout_keyword::BORDER_STYLE_DASHED)
}

/// Whether a node's computed `border-*-style` (`id`) equals the reserved
/// `expected` keyword handle.
fn border_style_is(
    style: Option<&ComputedStyle>,
    id: PropertyId,
    expected: obscur_identity::InternedSymbol,
) -> bool {
    matches!(
        style
            .and_then(|style| style.get(id))
            .map(ComputedValue::value),
        Some(&PropertyValue::Keyword(sym)) if sym == expected
    )
}

#[doc = include_str!("../../docs/api/orchestration/border_paint.doc/uniform_dashed_border.md")]
pub(super) fn uniform_dashed_border(
    style: Option<&ComputedStyle>,
    widths: BorderWidths,
) -> Option<(f32, StyleColor)> {
    let [top, right, bottom, left] = widths;
    if !(top > 0.0 && top == right && top == bottom && top == left) {
        return None;
    }
    if !(border_is_dashed(style, PropertyId::BorderTopStyle)
        && border_is_dashed(style, PropertyId::BorderRightStyle)
        && border_is_dashed(style, PropertyId::BorderBottomStyle)
        && border_is_dashed(style, PropertyId::BorderLeftStyle))
    {
        return None;
    }
    let current = current_color_of(style);
    let ct = border_color_of(style, PropertyId::BorderTopColor, current);
    let cr = border_color_of(style, PropertyId::BorderRightColor, current);
    let cb = border_color_of(style, PropertyId::BorderBottomColor, current);
    let cl = border_color_of(style, PropertyId::BorderLeftColor, current);
    if ct == cr && ct == cb && ct == cl {
        Some((top, ct))
    } else {
        None
    }
}

/// The dash pattern (in logical pixels) for a dashed border of stroke `width`:
/// `[3·width on, 3·width off]`. CSS leaves the exact dash length implementation-
/// defined; a multiple of the border width gives dashes that scale with the
/// border and read as a normal dashed rule.
pub(super) fn dashed_border_pattern(width: f32) -> [f32; 2] {
    let dash = (3.0 * width).max(1.0);
    [dash, dash]
}

/// Builds the path a uniform dashed border strokes along: the border box inset by
/// `width / 2` on every side, so a stroke of `width` centered on it fills the CSS
/// border band exactly (CSS borders sit inside the border box). Rounded when the
/// element has a corner radius (each radius reduced by `width / 2`, the centerline
/// radius), else a plain rectangle.
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

#[doc = include_str!("../../docs/api/orchestration/border_paint.doc/outline_paint.md")]
pub(super) fn outline_paint(
    style: Option<&ComputedStyle>,
    bounds: Rect,
    radii: CornerRadii,
    has_radius: bool,
) -> Option<PaintCommand> {
    let width = border_width_of(style, PropertyId::OutlineWidth);
    if width <= 0.0 {
        return None;
    }
    let dashed = border_is_dashed(style, PropertyId::OutlineStyle);
    let solid = border_is_solid(style, PropertyId::OutlineStyle);
    // Only a `solid` or `dashed` (incl. folded `dotted`) outline paints; `none` or
    // an absent style paints nothing.
    if !dashed && !solid {
        return None;
    }
    let offset = border_width_of(style, PropertyId::OutlineOffset);
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
    let current = current_color_of(style);
    let color = border_color_of(style, PropertyId::OutlineColor, current);
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

/// Reads a node's computed `border-*-color` (`id`), falling back to `current`
/// (the node's computed `color`, i.e. CSS `currentColor`) when the color is unset.
fn border_color_of(
    style: Option<&ComputedStyle>,
    id: PropertyId,
    current: StyleColor,
) -> StyleColor {
    style
        .and_then(|style| style.get(id))
        .and_then(ComputedValue::as_color)
        .unwrap_or(current)
}
