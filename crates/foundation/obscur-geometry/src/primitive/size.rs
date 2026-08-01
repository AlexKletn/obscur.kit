#![doc = include_str!("size.doc/_module.md")]

#[doc = include_str!("size.doc/Size.md")]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Size {
    #[doc = include_str!("size.doc/width.md")]
    pub width: f32,
    #[doc = include_str!("size.doc/height.md")]
    pub height: f32,
}

impl Size {
    #[doc = include_str!("size.doc/ZERO.md")]
    pub const ZERO: Self = Self {
        width: 0.0,
        height: 0.0,
    };

    #[doc = include_str!("size.doc/new.md")]
    #[must_use]
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}
