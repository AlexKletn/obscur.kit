//! `EventContract`: the typed contract of a single component event (spec §2.3).

use obscur_identity::InternedSymbol;

#[doc = include_str!("../../docs/api/contract/event_contract.doc/EventContract.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventContract {
    /// Interned name of the event.
    pub name: InternedSymbol,
    #[doc = include_str!("../../docs/api/contract/event_contract.doc/payload_type.md")]
    pub payload_type: InternedSymbol,
    /// Whether the event travels through the bubble phase after reaching its
    /// target.
    pub bubbles: bool,
    /// Whether a handler may cancel the event's default action.
    pub cancelable: bool,
}

impl EventContract {
    #[doc = include_str!("../../docs/api/contract/event_contract.doc/new.md")]
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
