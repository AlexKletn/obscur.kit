//! Where a layer surface is in its life.

#[doc = include_str!("../../docs/api/layer/layer_surface_state.doc/LayerSurfaceState.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LayerSurfaceState {
    #[doc = include_str!("../../docs/api/layer/layer_surface_state.doc/Configuring.md")]
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
