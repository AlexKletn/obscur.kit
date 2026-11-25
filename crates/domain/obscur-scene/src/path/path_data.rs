#![doc = include_str!("../../docs/api/path/path_data.doc/_module.md")]

use obscur_geometry::Point;

use super::{
    Path, PathDataError, PathDataFault, path_data_cursor::PathDataCursor, svg_arc::SvgArc,
};

/// Parses SVG path data into a [`Path`].
///
/// # Errors
///
/// Returns the byte offset and the kind of the first fault: a number where a
/// command was expected, a letter naming no command, a command whose numbers ran
/// out, a number trailing a `Z`, an invalid arc flag, or a coordinate that
/// cannot be represented in the scene.
pub fn parse_path_data(data: &str) -> Result<Path, PathDataError> {
    PathDataParser::new(data).run()
}

/// The parse in progress: where the pen is, and what the previous curve left behind for `S` and `T` to reflect.
struct PathDataParser<'a> {
    cursor: PathDataCursor<'a>,
    path: Path,
    current: Point,
    subpath_start: Point,
    // The previous curve's second control point, reflected by `S` and `T`.
    last_cubic_control: Option<Point>,
    last_quad_control: Option<Point>,
    started: bool,
}

impl<'a> PathDataParser<'a> {
    fn new(data: &'a str) -> Self {
        Self {
            cursor: PathDataCursor::new(data),
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
            self.cursor.skip_separators();
            let Some(byte) = self.cursor.peek() else {
                return Ok(self.path);
            };
            let entry = self.cursor.offset();
            if byte.is_ascii_alphabetic() {
                command = byte;
                self.cursor.advance();
            } else if command == 0 {
                // A number before any command: SVG has no implicit opening verb,
                // and guessing one would turn a typo into a drawing.
                return Err(self.cursor.fault(PathDataFault::LeadingCommand));
            } else if matches!(command, b'Z' | b'z') {
                // A standing command repeats by consuming another argument list.
                // `Z` has none to consume, and SVG's grammar allows nothing but
                // a command letter after a closepath — so this byte belongs to
                // no verb. Left to repeat, `Z` read nothing and closed the
                // subpath again, forever, growing the path until the machine
                // ran out of memory.
                return Err(self.cursor.fault(PathDataFault::StrayNumber));
            } else if matches!(command, b'M') {
                // "If a moveto is followed by multiple pairs of coordinates,
                // the subsequent pairs are treated as implicit lineto commands."
                command = b'L';
            } else if matches!(command, b'm') {
                command = b'l';
            }
            self.step(command)?;
            if self.cursor.offset() == entry {
                // Every turn of this loop must consume a byte. As the arms of
                // `step` stand each one either reads a number — which cannot
                // succeed on nothing — or returns, so this is unreachable; it
                // is the net under them, so that no future arm has to be
                // trusted to advance. A parser that spins here does not merely
                // hang: a canvas re-parses its path data on every paint, so it
                // takes the frame thread and then the machine's memory with it.
                self.cursor.advance();
            }
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
                let x = self.coordinate(origin.x)?;
                let p = Point::new(x, self.current.y);
                self.line(p);
                self.forget_controls();
            }
            b'V' => {
                let y = self.coordinate(origin.y)?;
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
                let c1 = self.reflected_control(self.last_cubic_control)?;
                let (c2, end) = (self.point(origin)?, self.point(origin)?);
                self.cubic(c1, c2, end);
            }
            b'Q' => {
                let (control, end) = (self.point(origin)?, self.point(origin)?);
                self.quad(control, end);
            }
            b'T' => {
                let control = self.reflected_control(self.last_quad_control)?;
                let end = self.point(origin)?;
                self.quad(control, end);
            }
            b'A' => {
                let (rx, ry) = (self.cursor.number()?, self.cursor.number()?);
                let rotation = self.cursor.number()?;
                let large = self.cursor.flag()?;
                let sweep = self.cursor.flag()?;
                let end = self.point(origin)?;
                self.arc(
                    SvgArc {
                        radii: [rx, ry],
                        rotation_degrees: rotation,
                        large,
                        sweep,
                    },
                    end,
                )?;
                self.forget_controls();
            }
            b'Z' => {
                if self.started {
                    self.path.close();
                    self.current = self.subpath_start;
                }
                self.forget_controls();
            }
            _ => return Err(self.cursor.fault(PathDataFault::UnknownCommand)),
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

    #[allow(clippy::cast_possible_truncation)]
    fn reflected_control(&self, previous: Option<Point>) -> Result<Point, PathDataError> {
        let Some(previous) = previous else {
            return Ok(self.current);
        };
        // Double precision avoids overflowing the intermediate even when the reflection fits.
        let reflect = |end, control| (2.0 * f64::from(end) - f64::from(control)) as f32;
        let (x, y) = (
            reflect(self.current.x, previous.x),
            reflect(self.current.y, previous.y),
        );
        if !(x.is_finite() && y.is_finite()) {
            return Err(self.cursor.fault(PathDataFault::CoordinateOutOfRange));
        }
        Ok(Point::new(x, y))
    }

    fn arc(&mut self, arc: SvgArc, end: Point) -> Result<(), PathDataError> {
        let start = self.current;
        if start == end {
            return Ok(());
        }
        self.ensure_started();
        arc.append(&mut self.path, start, end)
            .map_err(|fault| self.cursor.fault(fault))?;
        self.current = end;
        Ok(())
    }

    fn point(&mut self, origin: Point) -> Result<Point, PathDataError> {
        Ok(Point::new(
            self.coordinate(origin.x)?,
            self.coordinate(origin.y)?,
        ))
    }

    fn coordinate(&mut self, origin: f32) -> Result<f32, PathDataError> {
        self.cursor.skip_separators();
        let at = self.cursor.offset();
        let coordinate = self.cursor.number()? + origin;
        coordinate
            .is_finite()
            .then_some(coordinate)
            .ok_or(PathDataError {
                at,
                what: PathDataFault::CoordinateOutOfRange,
            })
    }
}

#[cfg(test)]
mod tests {
    use super::{PathDataFault, parse_path_data};
    use crate::path::{PathSegment, PathTrim};
    use obscur_geometry::Point;

    #[test]
    fn rejects_relative_coordinates_that_overflow() {
        for data in ["M3e38 0 h3e38", "M0 3e38 v3e38", "M3e38 0 l3e38 0"] {
            let error = parse_path_data(data).unwrap_err();
            assert_eq!(error.what, PathDataFault::CoordinateOutOfRange);
            assert_eq!(&data[error.at..error.at + 4], "3e38");
        }
    }

    #[test]
    fn smooth_controls_are_checked_after_reflection() {
        assert!(parse_path_data("M3e38 0 Q3e38 0 3e38 1 T3e38 2").is_ok());
        let error = parse_path_data("M3e38 0 Q0 0 3e38 1 T3e38 2").unwrap_err();
        assert_eq!(error.what, PathDataFault::CoordinateOutOfRange);
    }

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

    /// Verifies bounds and cubic controls of shallow arcs with very large radii.
    #[test]
    fn shallow_arcs_with_large_radii_keep_their_chord() {
        for radius in ["1e18", "2e19", "1e30", "3e38"] {
            let path = parse(&format!("M0 0 A{radius} {radius} 0 0 1 100 0"));
            let [
                _,
                PathSegment::CubicTo {
                    control1,
                    control2,
                    end,
                },
            ] = path.segments()
            else {
                panic!("expected one shallow cubic");
            };
            assert_eq!(*end, Point::new(100.0, 0.0));
            assert!((control1.x - 100.0 / 3.0).abs() < 1e-4);
            assert!((control2.x - 200.0 / 3.0).abs() < 1e-4);
            assert!(control1.y.abs() < 1e-4 && control2.y.abs() < 1e-4);
            assert_eq!(path.bounds().unwrap().size.width, 100.0);
        }
        assert_eq!(
            parse_path_data("M0 0 A3e38 3e38 0 1 1 100 0")
                .unwrap_err()
                .what,
            PathDataFault::ArcOutOfRange,
        );
    }

    /// Compares compact and spaced arc flags and rejects non-binary flags.
    #[test]
    fn arc_flags_can_touch_one_another_and_the_coordinate() {
        for (compact, spaced) in [
            ("M0 0 A50 50 0 11 100 0", "M0 0 A50 50 0 1 1 100 0"),
            ("M0 0 A50 50 0 1 1100 0", "M0 0 A50 50 0 1 1 100 0"),
            ("M0 0 A50 50 0 0150 0", "M0 0 A50 50 0 0 1 50 0"),
            ("M0 0 a50 50 0 0150 0", "M0 0 a50 50 0 0 1 50 0"),
        ] {
            assert_eq!(parse(compact), parse(spaced));
        }
        assert_eq!(
            parse_path_data("M0 0 A50 50 0 2 1 100 0").unwrap_err().what,
            PathDataFault::InvalidArcFlag,
        );
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
    fn a_number_after_a_close_is_refused_rather_than_repeated_forever() {
        // `Z` takes no arguments, so it consumes no bytes; the standing command
        // stayed `Z` until a letter replaced it, and no letter follows. This
        // input closed the subpath again every turn and never returned — a
        // canvas re-parsing its path data hung the frame thread on the first
        // paint and then ate the machine's memory.
        let error = parse_path_data("M0 0 L10 0 Z 1").expect_err("closepath takes no arguments");
        assert_eq!(error.what, PathDataFault::StrayNumber);
        assert_eq!(error.at, 13, "the position points at the stray number");
    }

    #[test]
    fn a_relative_close_is_no_more_repeatable_than_an_absolute_one() {
        assert_eq!(
            parse_path_data("M0 0 z 5")
                .expect_err("z takes none either")
                .what,
            PathDataFault::StrayNumber
        );
    }

    #[test]
    fn no_command_can_spin_the_loop_on_a_byte_it_cannot_use() {
        // The property the net under `run`'s loop exists for, asserted of every
        // verb: put a byte in front of one that is neither a letter nor a
        // separator and it must decide something, in finite time.
        for command in "MmLlHhVvCcSsQqTtAaZz".chars() {
            let data = format!("M0 0 {command} !");
            assert!(
                parse_path_data(&data).is_err(),
                "`{data}` names no drawing and has to say so rather than loop"
            );
        }
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
