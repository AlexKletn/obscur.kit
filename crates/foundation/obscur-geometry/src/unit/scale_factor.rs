#![doc = include_str!("scale_factor.doc/_module.md")]

#[doc = include_str!("scale_factor.doc/ScaleFactor.md")]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct ScaleFactor(f32);

impl ScaleFactor {
    #[doc = include_str!("scale_factor.doc/IDENTITY.md")]
    pub const IDENTITY: Self = Self(1.0);

    #[doc = include_str!("scale_factor.doc/new.md")]
    #[must_use]
    pub fn new(value: f32) -> Self {
        if value > 0.0 {
            Self(value)
        } else {
            Self::IDENTITY
        }
    }

    #[doc = include_str!("scale_factor.doc/get.md")]
    #[must_use]
    pub fn get(self) -> f32 {
        self.0
    }
}

impl Default for ScaleFactor {
    fn default() -> Self {
        Self::IDENTITY
    }
}
