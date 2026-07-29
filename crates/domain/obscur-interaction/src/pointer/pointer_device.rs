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

    /// Returns whether this device is a touchscreen contact ([`Touch`](Self::Touch)).
    ///
    /// Distinct from [`is_direct`](Self::is_direct), which a pen shares: §2.4
    /// treats touch as its own *source* — *"Touch does not synthesize persistent
    /// hover"* — so the two cases separate wherever a contact and a stylus behave
    /// differently, and this names that one device rather than the class.
    #[must_use]
    pub const fn is_touchscreen(self) -> bool {
        matches!(self, Self::Touch)
    }

    /// Returns §2.4's default **hover capability** for this device: whether it can
    /// be *over* a target without touching it.
    ///
    /// §2.4 rules two of the four cases outright — *"Touch does not synthesize
    /// persistent hover. Pen hover depends on device capability."* — so
    /// [`Touch`](Self::Touch) is `false` and [`Pen`](Self::Pen) is `false` **by
    /// default only**: a pen's answer is a property of the tablet, not of the
    /// enum, so a platform that knows its digitizer reports hover says so per
    /// event with
    /// [`PointerEvent::with_hover_capability`](super::PointerEvent::with_hover_capability).
    /// Defaulting the pen to `false` fails in the direction that costs a hover
    /// highlight rather than the one that invents a hover the device never
    /// reported.
    ///
    /// A [`Mouse`](Self::Mouse) and a [`Trackpad`](Self::Trackpad) drive a resting
    /// cursor, so both hover by construction.
    #[must_use]
    pub const fn hovers_by_default(self) -> bool {
        matches!(self, Self::Mouse | Self::Trackpad)
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

    /// A pen is direct but is not a touchscreen — the distinction §2.4's hover
    /// sentences rest on.
    #[test]
    fn is_touchscreen_is_narrower_than_is_direct() {
        assert!(PointerDevice::Touch.is_touchscreen());
        assert!(!PointerDevice::Pen.is_touchscreen());
        assert!(!PointerDevice::Mouse.is_touchscreen());
        assert!(!PointerDevice::Trackpad.is_touchscreen());
    }

    /// §2.4: "Touch does not synthesize persistent hover. Pen hover depends on
    /// device capability." — so neither hovers by default, and only a cursor
    /// device does.
    #[test]
    fn only_cursor_devices_hover_by_default() {
        assert!(PointerDevice::Mouse.hovers_by_default());
        assert!(PointerDevice::Trackpad.hovers_by_default());
        assert!(!PointerDevice::Touch.hovers_by_default());
        assert!(!PointerDevice::Pen.hovers_by_default());
    }
}
