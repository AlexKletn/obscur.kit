//! Version stamp used to detect changed snapshots between frames.

/// A coarse version stamp attached to snapshots and cached derivations so a
/// consumer can cheaply tell whether its input changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct VersionStamp(u64);

impl VersionStamp {
    /// Wraps a raw version value.
    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    /// Returns the raw version value.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }

    /// Returns the next version stamp, wrapping on overflow.
    #[must_use]
    pub const fn bump(self) -> Self {
        Self(self.0.wrapping_add(1))
    }
}
