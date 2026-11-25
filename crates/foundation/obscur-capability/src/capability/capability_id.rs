//! Stable, name-like capability identifier.

/// A stable identifier for a capability. Plugins query capabilities by id, not
/// by platform name (spec §8.58).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CapabilityId(Box<str>);

impl CapabilityId {
    /// Creates a capability id from a stable name.
    #[must_use]
    pub fn new(name: impl Into<Box<str>>) -> Self {
        Self(name.into())
    }

    /// Returns the capability name.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
