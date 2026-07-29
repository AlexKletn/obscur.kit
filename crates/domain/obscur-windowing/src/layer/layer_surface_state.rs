//! Where a layer surface is in its life (spec §6.5).

/// The state a layer surface is in, as a host can actually observe it.
///
/// Deliberately smaller than the full protocol dance. A surface being *created*
/// is not observable from outside — the call that creates one either returns a
/// window or an error — and *failed* is carried by that error rather than by a
/// state a caller could poll. What remains is what a running application can
/// meaningfully branch on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LayerSurfaceState {
    /// Created and committed, waiting for the compositor's first configure.
    ///
    /// **Nothing may be drawn yet.** Attaching a buffer before the first
    /// configure is a protocol error, not a race.
    #[default]
    Configuring,
    /// Configured: the size is real and the surface can be drawn to.
    Ready,
    /// The compositor closed it. Every handle is dead and the host should stop.
    Closed,
}

impl LayerSurfaceState {
    /// Whether the surface may be drawn to.
    #[must_use]
    pub const fn is_drawable(self) -> bool {
        matches!(self, Self::Ready)
    }

    /// Whether the surface is gone for good.
    #[must_use]
    pub const fn is_closed(self) -> bool {
        matches!(self, Self::Closed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_configured_surface_may_be_drawn_to() {
        assert!(!LayerSurfaceState::Configuring.is_drawable());
        assert!(LayerSurfaceState::Ready.is_drawable());
        assert!(!LayerSurfaceState::Closed.is_drawable());
    }

    /// A surface starts unconfigured, because the protocol says the first thing
    /// that happens after creation is waiting to be told the size.
    #[test]
    fn a_surface_starts_waiting_to_be_configured() {
        assert_eq!(LayerSurfaceState::default(), LayerSurfaceState::Configuring);
        assert!(!LayerSurfaceState::default().is_closed());
    }
}
