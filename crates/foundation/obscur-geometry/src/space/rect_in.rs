//! Rectangles with a coordinate space marker.

use crate::Rect;
use core::marker::PhantomData;

/// An axis-aligned rectangle tagged with its coordinate space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RectIn<S> {
    rect: Rect,
    space: PhantomData<fn() -> S>,
}

impl<S> RectIn<S> {
    /// Tags a raw rectangle with its coordinate space.
    #[must_use]
    pub const fn new(rect: Rect) -> Self {
        Self {
            rect,
            space: PhantomData,
        }
    }

    /// Returns the underlying rectangle.
    #[must_use]
    pub const fn rect(self) -> Rect {
        self.rect
    }
}
