//! Typed identifier: `SignalId`.

/// Stable, opaque identifier for an observable state source (spec §3.1).
///
/// State enters the UI through typed props, local component state, and
/// **read-only signal/observable sources**. A `SignalId` names one such source so
/// a component can declare that its render depends on it, and the runtime can
/// invalidate exactly the components that depend on a source that changed —
/// rather than everything.
///
/// Backed by an integer handle; the numeric value carries no meaning outside the
/// subsystem that mints it and must never be treated as a public schema. An
/// application typically declares one `const` per source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SignalId(u64);

impl SignalId {
    /// Wraps a raw handle value into a `SignalId`.
    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    /// Returns the raw handle value backing this identifier.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::SignalId;

    #[test]
    fn round_trips_its_raw_handle() {
        assert_eq!(SignalId::from_raw(7).raw(), 7);
    }

    #[test]
    fn distinct_handles_are_distinct_signals() {
        assert_eq!(SignalId::from_raw(1), SignalId::from_raw(1));
        assert_ne!(SignalId::from_raw(1), SignalId::from_raw(2));
    }
}
