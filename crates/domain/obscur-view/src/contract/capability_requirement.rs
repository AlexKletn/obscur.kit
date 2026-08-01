#![doc = include_str!("capability_requirement.doc/_module.md")]

#[doc = include_str!("capability_requirement.doc/CapabilityRequirement.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityRequirement {
    #[doc = include_str!("capability_requirement.doc/Required.md")]
    Required,
    #[doc = include_str!("capability_requirement.doc/Optional.md")]
    Optional,
}

impl CapabilityRequirement {
    #[doc = include_str!("capability_requirement.doc/is_required.md")]
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
