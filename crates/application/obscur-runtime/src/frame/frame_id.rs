//! Monotonic frame identity: [`FrameId`].

/// A monotonic frame counter that uniquely labels each produced frame.
///
/// `FrameId` is a lightweight newtype over `u64`. The runtime starts at
/// [`FrameId::FIRST`] and advances by [`FrameId::next`] once per produced frame,
/// so a larger id always denotes a later frame (until the counter wraps, which
/// is astronomically distant for a `u64` frame clock).
///
/// The type is a plain value: `Copy`, totally ordered, hashable, and `Default`
/// (equal to [`FrameId::FIRST`]), which makes it usable as a map key or in an
/// ordered log without any allocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct FrameId(u64);

impl FrameId {
    /// The first frame id, `0`.
    ///
    /// This is also the [`Default`] value, so a freshly constructed runtime
    /// begins producing frames from `FIRST`.
    pub const FIRST: Self = Self(0);

    /// Creates a `FrameId` from its raw `u64` value.
    ///
    /// This is the inverse of [`FrameId::raw`] and is primarily useful for
    /// reconstructing an id from persisted or transmitted state.
    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    /// Returns the raw `u64` value backing this id.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }

    /// Returns the next frame id.
    ///
    /// The counter advances by one, wrapping on overflow so the operation is
    /// total and can never panic. In practice a `u64` frame clock never reaches
    /// the wrap point during any real runtime session.
    #[must_use]
    pub const fn next(self) -> Self {
        Self(self.0.wrapping_add(1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_is_zero_and_default() {
        assert_eq!(FrameId::FIRST.raw(), 0);
        assert_eq!(FrameId::default(), FrameId::FIRST);
    }

    #[test]
    fn from_raw_round_trips() {
        assert_eq!(FrameId::from_raw(42).raw(), 42);
    }

    #[test]
    fn next_advances_monotonically() {
        let a = FrameId::FIRST;
        let b = a.next();
        let c = b.next();
        assert_eq!(b.raw(), 1);
        assert_eq!(c.raw(), 2);
        assert!(a < b && b < c);
    }

    #[test]
    fn next_wraps_at_max_without_panicking() {
        let max = FrameId::from_raw(u64::MAX);
        assert_eq!(max.next(), FrameId::FIRST);
    }
}
