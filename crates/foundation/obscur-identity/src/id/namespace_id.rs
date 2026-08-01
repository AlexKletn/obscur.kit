#![doc = include_str!("namespace_id.doc/_module.md")]

#[doc = include_str!("namespace_id.doc/NamespaceId.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NamespaceId(u64);

impl NamespaceId {
    #[doc = include_str!("namespace_id.doc/from_raw.md")]
    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    #[doc = include_str!("namespace_id.doc/raw.md")]
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}
