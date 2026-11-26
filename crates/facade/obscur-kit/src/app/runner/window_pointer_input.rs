//! Native pointer events for the standalone window.

use std::ops::ControlFlow;

use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, Touch, TouchPhase};

use super::AppRunner;
use crate::geometry::Point;
use crate::interaction::pointer::{
    PointerButton, PointerButtons, PointerEvent, PointerId, WheelPhase,
};
use crate::runtime::{UiRuntime, frame::FrameReason};
use crate::window_winit::conversion::PointerConverter;

impl AppRunner {
    pub(super) fn handle_mouse_move(&mut self, position: PhysicalPosition<f64>) {
        self.cursor = (position.x, position.y);
        self.update_resize_cursor(self.resize_direction());
        let converter = PointerConverter::new();
        // CursorEntered has no position, so hit testing waits for a move.
        let event = if std::mem::take(&mut self.entry_pending) {
            converter.entered(PointerId::MOUSE, position, self.held_buttons)
        } else {
            converter.moved(PointerId::MOUSE, position, self.held_buttons)
        };
        self.runtime
            .handle_pointer(event.with_timestamp(self.now_micros()));
        self.request_redraw();
    }

    pub(super) fn handle_mouse_leave(&mut self) {
        self.update_resize_cursor(None);
        self.entry_pending = true;
        // The runtime clears hover on leave even at the last known position.
        let at = PhysicalPosition::new(self.cursor.0, self.cursor.1);
        let left = PointerConverter::new()
            .left(PointerId::MOUSE, at, self.held_buttons)
            .with_timestamp(self.now_micros());
        self.runtime.handle_pointer(left);
        self.request_redraw();
    }

    /// Breaks when the platform takes ownership of the press for window resizing.
    pub(super) fn handle_mouse_button(
        &mut self,
        button: MouseButton,
        state: ElementState,
    ) -> ControlFlow<(), Option<PointerEvent>> {
        let Some(button) = PointerConverter::button(button) else {
            return ControlFlow::Continue(None);
        };
        let primary_press = button == PointerButton::Primary && state == ElementState::Pressed;
        if primary_press
            && !self.entry_pending
            && let Some(direction) = self.resize_direction()
            && self.try_edge_resize(direction)
        {
            return ControlFlow::Break(());
        }
        let event = self.pointer_button_event(button, state);
        self.runtime.handle_pointer(event);
        self.request_redraw();
        ControlFlow::Continue(primary_press.then_some(event))
    }

    pub(super) fn handle_touch_contact(&mut self, contact: Touch) -> PointerEvent {
        // Updating the mouse cursor here would change the next wheel target.
        let event = PointerConverter::new()
            .touch(contact)
            .with_timestamp(self.now_micros());
        self.runtime.handle_pointer(event);
        self.request_redraw();
        event
    }

    pub(super) fn handle_mouse_wheel(&mut self, delta: MouseScrollDelta, phase: TouchPhase) {
        #[allow(clippy::cast_possible_truncation)]
        let at = Point::new(self.cursor.0 as f32, self.cursor.1 as f32);
        let phase = match phase {
            TouchPhase::Started => WheelPhase::Started,
            TouchPhase::Moved => WheelPhase::Moved,
            TouchPhase::Ended => WheelPhase::Ended,
            TouchPhase::Cancelled => WheelPhase::Cancelled,
        };
        let wheel = PointerConverter::wheel(delta, at)
            .with_phase(phase)
            .with_timestamp(self.now_micros());
        let step = wheel.pixels_y(UiRuntime::WHEEL_LINE_STEP);
        let reason = if self.runtime.handle_wheel(wheel).is_some() {
            FrameReason::WindowChange
        } else if let Some(handler) = self.scroll.as_mut() {
            // An app with its own virtual list can own the scroll position.
            handler(step / self.scale_factor.max(0.05) as f32);
            FrameReason::UiCommit
        } else {
            return;
        };
        self.runtime.request_frame(reason);
        self.request_redraw();
    }

    /// Assembles a domain button [`PointerEvent`] at the tracked cursor, folding
    /// `button` into the held set first so the event carries both the set held
    /// **after** the transition and the one member that moved.
    #[allow(clippy::cast_possible_truncation)]
    #[doc = include_str!("../../../docs/api/app/runner.doc/pointer_button_event_2.md")]
    fn pointer_button_event(&mut self, button: PointerButton, state: ElementState) -> PointerEvent {
        let at = PhysicalPosition::new(self.cursor.0, self.cursor.1);
        let converter = PointerConverter::new();
        self.held_buttons = held_after(self.held_buttons, button, state);
        match state {
            ElementState::Pressed => {
                converter.pressed(PointerId::MOUSE, at, button, self.held_buttons)
            }
            ElementState::Released => {
                converter.released(PointerId::MOUSE, at, button, self.held_buttons)
            }
        }
        .with_timestamp(self.now_micros())
    }
}

#[doc = include_str!("../../../docs/api/app/runner.doc/held_after.md")]
fn held_after(held: PointerButtons, button: PointerButton, state: ElementState) -> PointerButtons {
    let mut held = held;
    match state {
        ElementState::Pressed => held.press(button),
        ElementState::Released => held.release(button),
    }
    held
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A chord is a real state, and winit never reports one: it sends a
    /// transition per button and leaves the set to whoever is counting. Before
    /// this the runner counted nothing — a press was always exactly
    /// `{PRIMARY}` and a release always `{}` — so two buttons held at once was
    /// unrepresentable and, since the guard dropped every button but the left,
    /// unreachable as well.
    #[test]
    fn two_buttons_held_at_once_are_two_buttons_held_at_once() {
        let mut held = PointerButtons::new();

        held = held_after(held, PointerButton::Primary, ElementState::Pressed);
        assert!(held.contains(PointerButton::Primary));
        assert!(!held.contains(PointerButton::Secondary));

        held = held_after(held, PointerButton::Secondary, ElementState::Pressed);
        assert!(
            held.contains(PointerButton::Primary) && held.contains(PointerButton::Secondary),
            "the second press must join the first, not replace it"
        );

        // And releasing one leaves the other, which is the half a per-event
        // guess gets wrong: the set is not "the button that just moved".
        held = held_after(held, PointerButton::Primary, ElementState::Released);
        assert!(!held.contains(PointerButton::Primary));
        assert!(
            held.contains(PointerButton::Secondary),
            "the button still down is still down"
        );

        held = held_after(held, PointerButton::Secondary, ElementState::Released);
        assert!(held.is_empty());
    }

    /// The set an event carries is the one held *after* the transition — a
    /// press includes itself, a release excludes itself. That is what
    /// `PointerEvent::buttons` means, and it is why the press half of the old
    /// hardcoding (`{PRIMARY}`, then `{}`) looked right for one button.
    #[test]
    fn the_set_is_taken_after_the_transition_not_before() {
        let pressed = held_after(
            PointerButtons::new(),
            PointerButton::Middle,
            ElementState::Pressed,
        );
        assert!(pressed.contains(PointerButton::Middle));

        let released = held_after(pressed, PointerButton::Middle, ElementState::Released);
        assert!(released.is_empty());
    }

    /// A release of a button that was never pressed — the window was not
    /// focused when it went down, or the fold started mid-chord — must not
    /// corrupt the set. It clears a bit that is already clear.
    #[test]
    fn releasing_an_unheld_button_is_inert() {
        let mut held = PointerButtons::new();
        held.press(PointerButton::Primary);
        let after = held_after(held, PointerButton::Forward, ElementState::Released);
        assert_eq!(after, held);
    }
}
