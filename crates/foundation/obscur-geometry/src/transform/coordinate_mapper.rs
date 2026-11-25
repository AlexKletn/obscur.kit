//! Typed affine mappings between coordinate spaces.

use crate::{
    AffineTransform, Size,
    space::{PointIn, RectIn},
};
use core::marker::PhantomData;

/// An affine transform with distinct source and destination spaces.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CoordinateMapper<From, To> {
    transform: AffineTransform,
    spaces: PhantomData<fn(From) -> To>,
}

impl<From, To> CoordinateMapper<From, To> {
    /// Builds a mapper from a source-to-destination affine transform.
    #[must_use]
    pub const fn new(transform: AffineTransform) -> Self {
        Self {
            transform,
            spaces: PhantomData,
        }
    }

    /// Transforms a point from the source space to the destination space.
    #[must_use]
    pub fn map(&self, point: PointIn<From>) -> PointIn<To> {
        PointIn::new(self.transform.apply(point.point()))
    }

    /// Returns the axis-aligned bounding rectangle of the four transformed corners.
    #[must_use]
    pub fn map_rect(&self, rect: RectIn<From>) -> RectIn<To> {
        RectIn::new(self.transform.map_rect(rect.rect()))
    }

    /// Maps the size of an axis-aligned rectangle to the size of its transformed bounds. Translation does not affect the extent.
    #[must_use]
    pub fn map_extent(&self, extent: Size) -> Size {
        let [xx, yx, xy, yy, _, _] = self.transform.coeffs;
        Size::new(
            xx.abs() * extent.width + xy.abs() * extent.height,
            yx.abs() * extent.width + yy.abs() * extent.height,
        )
    }

    /// Reverses the mapping if its affine transform is invertible.
    #[must_use]
    pub fn inverse(&self) -> Option<CoordinateMapper<To, From>> {
        self.transform.inverse().map(CoordinateMapper::new)
    }

    /// Applies this mapping followed by the next mapping.
    #[must_use]
    pub fn then<Next>(&self, next: &CoordinateMapper<To, Next>) -> CoordinateMapper<From, Next> {
        CoordinateMapper::new(next.transform.multiply(self.transform))
    }
}
