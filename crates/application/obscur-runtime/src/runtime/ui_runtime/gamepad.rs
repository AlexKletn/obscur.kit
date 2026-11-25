use obscur_interaction::event::{EventOrigin, EventPayload, UiEvent};
use obscur_interaction::gamepad::{GamepadButton, GamepadEvent, GamepadEventKind};
use obscur_interaction::spatial::NavigationDirection;
use obscur_view::view::EVENT_GAMEPAD;

use crate::runtime::CommandDispatch;

use super::UiRuntime;

impl UiRuntime {
    /// Dispatches a trusted gamepad report from the host. A focused binding or the window or application command can prevent default navigation or activation. Returns true if a handler ran or the runtime performed a default action.
    pub fn handle_gamepad(&mut self, input: GamepadEvent) -> bool {
        let mut event = UiEvent::detached(EventOrigin::Gamepad)
            .as_trusted()
            .with_timestamp(input.timestamp());
        event.payload = EventPayload::Gamepad(input);
        let focused = self.focused;
        let (origin, command) = focused
            .and_then(|target| CommandDispatch::binding_for(&self.tree, target, EVENT_GAMEPAD))
            .map_or((focused, EVENT_GAMEPAD), |(node, command)| {
                (Some(node), command)
            });
        let dispatched = self.dispatch_command_outcome(origin, command, event);
        if dispatched.default_prevented {
            return true;
        }

        if !self.spatial_navigation {
            return dispatched.anything_ran;
        }
        let GamepadEventKind::ButtonDown(button) = input.kind() else {
            return dispatched.anything_ran;
        };
        let direction = match button {
            GamepadButton::DpadUp => NavigationDirection::Up,
            GamepadButton::DpadDown => NavigationDirection::Down,
            GamepadButton::DpadLeft => NavigationDirection::Left,
            GamepadButton::DpadRight => NavigationDirection::Right,
            GamepadButton::South => {
                return focused.is_some_and(|node| self.activate(node, event.at_target(node)))
                    || dispatched.anything_ran;
            }
            _ => return false,
        };
        self.move_directional_focus(direction, event) || dispatched.anything_ran
    }
}
