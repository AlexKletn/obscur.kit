//! A point on an abstract monotonic timeline.

#[doc = include_str!("../../docs/api/time/instant.doc/Instant.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Instant(u64);

impl Instant {
    /// Creates an instant from nanoseconds since the clock origin.
    #[must_use]
    pub const fn from_nanos(nanos: u64) -> Self {
        Self(nanos)
    }

    /// Returns nanoseconds since the clock origin.
    #[must_use]
    pub const fn as_nanos(self) -> u64 {
        self.0
    }
}
