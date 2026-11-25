//! The [`PointerId`] identity for a single pointer or contact (spec §4.7).

#[doc = include_str!("../../docs/api/pointer/pointer_id.doc/PointerId.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PointerId(u64);

/// The bit that marks a touch contact's id, keeping the platform's contact
/// numbering clear of [`PointerId::MOUSE`] and of any host-minted handle.
const TOUCH_NAMESPACE: u64 = 1 << 63;

impl PointerId {
    /// The conventional identity for the system mouse pointer (raw `0`).
    pub const MOUSE: Self = Self::from_raw(0);

    /// Wraps a raw handle value into a `PointerId`.
    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    #[doc = include_str!("../../docs/api/pointer/pointer_id.doc/touch.md")]
    #[must_use]
    pub const fn touch(contact: u64) -> Self {
        Self(contact | TOUCH_NAMESPACE)
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
    fn the_first_touch_contact_is_not_the_mouse() {
        // The collision that is certain rather than possible: a platform numbers
        // its contacts from zero, and the mouse is raw zero.
        assert_ne!(PointerId::touch(0), PointerId::MOUSE);
    }

    #[test]
    fn touch_contacts_stay_distinct_from_each_other() {
        // Two fingers must remain two pointers — the namespace must not fold the
        // contact number away.
        assert_ne!(PointerId::touch(0), PointerId::touch(1));
        assert_ne!(PointerId::touch(1), PointerId::touch(2));
    }

    #[test]
    fn a_touch_contact_never_aliases_a_low_range_host_handle() {
        // A host minting ids the obvious way (0, 1, 2, …) can never land on a
        // contact, so the two id spaces coexist without coordination.
        for host in 0..64_u64 {
            for contact in 0..64_u64 {
                assert_ne!(
                    PointerId::from_raw(host),
                    PointerId::touch(contact),
                    "host {host} aliased contact {contact}"
                );
            }
        }
    }

    #[test]
    fn distinct_ids_compare_unequal_and_order() {
        assert_ne!(PointerId::from_raw(1), PointerId::from_raw(2));
        assert!(PointerId::from_raw(1) < PointerId::from_raw(2));
    }
}
