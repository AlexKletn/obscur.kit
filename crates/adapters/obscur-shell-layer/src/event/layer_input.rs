#![doc = include_str!("layer_input.doc/_module.md")]

use obscur_geometry::Point;
use obscur_interaction::pointer::PointerEvent;

#[doc = include_str!("layer_input.doc/LayerInput.md")]
#[derive(Debug, Clone)]
pub enum LayerInput {
    #[doc = include_str!("layer_input.doc/Pointer.md")]
    Pointer(PointerEvent),
    #[doc = include_str!("layer_input.doc/Scroll.md")]
    Scroll {
        #[doc = include_str!("layer_input.doc/delta_y.md")]
        delta_y: f32,
        #[doc = include_str!("layer_input.doc/at.md")]
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
