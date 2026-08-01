#![doc = include_str!("layer_handle.doc/_module.md")]

use std::cell::RefCell;
use std::rc::Rc;

use crate::geometry::Size;
use crate::windowing::layer::{LayerSurfaceConfig, LayerSurfaceState};

#[doc = include_str!("layer_handle.doc/LayerHandle.md")]
#[derive(Clone, Default)]
pub struct LayerHandle {
    inner: Rc<RefCell<HandleState>>,
}

#[doc = include_str!("layer_handle.doc/HandleState.md")]
#[derive(Default)]
struct HandleState {
    #[doc = include_str!("layer_handle.doc/pending.md")]
    pending: Option<LayerSurfaceConfig>,
    #[doc = include_str!("layer_handle.doc/close_requested.md")]
    close_requested: bool,
    #[doc = include_str!("layer_handle.doc/size.md")]
    size: Size,
    #[doc = include_str!("layer_handle.doc/scale.md")]
    scale: f32,
    #[doc = include_str!("layer_handle.doc/state.md")]
    state: LayerSurfaceState,
}

impl LayerHandle {
    #[doc = include_str!("layer_handle.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(HandleState {
                scale: 1.0,
                ..HandleState::default()
            })),
        }
    }

    #[doc = include_str!("layer_handle.doc/reconfigure.md")]
    pub fn reconfigure(&self, config: LayerSurfaceConfig) {
        self.inner.borrow_mut().pending = Some(config);
    }

    #[doc = include_str!("layer_handle.doc/close.md")]
    pub fn close(&self) {
        self.inner.borrow_mut().close_requested = true;
    }

    #[doc = include_str!("layer_handle.doc/size_2.md")]
    #[must_use]
    pub fn size(&self) -> Size {
        self.inner.borrow().size
    }

    #[doc = include_str!("layer_handle.doc/scale_2.md")]
    #[must_use]
    pub fn scale(&self) -> f32 {
        self.inner.borrow().scale
    }

    #[doc = include_str!("layer_handle.doc/state_2.md")]
    #[must_use]
    pub fn state(&self) -> LayerSurfaceState {
        self.inner.borrow().state
    }

    #[doc = include_str!("layer_handle.doc/take_pending.md")]
    pub(super) fn take_pending(&self) -> Option<LayerSurfaceConfig> {
        self.inner.borrow_mut().pending.take()
    }

    #[doc = include_str!("layer_handle.doc/close_requested_2.md")]
    pub(super) fn close_requested(&self) -> bool {
        self.inner.borrow().close_requested
    }

    #[doc = include_str!("layer_handle.doc/publish.md")]
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
