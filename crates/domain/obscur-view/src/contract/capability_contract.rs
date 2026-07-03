//! `CapabilityContract`: a capability a component depends on (spec §2.13).

use obscur_capability::CapabilityId;

use crate::contract::CapabilityRequirement;

/// The exported contract for one capability a component depends on
/// (spec §2.13).
///
/// A component declares the capabilities it needs so the runtime can decide
/// whether to instantiate it. Each dependency names a capability by its stable
/// [`CapabilityId`] and records whether it is [`CapabilityRequirement::Required`]
/// or merely [`CapabilityRequirement::Optional`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityContract {
    /// Stable identifier of the capability being depended on.
    pub capability: CapabilityId,
    /// How strongly the component needs the capability.
    pub requirement: CapabilityRequirement,
}

impl CapabilityContract {
    /// Creates a contract declaring `capability` with the given `requirement`.
    #[must_use]
    pub const fn new(capability: CapabilityId, requirement: CapabilityRequirement) -> Self {
        Self {
            capability,
            requirement,
        }
    }

    /// Creates a contract for a strictly required capability.
    #[must_use]
    pub const fn required(capability: CapabilityId) -> Self {
        Self::new(capability, CapabilityRequirement::Required)
    }

    /// Creates a contract for an optional capability.
    #[must_use]
    pub const fn optional(capability: CapabilityId) -> Self {
        Self::new(capability, CapabilityRequirement::Optional)
    }

    /// Returns whether the capability is strictly required.
    #[must_use]
    pub const fn is_required(&self) -> bool {
        self.requirement.is_required()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn required_constructor_is_mandatory() {
        let contract = CapabilityContract::required(CapabilityId::new("clipboard.read"));
        assert!(contract.is_required());
        assert_eq!(contract.capability.as_str(), "clipboard.read");
    }

    #[test]
    fn optional_constructor_is_not_mandatory() {
        let contract = CapabilityContract::optional(CapabilityId::new("clipboard.read"));
        assert!(!contract.is_required());
    }
}
