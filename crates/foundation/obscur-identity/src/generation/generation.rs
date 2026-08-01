#![doc = include_str!("generation.doc/_module.md")]

#[doc = include_str!("generation.doc/Generation.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct Generation(u32);

impl Generation {
    #[doc = include_str!("generation.doc/FIRST.md")]
    pub const FIRST: Self = Self(1);

    #[doc = include_str!("generation.doc/from_raw.md")]
    #[must_use]
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    #[doc = include_str!("generation.doc/raw.md")]
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }

    #[doc = include_str!("generation.doc/next.md")]
    #[must_use]
    pub const fn next(self) -> Self {
        Self(self.0.wrapping_add(1))
    }
}
