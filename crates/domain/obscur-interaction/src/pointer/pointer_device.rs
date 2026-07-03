//! The [`PointerDevice`] kinds that can produce pointer input (spec §4.7).

/// The kind of physical device that produced a pointer event (spec §4.7).
///
/// Pointer routing is device-agnostic, but handlers sometimes need to adapt to
/// the source: whether it reports pressure, or whether it is a direct
/// (touch-the-target) rather than indirect (move-a-cursor) manipulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PointerDevice {
    /// An indirect pointing device that drives an on-screen cursor.
    #[default]
    Mouse,
    /// A direct touchscreen contact.
    Touch,
    /// A stylus or digital pen.
    Pen,
    /// An indirect trackpad surface driving an on-screen cursor.
    Trackpad,
}

impl PointerDevice {
    /// Returns whether this device reports a meaningful contact pressure
    /// ([`Pen`](Self::Pen) or [`Touch`](Self::Touch)).
    #[must_use]
    pub const fn supports_pressure(self) -> bool {
        matches!(self, Self::Pen | Self::Touch)
    }

    /// Returns whether this device is a *direct* manipulation device — the user
    /// touches the target itself ([`Touch`](Self::Touch), [`Pen`](Self::Pen)) —
    /// rather than an indirect device that moves a cursor
    /// ([`Mouse`](Self::Mouse), [`Trackpad`](Self::Trackpad)).
    #[must_use]
    pub const fn is_direct(self) -> bool {
        matches!(self, Self::Touch | Self::Pen)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_mouse() {
        assert_eq!(PointerDevice::default(), PointerDevice::Mouse);
    }

    #[test]
    fn supports_pressure_only_for_pen_and_touch() {
        assert!(PointerDevice::Pen.supports_pressure());
        assert!(PointerDevice::Touch.supports_pressure());
        assert!(!PointerDevice::Mouse.supports_pressure());
        assert!(!PointerDevice::Trackpad.supports_pressure());
    }

    #[test]
    fn is_direct_only_for_touch_and_pen() {
        assert!(PointerDevice::Touch.is_direct());
        assert!(PointerDevice::Pen.is_direct());
        assert!(!PointerDevice::Mouse.is_direct());
        assert!(!PointerDevice::Trackpad.is_direct());
    }
}
