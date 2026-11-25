//! Typed identifier: `WindowId`.

#[doc = include_str!("../../docs/api/id/window_id.doc/WindowId.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WindowId(u64);

impl WindowId {
    /// Wraps a raw handle value into a `WindowId`.
    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    /// Returns the raw handle value backing this identifier.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}
