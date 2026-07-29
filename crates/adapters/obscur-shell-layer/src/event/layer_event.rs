//! What a layer surface reports back to its host.

use obscur_geometry::{Point, Size};
use obscur_interaction::pointer::PointerEvent;

/// One thing that happened to a layer surface, in the host's own vocabulary.
///
/// Pointer and touch arrive already converted into the kit's
/// [`PointerEvent`], so a host feeds them to `UiRuntime::handle_pointer`
/// unchanged and every device-agnostic behaviour — hover, focus, gestures,
/// capture — works as it does in a window.
#[derive(Debug, Clone)]
pub enum LayerEvent {
    /// The compositor decided the surface's size. **This, not the request, is
    /// the real extent**: an anchored axis is decided by the compositor, so a bar
    /// that asked for a width and got the output's is normal, not an error.
    Configured {
        /// The size in logical pixels.
        size: Size,
    },
    /// The output's scale changed; the host should re-derive its physical
    /// surface extent.
    ScaleChanged {
        /// The new device-pixel ratio.
        scale: f32,
    },
    /// Something outside the compositor asked the loop to turn — a worker
    /// thread, a timer, another protocol's connection — through a
    /// [`LayerWaker`](crate::surface::LayerWaker).
    ///
    /// It carries nothing, and cannot: the waker does not know why it was rung.
    /// A host treats it as "reconsider the interface", which is what every
    /// caller of a waker actually means.
    Woken,
    /// The compositor closed the surface. Every handle to it is now dead and the
    /// host should stop.
    Closed,
    /// A pointer, touch or pen event, already in the kit's model.
    Pointer(PointerEvent),
    /// A scroll of `delta_y` pixels at the pointer's position.
    ///
    /// The position travels with the delta because scrolling is routed to the
    /// container *under the pointer*, and a host that had to guess it — from the
    /// hovered node's box, say — would send the notch to the wrong place the
    /// moment the pointer sat over a child.
    Scroll {
        /// Pixels to scroll; positive reveals content further down.
        delta_y: f32,
        /// Where the pointer was, in physical pixels.
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
