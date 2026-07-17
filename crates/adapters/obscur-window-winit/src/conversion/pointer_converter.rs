//! The [`PointerConverter`] pure winit→domain pointer-event converter
//! (spec §4.7 pointer input; §6.1 windowing; §2.7 "преобразует native events в
//! доменные события").

use obscur_geometry::Point;
use obscur_interaction::pointer::{
    PointerButton, PointerButtons, PointerDevice, PointerEvent, PointerEventKind, PointerId,
};

/// A stateless converter from winit's native mouse input to the domain's
/// [`PointerEvent`] model (spec §4.7, §2.7).
///
/// These are **pure**, per-event conversions with no event-loop state: each maps
/// one winit sub-value (a button, an element state, a cursor position) to the
/// matching domain value. The converters here intentionally do *not* track the
/// held-button set across events — full button-state tracking is the backend's
/// job, which threads a running [`PointerButtons`](obscur_interaction::pointer::PointerButtons)
/// through these per-event conversions. Every event produced here uses
/// [`PointerDevice::Mouse`] and the conventional [`PointerId::MOUSE`].
#[derive(Debug, Clone, Copy, Default)]
pub struct PointerConverter;

impl PointerConverter {
    /// Creates a converter.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Maps a winit [`MouseButton`](winit::event::MouseButton) to the domain
    /// [`PointerButton`], or `None` for a button the domain does not model
    /// ([`MouseButton::Other`](winit::event::MouseButton::Other)).
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

    /// Maps a winit [`ElementState`](winit::event::ElementState) to the pointer
    /// [`PointerEventKind`] for a button transition
    /// (`Pressed` → [`Down`](PointerEventKind::Down),
    /// `Released` → [`Up`](PointerEventKind::Up)).
    #[must_use]
    pub fn kind(state: winit::event::ElementState) -> PointerEventKind {
        match state {
            winit::event::ElementState::Pressed => PointerEventKind::Down,
            winit::event::ElementState::Released => PointerEventKind::Up,
        }
    }

    /// Builds a [`PointerEventKind::Move`] event at `position` for pointer `id`
    /// (spec §4.7).
    #[must_use]
    pub fn moved(
        &self,
        id: PointerId,
        position: winit::dpi::PhysicalPosition<f64>,
    ) -> PointerEvent {
        self.event(id, PointerEventKind::Move, position)
    }

    /// Builds a [`PointerEventKind::Enter`] event at `position` for pointer `id`
    /// (spec §4.7).
    #[must_use]
    pub fn entered(
        &self,
        id: PointerId,
        position: winit::dpi::PhysicalPosition<f64>,
    ) -> PointerEvent {
        self.event(id, PointerEventKind::Enter, position)
    }

    /// Builds a [`PointerEventKind::Leave`] event at `position` for pointer `id`
    /// (spec §4.7).
    #[must_use]
    pub fn left(&self, id: PointerId, position: winit::dpi::PhysicalPosition<f64>) -> PointerEvent {
        self.event(id, PointerEventKind::Leave, position)
    }

    /// Builds a [`PointerEventKind::Down`] event at `position` for pointer `id`,
    /// with `buttons` held **after** the press (spec §4.7).
    ///
    /// This exists because without it §2.3's trusted flag answers nothing where it
    /// matters. A host had no way to obtain a *trusted* press: the builders were
    /// move/enter/leave only, so a real click came in through `PointerEvent::new`
    /// as **untrusted** — exactly like a `§7.6` injected one. The two kinds
    /// DevTools can inject are `PointerDown`/`PointerUp`, and those were precisely
    /// the two this converter could not produce, so the flag's trusted and
    /// injectable sets were disjoint and it could never tell them apart.
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

    /// Builds a [`PointerEventKind::Up`] event at `position` for pointer `id`, with
    /// `buttons` still held **after** the release (spec §4.7). See
    /// [`pressed`](Self::pressed) for why the trusted flag needs this.
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

    /// Assembles a mouse pointer event of `kind` at `position` with no held
    /// buttons — the shared body of the per-event builders above.
    ///
    /// Takes `&self` for symmetry with the public builders (spec: converter
    /// methods take `&self`); it needs no instance state.
    ///
    /// # This is where trust enters the runtime
    ///
    /// Every event built here descends from a `winit::event::WindowEvent` the
    /// compositor delivered for an actual device, so this converter — and, in this
    /// workspace, *only* this converter — marks its output
    /// [`Trusted`](obscur_interaction::event::EventTrust::Trusted) (§2.3). The
    /// other two producers of a `PointerEvent` are synthetic by definition
    /// (§7.6's DevTools `Inject`, §7.4's headless harness) and leave the default
    /// untrusted standing, so an application can tell a real click from a driven
    /// one — but only for the kinds this converter can actually build. That is why
    /// [`pressed`](Self::pressed) and [`released`](Self::released) exist: while the
    /// builders were move/enter/leave only, the trusted set and the injectable set
    /// (`PointerDown`/`PointerUp`) were **disjoint**, and the flag distinguished
    /// nothing on the one gesture an attacker would forge. Key events still have no
    /// converter here at all, so a real `KeyDown` remains untrusted.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::unused_self,
        clippy::trivially_copy_pass_by_ref
    )]
    fn event(
        &self,
        id: PointerId,
        kind: PointerEventKind,
        position: winit::dpi::PhysicalPosition<f64>,
    ) -> PointerEvent {
        PointerEvent::new(
            id,
            PointerDevice::Mouse,
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
}
