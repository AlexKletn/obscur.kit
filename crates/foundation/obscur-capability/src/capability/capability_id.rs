#![doc = include_str!("capability_id.doc/_module.md")]

#[doc = include_str!("capability_id.doc/CapabilityId.md")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CapabilityId(Box<str>);

impl CapabilityId {
    #[doc = include_str!("capability_id.doc/new.md")]
    #[must_use]
    pub fn new(name: impl Into<Box<str>>) -> Self {
        Self(name.into())
    }

    #[doc = include_str!("capability_id.doc/as_str.md")]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
