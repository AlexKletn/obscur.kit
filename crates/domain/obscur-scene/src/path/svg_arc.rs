//! Converts an SVG endpoint arc into cubic segments.

use core::f64::consts::{FRAC_PI_2, TAU};

use obscur_geometry::Point;

use super::{Path, PathDataFault};

pub(super) struct SvgArc {
    pub(super) radii: [f32; 2],
    pub(super) rotation_degrees: f32,
    pub(super) large: bool,
    pub(super) sweep: bool,
}

impl SvgArc {
    /// Appends the arc between distinct endpoints after validating its control points.
    pub(super) fn append(
        self,
        path: &mut Path,
        start: Point,
        end: Point,
    ) -> Result<(), PathDataFault> {
        let [rx, ry] = self.radii;
        if self.radii.contains(&0.0) {
            path.line_to(end);
            return Ok(());
        }
        let Self {
            rotation_degrees,
            large,
            sweep,
            ..
        } = self;
        let (mut rx, mut ry) = (f64::from(rx).abs(), f64::from(ry).abs());
        let (sin, cos) = f64::from(rotation_degrees)
            .rem_euclid(360.0)
            .to_radians()
            .sin_cos();
        let dx = (f64::from(start.x) - f64::from(end.x)) / 2.0;
        let dy = (f64::from(start.y) - f64::from(end.y)) / 2.0;
        let mut half_chord_x = (cos * dx + sin * dy) / rx;
        let mut half_chord_y = (-sin * dx + cos * dy) / ry;
        let mut chord = half_chord_x.hypot(half_chord_y);
        if chord > 1.0 {
            rx *= chord;
            ry *= chord;
            half_chord_x /= chord;
            half_chord_y /= chord;
            chord = 1.0;
        }

        // Work in the unit circle. The half chord gives the sweep directly,
        // including shallow arcs where subtracting endpoint angles loses it.
        let sign = if large == sweep { -1.0 } else { 1.0 };
        let normal = sign * (1.0 - chord * chord).sqrt();
        let centre_x = (half_chord_y / chord) * normal;
        let centre_y = -(half_chord_x / chord) * normal;
        let (start_cos, start_sin) = (half_chord_x - centre_x, half_chord_y - centre_y);
        let minor = 2.0 * chord.asin();
        let sweep_radians =
            if large { TAU - minor } else { minor } * if sweep { 1.0 } else { -1.0 };
        let segments = (sweep_radians.abs() / FRAC_PI_2).ceil().max(1.0);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let count = segments as usize;
        let step = sweep_radians / segments;
        let control_scale = (4.0 / 3.0) * (step / 4.0).tan();

        let map = |x, y| (rx * x * cos - ry * y * sin, rx * x * sin + ry * y * cos);
        let point_at = |t: f64| {
            // Evaluate relative to the known start instead of adding large
            // terms about the ellipse centre. Half angles avoid cancellation.
            let (sh, ch) = (t / 2.0).sin_cos();
            let diff_cos = -2.0 * sh * sh;
            let diff_sin = 2.0 * sh * ch;
            let (x, y) = map(
                start_cos * diff_cos - start_sin * diff_sin,
                start_sin * diff_cos + start_cos * diff_sin,
            );
            (f64::from(start.x) + x, f64::from(start.y) + y)
        };
        let tangent_at = |t: f64| {
            let (s, c) = t.sin_cos();
            map(
                -start_sin * c - start_cos * s,
                start_cos * c - start_sin * s,
            )
        };
        let mut p0 = (f64::from(start.x), f64::from(start.y));
        let mut start_angle = 0.0;
        // A full turn fits in four quarter-turn cubics. Stage them so an
        // unrepresentable point leaves the destination path untouched.
        let mut cubics = [[Point::ORIGIN; 3]; 4];
        let cubics = cubics
            .get_mut(..count)
            .ok_or(PathDataFault::ArcOutOfRange)?;
        for (index, cubic) in cubics.iter_mut().enumerate() {
            let end_angle = start_angle + step;
            let last = index + 1 == count;
            let p3 = if last {
                (f64::from(end.x), f64::from(end.y))
            } else {
                point_at(end_angle)
            };
            let d0 = tangent_at(start_angle);
            let d3 = if last {
                map(half_chord_y + centre_y, -half_chord_x - centre_x)
            } else {
                tangent_at(end_angle)
            };
            let c1 = arc_point(p0.0 + control_scale * d0.0, p0.1 + control_scale * d0.1);
            let c2 = arc_point(p3.0 - control_scale * d3.0, p3.1 - control_scale * d3.1);
            let target = arc_point(p3.0, p3.1);
            let (Some(c1), Some(c2), Some(target)) = (c1, c2, target) else {
                return Err(PathDataFault::ArcOutOfRange);
            };
            *cubic = [c1, c2, target];
            p0 = p3;
            start_angle = end_angle;
        }
        for &[c1, c2, target] in cubics.iter() {
            path.cubic_to(c1, c2, target);
        }
        Ok(())
    }
}

/// Converts a computed arc point only if both coordinates are representable.
#[allow(clippy::cast_possible_truncation)]
fn arc_point(x: f64, y: f64) -> Option<Point> {
    let (x, y) = (x as f32, y as f32);
    (x.is_finite() && y.is_finite()).then(|| Point::new(x, y))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn out_of_range_arc_leaves_the_path_untouched() {
        let mut path = Path::new();
        path.move_to(Point::ORIGIN);
        let original = path.clone();
        let arc = SvgArc {
            radii: [3e38; 2],
            rotation_degrees: 0.0,
            large: true,
            sweep: true,
        };
        assert_eq!(
            arc.append(&mut path, Point::ORIGIN, Point::new(100.0, 0.0)),
            Err(PathDataFault::ArcOutOfRange)
        );
        assert_eq!(path, original);
    }
}
