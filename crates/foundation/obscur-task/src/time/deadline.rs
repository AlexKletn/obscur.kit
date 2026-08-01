#![doc = include_str!("deadline.doc/_module.md")]

use super::Instant;

#[doc = include_str!("deadline.doc/Deadline.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Deadline(Instant);

impl Deadline {
    #[doc = include_str!("deadline.doc/at.md")]
    #[must_use]
    pub const fn at(instant: Instant) -> Self {
        Self(instant)
    }

    #[doc = include_str!("deadline.doc/is_expired.md")]
    #[must_use]
    pub fn is_expired(self, now: Instant) -> bool {
        now >= self.0
    }
}
