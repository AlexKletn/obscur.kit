//! CSS clip geometry resolved in the node’s local layout space.

use obscur_geometry::{CornerRadii, CornerRadius, Point, Rect, Size};
use obscur_layout::model::LayoutBox;
use obscur_scene::clip::Clip;
use obscur_scene::path::{Path, PathSegment, PathTrim};
use obscur_style::computed::ComputedStyle;
use obscur_style::property::{
    ClipRadius, ClipRectangleKind, Length, LengthExpr, PropertyId, PropertyValue,
};

mod clip_polygon;
use clip_polygon::ClipPolygon;

pub(crate) enum ResolvedClipPath<'a> {
    Box(Rect, CornerRadii),
    Polygon(ClipPolygon<'a>),
    Svg {
        path: &'a Path,
        outline: &'a PathTrim,
        origin: Point,
    },
}

impl<'a> ResolvedClipPath<'a> {
    pub(crate) fn of(style: Option<&'a ComputedStyle>, layout: LayoutBox) -> Option<Self> {
        use super::background_paint::{background_box_rect, inset_background_radii};
        match style?.get(PropertyId::ClipPath)?.value() {
            PropertyValue::ClipSvgPath {
                reference_box,
                path,
                outline,
            } => {
                let bounds = background_box_rect(*reference_box, style, layout);
                Some(Self::Svg {
                    path,
                    outline,
                    origin: bounds.origin,
                })
            }
            PropertyValue::ClipRectangle {
                reference_box,
                kind,
                coordinates,
                radii,
            } => {
                let bounds = background_box_rect(*reference_box, style, layout);
                Some(resolve_rectangle(
                    bounds,
                    *kind,
                    coordinates,
                    radii.as_deref(),
                ))
            }
            PropertyValue::ClipEllipse {
                reference_box,
                center,
                radii,
                circle,
            } => {
                let bounds = background_box_rect(*reference_box, style, layout);
                Some(resolve_ellipse(bounds, center, radii, *circle))
            }
            PropertyValue::ClipPolygon {
                points,
                even_odd,
                reference_box,
            } => {
                let bounds = background_box_rect(*reference_box, style, layout);
                Some(Self::Polygon(ClipPolygon::new(points, bounds, *even_odd)))
            }
            PropertyValue::ClipBox(reference_box) => {
                let bounds = background_box_rect(*reference_box, style, layout);
                let outer =
                    super::corner_radii_of(style, Some(layout.bounds)).normalized(layout.bounds);
                Some(Self::Box(
                    bounds,
                    inset_background_radii(outer, layout.bounds, bounds),
                ))
            }
            _ => None,
        }
    }

    pub(crate) fn clip(&self) -> Clip {
        match self {
            Self::Box(bounds, radii) => Clip::rounded_rect(*bounds, *radii),
            Self::Polygon(polygon) => Clip::path(polygon.path()),
            Self::Svg { path, origin, .. } => {
                let translate = |p: Point| Point::new(p.x + origin.x, p.y + origin.y);
                let mut translated = Path::with_fill_rule(path.fill_rule());
                for &segment in path.segments() {
                    match segment {
                        PathSegment::MoveTo(p) => translated.move_to(translate(p)),
                        PathSegment::LineTo(p) => translated.line_to(translate(p)),
                        PathSegment::QuadTo { control, end } => {
                            translated.quad_to(translate(control), translate(end))
                        }
                        PathSegment::CubicTo {
                            control1,
                            control2,
                            end,
                        } => translated.cubic_to(
                            translate(control1),
                            translate(control2),
                            translate(end),
                        ),
                        PathSegment::Close => translated.close(),
                    };
                }
                Clip::path(translated)
            }
        }
    }
}

fn resolve_rectangle(
    reference: Rect,
    kind: ClipRectangleKind,
    [a, b, c, d]: &[LengthExpr; 4],
    radii: Option<&[[LengthExpr; 2]; 4]>,
) -> ResolvedClipPath<'static> {
    let width = reference.size.width;
    let height = reference.size.height;
    let (x, y, w, h) = match kind {
        ClipRectangleKind::Inset => {
            let (left, right) = inset_pair(d, b, width);
            let (top, bottom) = inset_pair(a, c, height);
            (left, top, width - left - right, height - top - bottom)
        }
        ClipRectangleKind::Edges => {
            let top = resolve_length(a, height);
            let right = if matches!(b, LengthExpr::Leaf(Length::Auto)) {
                width
            } else {
                resolve_length(b, width)
            };
            let bottom = if matches!(c, LengthExpr::Leaf(Length::Auto)) {
                height
            } else {
                resolve_length(c, height)
            };
            let left = resolve_length(d, width);
            (left, top, right - left, bottom - top)
        }
        ClipRectangleKind::OriginAndSize => (
            resolve_length(a, width),
            resolve_length(b, height),
            resolve_length(c, width),
            resolve_length(d, height),
        ),
    };
    let bounds = Rect::new(
        Point::new(reference.origin.x + x, reference.origin.y + y),
        Size::new(w.max(0.0), h.max(0.0)),
    );
    let radii = radii.map_or_else(CornerRadii::default, |radii| {
        let [top_left, top_right, bottom_right, bottom_left] = radii.each_ref().map(|[x, y]| {
            CornerRadius::new(
                resolve_length(x, bounds.size.width),
                resolve_length(y, bounds.size.height),
            )
        });
        CornerRadii {
            top_left,
            top_right,
            bottom_right,
            bottom_left,
        }
        .normalized(bounds)
    });
    ResolvedClipPath::Box(bounds, radii)
}

fn inset_pair(near: &LengthExpr, far: &LengthExpr, extent: f32) -> (f32, f32) {
    let near = resolve_length(near, extent);
    let far = resolve_length(far, extent);
    let sum = f64::from(near) + f64::from(far);
    if sum > f64::from(extent) {
        let scale = f64::from(extent) / sum;
        let near = (f64::from(near) * scale) as f32;
        (near, extent - near)
    } else {
        (near, far)
    }
}

fn resolve_length(length: &LengthExpr, extent: f32) -> f32 {
    length.resolve_with_percentage_basis(extent).unwrap_or(0.0)
}

fn resolve_ellipse(
    bounds: Rect,
    [center_x, center_y]: &[LengthExpr; 2],
    radii: &[ClipRadius; 2],
    circle: bool,
) -> ResolvedClipPath<'static> {
    let width = bounds.size.width;
    let height = bounds.size.height;
    let x = resolve_length(center_x, width);
    let y = resolve_length(center_y, height);
    let horizontal = [x.abs(), (width - x).abs()];
    let vertical = [y.abs(), (height - y).abs()];
    let (rx, ry) = if circle {
        let diagonal =
            (f64::from(width).hypot(f64::from(height)) * std::f64::consts::FRAC_1_SQRT_2) as f32;
        let radius = resolve_radius(&radii[0], diagonal, horizontal.into_iter().chain(vertical));
        (radius, radius)
    } else {
        (
            resolve_radius(&radii[0], width, horizontal),
            resolve_radius(&radii[1], height, vertical),
        )
    };
    let ellipse = Rect::new(
        Point::new(bounds.origin.x + x - rx, bounds.origin.y + y - ry),
        Size::new(rx * 2.0, ry * 2.0),
    );
    ResolvedClipPath::Box(ellipse, CornerRadii::elliptical(rx, ry))
}

fn resolve_radius(radius: &ClipRadius, basis: f32, sides: impl IntoIterator<Item = f32>) -> f32 {
    match radius {
        ClipRadius::ClosestSide => sides.into_iter().fold(f32::INFINITY, f32::min),
        ClipRadius::FarthestSide => sides.into_iter().fold(0.0, f32::max),
        ClipRadius::Length(length) => resolve_length(length, basis),
    }
    .max(0.0)
}
