//! [`LayerHandle`]: the running surface, as the application sees it.

use std::cell::RefCell;
use std::rc::Rc;

use crate::geometry::Size;
use crate::windowing::layer::{LayerSurfaceConfig, LayerSurfaceState};

/// A live handle to the surface a [`LayerApp`](super::LayerApp) is running.
///
/// # Why a handle and not `&mut` anything
///
/// The event loop owns the surface, and an application's command handlers run
/// *inside* it — so they cannot be given a reference to it without borrowing the
/// loop from itself. A handle sidesteps that entirely: a handler leaves a
/// request, and the loop picks it up at the top of the next turn. That also
/// means every change lands at a defined point in the frame rather than halfway
/// through one.
///
/// Clone it freely; every clone names the same surface.
///
/// # What it is for
///
/// Reconfiguring a live surface is how an application changes *how it is
/// presented* without restarting: a bar that grows, a panel that stops
/// reserving space and floats, a surface that moves from `Top` to `Overlay`.
/// None of layer-shell's placement properties need a new surface — band, anchor,
/// exclusive zone, margins and size are all live requests — so this is the whole
/// mechanism, and there is deliberately no "recreate" to reach for.
#[derive(Clone, Default)]
pub struct LayerHandle {
    inner: Rc<RefCell<HandleState>>,
}

/// What the loop publishes and the application requests.
#[derive(Default)]
struct HandleState {
    /// A configuration waiting to be applied.
    pending: Option<LayerSurfaceConfig>,
    /// Set when the application asks the loop to finish.
    close_requested: bool,
    /// The surface's logical size, as last configured.
    size: Size,
    /// The surface's scale, integral or fractional.
    scale: f32,
    /// Where the surface is in its life.
    state: LayerSurfaceState,
}

impl LayerHandle {
    /// A handle not yet attached to a surface.
    ///
    /// Build one before [`LayerApp::run`](super::LayerApp::run), hand clones to
    /// the handlers that need it, and give one to
    /// [`LayerApp::handle`](super::LayerApp::handle). Until the loop starts it
    /// reports [`LayerSurfaceState::Configuring`] and a zero size.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(HandleState {
                scale: 1.0,
                ..HandleState::default()
            })),
        }
    }

    /// Asks for the surface to be reconfigured.
    ///
    /// Applied at the top of the next loop turn, not immediately. The compositor
    /// answers with a fresh configure, so the resulting size is whatever it
    /// decides — read it back from [`size`](Self::size) rather than assuming the
    /// request was honoured verbatim.
    ///
    /// A second request before the first is applied replaces it: only the latest
    /// intent matters.
    pub fn reconfigure(&self, config: LayerSurfaceConfig) {
        self.inner.borrow_mut().pending = Some(config);
    }

    /// Asks the loop to finish and the surface to be torn down.
    pub fn close(&self) {
        self.inner.borrow_mut().close_requested = true;
    }

    /// The surface's logical size, as last configured.
    #[must_use]
    pub fn size(&self) -> Size {
        self.inner.borrow().size
    }

    /// The surface's scale — fractional where the compositor offers it.
    #[must_use]
    pub fn scale(&self) -> f32 {
        self.inner.borrow().scale
    }

    /// Where the surface is in its life.
    #[must_use]
    pub fn state(&self) -> LayerSurfaceState {
        self.inner.borrow().state
    }

    /// Takes the pending configuration, if one was requested.
    pub(super) fn take_pending(&self) -> Option<LayerSurfaceConfig> {
        self.inner.borrow_mut().pending.take()
    }

    /// Whether the application asked to stop.
    pub(super) fn close_requested(&self) -> bool {
        self.inner.borrow().close_requested
    }

    /// Publishes what the loop knows about the surface.
    pub(super) fn publish(&self, size: Size, scale: f32, state: LayerSurfaceState) {
        let mut inner = self.inner.borrow_mut();
        inner.size = size;
        inner.scale = scale;
        inner.state = state;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::windowing::layer::LayerSurfaceConfig;

    #[test]
    fn a_fresh_handle_reports_an_unconfigured_surface() {
        let handle = LayerHandle::new();
        assert_eq!(handle.state(), LayerSurfaceState::Configuring);
        assert_eq!(handle.scale(), 1.0);
        assert!(handle.take_pending().is_none());
        assert!(!handle.close_requested());
    }

    /// Only the latest intent matters: a handler that reconfigures twice before
    /// the loop turns should not make the surface flick through an intermediate
    /// shape.
    #[test]
    fn a_second_request_replaces_the_first() {
        let handle = LayerHandle::new();
        handle.reconfigure(LayerSurfaceConfig::top_bar("a", 10));
        handle.reconfigure(LayerSurfaceConfig::bottom_bar("b", 20));
        let taken = handle.take_pending().expect("a pending config");
        assert_eq!(taken.namespace, "b");
        assert!(handle.take_pending().is_none(), "taken exactly once");
    }

    #[test]
    fn clones_name_the_same_surface() {
        let handle = LayerHandle::new();
        let clone = handle.clone();
        clone.close();
        assert!(handle.close_requested());
        handle.publish(Size::new(800.0, 40.0), 1.5, LayerSurfaceState::Ready);
        assert_eq!(clone.size(), Size::new(800.0, 40.0));
        assert_eq!(clone.scale(), 1.5);
        assert!(clone.state().is_drawable());
    }
}
