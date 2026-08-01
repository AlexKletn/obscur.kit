#![doc = include_str!("version_stamp.doc/_module.md")]

#[doc = include_str!("version_stamp.doc/VersionStamp.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct VersionStamp(u64);

impl VersionStamp {
    #[doc = include_str!("version_stamp.doc/from_raw.md")]
    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    #[doc = include_str!("version_stamp.doc/raw.md")]
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }

    #[doc = include_str!("version_stamp.doc/bump.md")]
    #[must_use]
    pub const fn bump(self) -> Self {
        Self(self.0.wrapping_add(1))
    }
}
