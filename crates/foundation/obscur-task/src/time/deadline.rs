//! An absolute deadline on the clock timeline.

use super::Instant;

/// An absolute point in time by which an operation should complete.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Deadline(Instant);

impl Deadline {
    /// Creates a deadline at the given instant.
    #[must_use]
    pub const fn at(instant: Instant) -> Self {
        Self(instant)
    }

    /// Returns whether `now` has reached or passed this deadline.
    #[must_use]
    pub fn is_expired(self, now: Instant) -> bool {
        now >= self.0
    }
}
