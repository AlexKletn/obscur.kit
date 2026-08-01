#![doc = include_str!("touch_contact.doc/_module.md")]

use obscur_geometry::Point;
use obscur_interaction::pointer::{PointerDevice, PointerEvent, PointerEventKind, PointerId};

#[doc = include_str!("touch_contact.doc/finger.md")]
#[must_use]
pub fn finger(contact: u64, kind: PointerEventKind, x: f32, y: f32) -> PointerEvent {
    untrusted_finger(contact, kind, x, y).as_trusted()
}

#[doc = include_str!("touch_contact.doc/finger_at.md")]
#[must_use]
pub fn finger_at(
    contact: u64,
    kind: PointerEventKind,
    x: f32,
    y: f32,
    micros: u64,
) -> PointerEvent {
    finger(contact, kind, x, y).with_timestamp(micros)
}

#[doc = include_str!("touch_contact.doc/untrusted_finger.md")]
#[must_use]
pub fn untrusted_finger(contact: u64, kind: PointerEventKind, x: f32, y: f32) -> PointerEvent {
    PointerEvent::new(
        PointerId::touch(contact),
        PointerDevice::Touch,
        kind,
        Point::new(x, y),
    )
}
