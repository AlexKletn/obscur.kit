#![doc = include_str!("../../docs/api/fixture/key_event.doc/_module.md")]

use obscur_interaction::command::{KeyEvent, LogicalKey, NamedKey, PhysicalKey};

#[doc = include_str!("../../docs/api/fixture/key_event.doc/char_press.md")]
#[must_use]
pub fn char_press(character: char) -> KeyEvent {
    KeyEvent::press(PhysicalKey::KeyA, Some(LogicalKey::character(character)))
}

#[doc = include_str!("../../docs/api/fixture/key_event.doc/named_press.md")]
#[must_use]
pub fn named_press(named: NamedKey) -> KeyEvent {
    KeyEvent::press(PhysicalKey::KeyA, Some(LogicalKey::named(named)))
}
