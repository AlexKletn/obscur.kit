//! A point on an abstract monotonic timeline.

/// An instant expressed as nanoseconds since an unspecified monotonic origin.
///
/// The origin is defined by the [`Clock`](super::Clock) that produced it;
/// headless clocks are fully deterministic (spec §3.3, §7.4).
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
