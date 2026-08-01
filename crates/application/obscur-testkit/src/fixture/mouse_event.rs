#![doc = include_str!("mouse_event.doc/_module.md")]

use obscur_geometry::Point;
use obscur_interaction::pointer::{PointerDevice, PointerEvent, PointerEventKind, PointerId};

#[doc = include_str!("mouse_event.doc/mouse.md")]
#[must_use]
pub fn mouse(kind: PointerEventKind, x: f32, y: f32) -> PointerEvent {
    PointerEvent::new(
        PointerId::MOUSE,
        PointerDevice::Mouse,
        kind,
        Point::new(x, y),
    )
    .as_trusted()
}

#[doc = include_str!("mouse_event.doc/mouse_at.md")]
#[must_use]
pub fn mouse_at(kind: PointerEventKind, x: f32, y: f32, micros: u64) -> PointerEvent {
    mouse(kind, x, y).with_timestamp(micros)
}
