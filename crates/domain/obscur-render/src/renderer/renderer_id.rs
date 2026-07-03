//! Identifier for a renderer backend instance.

/// Identifies a renderer backend instance for diagnostics and multi-backend
/// setups (spec §5.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RendererId(u32);

impl RendererId {
    /// Wraps a raw renderer index.
    #[must_use]
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    /// Returns the raw renderer index.
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }
}
