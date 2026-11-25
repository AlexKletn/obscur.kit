//! Points tagged with their coordinate space.

use crate::Point;
use core::marker::PhantomData;

/// A point whose coordinate space is carried in its type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointIn<S> {
    point: Point,
    space: PhantomData<fn() -> S>,
}

impl<S> PointIn<S> {
    /// Tags a raw point as coordinates in this space.
    #[must_use]
    pub const fn new(point: Point) -> Self {
        Self {
            point,
            space: PhantomData,
        }
    }

    /// Returns the underlying numeric coordinates.
    #[must_use]
    pub const fn point(self) -> Point {
        self.point
    }
}
