//! Where an input event originated before entering the event model (spec §4.7).

#[doc = include_str!("../../docs/api/event/event_origin.doc/EventOrigin.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventOrigin {
    /// A pointing device (mouse, pen, trackpad).
    Pointer,
    /// A physical or virtual keyboard.
    Keyboard,
    /// A touchscreen contact.
    Touch,
    /// A recognized higher-level gesture (pinch, swipe, rotate).
    Gesture,
    /// A gamepad or game controller.
    Gamepad,
    /// An accessibility action requested by assistive technology.
    Accessibility,
    /// A test-injected synthetic input (spec §7.4).
    Synthetic,
}

impl EventOrigin {
    /// Returns whether this event was injected synthetically for testing
    /// (spec §7.4).
    #[must_use]
    pub const fn is_synthetic(self) -> bool {
        matches!(self, Self::Synthetic)
    }

    /// Returns whether this event came directly from a physical input device
    /// ([`Pointer`](Self::Pointer), [`Keyboard`](Self::Keyboard),
    /// [`Touch`](Self::Touch), or [`Gamepad`](Self::Gamepad)) rather than from a
    /// recognizer, assistive technology, or a test harness.
    #[must_use]
    pub const fn is_direct_input(self) -> bool {
        matches!(
            self,
            Self::Pointer | Self::Keyboard | Self::Touch | Self::Gamepad
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synthetic_predicate() {
        assert!(EventOrigin::Synthetic.is_synthetic());
        assert!(!EventOrigin::Pointer.is_synthetic());
    }

    #[test]
    fn direct_input_predicate() {
        assert!(EventOrigin::Pointer.is_direct_input());
        assert!(EventOrigin::Keyboard.is_direct_input());
        assert!(EventOrigin::Touch.is_direct_input());
        assert!(EventOrigin::Gamepad.is_direct_input());

        assert!(!EventOrigin::Gesture.is_direct_input());
        assert!(!EventOrigin::Accessibility.is_direct_input());
        assert!(!EventOrigin::Synthetic.is_direct_input());
    }
}
