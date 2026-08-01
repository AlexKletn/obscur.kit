#![doc = include_str!("pointer_converter.doc/_module.md")]

use obscur_geometry::Point;
use obscur_interaction::pointer::{
    PointerButton, PointerButtons, PointerDevice, PointerEvent, PointerEventKind, PointerId,
};

#[doc = include_str!("pointer_converter.doc/PointerConverter.md")]
#[derive(Debug, Clone, Copy, Default)]
pub struct PointerConverter;

impl PointerConverter {
    #[doc = include_str!("pointer_converter.doc/new.md")]
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    #[doc = include_str!("pointer_converter.doc/button.md")]
    #[must_use]
    pub fn button(button: winit::event::MouseButton) -> Option<PointerButton> {
        use winit::event::MouseButton as Winit;
        match button {
            Winit::Left => Some(PointerButton::Primary),
            Winit::Right => Some(PointerButton::Secondary),
            Winit::Middle => Some(PointerButton::Middle),
            Winit::Back => Some(PointerButton::Back),
            Winit::Forward => Some(PointerButton::Forward),
            Winit::Other(_) => None,
        }
    }

    #[doc = include_str!("pointer_converter.doc/kind.md")]
    #[must_use]
    pub fn kind(state: winit::event::ElementState) -> PointerEventKind {
        match state {
            winit::event::ElementState::Pressed => PointerEventKind::Down,
            winit::event::ElementState::Released => PointerEventKind::Up,
        }
    }

    #[doc = include_str!("pointer_converter.doc/moved.md")]
    #[must_use]
    pub fn moved(
        &self,
        id: PointerId,
        position: winit::dpi::PhysicalPosition<f64>,
    ) -> PointerEvent {
        self.event(id, PointerEventKind::Move, position)
    }

    #[doc = include_str!("pointer_converter.doc/entered.md")]
    #[must_use]
    pub fn entered(
        &self,
        id: PointerId,
        position: winit::dpi::PhysicalPosition<f64>,
    ) -> PointerEvent {
        self.event(id, PointerEventKind::Enter, position)
    }

    #[doc = include_str!("pointer_converter.doc/left.md")]
    #[must_use]
    pub fn left(&self, id: PointerId, position: winit::dpi::PhysicalPosition<f64>) -> PointerEvent {
        self.event(id, PointerEventKind::Leave, position)
    }

    #[doc = include_str!("pointer_converter.doc/pressed.md")]
    #[must_use]
    pub fn pressed(
        &self,
        id: PointerId,
        position: winit::dpi::PhysicalPosition<f64>,
        buttons: PointerButtons,
    ) -> PointerEvent {
        self.event(id, PointerEventKind::Down, position)
            .with_buttons(buttons)
    }

    #[doc = include_str!("pointer_converter.doc/released.md")]
    #[must_use]
    pub fn released(
        &self,
        id: PointerId,
        position: winit::dpi::PhysicalPosition<f64>,
        buttons: PointerButtons,
    ) -> PointerEvent {
        self.event(id, PointerEventKind::Up, position)
            .with_buttons(buttons)
    }

    #[doc = include_str!("pointer_converter.doc/touch.md")]
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn touch(&self, contact: winit::event::Touch) -> PointerEvent {
        use winit::event::TouchPhase;

        let kind = match contact.phase {
            TouchPhase::Started => PointerEventKind::Down,
            TouchPhase::Moved => PointerEventKind::Move,
            TouchPhase::Ended => PointerEventKind::Up,
            TouchPhase::Cancelled => PointerEventKind::Cancel,
        };
        let mut held = PointerButtons::new();
        if matches!(kind, PointerEventKind::Down | PointerEventKind::Move) {
            held.press(PointerButton::Primary);
        }
        let pressure = contact.force.map_or(0.0, |force| force.normalized() as f32);

        self.event_of(
            PointerDevice::Touch,
            PointerId::touch(contact.id),
            kind,
            contact.location,
        )
        .with_buttons(held)
        .with_pressure(pressure)
    }

    #[doc = include_str!("pointer_converter.doc/event.md")]
    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn event(
        &self,
        id: PointerId,
        kind: PointerEventKind,
        position: winit::dpi::PhysicalPosition<f64>,
    ) -> PointerEvent {
        self.event_of(PointerDevice::Mouse, id, kind, position)
    }

    #[doc = include_str!("pointer_converter.doc/clippy.md")]
    #[allow(
        clippy::cast_possible_truncation,
        clippy::unused_self,
        clippy::trivially_copy_pass_by_ref
    )]
    #[allow(
        clippy::cast_possible_truncation,
        clippy::unused_self,
        clippy::trivially_copy_pass_by_ref
    )]
    fn event_of(
        &self,
        device: PointerDevice,
        id: PointerId,
        kind: PointerEventKind,
        position: winit::dpi::PhysicalPosition<f64>,
    ) -> PointerEvent {
        PointerEvent::new(
            id,
            device,
            kind,
            Point::new(position.x as f32, position.y as f32),
        )
        .as_trusted()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_interaction::event::EventTrust;
    use winit::dpi::PhysicalPosition;
    use winit::event::{ElementState, MouseButton};

    /// §2.3's trusted flag has to answer on the gesture an attacker would forge.
    /// Before `pressed`/`released` existed, this converter could build only
    /// move/enter/leave — while DevTools' `Inject` can forge exactly
    /// `PointerDown`/`PointerUp`. The trusted set and the injectable set were
    /// **disjoint**, so a real click and a forged one were both untrusted and the
    /// flag distinguished nothing.
    #[test]
    fn a_real_press_is_trusted_and_an_injected_one_is_not() {
        let converter = PointerConverter::new();
        let mut held = PointerButtons::new();
        held.press(PointerButton::Primary);

        let real = converter.pressed(PointerId::MOUSE, PhysicalPosition::new(4.0, 8.0), held);
        assert_eq!(
            real.trust,
            EventTrust::Trusted,
            "a press the compositor delivered must be trusted, or the flag says \
             nothing about the one event that matters"
        );
        assert_eq!(real.kind, PointerEventKind::Down);
        assert_eq!(real.buttons, held);

        // What §7.6's Inject builds: the same kind, through the public constructor,
        // with nothing vouching for it.
        let injected = PointerEvent::new(
            PointerId::MOUSE,
            PointerDevice::Mouse,
            PointerEventKind::Down,
            Point::new(4.0, 8.0),
        );
        assert_eq!(injected.trust, EventTrust::Untrusted);
        assert_ne!(
            real.trust, injected.trust,
            "a real press and an injected press must be distinguishable"
        );
    }

    /// The release half, so neither builder can carry the other.
    #[test]
    fn a_real_release_is_trusted() {
        let converter = PointerConverter::new();
        let event = converter.released(
            PointerId::MOUSE,
            PhysicalPosition::new(1.0, 2.0),
            PointerButtons::new(),
        );
        assert_eq!(event.trust, EventTrust::Trusted);
        assert_eq!(event.kind, PointerEventKind::Up);
    }

    #[test]
    fn buttons_map_to_domain_buttons() {
        assert_eq!(
            PointerConverter::button(MouseButton::Left),
            Some(PointerButton::Primary)
        );
        assert_eq!(
            PointerConverter::button(MouseButton::Right),
            Some(PointerButton::Secondary)
        );
        assert_eq!(
            PointerConverter::button(MouseButton::Middle),
            Some(PointerButton::Middle)
        );
        assert_eq!(
            PointerConverter::button(MouseButton::Back),
            Some(PointerButton::Back)
        );
        assert_eq!(
            PointerConverter::button(MouseButton::Forward),
            Some(PointerButton::Forward)
        );
    }

    #[test]
    fn unmodeled_button_is_none() {
        assert_eq!(PointerConverter::button(MouseButton::Other(9)), None);
    }

    #[test]
    fn element_state_maps_to_kind() {
        assert_eq!(
            PointerConverter::kind(ElementState::Pressed),
            PointerEventKind::Down
        );
        assert_eq!(
            PointerConverter::kind(ElementState::Released),
            PointerEventKind::Up
        );
    }

    #[test]
    fn moved_builds_a_move_at_the_position() {
        let converter = PointerConverter::new();
        let event = converter.moved(PointerId::MOUSE, PhysicalPosition::new(12.0, 34.0));
        assert_eq!(event.pointer(), PointerId::MOUSE);
        assert_eq!(event.device(), PointerDevice::Mouse);
        assert_eq!(event.kind(), PointerEventKind::Move);
        assert_eq!(event.position(), Point::new(12.0, 34.0));
        assert!(event.buttons().is_empty());
    }

    #[test]
    fn entered_and_left_build_the_matching_kinds() {
        let converter = PointerConverter::new();
        let pos = PhysicalPosition::new(1.0, 2.0);
        assert_eq!(
            converter.entered(PointerId::MOUSE, pos).kind(),
            PointerEventKind::Enter
        );
        assert_eq!(
            converter.left(PointerId::MOUSE, pos).kind(),
            PointerEventKind::Leave
        );
    }

    /// §2.3: this converter is the *only* place in the workspace where a real OS
    /// event becomes a domain event, so it is the only place trust may be
    /// conferred. This is the trusted half of the pair whose untrusted half is
    /// `injected_pointer_is_not_trusted` (obscur-kit `tests/devtools_live.rs`) —
    /// together they prove the two paths are distinguishable, which is the whole
    /// point of the flag. The two halves fail independently: deleting
    /// `.as_trusted()` here fails only this one.
    #[test]
    fn a_converted_os_event_is_trusted() {
        let converter = PointerConverter::new();
        let pos = PhysicalPosition::new(5.0, 6.0);

        // Every builder funnels through `event`, so every one must be trusted.
        assert!(
            converter.moved(PointerId::MOUSE, pos).is_trusted(),
            "a real cursor move is a user gesture"
        );
        assert!(converter.entered(PointerId::MOUSE, pos).is_trusted());
        assert!(converter.left(PointerId::MOUSE, pos).is_trusted());
        assert_eq!(
            converter.moved(PointerId::MOUSE, pos).trust(),
            EventTrust::Trusted
        );
    }

    /// The guard on the guard: the trusted mark above must come from *this
    /// converter*, not from a permissive default. If `PointerEvent::new` ever
    /// starts handing out trust, `a_converted_os_event_is_trusted` would pass for
    /// the wrong reason and every injected event would silently become a "real
    /// gesture" — so pin the default here.
    #[test]
    fn a_bare_pointer_event_is_untrusted_so_trust_comes_from_this_converter() {
        let bare = PointerEvent::new(
            PointerId::MOUSE,
            PointerDevice::Mouse,
            PointerEventKind::Move,
            Point::new(5.0, 6.0),
        );
        assert!(
            !bare.is_trusted(),
            "PointerEvent::new must fail closed; trust is conferred, never default"
        );
    }

    /// Builds the winit touch event for `contact` in `phase` at `(x, y)`, with no
    /// force reported — the shape a Wayland touchscreen actually delivers.
    fn contact(id: u64, phase: winit::event::TouchPhase, x: f64, y: f64) -> winit::event::Touch {
        winit::event::Touch {
            device_id: winit::event::DeviceId::dummy(),
            phase,
            location: PhysicalPosition::new(x, y),
            force: None,
            id,
        }
    }

    #[test]
    fn a_contact_becomes_a_trusted_touch_pointer_at_its_location() {
        let event = PointerConverter::new().touch(contact(
            7,
            winit::event::TouchPhase::Started,
            12.0,
            34.0,
        ));

        assert_eq!(event.kind, PointerEventKind::Down);
        assert_eq!(event.device, PointerDevice::Touch);
        assert_eq!(event.pointer, PointerId::touch(7));
        assert_eq!(event.position, Point::new(12.0, 34.0));
        assert!(
            event.buttons.contains(PointerButton::Primary),
            "a contact is the engagement device-agnostic code tests for"
        );
        assert!(
            event.is_trusted(),
            "a finger the compositor delivered is as real a gesture as a click"
        );
    }

    /// The distinction the whole conversion turns on: the system taking a
    /// sequence away is **not** a release.
    ///
    /// They are one phase apart in winit and worlds apart downstream — an `Up` on
    /// the pressed node activates it (runs the bound command), a `Cancel` does
    /// not. Mapping `Cancelled` to `Up` would make an interrupted gesture — a
    /// system edge-swipe stealing the sequence — press whatever button the finger
    /// happened to be resting on.
    #[test]
    fn a_cancelled_contact_is_a_cancel_and_not_a_release() {
        let converter = PointerConverter::new();
        let cancelled = converter.touch(contact(1, winit::event::TouchPhase::Cancelled, 5.0, 5.0));
        let ended = converter.touch(contact(1, winit::event::TouchPhase::Ended, 5.0, 5.0));

        assert_eq!(cancelled.kind, PointerEventKind::Cancel);
        assert_eq!(ended.kind, PointerEventKind::Up);
        assert_ne!(cancelled.kind, ended.kind);
        // Both end the contact, so neither leaves the primary button held.
        assert!(cancelled.buttons.is_empty());
        assert!(ended.buttons.is_empty());
    }

    #[test]
    fn two_fingers_are_two_pointers_and_neither_is_the_mouse() {
        let converter = PointerConverter::new();
        let first = converter.touch(contact(0, winit::event::TouchPhase::Started, 1.0, 1.0));
        let second = converter.touch(contact(1, winit::event::TouchPhase::Started, 9.0, 9.0));

        assert_ne!(first.pointer, second.pointer);
        assert_ne!(
            first.pointer,
            PointerId::MOUSE,
            "contact 0 must not be the mouse, or the first finger down steals the \
             cursor's identity"
        );
    }

    #[test]
    fn a_reported_force_becomes_the_contact_pressure() {
        let mut heavy = contact(3, winit::event::TouchPhase::Moved, 2.0, 2.0);
        heavy.force = Some(winit::event::Force::Normalized(0.5));
        let event = PointerConverter::new().touch(heavy);
        assert!((event.pressure - 0.5).abs() < f32::EPSILON);

        // No force reported (this workspace's Wayland target) is the domain's
        // "device without pressure", not a feather-light touch.
        let plain =
            PointerConverter::new().touch(contact(3, winit::event::TouchPhase::Moved, 2.0, 2.0));
        assert!((plain.pressure - 0.0).abs() < f32::EPSILON);
    }
}
