#![doc = include_str!("instant.doc/_module.md")]

#[doc = include_str!("instant.doc/Instant.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Instant(u64);

impl Instant {
    #[doc = include_str!("instant.doc/from_nanos.md")]
    #[must_use]
    pub const fn from_nanos(nanos: u64) -> Self {
        Self(nanos)
    }

    #[doc = include_str!("instant.doc/as_nanos.md")]
    #[must_use]
    pub const fn as_nanos(self) -> u64 {
        self.0
    }
}
