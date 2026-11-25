//! Point alignment against a scene grid and guides.

use obscur_geometry::space::{PointIn, WorldSpace};
use obscur_geometry::{Point, Size};

use super::WorldGrid;

/// Snaps each axis to its closest grid line or guide within the world tolerance. X guides are vertical and y guides are horizontal. Guides win ties, in input order.
#[must_use]
pub fn snap_world_point(
    point: PointIn<WorldSpace>,
    grid: Option<WorldGrid>,
    x_guides: &[f32],
    y_guides: &[f32],
    tolerance: Size,
) -> PointIn<WorldSpace> {
    let value = point.point();
    let grid = grid.map(|grid| grid.nearest(point).point());
    PointIn::new(Point::new(
        snap_axis(value.x, grid.map(|p| p.x), x_guides, tolerance.width),
        snap_axis(value.y, grid.map(|p| p.y), y_guides, tolerance.height),
    ))
}

fn snap_axis(value: f32, grid: Option<f32>, guides: &[f32], tolerance: f32) -> f32 {
    if !value.is_finite() || !tolerance.is_finite() || tolerance < 0.0 {
        return value;
    }
    guides
        .iter()
        .copied()
        .chain(grid)
        .map(|candidate| (candidate, (f64::from(candidate) - f64::from(value)).abs()))
        .filter(|(_, distance)| *distance <= f64::from(tolerance))
        .min_by(|(_, a), (_, b)| a.total_cmp(b))
        .map_or(value, |(candidate, _)| candidate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_and_guides_compete_per_axis_within_tolerance() {
        let point = PointIn::new(Point::new(-8.0, 17.0));
        let grid = WorldGrid::new(PointIn::new(Point::ORIGIN), Size::new(10.0, 10.0)).unwrap();
        let result = snap_world_point(
            point,
            Some(grid),
            &[-6.0, f32::NAN],
            &[40.0],
            Size::new(2.0, 3.0),
        );
        assert_eq!(result.point(), Point::new(-6.0, 20.0));
        let result = snap_world_point(point, Some(grid), &[], &[], Size::new(2.0, 2.0));
        assert_eq!(result.point(), Point::new(-10.0, 17.0));
    }
}
