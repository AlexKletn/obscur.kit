#![doc = include_str!("pointer_device.doc/_module.md")]

#[doc = include_str!("pointer_device.doc/PointerDevice.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PointerDevice {
    #[doc = include_str!("pointer_device.doc/Mouse.md")]
    #[default]
    Mouse,
    #[doc = include_str!("pointer_device.doc/Touch.md")]
    Touch,
    #[doc = include_str!("pointer_device.doc/Pen.md")]
    Pen,
    #[doc = include_str!("pointer_device.doc/Trackpad.md")]
    Trackpad,
}

impl PointerDevice {
    #[doc = include_str!("pointer_device.doc/supports_pressure.md")]
    #[must_use]
    pub const fn supports_pressure(self) -> bool {
        matches!(self, Self::Pen | Self::Touch)
    }

    #[doc = include_str!("pointer_device.doc/is_direct.md")]
    #[must_use]
    pub const fn is_direct(self) -> bool {
        matches!(self, Self::Touch | Self::Pen)
    }

    #[doc = include_str!("pointer_device.doc/is_touchscreen.md")]
    #[must_use]
    pub const fn is_touchscreen(self) -> bool {
        matches!(self, Self::Touch)
    }

    #[doc = include_str!("pointer_device.doc/hovers_by_default.md")]
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
