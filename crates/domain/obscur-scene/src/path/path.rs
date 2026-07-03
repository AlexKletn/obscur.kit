//! A vector outline: an ordered list of segments plus a fill rule.

use obscur_geometry::{CornerRadii, Point, Rect, Size};

use super::{FillRule, PathSegment};

/// The Bézier arc approximation constant (`4/3 * (sqrt(2) - 1)`), used to place
/// cubic control points that closely approximate a quarter-circle.
const KAPPA: f32 = 0.552_284_75;

/// A vector outline (spec §5.1): an ordered sequence of [`PathSegment`]s plus a
/// [`FillRule`] describing how the interior is filled.
///
/// A `Path` is plain, backend-independent data. It carries no rendering state
/// and references no GPU or renderer types; a backend interprets the segments.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Path {
    segments: Vec<PathSegment>,
    fill_rule: FillRule,
}

impl Path {
    /// Creates an empty path with the default [`FillRule::NonZero`].
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an empty path with the given fill rule.
    #[must_use]
    pub fn with_fill_rule(rule: FillRule) -> Self {
        Self {
            segments: Vec::new(),
            fill_rule: rule,
        }
    }

    /// Starts a new subpath at `p`. Returns `&mut Self` for chaining.
    pub fn move_to(&mut self, p: Point) -> &mut Self {
        self.segments.push(PathSegment::MoveTo(p));
        self
    }

    /// Adds a straight line to `p`. Returns `&mut Self` for chaining.
    pub fn line_to(&mut self, p: Point) -> &mut Self {
        self.segments.push(PathSegment::LineTo(p));
        self
    }

    /// Adds a quadratic Bézier curve to `end` via `control`. Returns
    /// `&mut Self` for chaining.
    pub fn quad_to(&mut self, control: Point, end: Point) -> &mut Self {
        self.segments.push(PathSegment::QuadTo { control, end });
        self
    }

    /// Adds a cubic Bézier curve to `end` via `c1` and `c2`. Returns
    /// `&mut Self` for chaining.
    pub fn cubic_to(&mut self, c1: Point, c2: Point, end: Point) -> &mut Self {
        self.segments.push(PathSegment::CubicTo {
            control1: c1,
            control2: c2,
            end,
        });
        self
    }

    /// Closes the current subpath. Returns `&mut Self` for chaining.
    pub fn close(&mut self) -> &mut Self {
        self.segments.push(PathSegment::Close);
        self
    }

    /// The ordered segments making up this path.
    #[must_use]
    pub fn segments(&self) -> &[PathSegment] {
        &self.segments
    }

    /// The fill rule used to determine this path's interior.
    #[must_use]
    pub fn fill_rule(&self) -> FillRule {
        self.fill_rule
    }

    /// Replaces the fill rule.
    pub fn set_fill_rule(&mut self, rule: FillRule) -> &mut Self {
        self.fill_rule = rule;
        self
    }

    /// Whether the path has no segments.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    /// The number of segments in the path.
    #[must_use]
    pub fn len(&self) -> usize {
        self.segments.len()
    }

    /// Builds a closed rectangular path (four line segments plus a close).
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

    /// Builds a closed rounded-rectangle path.
    ///
    /// Each corner is drawn with a single cubic Bézier segment whose control
    /// points use the [`KAPPA`] constant; this is a close but not exact
    /// approximation of a quarter-circle, which is standard for vector
    /// rounded rectangles. Each radius is clamped to at most half the shorter
    /// side of `rect` (and to be non-negative) so that adjacent corners never
    /// overlap.
    #[must_use]
    pub fn rounded_rect(rect: Rect, radii: CornerRadii) -> Path {
        let x0 = rect.origin.x;
        let y0 = rect.origin.y;
        let x1 = rect.right();
        let y1 = rect.bottom();

        let max_r = (rect.size.width.min(rect.size.height) / 2.0).max(0.0);
        let tl = radii.top_left.clamp(0.0, max_r);
        let tr = radii.top_right.clamp(0.0, max_r);
        let br = radii.bottom_right.clamp(0.0, max_r);
        let bl = radii.bottom_left.clamp(0.0, max_r);

        let mut path = Path::new();
        // Top edge, starting after the top-left corner.
        path.move_to(Point::new(x0 + tl, y0))
            .line_to(Point::new(x1 - tr, y0));
        // Top-right corner.
        path.cubic_to(
            Point::new(x1 - tr * (1.0 - KAPPA), y0),
            Point::new(x1, y0 + tr * (1.0 - KAPPA)),
            Point::new(x1, y0 + tr),
        );
        // Right edge.
        path.line_to(Point::new(x1, y1 - br));
        // Bottom-right corner.
        path.cubic_to(
            Point::new(x1, y1 - br * (1.0 - KAPPA)),
            Point::new(x1 - br * (1.0 - KAPPA), y1),
            Point::new(x1 - br, y1),
        );
        // Bottom edge.
        path.line_to(Point::new(x0 + bl, y1));
        // Bottom-left corner.
        path.cubic_to(
            Point::new(x0 + bl * (1.0 - KAPPA), y1),
            Point::new(x0, y1 - bl * (1.0 - KAPPA)),
            Point::new(x0, y1 - bl),
        );
        // Left edge.
        path.line_to(Point::new(x0, y0 + tl));
        // Top-left corner.
        path.cubic_to(
            Point::new(x0, y0 + tl * (1.0 - KAPPA)),
            Point::new(x0 + tl * (1.0 - KAPPA), y0),
            Point::new(x0 + tl, y0),
        );
        path.close();
        path
    }

    /// The axis-aligned bounding box of every point referenced by the path, or
    /// `None` if the path is empty (has no points).
    ///
    /// Control points of quadratic and cubic segments are included, so the box
    /// is a conservative bound: it always contains the drawn curves but may be
    /// larger than their tight bounds. Never panics.
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

        bounds.map(|(min, max)| {
            Rect::new(min, Size::new(max.x - min.x, max.y - min.y))
        })
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
        assert_eq!(path.segments()[0], PathSegment::MoveTo(Point::new(1.0, 2.0)));
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
    fn rounded_rect_negative_size_does_not_panic() {
        let r = Rect::new(Point::new(0.0, 0.0), Size::new(-5.0, -5.0));
        let path = Path::rounded_rect(r, CornerRadii::uniform(2.0));
        assert!(!path.is_empty());
    }
}
