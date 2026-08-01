#![doc = include_str!("capability_contract.doc/_module.md")]

use obscur_capability::CapabilityId;

use crate::contract::CapabilityRequirement;

#[doc = include_str!("capability_contract.doc/CapabilityContract.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityContract {
    #[doc = include_str!("capability_contract.doc/capability.md")]
    pub capability: CapabilityId,
    #[doc = include_str!("capability_contract.doc/requirement.md")]
    pub requirement: CapabilityRequirement,
}

impl CapabilityContract {
    #[doc = include_str!("capability_contract.doc/new.md")]
    #[must_use]
    pub const fn new(capability: CapabilityId, requirement: CapabilityRequirement) -> Self {
        Self {
            capability,
            requirement,
        }
    }

    #[doc = include_str!("capability_contract.doc/required.md")]
    #[must_use]
    pub const fn required(capability: CapabilityId) -> Self {
        Self::new(capability, CapabilityRequirement::Required)
    }

    #[doc = include_str!("capability_contract.doc/optional.md")]
    #[must_use]
    pub const fn optional(capability: CapabilityId) -> Self {
        Self::new(capability, CapabilityRequirement::Optional)
    }

    #[doc = include_str!("capability_contract.doc/is_required.md")]
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
