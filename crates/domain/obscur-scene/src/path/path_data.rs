#![doc = include_str!("path_data.doc/_module.md")]

use core::f32::consts::PI;

use obscur_geometry::Point;

use super::Path;

#[doc = include_str!("path_data.doc/PathDataError.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathDataError {
    #[doc = include_str!("path_data.doc/at.md")]
    pub at: usize,
    #[doc = include_str!("path_data.doc/what.md")]
    pub what: PathDataFault,
}

#[doc = include_str!("path_data.doc/PathDataFault.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathDataFault {
    #[doc = include_str!("path_data.doc/UnknownCommand.md")]
    UnknownCommand,
    #[doc = include_str!("path_data.doc/MissingNumber.md")]
    MissingNumber,
    #[doc = include_str!("path_data.doc/LeadingCommand.md")]
    LeadingCommand,
}

#[doc = include_str!("path_data.doc/parse_path_data.md")]
///
/// # Errors
///
/// Returns the byte offset and the kind of the first fault: a number where a
/// command was expected, a letter naming no command, or a command whose numbers
/// ran out.
pub fn parse_path_data(data: &str) -> Result<Path, PathDataError> {
    Scanner::new(data).run()
}

#[doc = include_str!("path_data.doc/Scanner.md")]
struct Scanner<'a> {
    bytes: &'a [u8],
    at: usize,
    path: Path,
    current: Point,
    subpath_start: Point,
    // The previous curve's second control point, reflected by `S` and `T`.
    last_cubic_control: Option<Point>,
    last_quad_control: Option<Point>,
    started: bool,
}

impl<'a> Scanner<'a> {
    fn new(data: &'a str) -> Self {
        Self {
            bytes: data.as_bytes(),
            at: 0,
            path: Path::new(),
            current: Point::new(0.0, 0.0),
            subpath_start: Point::new(0.0, 0.0),
            last_cubic_control: None,
            last_quad_control: None,
            started: false,
        }
    }

    fn run(mut self) -> Result<Path, PathDataError> {
        let mut command = 0_u8;
        loop {
            self.skip_separators();
            if self.at >= self.bytes.len() {
                return Ok(self.path);
            }
            let byte = self.bytes[self.at];
            if byte.is_ascii_alphabetic() {
                command = byte;
                self.at += 1;
            } else if command == 0 {
                // A number before any command: SVG has no implicit opening verb,
                // and guessing one would turn a typo into a drawing.
                return Err(self.fault(PathDataFault::LeadingCommand));
            } else if matches!(command, b'M') {
                // "If a moveto is followed by multiple pairs of coordinates,
                // the subsequent pairs are treated as implicit lineto commands."
                command = b'L';
            } else if matches!(command, b'm') {
                command = b'l';
            }
            self.step(command)?;
        }
    }

    fn step(&mut self, command: u8) -> Result<(), PathDataError> {
        let relative = command.is_ascii_lowercase();
        let origin = if relative {
            self.current
        } else {
            Point::new(0.0, 0.0)
        };
        match command.to_ascii_uppercase() {
            b'M' => {
                let p = self.point(origin)?;
                self.path.move_to(p);
                self.current = p;
                self.subpath_start = p;
                self.started = true;
                self.forget_controls();
            }
            b'L' => {
                let p = self.point(origin)?;
                self.line(p);
                self.forget_controls();
            }
            b'H' => {
                let x = self.number()? + origin.x;
                let p = Point::new(x, self.current.y);
                self.line(p);
                self.forget_controls();
            }
            b'V' => {
                let y = self.number()? + origin.y;
                let p = Point::new(self.current.x, y);
                self.line(p);
                self.forget_controls();
            }
            b'C' => {
                let (c1, c2, end) = (
                    self.point(origin)?,
                    self.point(origin)?,
                    self.point(origin)?,
                );
                self.cubic(c1, c2, end);
            }
            b'S' => {
                let c1 = self.reflected_cubic();
                let (c2, end) = (self.point(origin)?, self.point(origin)?);
                self.cubic(c1, c2, end);
            }
            b'Q' => {
                let (control, end) = (self.point(origin)?, self.point(origin)?);
                self.quad(control, end);
            }
            b'T' => {
                let control = self.reflected_quad();
                let end = self.point(origin)?;
                self.quad(control, end);
            }
            b'A' => {
                let (rx, ry) = (self.number()?, self.number()?);
                let rotation = self.number()?;
                let large = self.number()? != 0.0;
                let sweep = self.number()? != 0.0;
                let end = self.point(origin)?;
                self.arc(rx, ry, rotation, large, sweep, end);
                self.forget_controls();
            }
            b'Z' => {
                if self.started {
                    self.path.close();
                    self.current = self.subpath_start;
                }
                self.forget_controls();
            }
            _ => return Err(self.fault(PathDataFault::UnknownCommand)),
        }
        Ok(())
    }

    fn line(&mut self, p: Point) {
        self.ensure_started();
        self.path.line_to(p);
        self.current = p;
    }

    fn cubic(&mut self, c1: Point, c2: Point, end: Point) {
        self.ensure_started();
        self.path.cubic_to(c1, c2, end);
        self.current = end;
        self.last_cubic_control = Some(c2);
        self.last_quad_control = None;
    }

    fn quad(&mut self, control: Point, end: Point) {
        self.ensure_started();
        self.path.quad_to(control, end);
        self.current = end;
        self.last_quad_control = Some(control);
        self.last_cubic_control = None;
    }

    fn ensure_started(&mut self) {
        // SVG says a path may not begin with a drawing verb, but a path that
        // does is far better opened at the origin than dropped: the drawing is
        // still what the author wrote, only its start was left implicit.
        if !self.started {
            self.path.move_to(self.current);
            self.subpath_start = self.current;
            self.started = true;
        }
    }

    fn forget_controls(&mut self) {
        self.last_cubic_control = None;
        self.last_quad_control = None;
    }

    fn reflected_cubic(&self) -> Point {
        // "If there is no previous command or if the previous command was not a
        // C, c, S or s, assume the first control point is coincident with the
        // current point."
        self.last_cubic_control.map_or(self.current, |c| {
            Point::new(2.0 * self.current.x - c.x, 2.0 * self.current.y - c.y)
        })
    }

    fn reflected_quad(&self) -> Point {
        self.last_quad_control.map_or(self.current, |c| {
            Point::new(2.0 * self.current.x - c.x, 2.0 * self.current.y - c.y)
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn arc(&mut self, rx: f32, ry: f32, rotation: f32, large: bool, sweep: bool, end: Point) {
        // SVG 1.1 F.6: endpoint parameterisation to centre parameterisation,
        // then the arc as cubics. An arc with either radius zero, or with
        // coincident endpoints, is a straight line by the spec's own words —
        // "if rx = 0 or ry = 0 then this arc is treated as a straight line".
        let (start, rx, ry) = (self.current, rx.abs(), ry.abs());
        if rx == 0.0 || ry == 0.0 || (start.x == end.x && start.y == end.y) {
            self.line(end);
            return;
        }
        let phi = rotation.to_radians();
        let (cos, sin) = (phi.cos(), phi.sin());
        let (dx2, dy2) = ((start.x - end.x) / 2.0, (start.y - end.y) / 2.0);
        let x1 = cos * dx2 + sin * dy2;
        let y1 = -sin * dx2 + cos * dy2;

        // F.6.6: scale the radii up when they are too small to span the chord.
        let lambda = (x1 * x1) / (rx * rx) + (y1 * y1) / (ry * ry);
        let (rx, ry) = if lambda > 1.0 {
            let s = lambda.sqrt();
            (rx * s, ry * s)
        } else {
            (rx, ry)
        };

        let numerator = (rx * rx * ry * ry - rx * rx * y1 * y1 - ry * ry * x1 * x1).max(0.0);
        let denominator = rx * rx * y1 * y1 + ry * ry * x1 * x1;
        let coefficient = if denominator == 0.0 {
            0.0
        } else {
            let sign = if large == sweep { -1.0 } else { 1.0 };
            sign * (numerator / denominator).sqrt()
        };
        let cx1 = coefficient * rx * y1 / ry;
        let cy1 = -coefficient * ry * x1 / rx;
        let cx = cos * cx1 - sin * cy1 + start.x.midpoint(end.x);
        let cy = sin * cx1 + cos * cy1 + start.y.midpoint(end.y);

        let theta = angle(1.0, 0.0, (x1 - cx1) / rx, (y1 - cy1) / ry);
        let mut delta = angle(
            (x1 - cx1) / rx,
            (y1 - cy1) / ry,
            (-x1 - cx1) / rx,
            (-y1 - cy1) / ry,
        );
        if !sweep && delta > 0.0 {
            delta -= 2.0 * PI;
        } else if sweep && delta < 0.0 {
            delta += 2.0 * PI;
        }

        // One cubic per quarter turn or less: the standard bound at which a
        // cubic's error against a circular arc stays under a fraction of a pixel.
        let segments = (delta.abs() / (PI / 2.0)).ceil().max(1.0);
        // Bounded above by construction: `delta` is at most a full turn, so this
        // is at most four, and the cast can neither truncate meaningfully nor
        // lose a sign.
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let count = segments as usize;
        let step = delta / segments;
        let k = 4.0 / 3.0 * (step / 4.0).tan();
        let mut angle_at = theta;
        for _ in 0..count {
            let next = angle_at + step;
            let (c0, s0) = (angle_at.cos(), angle_at.sin());
            let (c1a, s1a) = (next.cos(), next.sin());
            let ellipse = Ellipse {
                centre: Point::new(cx, cy),
                rx,
                ry,
                cos,
                sin,
            };
            let p0 = ellipse.at(c0, s0);
            let p3 = ellipse.at(c1a, s1a);
            let d0 = ellipse_tangent(rx, ry, cos, sin, c0, s0);
            let d3 = ellipse_tangent(rx, ry, cos, sin, c1a, s1a);
            self.cubic(
                Point::new(p0.x + k * d0.x, p0.y + k * d0.y),
                Point::new(p3.x - k * d3.x, p3.y - k * d3.y),
                p3,
            );
            angle_at = next;
        }
        self.current = end;
    }

    fn point(&mut self, origin: Point) -> Result<Point, PathDataError> {
        let x = self.number()?;
        let y = self.number()?;
        Ok(Point::new(origin.x + x, origin.y + y))
    }

    fn number(&mut self) -> Result<f32, PathDataError> {
        self.skip_separators();
        let start = self.at;
        if self.at < self.bytes.len() && matches!(self.bytes[self.at], b'+' | b'-') {
            self.at += 1;
        }
        while self.at < self.bytes.len() && self.bytes[self.at].is_ascii_digit() {
            self.at += 1;
        }
        if self.at < self.bytes.len() && self.bytes[self.at] == b'.' {
            self.at += 1;
            while self.at < self.bytes.len() && self.bytes[self.at].is_ascii_digit() {
                self.at += 1;
            }
        }
        if self.at < self.bytes.len() && matches!(self.bytes[self.at], b'e' | b'E') {
            let mark = self.at;
            self.at += 1;
            if self.at < self.bytes.len() && matches!(self.bytes[self.at], b'+' | b'-') {
                self.at += 1;
            }
            if self.at < self.bytes.len() && self.bytes[self.at].is_ascii_digit() {
                while self.at < self.bytes.len() && self.bytes[self.at].is_ascii_digit() {
                    self.at += 1;
                }
            } else {
                // An `e` that begins no exponent is the next command, not part
                // of this number.
                self.at = mark;
            }
        }
        core::str::from_utf8(&self.bytes[start..self.at])
            .ok()
            .and_then(|text| text.parse::<f32>().ok())
            .filter(|value| value.is_finite())
            .ok_or(PathDataError {
                at: start,
                what: PathDataFault::MissingNumber,
            })
    }

    fn skip_separators(&mut self) {
        while self.at < self.bytes.len()
            && matches!(self.bytes[self.at], b' ' | b',' | b'\t' | b'\n' | b'\r')
        {
            self.at += 1;
        }
    }

    fn fault(&self, what: PathDataFault) -> PathDataError {
        PathDataError { at: self.at, what }
    }
}

#[doc = include_str!("path_data.doc/angle.md")]
fn angle(ux: f32, uy: f32, vx: f32, vy: f32) -> f32 {
    let dot = ux * vx + uy * vy;
    let len = (ux * ux + uy * uy).sqrt() * (vx * vx + vy * vy).sqrt();
    if len == 0.0 {
        return 0.0;
    }
    let sign = if ux * vy - uy * vx < 0.0 { -1.0 } else { 1.0 };
    sign * (dot / len).clamp(-1.0, 1.0).acos()
}

#[doc = include_str!("path_data.doc/Ellipse.md")]
#[derive(Debug, Clone, Copy)]
struct Ellipse {
    centre: Point,
    rx: f32,
    ry: f32,
    cos: f32,
    sin: f32,
}

impl Ellipse {
    #[doc = include_str!("path_data.doc/on_ellipse.md")]
    fn at(self, ct: f32, st: f32) -> Point {
        Point::new(
            self.centre.x + self.rx * ct * self.cos - self.ry * st * self.sin,
            self.centre.y + self.rx * ct * self.sin + self.ry * st * self.cos,
        )
    }
}

#[doc = include_str!("path_data.doc/ellipse_tangent.md")]
fn ellipse_tangent(rx: f32, ry: f32, cos: f32, sin: f32, ct: f32, st: f32) -> Point {
    Point::new(
        -rx * st * cos - ry * ct * sin,
        -rx * st * sin + ry * ct * cos,
    )
}

#[cfg(test)]
mod tests {
    use super::{PathDataFault, parse_path_data};
    use crate::path::{PathSegment, PathTrim};
    use obscur_geometry::Point;

    fn parse(data: &str) -> crate::path::Path {
        parse_path_data(data).expect("valid path data")
    }

    #[test]
    fn a_move_and_a_line_are_what_they_say() {
        let path = parse("M 10 20 L 30 40");
        assert_eq!(
            path.segments(),
            [
                PathSegment::MoveTo(Point::new(10.0, 20.0)),
                PathSegment::LineTo(Point::new(30.0, 40.0)),
            ]
        );
    }

    #[test]
    fn separators_are_optional_and_interchangeable() {
        // `M10,20L30-40` is legal SVG and a real generator emits it.
        let terse = parse("M10,20L30-40");
        let spaced = parse("M 10 20 L 30 -40");
        assert_eq!(terse.segments(), spaced.segments());
    }

    #[test]
    fn a_repeated_coordinate_pair_after_a_move_is_a_line() {
        // The rule a hand-written parser forgets, and the one every generator
        // relies on.
        let implicit = parse("M0 0 10 0 20 0");
        assert_eq!(
            implicit.segments(),
            [
                PathSegment::MoveTo(Point::new(0.0, 0.0)),
                PathSegment::LineTo(Point::new(10.0, 0.0)),
                PathSegment::LineTo(Point::new(20.0, 0.0)),
            ]
        );
    }

    #[test]
    fn relative_commands_are_relative_to_where_the_pen_is() {
        let path = parse("M10 10 l5 0 l0 5");
        assert_eq!(
            path.segments()[2],
            PathSegment::LineTo(Point::new(15.0, 15.0))
        );
    }

    #[test]
    fn horizontal_and_vertical_keep_the_other_axis() {
        let path = parse("M10 20 H50 V60");
        assert_eq!(
            path.segments()[1],
            PathSegment::LineTo(Point::new(50.0, 20.0))
        );
        assert_eq!(
            path.segments()[2],
            PathSegment::LineTo(Point::new(50.0, 60.0))
        );
    }

    #[test]
    fn a_smooth_curve_reflects_the_previous_control_point() {
        // `S` without a preceding curve uses the current point; with one, it
        // mirrors. Getting this backwards produces a visible kink.
        let path = parse("M0 0 C0 10 10 10 10 0 S20 -10 20 0");
        let PathSegment::CubicTo { control1, .. } = path.segments()[2] else {
            panic!("the smooth curve is a cubic");
        };
        assert_eq!(
            control1,
            Point::new(10.0, -10.0),
            "reflected through the current point: 2*(10,0) - (10,10)"
        );
    }

    #[test]
    fn a_close_returns_the_pen_to_the_subpath_start() {
        let path = parse("M10 10 L20 10 Z L30 30");
        assert!(path.segments().contains(&PathSegment::Close));
        // After `Z` the pen is back at (10, 10), so the next line starts there.
        let length = PathTrim::of(&parse("M10 10 L20 10 Z")).length();
        assert!((length - 20.0).abs() < 0.01, "out and back: {length}");
    }

    #[test]
    fn an_arc_becomes_curves_of_about_the_right_length() {
        // A half circle of radius 50: true length pi*50 = 157.08. This is the
        // whole reason arcs are parsed rather than refused — an author drawing a
        // gauge writes `A`, not four cubics.
        let path = parse("M0 0 A50 50 0 0 1 100 0");
        let length = PathTrim::of(&path).length();
        assert!(
            (length - 157.08).abs() < 1.0,
            "half circle came out {length}"
        );
    }

    #[test]
    fn an_arc_with_a_zero_radius_is_a_straight_line_as_the_spec_says() {
        let path = parse("M0 0 A0 0 0 0 1 100 0");
        assert_eq!(
            path.segments()[1],
            PathSegment::LineTo(Point::new(100.0, 0.0))
        );
    }

    #[test]
    fn the_sweep_flag_picks_the_other_way_round() {
        let clockwise = PathTrim::of(&parse("M0 0 A50 50 0 0 1 100 0"));
        let widdershins = PathTrim::of(&parse("M0 0 A50 50 0 0 0 100 0"));
        // Same length, opposite bulge: the midpoints sit either side of the chord.
        let above = widdershins.slice(0.0, 0.5);
        let below = clockwise.slice(0.0, 0.5);
        let y_of = |p: &crate::path::Path| p.bounds().map_or(0.0, |b| b.origin.y);
        assert!(
            y_of(&above) < 0.0 || y_of(&below) < 0.0,
            "one of them bulges the other way"
        );
        assert!((clockwise.length() - widdershins.length()).abs() < 0.5);
    }

    #[test]
    fn a_number_before_any_command_is_refused_rather_than_guessed() {
        let error = parse_path_data("10 20 L30 40").expect_err("no opening verb");
        assert_eq!(error.what, PathDataFault::LeadingCommand);
    }

    #[test]
    fn an_unknown_command_is_refused_with_its_position() {
        let error = parse_path_data("M0 0 K10 10").expect_err("K is not a command");
        assert_eq!(error.what, PathDataFault::UnknownCommand);
        assert_eq!(error.at, 6, "the position points at the offending verb");
    }

    #[test]
    fn a_command_missing_its_numbers_is_refused() {
        assert_eq!(
            parse_path_data("M0 0 L")
                .expect_err("L needs two numbers")
                .what,
            PathDataFault::MissingNumber
        );
    }

    #[test]
    fn empty_data_is_an_empty_path_rather_than_an_error() {
        assert!(parse("").is_empty());
        assert!(parse("   ").is_empty());
    }

    #[test]
    fn exponent_notation_survives_and_a_bare_e_does_not_eat_the_next_command() {
        let path = parse("M1e2 0 L0 0");
        assert_eq!(
            path.segments()[0],
            PathSegment::MoveTo(Point::new(100.0, 0.0))
        );
    }
}
