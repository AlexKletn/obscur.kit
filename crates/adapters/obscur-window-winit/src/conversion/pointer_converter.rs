//! The [`PointerConverter`] pure winit→domain pointer-event converter
//! (spec §4.7 pointer input; §6.1 windowing; §2.7 "преобразует native events в
//! доменные события").

use obscur_geometry::Point;
use obscur_interaction::pointer::{
    PointerButton, PointerDevice, PointerEvent, PointerEventKind, PointerId,
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

    /// Assembles a mouse pointer event of `kind` at `position` with no held
    /// buttons — the shared body of the per-event builders above.
    ///
    /// Takes `&self` for symmetry with the public builders (spec: converter
    /// methods take `&self`); it needs no instance state.
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use winit::dpi::PhysicalPosition;
    use winit::event::{ElementState, MouseButton};

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
}
