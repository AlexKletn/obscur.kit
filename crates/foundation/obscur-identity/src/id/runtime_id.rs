#![doc = include_str!("runtime_id.doc/_module.md")]

#[doc = include_str!("runtime_id.doc/RuntimeId.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RuntimeId(u64);

impl RuntimeId {
    #[doc = include_str!("runtime_id.doc/from_raw.md")]
    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    #[doc = include_str!("runtime_id.doc/raw.md")]
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}
