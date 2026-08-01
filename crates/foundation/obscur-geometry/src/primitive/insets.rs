#![doc = include_str!("insets.doc/_module.md")]

#[doc = include_str!("insets.doc/Insets.md")]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Insets {
    #[doc = include_str!("insets.doc/top.md")]
    pub top: f32,
    #[doc = include_str!("insets.doc/right.md")]
    pub right: f32,
    #[doc = include_str!("insets.doc/bottom.md")]
    pub bottom: f32,
    #[doc = include_str!("insets.doc/left.md")]
    pub left: f32,
}

impl Insets {
    #[doc = include_str!("insets.doc/uniform.md")]
    #[must_use]
    pub const fn uniform(value: f32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }
}
