//! Whether an event came from a real user gesture or was injected (spec §2.3).

/// §2.3's trusted/untrusted origin: whether an event was produced by a real
/// user gesture on a real device, or synthesized by something else.
///
/// # Why this is not decoration
///
/// This crate's [`PointerEvent`](crate::pointer::PointerEvent) is reachable from
/// three producers, and only one of them is a person: the winit adapter's
/// converter (a real OS event), `obscur_kit::devtools::apply_injected` (§7.6's
/// DevTools `Inject`, driven by whoever holds an `Inject`-scoped token), and the
/// headless harness's synthetic dispatcher (§7.4). Before this type the three
/// were **byte-identical** at the runtime's door, so an application could not
/// have declined to act on a gesture it never received.
///
/// # Fails closed
///
/// [`Untrusted`](Self::Untrusted) is the [`Default`], and
/// [`PointerEvent::new`](crate::pointer::PointerEvent::new) produces it. Trust is
/// *conferred* at the platform boundary by the adapter that received the OS
/// event — it is never assumed. A new injection path that forgets this type is
/// therefore untrusted, which is the harmless direction; the dangerous direction
/// (an injected event that passes for a real one) requires someone to write
/// [`Trusted`](Self::Trusted) on purpose.
///
/// A false negative — a real gesture marked untrusted, e.g. by a platform
/// adapter that never opts in — costs an application a legitimate action. That
/// is the trade this default makes deliberately.
///
/// # What this crate does *not* do with it
///
/// Nothing here gates on trust, and that is by design rather than an omission:
/// §7.6's agent loop exists to drive real clicks, so a runtime that refused
/// untrusted input would break the feature the flag describes. The runtime
/// records trust and reports it out through
/// [`PointerDispatch`](../../obscur_runtime/runtime/struct.PointerDispatch.html);
/// deciding that *this particular action* needs a real finger is the
/// application's call, because only the application knows which of its actions
/// are consequential.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EventTrust {
    /// Synthesized: injected over the DevTools protocol (§7.6), produced by a
    /// test harness (§7.4), or otherwise not vouched for by a platform adapter.
    ///
    /// The default, so trust is never acquired by omission.
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
