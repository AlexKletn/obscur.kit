//! `CapabilityRequirement`: how strongly a component needs a capability.

/// Whether a component strictly requires a capability or merely benefits from
/// it (spec §2.13).
///
/// This is an independent trait of a capability dependency, so it lives in its
/// own type. `Required` means the component cannot function without the
/// capability; `Optional` means the component adapts gracefully when it is
/// absent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityRequirement {
    /// The component cannot function without the capability.
    Required,
    /// The component can function without the capability, using it only when
    /// present.
    Optional,
}

impl CapabilityRequirement {
    /// Returns whether this requirement is mandatory.
    #[must_use]
    pub const fn is_required(self) -> bool {
        matches!(self, Self::Required)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn required_is_mandatory_optional_is_not() {
        assert!(CapabilityRequirement::Required.is_required());
        assert!(!CapabilityRequirement::Optional.is_required());
    }
}
