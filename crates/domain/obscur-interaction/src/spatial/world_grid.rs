//! Regular grid in world coordinates.

use obscur_geometry::Size;
use obscur_geometry::space::{PointIn, WorldSpace};

/// A scene grid with an origin and independent spacing for each axis.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldGrid {
    origin: PointIn<WorldSpace>,
    spacing: Size,
}

impl WorldGrid {
    /// Creates a grid with finite origin and strictly positive finite spacing.
    #[must_use]
    pub fn new(origin: PointIn<WorldSpace>, spacing: Size) -> Option<Self> {
        let point = origin.point();
        if !point.x.is_finite()
            || !point.y.is_finite()
            || !spacing.width.is_finite()
            || !spacing.height.is_finite()
            || spacing.width <= 0.0
            || spacing.height <= 0.0
        {
            return None;
        }
        Some(Self { origin, spacing })
    }

    /// Returns the world origin of the grid.
    #[must_use]
    pub const fn origin(self) -> PointIn<WorldSpace> {
        self.origin
    }

    /// Returns horizontal and vertical spacing in world units.
    #[must_use]
    pub const fn spacing(self) -> Size {
        self.spacing
    }

    /// Returns the nearest grid intersection to a world point.
    #[must_use]
    pub fn nearest(self, point: PointIn<WorldSpace>) -> PointIn<WorldSpace> {
        let value = point.point();
        let origin = self.origin.point();
        let axis = |value: f32, origin: f32, step: f32| {
            let step = f64::from(step);
            let origin = f64::from(origin);
            let snapped = (origin + ((f64::from(value) - origin) / step).round() * step) as f32;
            if snapped.is_finite() { snapped } else { value }
        };
        PointIn::new(obscur_geometry::Point::new(
            axis(value.x, origin.x, self.spacing.width),
            axis(value.y, origin.y, self.spacing.height),
        ))
    }
}
