//! `InstanceId`: identity of a single mounted component instance.

/// Stable, opaque identifier for one mounted component instance.
///
/// Distinct from [`ComponentId`](obscur_identity::ComponentId): a `ComponentId`
/// names a *kind* of component, while an `InstanceId` names a single live
/// instance of that kind. Backed by an integer handle whose value carries no
/// meaning outside the runtime that mints it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct InstanceId(u64);

impl InstanceId {
    /// Wraps a raw handle value into an `InstanceId`.
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
    use super::InstanceId;

    #[test]
    fn round_trips_raw_handle() {
        let id = InstanceId::from_raw(42);
        assert_eq!(id.raw(), 42);
    }

    #[test]
    fn distinct_handles_are_unequal() {
        assert_ne!(InstanceId::from_raw(1), InstanceId::from_raw(2));
    }
}
