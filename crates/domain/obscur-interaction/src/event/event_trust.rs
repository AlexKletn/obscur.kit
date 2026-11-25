//! Whether an event came from a real user gesture or was injected (spec §2.3).

#[doc = include_str!("../../docs/api/event/event_trust.doc/EventTrust.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EventTrust {
    #[doc = include_str!("../../docs/api/event/event_trust.doc/Untrusted.md")]
    #[default]
    Untrusted,
    /// Produced by a real user gesture on a real device, as reported by the
    /// platform and vouched for by the adapter that received it.
    Trusted,
}

impl EventTrust {
    /// Returns whether this is a real user gesture ([`Trusted`](Self::Trusted)).
    #[must_use]
    pub const fn is_trusted(self) -> bool {
        matches!(self, Self::Trusted)
    }

    /// Returns whether this event was synthesized ([`Untrusted`](Self::Untrusted)).
    #[must_use]
    pub const fn is_untrusted(self) -> bool {
        matches!(self, Self::Untrusted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn predicates_are_exclusive() {
        assert!(EventTrust::Trusted.is_trusted());
        assert!(!EventTrust::Trusted.is_untrusted());

        assert!(EventTrust::Untrusted.is_untrusted());
        assert!(!EventTrust::Untrusted.is_trusted());
    }

    #[test]
    fn default_fails_closed() {
        // The whole security argument rests on this: trust is never the default.
        assert_eq!(EventTrust::default(), EventTrust::Untrusted);
        assert!(!EventTrust::default().is_trusted());
    }
}
