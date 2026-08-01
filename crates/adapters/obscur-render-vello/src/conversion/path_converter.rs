#![doc = include_str!("path_converter.doc/_module.md")]

use obscur_scene::path::{FillRule, Path, PathSegment};
use vello::kurbo::BezPath;
use vello::peniko;

#[doc = include_str!("path_converter.doc/PathConverter.md")]
#[derive(Debug, Default, Clone, Copy)]
pub struct PathConverter;

impl PathConverter {
    #[doc = include_str!("path_converter.doc/new.md")]
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    #[doc = include_str!("path_converter.doc/convert.md")]
    #[must_use]
    pub fn convert(&self, path: &Path) -> BezPath {
        let mut bez = BezPath::new();
        for segment in path.segments() {
            match *segment {
                PathSegment::MoveTo(p) => bez.move_to((f64::from(p.x), f64::from(p.y))),
                PathSegment::LineTo(p) => bez.line_to((f64::from(p.x), f64::from(p.y))),
                PathSegment::QuadTo { control, end } => bez.quad_to(
                    (f64::from(control.x), f64::from(control.y)),
                    (f64::from(end.x), f64::from(end.y)),
                ),
                PathSegment::CubicTo {
                    control1,
                    control2,
                    end,
                } => bez.curve_to(
                    (f64::from(control1.x), f64::from(control1.y)),
                    (f64::from(control2.x), f64::from(control2.y)),
                    (f64::from(end.x), f64::from(end.y)),
                ),
                PathSegment::Close => bez.close_path(),
            }
        }
        bez
    }

    #[doc = include_str!("path_converter.doc/fill_rule.md")]
    #[must_use]
    pub fn fill_rule(&self, rule: FillRule) -> peniko::Fill {
        match rule {
            FillRule::NonZero => peniko::Fill::NonZero,
            FillRule::EvenOdd => peniko::Fill::EvenOdd,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_geometry::{Point, Rect, Size};
    use vello::kurbo::PathEl;

    #[test]
    fn rectangle_path_has_five_elements() {
        let path = Path::rect(Rect::new(Point::new(1.0, 2.0), Size::new(4.0, 6.0)));
        let bez = PathConverter::new().convert(&path);
        // move + 3 lines + close, matching the 5 source segments.
        assert_eq!(bez.elements().len(), 5);
        assert!(matches!(bez.elements()[0], PathEl::MoveTo(_)));
        assert!(matches!(bez.elements()[4], PathEl::ClosePath));
    }

    #[test]
    fn every_segment_kind_maps_to_one_element() {
        let mut path = Path::new();
        path.move_to(Point::new(0.0, 0.0))
            .line_to(Point::new(10.0, 0.0))
            .quad_to(Point::new(10.0, 5.0), Point::new(5.0, 10.0))
            .cubic_to(
                Point::new(0.0, 10.0),
                Point::new(0.0, 5.0),
                Point::new(0.0, 0.0),
            )
            .close();
        let bez = PathConverter::new().convert(&path);
        assert_eq!(bez.elements().len(), path.segments().len());
        assert!(matches!(bez.elements()[1], PathEl::LineTo(_)));
        assert!(matches!(bez.elements()[2], PathEl::QuadTo(..)));
        assert!(matches!(bez.elements()[3], PathEl::CurveTo(..)));
    }

    #[test]
    fn empty_path_is_empty() {
        let bez = PathConverter::new().convert(&Path::new());
        assert!(bez.elements().is_empty());
    }

    #[test]
    fn fill_rule_maps_both_variants() {
        let c = PathConverter::new();
        assert_eq!(c.fill_rule(FillRule::NonZero), peniko::Fill::NonZero);
        assert_eq!(c.fill_rule(FillRule::EvenOdd), peniko::Fill::EvenOdd);
    }
}
