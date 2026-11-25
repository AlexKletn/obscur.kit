//! Lower SVG path data into the scene commands used by CSS clipping.

use obscur_geometry::Point;
use obscur_scene::path::{FillRule, Path};
use svgtypes::{PathParser, PathSegment, SimplePathSegment, SimplifyingPathParser};

pub(super) fn lower_svg_path(data: &str, rule: FillRule) -> Option<Path> {
    // Validate the authored coordinates before arc-to-cubic expansion.
    // An extreme arc radius would otherwise generate excessive segments.
    for segment in PathParser::from(data) {
        if !safe_segment(segment.ok()?) {
            return None;
        }
    }

    let mut path = Path::with_fill_rule(rule);
    let mut has_geometry = false;
    let point = |x: f64, y: f64| {
        let point = Point::new(x as f32, y as f32);
        (point.x.is_finite() && point.y.is_finite()).then_some(point)
    };
    for segment in SimplifyingPathParser::from(data) {
        match segment.ok()? {
            SimplePathSegment::MoveTo { x, y } => {
                path.move_to(point(x, y)?);
            }
            SimplePathSegment::LineTo { x, y } => {
                path.line_to(point(x, y)?);
                has_geometry = true;
            }
            SimplePathSegment::Quadratic { x1, y1, x, y } => {
                path.quad_to(point(x1, y1)?, point(x, y)?);
                has_geometry = true;
            }
            SimplePathSegment::CurveTo {
                x1,
                y1,
                x2,
                y2,
                x,
                y,
            } => {
                path.cubic_to(point(x1, y1)?, point(x2, y2)?, point(x, y)?);
                has_geometry = true;
            }
            SimplePathSegment::ClosePath => {
                path.close();
            }
        }
    }
    has_geometry.then_some(path)
}

fn safe_segment(segment: PathSegment) -> bool {
    const COORDINATE_LIMIT: f64 = 1.0e9;
    let safe = |values: &[f64]| values.iter().all(|value| value.abs() <= COORDINATE_LIMIT);
    match segment {
        PathSegment::MoveTo { x, y, .. }
        | PathSegment::LineTo { x, y, .. }
        | PathSegment::SmoothQuadratic { x, y, .. } => safe(&[x, y]),
        PathSegment::HorizontalLineTo { x, .. } => safe(&[x]),
        PathSegment::VerticalLineTo { y, .. } => safe(&[y]),
        PathSegment::CurveTo {
            x1,
            y1,
            x2,
            y2,
            x,
            y,
            ..
        } => safe(&[x1, y1, x2, y2, x, y]),
        PathSegment::SmoothCurveTo { x2, y2, x, y, .. } => safe(&[x2, y2, x, y]),
        PathSegment::Quadratic { x1, y1, x, y, .. } => safe(&[x1, y1, x, y]),
        PathSegment::EllipticalArc {
            rx,
            ry,
            x_axis_rotation,
            x,
            y,
            ..
        } => safe(&[rx, ry, x_axis_rotation, x, y]),
        PathSegment::ClosePath { .. } => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_data_accepts_arcs_and_rejects_invalid_geometry() {
        let path = lower_svg_path("M0 8 A8 8 0 0 1 16 8 L16 16 H0 Z", FillRule::NonZero)
            .expect("closed path");
        assert!(
            path.segments().iter().any(|segment| {
                matches!(segment, obscur_scene::path::PathSegment::CubicTo { .. })
            })
        );
        assert!(lower_svg_path("M0 0 L", FillRule::NonZero).is_none());
        assert!(lower_svg_path("M0 0", FillRule::NonZero).is_none());
        assert!(lower_svg_path("M0 0 A1e300 10 0 0 0 1 1", FillRule::NonZero).is_none());
    }
}
