#![doc = include_str!("synthetic_input.doc/_module.md")]

use crate::input::pointer_button::PointerButton;
use crate::input::selector_target::SelectorTarget;

#[doc = include_str!("synthetic_input.doc/SyntheticInput.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum SyntheticInput {
    #[doc = include_str!("synthetic_input.doc/PointerMove.md")]
    PointerMove {
        #[doc = include_str!("synthetic_input.doc/target.md")]
        target: SelectorTarget,
    },
    #[doc = include_str!("synthetic_input.doc/PointerDown.md")]
    PointerDown {
        #[doc = include_str!("synthetic_input.doc/target_2.md")]
        target: SelectorTarget,
        #[doc = include_str!("synthetic_input.doc/button.md")]
        button: PointerButton,
    },
    #[doc = include_str!("synthetic_input.doc/PointerUp.md")]
    PointerUp {
        #[doc = include_str!("synthetic_input.doc/target_3.md")]
        target: SelectorTarget,
        #[doc = include_str!("synthetic_input.doc/button_2.md")]
        button: PointerButton,
    },
    #[doc = include_str!("synthetic_input.doc/KeyDown.md")]
    KeyDown {
        #[doc = include_str!("synthetic_input.doc/key.md")]
        key: u32,
    },
    #[doc = include_str!("synthetic_input.doc/KeyUp.md")]
    KeyUp {
        #[doc = include_str!("synthetic_input.doc/key_2.md")]
        key: u32,
    },
}

impl SyntheticInput {
    #[doc = include_str!("synthetic_input.doc/pointer_move.md")]
    #[must_use]
    pub const fn pointer_move(target: SelectorTarget) -> Self {
        Self::PointerMove { target }
    }

    #[doc = include_str!("synthetic_input.doc/pointer_down.md")]
    #[must_use]
    pub const fn pointer_down(target: SelectorTarget, button: PointerButton) -> Self {
        Self::PointerDown { target, button }
    }

    #[doc = include_str!("synthetic_input.doc/pointer_up.md")]
    #[must_use]
    pub const fn pointer_up(target: SelectorTarget, button: PointerButton) -> Self {
        Self::PointerUp { target, button }
    }

    #[doc = include_str!("synthetic_input.doc/key_down.md")]
    #[must_use]
    pub const fn key_down(key: u32) -> Self {
        Self::KeyDown { key }
    }

    #[doc = include_str!("synthetic_input.doc/key_up.md")]
    #[must_use]
    pub const fn key_up(key: u32) -> Self {
        Self::KeyUp { key }
    }

    #[doc = include_str!("synthetic_input.doc/target_4.md")]
    #[must_use]
    pub const fn target(&self) -> Option<&SelectorTarget> {
        match self {
            Self::PointerMove { target }
            | Self::PointerDown { target, .. }
            | Self::PointerUp { target, .. } => Some(target),
            Self::KeyDown { .. } | Self::KeyUp { .. } => None,
        }
    }

    #[doc = include_str!("synthetic_input.doc/is_pointer.md")]
    #[must_use]
    pub const fn is_pointer(&self) -> bool {
        matches!(
            self,
            Self::PointerMove { .. } | Self::PointerDown { .. } | Self::PointerUp { .. }
        )
    }

    #[doc = include_str!("synthetic_input.doc/is_key.md")]
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
