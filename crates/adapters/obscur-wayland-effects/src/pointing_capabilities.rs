//! Pointing capabilities aggregated across Wayland seats.

/// Union of pointing capabilities reported by the available Wayland seats.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WaylandPointingCapabilities {
    /// At least one seat exposes a pointer.
    pub pointer: bool,
    /// At least one seat exposes a touch interface.
    pub touch: bool,
}
