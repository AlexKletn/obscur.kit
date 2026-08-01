#![doc = include_str!("event_contract.doc/_module.md")]

use obscur_identity::InternedSymbol;

#[doc = include_str!("event_contract.doc/EventContract.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventContract {
    #[doc = include_str!("event_contract.doc/name.md")]
    pub name: InternedSymbol,
    #[doc = include_str!("event_contract.doc/payload_type.md")]
    pub payload_type: InternedSymbol,
    #[doc = include_str!("event_contract.doc/bubbles.md")]
    pub bubbles: bool,
    #[doc = include_str!("event_contract.doc/cancelable.md")]
    pub cancelable: bool,
}

impl EventContract {
    #[doc = include_str!("event_contract.doc/new.md")]
    #[must_use]
    pub const fn new(name: InternedSymbol, payload_type: InternedSymbol) -> Self {
        Self {
            name,
            payload_type,
            bubbles: false,
            cancelable: false,
        }
    }

    #[doc = include_str!("event_contract.doc/bubbling.md")]
    #[must_use]
    pub const fn bubbling(mut self) -> Self {
        self.bubbles = true;
        self
    }

    #[doc = include_str!("event_contract.doc/cancelable_2.md")]
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
