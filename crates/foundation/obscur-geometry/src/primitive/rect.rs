#![doc = include_str!("rect.doc/_module.md")]

use super::{Point, Size};

#[doc = include_str!("rect.doc/Rect.md")]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rect {
    #[doc = include_str!("rect.doc/origin.md")]
    pub origin: Point,
    #[doc = include_str!("rect.doc/size.md")]
    pub size: Size,
}

impl Rect {
    #[doc = include_str!("rect.doc/new.md")]
    #[must_use]
    pub const fn new(origin: Point, size: Size) -> Self {
        Self { origin, size }
    }

    #[doc = include_str!("rect.doc/right.md")]
    #[must_use]
    pub fn right(self) -> f32 {
        self.origin.x + self.size.width
    }

    #[doc = include_str!("rect.doc/bottom.md")]
    #[must_use]
    pub fn bottom(self) -> f32 {
        self.origin.y + self.size.height
    }

    #[doc = include_str!("rect.doc/contains.md")]
    #[must_use]
    pub fn contains(self, p: Point) -> bool {
        p.x >= self.origin.x && p.x < self.right() && p.y >= self.origin.y && p.y < self.bottom()
    }
}
