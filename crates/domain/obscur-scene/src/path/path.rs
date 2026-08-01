#![doc = include_str!("path.doc/_module.md")]

use obscur_geometry::{CornerRadii, Point, Rect, Size};

use super::{FillRule, PathSegment};

#[doc = include_str!("path.doc/KAPPA.md")]
const KAPPA: f32 = 0.552_284_8;

#[doc = include_str!("path.doc/Path.md")]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Path {
    segments: Vec<PathSegment>,
    fill_rule: FillRule,
}

impl Path {
    #[doc = include_str!("path.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[doc = include_str!("path.doc/with_fill_rule.md")]
    #[must_use]
    pub fn with_fill_rule(rule: FillRule) -> Self {
        Self {
            segments: Vec::new(),
            fill_rule: rule,
        }
    }

    #[doc = include_str!("path.doc/move_to.md")]
    pub fn move_to(&mut self, p: Point) -> &mut Self {
        self.segments.push(PathSegment::MoveTo(p));
        self
    }

    #[doc = include_str!("path.doc/line_to.md")]
    pub fn line_to(&mut self, p: Point) -> &mut Self {
        self.segments.push(PathSegment::LineTo(p));
        self
    }

    #[doc = include_str!("path.doc/quad_to.md")]
    pub fn quad_to(&mut self, control: Point, end: Point) -> &mut Self {
        self.segments.push(PathSegment::QuadTo { control, end });
        self
    }

    #[doc = include_str!("path.doc/cubic_to.md")]
    pub fn cubic_to(&mut self, c1: Point, c2: Point, end: Point) -> &mut Self {
        self.segments.push(PathSegment::CubicTo {
            control1: c1,
            control2: c2,
            end,
        });
        self
    }

    #[doc = include_str!("path.doc/close.md")]
    pub fn close(&mut self) -> &mut Self {
        self.segments.push(PathSegment::Close);
        self
    }

    #[doc = include_str!("path.doc/segments.md")]
    #[must_use]
    pub fn segments(&self) -> &[PathSegment] {
        &self.segments
    }

    #[doc = include_str!("path.doc/fill_rule.md")]
    #[must_use]
    pub fn fill_rule(&self) -> FillRule {
        self.fill_rule
    }

    #[doc = include_str!("path.doc/set_fill_rule.md")]
    pub fn set_fill_rule(&mut self, rule: FillRule) -> &mut Self {
        self.fill_rule = rule;
        self
    }

    #[doc = include_str!("path.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    #[doc = include_str!("path.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.segments.len()
    }

    #[doc = include_str!("path.doc/rect.md")]
    #[must_use]
    pub fn rect(rect: Rect) -> Path {
        let x0 = rect.origin.x;
        let y0 = rect.origin.y;
        let x1 = rect.right();
        let y1 = rect.bottom();
        let mut path = Path::new();
        path.move_to(Point::new(x0, y0))
            .line_to(Point::new(x1, y0))
            .line_to(Point::new(x1, y1))
            .line_to(Point::new(x0, y1))
            .close();
        path
    }

    #[doc = include_str!("path.doc/rounded_rect.md")]
    #[must_use]
    pub fn rounded_rect(rect: Rect, radii: CornerRadii) -> Path {
        let x0 = rect.origin.x;
        let y0 = rect.origin.y;
        let x1 = rect.right();
        let y1 = rect.bottom();

        let radii = radii.non_negative().normalized(rect);
        let (tl, tr, br, bl) = (
            radii.top_left,
            radii.top_right,
            radii.bottom_right,
            radii.bottom_left,
        );
        // The Bézier handle length along each axis: the distance from the corner
        // point back towards the edge tangent.
        let k = |r: f32| r * (1.0 - KAPPA);

        let mut path = Path::new();
        // Top edge, starting after the top-left corner.
        path.move_to(Point::new(x0 + tl.x, y0))
            .line_to(Point::new(x1 - tr.x, y0));
        // Top-right corner.
        path.cubic_to(
            Point::new(x1 - k(tr.x), y0),
            Point::new(x1, y0 + k(tr.y)),
            Point::new(x1, y0 + tr.y),
        );
        // Right edge.
        path.line_to(Point::new(x1, y1 - br.y));
        // Bottom-right corner.
        path.cubic_to(
            Point::new(x1, y1 - k(br.y)),
            Point::new(x1 - k(br.x), y1),
            Point::new(x1 - br.x, y1),
        );
        // Bottom edge.
        path.line_to(Point::new(x0 + bl.x, y1));
        // Bottom-left corner.
        path.cubic_to(
            Point::new(x0 + k(bl.x), y1),
            Point::new(x0, y1 - k(bl.y)),
            Point::new(x0, y1 - bl.y),
        );
        // Left edge.
        path.line_to(Point::new(x0, y0 + tl.y));
        // Top-left corner.
        path.cubic_to(
            Point::new(x0, y0 + k(tl.y)),
            Point::new(x0 + k(tl.x), y0),
            Point::new(x0 + tl.x, y0),
        );
        path.close();
        path
    }

    #[doc = include_str!("path.doc/rounded_rect_ring.md")]
    #[must_use]
    pub fn rounded_rect_ring(
        outer: Rect,
        outer_radii: CornerRadii,
        inner: Rect,
        inner_radii: CornerRadii,
    ) -> Path {
        let mut path = Path::rounded_rect(outer, outer_radii);
        let inner_path = Path::rounded_rect(inner, inner_radii);
        path.segments.extend_from_slice(inner_path.segments());
        path.fill_rule = FillRule::EvenOdd;
        path
    }

    #[doc = include_str!("path.doc/bounds.md")]
    #[must_use]
    pub fn bounds(&self) -> Option<Rect> {
        let mut bounds: Option<(Point, Point)> = None;
        let mut include = |p: Point| match &mut bounds {
            Some((min, max)) => {
                min.x = min.x.min(p.x);
                min.y = min.y.min(p.y);
                max.x = max.x.max(p.x);
                max.y = max.y.max(p.y);
            }
            None => bounds = Some((p, p)),
        };

        for segment in &self.segments {
            match segment {
                PathSegment::MoveTo(p) | PathSegment::LineTo(p) => include(*p),
                PathSegment::QuadTo { control, end } => {
                    include(*control);
                    include(*end);
                }
                PathSegment::CubicTo {
                    control1,
                    control2,
                    end,
                } => {
                    include(*control1);
                    include(*control2);
                    include(*end);
                }
                PathSegment::Close => {}
            }
        }

        bounds.map(|(min, max)| Rect::new(min, Size::new(max.x - min.x, max.y - min.y)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_is_empty_non_zero() {
        let path = Path::new();
        assert!(path.is_empty());
        assert_eq!(path.len(), 0);
        assert_eq!(path.fill_rule(), FillRule::NonZero);
        assert_eq!(path.bounds(), None);
    }

    #[test]
    fn builder_chaining_pushes_segments() {
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
        assert_eq!(path.len(), 5);
        assert!(!path.is_empty());
    }

    #[test]
    fn set_fill_rule_updates() {
        let mut path = Path::new();
        path.set_fill_rule(FillRule::EvenOdd);
        assert_eq!(path.fill_rule(), FillRule::EvenOdd);
    }

    #[test]
    fn rect_has_five_segments_and_bounds() {
        let r = Rect::new(Point::new(1.0, 2.0), Size::new(4.0, 6.0));
        let path = Path::rect(r);
        assert_eq!(path.len(), 5);
        assert_eq!(
            path.segments()[0],
            PathSegment::MoveTo(Point::new(1.0, 2.0))
        );
        assert_eq!(path.segments()[4], PathSegment::Close);
        assert_eq!(path.bounds(), Some(r));
    }

    #[test]
    fn rounded_rect_stays_within_rect_bounds() {
        let r = Rect::new(Point::new(0.0, 0.0), Size::new(20.0, 10.0));
        let path = Path::rounded_rect(r, CornerRadii::uniform(3.0));
        let Some(bounds) = path.bounds() else {
            panic!("rounded rect must have bounds")
        };
        // Bounds (conservative, includes control points) stay within the rect.
        assert!(bounds.origin.x >= r.origin.x - f32::EPSILON);
        assert!(bounds.origin.y >= r.origin.y - f32::EPSILON);
        assert!(bounds.right() <= r.right() + f32::EPSILON);
        assert!(bounds.bottom() <= r.bottom() + f32::EPSILON);
    }

    #[test]
    fn rounded_rect_clamps_oversized_radii() {
        let r = Rect::new(Point::new(0.0, 0.0), Size::new(10.0, 4.0));
        // Radius far larger than half the shorter side (2.0) must be clamped.
        let path = Path::rounded_rect(r, CornerRadii::uniform(100.0));
        let Some(bounds) = path.bounds() else {
            panic!("rounded rect must have bounds")
        };
        assert!(bounds.origin.x >= r.origin.x - f32::EPSILON);
        assert!(bounds.right() <= r.right() + f32::EPSILON);
        assert!(bounds.bottom() <= r.bottom() + f32::EPSILON);
    }

    #[test]
    fn rounded_rect_ring_has_two_subpaths_and_even_odd() {
        let outer = Rect::new(Point::new(0.0, 0.0), Size::new(40.0, 40.0));
        let inner = Rect::new(Point::new(5.0, 5.0), Size::new(30.0, 30.0));
        let ring = Path::rounded_rect_ring(
            outer,
            CornerRadii::uniform(14.0),
            inner,
            CornerRadii::uniform(9.0),
        );
        // Even-odd so the inner subpath punches a hole rather than filling solid.
        assert_eq!(ring.fill_rule(), FillRule::EvenOdd);
        // Two closed rounded-rect subpaths ⇒ two `MoveTo`s and two `Close`s.
        let moves = ring
            .segments()
            .iter()
            .filter(|s| matches!(s, PathSegment::MoveTo(_)))
            .count();
        let closes = ring
            .segments()
            .iter()
            .filter(|s| matches!(s, PathSegment::Close))
            .count();
        assert_eq!(moves, 2);
        assert_eq!(closes, 2);
        // Its segments are exactly the outer rounded rect's followed by the inner's.
        let outer_only = Path::rounded_rect(outer, CornerRadii::uniform(14.0));
        let inner_only = Path::rounded_rect(inner, CornerRadii::uniform(9.0));
        assert_eq!(&ring.segments()[..outer_only.len()], outer_only.segments());
        assert_eq!(&ring.segments()[outer_only.len()..], inner_only.segments());
    }

    #[test]
    fn rounded_rect_negative_size_does_not_panic() {
        let r = Rect::new(Point::new(0.0, 0.0), Size::new(-5.0, -5.0));
        let path = Path::rounded_rect(r, CornerRadii::uniform(2.0));
        assert!(!path.is_empty());
    }
}
