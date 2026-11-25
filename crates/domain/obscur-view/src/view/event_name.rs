#![doc = include_str!("../../docs/api/view/event_name.doc/_module.md")]

use obscur_identity::InternedSymbol;

/// Reserved symbol naming the primary **click** event — a press-then-release on
/// the same element (spec §2.3, §3.1). The deliverable event name: a
/// [`ViewEventBinding::new(EVENT_CLICK, command)`](super::ViewEventBinding::new)
/// on an element fires `command` when the runtime detects a click on it.
pub const EVENT_CLICK: InternedSymbol = InternedSymbol::from_raw(0x0000_0101);

/// Reserved symbol naming an **input** event — a value edit in progress (e.g. a
/// text field changing as the user types). Reserved now for later wiring; the
/// runtime does not yet fire it.
pub const EVENT_INPUT: InternedSymbol = InternedSymbol::from_raw(0x0000_0102);

/// Reserved symbol naming a **change** event — a committed value change (e.g. a
/// field losing focus after an edit, or a checkbox toggling). Reserved now for
/// later wiring; the runtime does not yet fire it.
pub const EVENT_CHANGE: InternedSymbol = InternedSymbol::from_raw(0x0000_0103);

#[doc = include_str!("../../docs/api/view/event_name.doc/EVENT_LONG_PRESS.md")]
pub const EVENT_LONG_PRESS: InternedSymbol = InternedSymbol::from_raw(0x0000_0104);

#[doc = include_str!("../../docs/api/view/event_name.doc/EVENT_KEY_DOWN.md")]
pub const EVENT_KEY_DOWN: InternedSymbol = InternedSymbol::from_raw(0x0000_0105);

#[doc = include_str!("../../docs/api/view/event_name.doc/EVENT_KEY_UP.md")]
pub const EVENT_KEY_UP: InternedSymbol = InternedSymbol::from_raw(0x0000_0106);

#[doc = include_str!("../../docs/api/view/event_name.doc/EVENT_POINTER_DOWN.md")]
pub const EVENT_POINTER_DOWN: InternedSymbol = InternedSymbol::from_raw(0x0000_0107);

#[doc = include_str!("../../docs/api/view/event_name.doc/EVENT_POINTER_UP.md")]
pub const EVENT_POINTER_UP: InternedSymbol = InternedSymbol::from_raw(0x0000_0108);

#[doc = include_str!("../../docs/api/view/event_name.doc/EVENT_POINTER_MOVE.md")]
pub const EVENT_POINTER_MOVE: InternedSymbol = InternedSymbol::from_raw(0x0000_0109);

#[doc = include_str!("../../docs/api/view/event_name.doc/EVENT_POINTER_CANCEL.md")]
pub const EVENT_POINTER_CANCEL: InternedSymbol = InternedSymbol::from_raw(0x0000_010A);

#[doc = include_str!("../../docs/api/view/event_name.doc/EVENT_POINTER_ENTER.md")]
pub const EVENT_POINTER_ENTER: InternedSymbol = InternedSymbol::from_raw(0x0000_010B);

#[doc = include_str!("../../docs/api/view/event_name.doc/EVENT_POINTER_LEAVE.md")]
pub const EVENT_POINTER_LEAVE: InternedSymbol = InternedSymbol::from_raw(0x0000_010C);

#[doc = include_str!("../../docs/api/view/event_name.doc/EVENT_WHEEL.md")]
pub const EVENT_WHEEL: InternedSymbol = InternedSymbol::from_raw(0x0000_010D);

#[doc = include_str!("../../docs/api/view/event_name.doc/EVENT_FOCUS.md")]
pub const EVENT_FOCUS: InternedSymbol = InternedSymbol::from_raw(0x0000_010E);

#[doc = include_str!("../../docs/api/view/event_name.doc/EVENT_BLUR.md")]
pub const EVENT_BLUR: InternedSymbol = InternedSymbol::from_raw(0x0000_010F);

#[doc = include_str!("../../docs/api/view/event_name.doc/EVENT_CONTEXT_MENU.md")]
pub const EVENT_CONTEXT_MENU: InternedSymbol = InternedSymbol::from_raw(0x0000_0110);

/// Native touchpad input event name (`touchpadgesture`).
pub const EVENT_TOUCHPAD_GESTURE: InternedSymbol = InternedSymbol::from_raw(0x0000_0111);

/// The application touch gesture candidate stream (`touchgesture`).
pub const EVENT_TOUCH_GESTURE: InternedSymbol = InternedSymbol::from_raw(0x0000_0112);

/// Predictive Back from the host of the current window (`systemback`).
pub const EVENT_SYSTEM_BACK: InternedSymbol = InternedSymbol::from_raw(0x0000_0113);

/// Application candidate for a phased pixel-scroll sequence (`scrollgesture`).
pub const EVENT_SCROLL_GESTURE: InternedSymbol = InternedSymbol::from_raw(0x0000_0114);

/// A host gamepad input report (`gamepad`).
pub const EVENT_GAMEPAD: InternedSymbol = InternedSymbol::from_raw(0x0000_0115);

#[doc = include_str!("../../docs/api/view/event_name.doc/RESERVED_EVENTS.md")]
pub const RESERVED_EVENTS: &[InternedSymbol] = &[
    EVENT_CLICK,
    EVENT_INPUT,
    EVENT_CHANGE,
    EVENT_LONG_PRESS,
    EVENT_KEY_DOWN,
    EVENT_KEY_UP,
    EVENT_POINTER_DOWN,
    EVENT_POINTER_UP,
    EVENT_POINTER_MOVE,
    EVENT_POINTER_CANCEL,
    EVENT_POINTER_ENTER,
    EVENT_POINTER_LEAVE,
    EVENT_WHEEL,
    EVENT_FOCUS,
    EVENT_BLUR,
    EVENT_TOUCHPAD_GESTURE,
    EVENT_TOUCH_GESTURE,
    EVENT_SYSTEM_BACK,
    EVENT_SCROLL_GESTURE,
    EVENT_GAMEPAD,
    EVENT_CONTEXT_MENU,
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn every_reserved_event_name_is_distinct() {
        // One assertion over the whole set rather than a quadratic list of
        // pairs, so adding a name cannot quietly skip its own collision check —
        // which is exactly what the pairwise version had started to invite.
        let unique: HashSet<u32> = RESERVED_EVENTS.iter().map(|name| name.raw()).collect();
        assert_eq!(
            unique.len(),
            RESERVED_EVENTS.len(),
            "two reserved event names share a symbol"
        );
    }

    #[test]
    fn the_reserved_event_range_is_dense_from_one() {
        // The allocation rule, pinned rather than remembered: names are taken in
        // order from `0x0000_0101` and no hole is left behind. A hole is how two
        // people later pick the same next number — the collision check above
        // catches that only if both spellings land in this list, and a name added
        // to one crate's mirror and not the other never does.
        let mut raws: Vec<u32> = RESERVED_EVENTS.iter().map(|name| name.raw()).collect();
        raws.sort_unstable();
        for (offset, raw) in raws.iter().enumerate() {
            let expected = 0x0000_0101 + u32::try_from(offset).expect("a small set");
            assert_eq!(*raw, expected, "the reserved event range has a hole");
        }
    }

    #[test]
    fn no_reserved_event_name_aliases_the_style_attribute_namespace() {
        // Style attributes live in `0x0000_00xx` and event names in
        // `0x0000_01xx`; a collision would make one node property answer two
        // questions.
        for name in RESERVED_EVENTS {
            assert_ne!(*name, super::super::CLASS);
            assert_ne!(*name, super::super::ID);
            assert_ne!(*name, super::super::TYPE);
            assert_ne!(*name, super::super::LOCAL_NAME);
            assert_eq!(
                name.raw() & 0xFFFF_FF00,
                0x0000_0100,
                "{name:?} is outside the reserved event range"
            );
        }
    }
}
