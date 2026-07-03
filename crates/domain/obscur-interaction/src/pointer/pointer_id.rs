//! The [`PointerId`] identity for a single pointer or contact (spec §4.7).

/// Stable identity for one pointer or contact point (spec §4.7).
///
/// A single interaction session may involve several concurrent pointers: a
/// mouse plus multiple simultaneous touch contacts. Each pointer carries a
/// `PointerId` so multi-pointer (touch) input can be tracked and routed
/// independently. By convention the mouse is [`PointerId::MOUSE`] (raw `0`) and
/// every touch contact is minted a distinct id. This is a *local* newtype:
/// the interaction crate mints and compares these handles itself. The numeric
/// value carries no meaning outside the subsystem that mints it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PointerId(u64);

impl PointerId {
    /// The conventional identity for the system mouse pointer (raw `0`).
    pub const MOUSE: Self = Self::from_raw(0);

    /// Wraps a raw handle value into a `PointerId`.
    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    /// Returns the raw handle value backing this identifier.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_raw_value() {
        assert_eq!(PointerId::from_raw(42).raw(), 42);
        assert_eq!(PointerId::from_raw(0).raw(), 0);
        assert_eq!(PointerId::from_raw(u64::MAX).raw(), u64::MAX);
    }

    #[test]
    fn mouse_is_raw_zero() {
        assert_eq!(PointerId::MOUSE, PointerId::from_raw(0));
        assert_eq!(PointerId::MOUSE.raw(), 0);
    }

    #[test]
    fn distinct_ids_compare_unequal_and_order() {
        assert_ne!(PointerId::from_raw(1), PointerId::from_raw(2));
        assert!(PointerId::from_raw(1) < PointerId::from_raw(2));
    }
}
