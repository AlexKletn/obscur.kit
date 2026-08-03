#![doc = include_str!("event_name.doc/_module.md")]

use obscur_identity::InternedSymbol;

#[doc = include_str!("event_name.doc/EVENT_CLICK.md")]
pub const EVENT_CLICK: InternedSymbol = InternedSymbol::from_raw(0x0000_0101);

#[doc = include_str!("event_name.doc/EVENT_INPUT.md")]
pub const EVENT_INPUT: InternedSymbol = InternedSymbol::from_raw(0x0000_0102);

#[doc = include_str!("event_name.doc/EVENT_CHANGE.md")]
pub const EVENT_CHANGE: InternedSymbol = InternedSymbol::from_raw(0x0000_0103);

#[doc = include_str!("event_name.doc/EVENT_LONG_PRESS.md")]
pub const EVENT_LONG_PRESS: InternedSymbol = InternedSymbol::from_raw(0x0000_0104);

#[doc = include_str!("event_name.doc/EVENT_KEY_DOWN.md")]
pub const EVENT_KEY_DOWN: InternedSymbol = InternedSymbol::from_raw(0x0000_0105);

#[doc = include_str!("event_name.doc/EVENT_KEY_UP.md")]
pub const EVENT_KEY_UP: InternedSymbol = InternedSymbol::from_raw(0x0000_0106);

#[doc = include_str!("event_name.doc/RESERVED_EVENTS.md")]
pub const RESERVED_EVENTS: &[InternedSymbol] = &[
    EVENT_CLICK,
    EVENT_INPUT,
    EVENT_CHANGE,
    EVENT_LONG_PRESS,
    EVENT_KEY_DOWN,
    EVENT_KEY_UP,
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
