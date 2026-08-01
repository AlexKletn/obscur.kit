#![doc = include_str!("component_id.doc/_module.md")]

#[doc = include_str!("component_id.doc/ComponentId.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ComponentId(u64);

impl ComponentId {
    #[doc = include_str!("component_id.doc/from_raw.md")]
    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    #[doc = include_str!("component_id.doc/raw.md")]
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}
