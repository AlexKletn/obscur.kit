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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reserved_event_names_are_distinct_and_outside_the_style_namespace() {
        // Distinct from one another.
        assert_ne!(EVENT_CLICK, EVENT_INPUT);
        assert_ne!(EVENT_INPUT, EVENT_CHANGE);
        assert_ne!(EVENT_CLICK, EVENT_CHANGE);
        assert_ne!(EVENT_LONG_PRESS, EVENT_CLICK);
        assert_ne!(EVENT_LONG_PRESS, EVENT_INPUT);
        assert_ne!(EVENT_LONG_PRESS, EVENT_CHANGE);
        // Distinct from the style-attribute namespace (`0x0000_00xx`).
        assert_ne!(EVENT_CLICK, super::super::CLASS);
        assert_ne!(EVENT_CLICK, super::super::ID);
        assert_ne!(EVENT_CLICK, super::super::TYPE);
    }
}
