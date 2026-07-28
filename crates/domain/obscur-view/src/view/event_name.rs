//! Well-known event-name symbols carried on element event bindings.
//!
//! An element view node binds an interaction event to a semantic command via a
//! [`ViewEventBinding`](super::ViewEventBinding) — `event → command`. The
//! *event* side of that binding is an [`InternedSymbol`], and both the
//! view-builder that authors the binding and the runtime that fires it must name
//! the same event. Rather than route these through an interner (which would need
//! the two sides to share a table), the toolkit reserves a small set of fixed
//! event-name symbols here, so the authored `on-click` binding and the runtime's
//! click lookup agree by *value* without any interner coherence.
//!
//! The reserved names occupy a low symbol namespace (`0x0000_01xx`), distinct
//! from the style-attribute namespace (`0x0000_00xx`, see
//! [`style_attribute`](super::style_attribute)) and the component/primitive prop
//! namespaces (e.g. `0x00C1_xxxx`), so an event name never collides with a style
//! attribute or an ordinary prop.
//!
//! Only the *event* is reserved; the *command* a binding maps to stays
//! app-owned — the application interns its own command symbol and uses the
//! **same** symbol when it registers the handler with the runtime, which only
//! compares symbols.

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

/// Reserved symbol naming a **long press** — a direct contact held still past
/// the runtime's threshold without lifting (spec §6.16 `LongPress`).
///
/// The touch counterpart to a secondary click. A touchscreen has one button and
/// no modifiers, so everything a right-click or a modifier would reach on a
/// desktop is reached by holding still, which makes this the only route to
/// context actions on a tablet. Bound the same way a click is —
/// [`ViewEventBinding::new(EVENT_LONG_PRESS, command)`](super::ViewEventBinding::new)
/// — and the runtime fires `command` when the hold is recognized, *before* the
/// finger lifts, and instead of the click that lift would otherwise have been.
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
