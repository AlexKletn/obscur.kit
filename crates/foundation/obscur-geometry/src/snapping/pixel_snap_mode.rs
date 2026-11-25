//! Pixel-snapping policy selector.

/// Controls how a subtree's geometry is snapped to the physical pixel grid at
/// paint time (spec §4.6). Layout itself never rounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PixelSnapMode {
    /// Let the renderer choose an appropriate snapping strategy.
    #[default]
    Auto,
    /// Do not snap; preserve exact fractional geometry.
    None,
    /// Snap outer edges only.
    Edges,
}
