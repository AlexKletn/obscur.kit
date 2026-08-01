#![doc = include_str!("layer_event.doc/_module.md")]

use obscur_geometry::{Point, Size};
use obscur_interaction::pointer::PointerEvent;

#[doc = include_str!("layer_event.doc/LayerEvent.md")]
#[derive(Debug, Clone)]
pub enum LayerEvent {
    #[doc = include_str!("layer_event.doc/Configured.md")]
    Configured {
        #[doc = include_str!("layer_event.doc/size.md")]
        size: Size,
    },
    #[doc = include_str!("layer_event.doc/ScaleChanged.md")]
    ScaleChanged {
        #[doc = include_str!("layer_event.doc/scale.md")]
        scale: f32,
    },
    #[doc = include_str!("layer_event.doc/Woken.md")]
    Woken,
    #[doc = include_str!("layer_event.doc/Closed.md")]
    Closed,
    #[doc = include_str!("layer_event.doc/Pointer.md")]
    Pointer(PointerEvent),
    #[doc = include_str!("layer_event.doc/Scroll.md")]
    Scroll {
        #[doc = include_str!("layer_event.doc/delta_y.md")]
        delta_y: f32,
        #[doc = include_str!("layer_event.doc/at.md")]
        at: Point,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_interaction::pointer::{PointerDevice, PointerEventKind, PointerId};

    #[test]
    fn a_pointer_event_passes_through_in_the_kits_own_model() {
        let event = LayerEvent::Pointer(PointerEvent::new(
            PointerId::touch(0),
            PointerDevice::Touch,
            PointerEventKind::Down,
            Point::new(4.0, 5.0),
        ));
        let LayerEvent::Pointer(inner) = event else {
            panic!("expected a pointer event");
        };
        assert_eq!(inner.device(), PointerDevice::Touch);
        assert_eq!(inner.position(), Point::new(4.0, 5.0));
    }
}
