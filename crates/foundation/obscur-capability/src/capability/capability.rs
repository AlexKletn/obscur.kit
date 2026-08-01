#![doc = include_str!("capability.doc/_module.md")]

use super::{CapabilityId, CapabilityScope};
use crate::support::SupportLevel;

#[doc = include_str!("capability.doc/Capability.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capability {
    #[doc = include_str!("capability.doc/id.md")]
    pub id: CapabilityId,
    #[doc = include_str!("capability.doc/scope.md")]
    pub scope: CapabilityScope,
    #[doc = include_str!("capability.doc/support.md")]
    pub support: SupportLevel,
}
