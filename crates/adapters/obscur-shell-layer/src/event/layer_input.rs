//! Input arriving on a layer surface, in the kit's own vocabulary.

use obscur_geometry::Point;
use obscur_interaction::command::{KeyEvent, Modifiers};
use obscur_interaction::pointer::PointerEvent;

#[doc = include_str!("../../docs/api/event/layer_input.doc/LayerInput.md")]
#[derive(Debug, Clone)]
pub enum LayerInput {
    /// A complete text input batch for an editable panel node.
    TextInput(obscur_identity::NodeId, obscur_text::ime::TextInputUpdate),
    /// Keyboard input delivered by the Wayland seat.
    Key(KeyEvent),
    /// Keyboard input delivered by the Wayland seat.
    Modifiers(Modifiers),
    /// Keyboard input delivered by the Wayland seat.
    KeyboardFocus(bool),
    /// A pointer or touch event, already converted to the kit's model.
    Pointer(PointerEvent),
    /// A scroll gesture, in logical pixels along the vertical axis.
    Scroll {
        /// How far to scroll, already multiplied through the surface scale.
        delta_y: f32,
        /// Where the cursor was when the scroll happened.
        at: Point,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_interaction::pointer::{PointerDevice, PointerEventKind, PointerId};

    #[test]
    fn a_pointer_event_passes_through_in_the_kits_own_model() {
        let event = LayerInput::Pointer(PointerEvent::new(
            PointerId::touch(0),
            PointerDevice::Touch,
            PointerEventKind::Down,
            Point::new(4.0, 5.0),
        ));
        let LayerInput::Pointer(inner) = event else {
            panic!("expected a pointer event");
        };
        assert_eq!(inner.device(), PointerDevice::Touch);
        assert_eq!(inner.position(), Point::new(4.0, 5.0));
    }
}
