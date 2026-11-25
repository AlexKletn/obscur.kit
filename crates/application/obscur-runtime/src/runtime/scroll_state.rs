//! Serializable scroll positions for application-owned persistence.

use obscur_geometry::Point;
use serde::{Deserialize, Serialize};

/// A versioned scroll position in logical pixels. Store it under an application
/// key and resolve the target node in the new session before restoring it.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ScrollState {
    version: u32,
    x: f32,
    y: f32,
}

impl ScrollState {
    const VERSION: u32 = 1;

    pub(super) fn capture(offset: Point) -> Self {
        Self {
            version: Self::VERSION,
            x: offset.x,
            y: offset.y,
        }
    }

    pub(super) fn offset(self) -> Option<Point> {
        (self.version == Self::VERSION && self.x.is_finite() && self.y.is_finite())
            .then_some(Point::new(self.x, self.y))
    }
}
