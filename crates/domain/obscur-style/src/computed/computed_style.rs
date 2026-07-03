//! Computed style for a node (skeleton).

/// The fully-resolved computed style of a node, produced after cascade and
/// inheritance (spec §4.14). This skeleton is a placeholder; concrete computed
/// properties are added in Phase 1.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ComputedStyle {
    /// Resolved opacity in `0.0..=1.0`.
    pub opacity: f32,
}
