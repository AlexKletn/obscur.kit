#![doc = include_str!("path_trim.doc/_module.md")]

use obscur_geometry::Point;

use super::{Path, PathSegment};

#[doc = include_str!("path_trim.doc/FLATTEN_TOLERANCE.md")]
const FLATTEN_TOLERANCE: f32 = 0.25;

#[doc = include_str!("path_trim.doc/MAX_STEPS.md")]
const MAX_STEPS: usize = 256;

#[doc = include_str!("path_trim.doc/PathTrim.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct PathTrim {
    #[doc = include_str!("path_trim.doc/points.md")]
    points: Vec<Point>,
    #[doc = include_str!("path_trim.doc/breaks.md")]
    breaks: Vec<usize>,
    #[doc = include_str!("path_trim.doc/lengths.md")]
    lengths: Vec<f32>,
}

impl PathTrim {
    #[doc = include_str!("path_trim.doc/of.md")]
    #[must_use]
    pub fn of(path: &Path) -> Self {
        let mut points: Vec<Point> = Vec::new();
        let mut breaks: Vec<usize> = Vec::new();
        let mut subpath_start = Point::new(0.0, 0.0);
        let mut current = Point::new(0.0, 0.0);
        let mut started = false;

        for segment in path.segments() {
            match segment {
                PathSegment::MoveTo(p) => {
                    if started {
                        breaks.push(points.len());
                    }
                    points.push(*p);
                    subpath_start = *p;
                    current = *p;
                    started = true;
                }
                PathSegment::LineTo(p) => {
                    if !started {
                        points.push(current);
                        started = true;
                    }
                    points.push(*p);
                    current = *p;
                }
                PathSegment::QuadTo { control, end } => {
                    if !started {
                        points.push(current);
                        started = true;
                    }
                    flatten_quad(current, *control, *end, &mut points);
                    current = *end;
                }
                PathSegment::CubicTo {
                    control1,
                    control2,
                    end,
                } => {
                    if !started {
                        points.push(current);
                        started = true;
                    }
                    flatten_cubic(current, *control1, *control2, *end, &mut points);
                    current = *end;
                }
                PathSegment::Close => {
                    if started && current != subpath_start {
                        points.push(subpath_start);
                    }
                    current = subpath_start;
                }
            }
        }

        // Cumulative arc length, one entry per point: `lengths[i]` is the
        // distance from the start of the path to `points[i]`. A break contributes
        // nothing, because the pen jumped rather than drew.
        let mut lengths = Vec::with_capacity(points.len());
        let mut total = 0.0;
        // `breaks` is built in increasing order, so a cursor answers "is this a
        // jump?" in constant time. `contains` would make this quadratic in the
        // number of subpaths — invisible for a ring, real for an icon.
        let mut next_break = 0;
        for (index, point) in points.iter().enumerate() {
            let jumped = breaks.get(next_break) == Some(&index);
            if jumped {
                next_break += 1;
            }
            if index > 0 && !jumped {
                total += distance(points[index - 1], *point);
            }
            lengths.push(total);
        }

        Self {
            points,
            breaks,
            lengths,
        }
    }

    #[doc = include_str!("path_trim.doc/length.md")]
    #[must_use]
    pub fn length(&self) -> f32 {
        self.lengths.last().copied().unwrap_or(0.0)
    }

    #[doc = include_str!("path_trim.doc/slice.md")]
    #[must_use]
    pub fn slice(&self, start: f32, end: f32) -> Path {
        let total = self.length();
        let mut out = Path::new();
        if total <= 0.0 || self.points.len() < 2 {
            return out;
        }
        // Out of order is empty rather than reversed: a caller asking for
        // `0.8..0.2` has a bug, and quietly drawing the complement would hide it.
        // Out of range clamps, because `0.0..1.0` is the whole path and asking
        // for more of it is not an error.
        let from = start.clamp(0.0, 1.0) * total;
        let to = end.clamp(0.0, 1.0) * total;
        if to <= from {
            return out;
        }

        let mut pen_down = false;
        // Same cursor as `of`, and for the same reason.
        let mut next_break = self.breaks.partition_point(|b| *b < 1);
        for index in 1..self.points.len() {
            if self.breaks.get(next_break) == Some(&index) {
                // The pen jumped here; the next drawn span starts fresh.
                next_break += 1;
                pen_down = false;
                continue;
            }
            let (a, b) = (self.points[index - 1], self.points[index]);
            let (sa, sb) = (self.lengths[index - 1], self.lengths[index]);
            if sb <= from || sa >= to {
                continue;
            }
            let span = sb - sa;
            let enter = if sa >= from {
                a
            } else {
                lerp(a, b, (from - sa) / span)
            };
            let leave = if sb <= to {
                b
            } else {
                lerp(a, b, (to - sa) / span)
            };
            if !pen_down {
                out.move_to(enter);
                pen_down = true;
            }
            out.line_to(leave);
        }
        out
    }
}

#[doc = include_str!("path_trim.doc/flatten_quad.md")]
fn flatten_quad(from: Point, control: Point, end: Point, out: &mut Vec<Point>) {
    let steps = steps_for(distance(from, control) + distance(control, end));
    for step in 1..=steps {
        let t = step as f32 / steps as f32;
        let inv = 1.0 - t;
        out.push(Point::new(
            inv * inv * from.x + 2.0 * inv * t * control.x + t * t * end.x,
            inv * inv * from.y + 2.0 * inv * t * control.y + t * t * end.y,
        ));
    }
}

#[doc = include_str!("path_trim.doc/flatten_cubic.md")]
fn flatten_cubic(from: Point, c1: Point, c2: Point, end: Point, out: &mut Vec<Point>) {
    let steps = steps_for(distance(from, c1) + distance(c1, c2) + distance(c2, end));
    for step in 1..=steps {
        let t = step as f32 / steps as f32;
        let inv = 1.0 - t;
        // The Bernstein basis, named so the four weights are readable as what
        // they are rather than as `a`, `b`, `c`, `d`.
        let w_from = inv * inv * inv;
        let w_c1 = 3.0 * inv * inv * t;
        let w_c2 = 3.0 * inv * t * t;
        let w_end = t * t * t;
        out.push(Point::new(
            w_from * from.x + w_c1 * c1.x + w_c2 * c2.x + w_end * end.x,
            w_from * from.y + w_c1 * c1.y + w_c2 * c2.y + w_end * end.y,
        ));
    }
}

#[doc = include_str!("path_trim.doc/steps_for.md")]
fn steps_for(control_polygon: f32) -> usize {
    // The control polygon is an upper bound on the curve's own length, so
    // stepping by it is conservative rather than optimistic — a curve is never
    // under-sampled, only sometimes over-sampled, which costs points and never
    // accuracy.
    if !control_polygon.is_finite() || control_polygon <= 0.0 {
        return 1;
    }
    let steps = (control_polygon / FLATTEN_TOLERANCE).sqrt().ceil();
    // Guarded above: finite and positive, so the cast can neither lose a sign
    // nor see a NaN. `clamp` bounds it whatever the magnitude.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let steps = steps as usize;
    steps.clamp(1, MAX_STEPS)
}

#[doc = include_str!("path_trim.doc/distance.md")]
fn distance(a: Point, b: Point) -> f32 {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    dx.hypot(dy)
}

#[doc = include_str!("path_trim.doc/lerp.md")]
fn lerp(a: Point, b: Point, t: f32) -> Point {
    Point::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t)
}

#[cfg(test)]
mod tests {
    use super::PathTrim;
    use crate::path::{Path, PathSegment};
    use obscur_geometry::{Point, Rect, Size};

    fn line(from: (f32, f32), to: (f32, f32)) -> Path {
        let mut path = Path::new();
        path.move_to(Point::new(from.0, from.1));
        path.line_to(Point::new(to.0, to.1));
        path
    }

    fn ends(path: &Path) -> Vec<Point> {
        path.segments()
            .iter()
            .filter_map(|s| match s {
                PathSegment::MoveTo(p) | PathSegment::LineTo(p) => Some(*p),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn a_straight_line_reports_its_own_length() {
        let trim = PathTrim::of(&line((0.0, 0.0), (30.0, 40.0)));
        assert!((trim.length() - 50.0).abs() < 0.001, "3-4-5");
    }

    #[test]
    fn the_whole_path_is_the_whole_path() {
        let trim = PathTrim::of(&line((0.0, 0.0), (100.0, 0.0)));
        assert_eq!(
            ends(&trim.slice(0.0, 1.0)),
            vec![Point::new(0.0, 0.0), Point::new(100.0, 0.0)]
        );
    }

    #[test]
    fn a_fraction_cuts_where_the_arc_length_says() {
        // The whole point: 0.4 of a 100px line ends at x = 40, and it must not
        // depend on how many segments the line happens to be made of.
        let trim = PathTrim::of(&line((0.0, 0.0), (100.0, 0.0)));
        let cut = ends(&trim.slice(0.0, 0.4));
        assert_eq!(cut[0], Point::new(0.0, 0.0));
        assert!((cut[1].x - 40.0).abs() < 0.001, "{cut:?}");
    }

    #[test]
    fn a_middle_span_starts_and_ends_where_it_should() {
        let trim = PathTrim::of(&line((0.0, 0.0), (100.0, 0.0)));
        let cut = ends(&trim.slice(0.25, 0.75));
        assert!((cut[0].x - 25.0).abs() < 0.001, "{cut:?}");
        assert!((cut[1].x - 75.0).abs() < 0.001, "{cut:?}");
    }

    #[test]
    fn an_empty_or_reversed_span_draws_nothing() {
        let trim = PathTrim::of(&line((0.0, 0.0), (100.0, 0.0)));
        assert!(trim.slice(0.5, 0.5).is_empty(), "zero width");
        assert!(
            trim.slice(0.8, 0.2).is_empty(),
            "out of order is empty, not the complement — quietly drawing the \
             complement would hide the caller's bug"
        );
    }

    #[test]
    fn a_fraction_out_of_range_clamps_rather_than_failing() {
        let trim = PathTrim::of(&line((0.0, 0.0), (100.0, 0.0)));
        assert_eq!(
            ends(&trim.slice(-1.0, 2.0)).len(),
            2,
            "clamped to the whole"
        );
    }

    #[test]
    fn a_trim_across_a_corner_keeps_the_corner() {
        // Two 100px arms; 0.25..0.75 spans the join, so the output must contain
        // the corner point itself and not cut it off.
        let mut path = Path::new();
        path.move_to(Point::new(0.0, 0.0));
        path.line_to(Point::new(100.0, 0.0));
        path.line_to(Point::new(100.0, 100.0));
        let trim = PathTrim::of(&path);
        assert!((trim.length() - 200.0).abs() < 0.001);

        let cut = ends(&trim.slice(0.25, 0.75));
        assert!(
            cut.contains(&Point::new(100.0, 0.0)),
            "the corner survives the cut: {cut:?}"
        );
        assert!((cut[0].x - 50.0).abs() < 0.001, "{cut:?}");
        assert!((cut[cut.len() - 1].y - 50.0).abs() < 0.001, "{cut:?}");
    }

    #[test]
    fn a_closed_rectangle_measures_its_whole_perimeter() {
        // `Close` draws the last side, so it has to count. A ring whose final
        // side were free would leave every fraction wrong.
        let path = Path::rect(Rect::new(Point::new(0.0, 0.0), Size::new(30.0, 20.0)));
        let trim = PathTrim::of(&path);
        assert!(
            (trim.length() - 100.0).abs() < 0.001,
            "2*(30+20), got {}",
            trim.length()
        );
    }

    #[test]
    fn a_curve_is_flattened_to_about_its_true_length() {
        // A cubic approximating a quarter circle of radius 100: true arc length
        // is pi*100/2 = 157.08. The flattening must land close, or every
        // fraction along a ring is wrong.
        const K: f32 = 0.552_285;
        let mut path = Path::new();
        path.move_to(Point::new(100.0, 0.0));
        path.cubic_to(
            Point::new(100.0, 100.0 * K),
            Point::new(100.0 * K, 100.0),
            Point::new(0.0, 100.0),
        );
        let length = PathTrim::of(&path).length();
        assert!(
            (length - 157.08).abs() < 1.0,
            "quarter circle came out {length}"
        );
    }

    #[test]
    fn two_subpaths_share_one_length_domain_and_the_gap_is_not_drawn() {
        // The decision: the whole path is one domain in document order, so a
        // caller trimming a two-subpath icon gets a consistent fraction. The pen
        // jump between them contributes no length and no line.
        let mut path = Path::new();
        path.move_to(Point::new(0.0, 0.0));
        path.line_to(Point::new(50.0, 0.0));
        path.move_to(Point::new(0.0, 100.0));
        path.line_to(Point::new(50.0, 100.0));
        let trim = PathTrim::of(&path);
        assert!(
            (trim.length() - 100.0).abs() < 0.001,
            "the jump adds nothing: {}",
            trim.length()
        );

        let whole = trim.slice(0.0, 1.0);
        let moves = whole
            .segments()
            .iter()
            .filter(|s| matches!(s, PathSegment::MoveTo(_)))
            .count();
        assert_eq!(moves, 2, "the gap stays a gap: {:?}", whole.segments());
    }

    #[test]
    fn an_empty_path_measures_nothing_and_slices_to_nothing() {
        let trim = PathTrim::of(&Path::new());
        assert_eq!(trim.length(), 0.0);
        assert!(trim.slice(0.0, 1.0).is_empty());
    }

    #[test]
    fn a_degenerate_path_of_one_point_does_not_divide_by_zero() {
        let mut path = Path::new();
        path.move_to(Point::new(5.0, 5.0));
        let trim = PathTrim::of(&path);
        assert_eq!(trim.length(), 0.0);
        assert!(trim.slice(0.0, 1.0).is_empty());
    }
}
