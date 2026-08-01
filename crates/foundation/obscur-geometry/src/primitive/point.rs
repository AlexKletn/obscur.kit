#![doc = include_str!("point.doc/_module.md")]

#[doc = include_str!("point.doc/Point.md")]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point {
    #[doc = include_str!("point.doc/x.md")]
    pub x: f32,
    #[doc = include_str!("point.doc/y.md")]
    pub y: f32,
}

impl Point {
    #[doc = include_str!("point.doc/ORIGIN.md")]
    pub const ORIGIN: Self = Self { x: 0.0, y: 0.0 };

    #[doc = include_str!("point.doc/new.md")]
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}
