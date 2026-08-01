#![doc = include_str!("window_id.doc/_module.md")]

#[doc = include_str!("window_id.doc/WindowId.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WindowId(u64);

impl WindowId {
    #[doc = include_str!("window_id.doc/from_raw.md")]
    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    #[doc = include_str!("window_id.doc/raw.md")]
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}
