//! Typed synthetic input events a test injects into a headless run (spec §7.4).

use crate::input::pointer_button::PointerButton;
use crate::input::selector_target::SelectorTarget;

/// A typed synthetic input event to inject into a headless run (spec §7.4).
///
/// Each variant is inert typed data describing one atomic input. The events are
/// injected into a headless run and dispatched — hit-tested, routed through
/// capture → target → bubble, and turned into a typed dispatch result — by the
/// interaction subsystem in a later phase (Phase 3, `obscur-interaction`). Here
/// nothing is resolved or dispatched; these descriptors only let tests build the
/// input they will replay.
#[derive(Debug, Clone, PartialEq)]
pub enum SyntheticInput {
    /// A pointer movement over the given target.
    PointerMove {
        /// Where the pointer moves to.
        target: SelectorTarget,
    },
    /// A pointer button press over the given target.
    PointerDown {
        /// Where the press lands.
        target: SelectorTarget,
        /// Which button is pressed.
        button: PointerButton,
    },
    /// A pointer button release over the given target.
    PointerUp {
        /// Where the release lands.
        target: SelectorTarget,
        /// Which button is released.
        button: PointerButton,
    },
    /// A key press, identified by an opaque key code.
    KeyDown {
        /// The opaque key code being pressed.
        key: u32,
    },
    /// A key release, identified by an opaque key code.
    KeyUp {
        /// The opaque key code being released.
        key: u32,
    },
}

impl SyntheticInput {
    /// Builds a [`SyntheticInput::PointerMove`] over `target`.
    #[must_use]
    pub const fn pointer_move(target: SelectorTarget) -> Self {
        Self::PointerMove { target }
    }

    /// Builds a [`SyntheticInput::PointerDown`] of `button` over `target`.
    #[must_use]
    pub const fn pointer_down(target: SelectorTarget, button: PointerButton) -> Self {
        Self::PointerDown { target, button }
    }

    /// Builds a [`SyntheticInput::PointerUp`] of `button` over `target`.
    #[must_use]
    pub const fn pointer_up(target: SelectorTarget, button: PointerButton) -> Self {
        Self::PointerUp { target, button }
    }

    /// Builds a [`SyntheticInput::KeyDown`] for the opaque `key` code.
    #[must_use]
    pub const fn key_down(key: u32) -> Self {
        Self::KeyDown { key }
    }

    /// Builds a [`SyntheticInput::KeyUp`] for the opaque `key` code.
    #[must_use]
    pub const fn key_up(key: u32) -> Self {
        Self::KeyUp { key }
    }

    /// Returns the targeted node selector for a pointer event, or `None` for a
    /// key event (which carries no [`SelectorTarget`]).
    #[must_use]
    pub const fn target(&self) -> Option<&SelectorTarget> {
        match self {
            Self::PointerMove { target }
            | Self::PointerDown { target, .. }
            | Self::PointerUp { target, .. } => Some(target),
            Self::KeyDown { .. } | Self::KeyUp { .. } => None,
        }
    }

    /// Returns `true` if this is a pointer event (move, down, or up).
    #[must_use]
    pub const fn is_pointer(&self) -> bool {
        matches!(
            self,
            Self::PointerMove { .. } | Self::PointerDown { .. } | Self::PointerUp { .. }
        )
    }

    /// Returns `true` if this is a key event (down or up).
    #[must_use]
    pub const fn is_key(&self) -> bool {
        matches!(self, Self::KeyDown { .. } | Self::KeyUp { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::SyntheticInput;
    use crate::input::pointer_button::PointerButton;
    use crate::input::selector_target::SelectorTarget;
    use obscur_identity::NodeId;

    #[test]
    fn pointer_down_carries_target_and_button() {
        let target = SelectorTarget::node(NodeId::from_raw(3));
        let event = SyntheticInput::pointer_down(target.clone(), PointerButton::Secondary);

        assert_eq!(
            event,
            SyntheticInput::PointerDown {
                target,
                button: PointerButton::Secondary,
            }
        );
    }

    #[test]
    fn pointer_events_expose_target() {
        let target = SelectorTarget::node(NodeId::from_raw(1));

        let move_event = SyntheticInput::pointer_move(target.clone());
        let down_event = SyntheticInput::pointer_down(target.clone(), PointerButton::Primary);
        let up_event = SyntheticInput::pointer_up(target.clone(), PointerButton::Middle);

        assert_eq!(move_event.target(), Some(&target));
        assert_eq!(down_event.target(), Some(&target));
        assert_eq!(up_event.target(), Some(&target));
        assert!(move_event.is_pointer());
        assert!(!move_event.is_key());
    }

    #[test]
    fn key_events_have_no_target() {
        let down = SyntheticInput::key_down(42);
        let up = SyntheticInput::key_up(42);

        assert_eq!(down.target(), None);
        assert_eq!(up.target(), None);
        assert!(down.is_key());
        assert!(!down.is_pointer());
        assert!(up.is_key());
        assert!(!up.is_pointer());
    }
}
