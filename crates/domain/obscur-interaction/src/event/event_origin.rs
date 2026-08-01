#![doc = include_str!("event_origin.doc/_module.md")]

#[doc = include_str!("event_origin.doc/EventOrigin.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventOrigin {
    #[doc = include_str!("event_origin.doc/Pointer.md")]
    Pointer,
    #[doc = include_str!("event_origin.doc/Keyboard.md")]
    Keyboard,
    #[doc = include_str!("event_origin.doc/Touch.md")]
    Touch,
    #[doc = include_str!("event_origin.doc/Gesture.md")]
    Gesture,
    #[doc = include_str!("event_origin.doc/Gamepad.md")]
    Gamepad,
    #[doc = include_str!("event_origin.doc/Accessibility.md")]
    Accessibility,
    #[doc = include_str!("event_origin.doc/Synthetic.md")]
    Synthetic,
}

impl EventOrigin {
    #[doc = include_str!("event_origin.doc/is_synthetic.md")]
    #[must_use]
    pub const fn is_synthetic(self) -> bool {
        matches!(self, Self::Synthetic)
    }

    #[doc = include_str!("event_origin.doc/is_direct_input.md")]
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
