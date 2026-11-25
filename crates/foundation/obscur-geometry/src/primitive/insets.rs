//! Directional insets in logical pixels.

/// Per-edge insets (padding/margin-like) in logical pixels. Physical edges are
/// resolved from logical ones after direction is applied (spec §7.3).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Insets {
    /// Top inset.
    pub top: f32,
    /// Right inset.
    pub right: f32,
    /// Bottom inset.
    pub bottom: f32,
    /// Left inset.
    pub left: f32,
}

impl Insets {
    /// Uniform insets on every edge.
    #[must_use]
    pub const fn uniform(value: f32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }
}
