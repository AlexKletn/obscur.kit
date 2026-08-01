#![doc = include_str!("event_trust.doc/_module.md")]

#[doc = include_str!("event_trust.doc/EventTrust.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EventTrust {
    #[doc = include_str!("event_trust.doc/Untrusted.md")]
    #[default]
    Untrusted,
    #[doc = include_str!("event_trust.doc/Trusted.md")]
    Trusted,
}

impl EventTrust {
    #[doc = include_str!("event_trust.doc/is_trusted.md")]
    #[must_use]
    pub const fn is_trusted(self) -> bool {
        matches!(self, Self::Trusted)
    }

    #[doc = include_str!("event_trust.doc/is_untrusted.md")]
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
