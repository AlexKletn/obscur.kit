//! Monotonic generation counter for stale-handle detection.

/// A monotonically increasing generation used to invalidate stale references
/// to a slot whose identity has been reused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct Generation(u32);

impl Generation {
    /// The first live generation. Generation zero is reserved for "never set".
    pub const FIRST: Self = Self(1);

    /// Wraps a raw generation value.
    #[must_use]
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    /// Returns the raw generation value.
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }

    /// Returns the next generation, wrapping on overflow.
    #[must_use]
    pub const fn next(self) -> Self {
        Self(self.0.wrapping_add(1))
    }
}
