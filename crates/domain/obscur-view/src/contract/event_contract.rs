//! `EventContract`: the typed contract of a single component event (spec §2.3).

use obscur_identity::InternedSymbol;

/// The exported contract for one event a component emits or accepts
/// (spec §2.3, §2.13).
///
/// Events are strictly typed: both the event name and its payload type are
/// referenced by interned handles, never by string dispatch. The two flags
/// describe how the event participates in the capture→target→bubble→default
/// propagation model: `bubbles` records whether it travels back up the bubble
/// phase, and `cancelable` records whether a handler may suppress the default
/// action. They are kept as distinct booleans because they are independent
/// traits rather than mutually exclusive modes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventContract {
    /// Interned name of the event.
    pub name: InternedSymbol,
    /// Interned name of the event's payload type.
    ///
    /// A reference to a type named and modelled elsewhere.
    pub payload_type: InternedSymbol,
    /// Whether the event travels through the bubble phase after reaching its
    /// target.
    pub bubbles: bool,
    /// Whether a handler may cancel the event's default action.
    pub cancelable: bool,
}

impl EventContract {
    /// Creates an event contract for `name` carrying payload type
    /// `payload_type`.
    ///
    /// The event starts non-bubbling and non-cancelable; use
    /// [`EventContract::bubbling`] and [`EventContract::cancelable`] to refine
    /// it in a builder style.
    #[must_use]
    pub const fn new(name: InternedSymbol, payload_type: InternedSymbol) -> Self {
        Self {
            name,
            payload_type,
            bubbles: false,
            cancelable: false,
        }
    }

    /// Marks the event as bubbling, returning the updated contract.
    #[must_use]
    pub const fn bubbling(mut self) -> Self {
        self.bubbles = true;
        self
    }

    /// Marks the event as cancelable, returning the updated contract.
    #[must_use]
    pub const fn cancelable(mut self) -> Self {
        self.cancelable = true;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_event_does_not_bubble_or_cancel() {
        let event = EventContract::new(InternedSymbol::from_raw(3), InternedSymbol::from_raw(4));
        assert!(!event.bubbles);
        assert!(!event.cancelable);
    }

    #[test]
    fn builder_sets_propagation_traits() {
        let event = EventContract::new(InternedSymbol::from_raw(3), InternedSymbol::from_raw(4))
            .bubbling()
            .cancelable();
        assert!(event.bubbles);
        assert!(event.cancelable);
    }
}
