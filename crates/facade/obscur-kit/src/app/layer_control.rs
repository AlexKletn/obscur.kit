#![doc = include_str!("layer_control.doc/_module.md")]

use std::cell::RefCell;
use std::rc::Rc;

use crate::geometry::Size;
use crate::windowing::layer::{LayerSettings, LayerSurfaceState};

#[doc = include_str!("layer_control.doc/LayerControl.md")]
#[derive(Clone, Default)]
pub struct LayerControl {
    inner: Rc<RefCell<ControlState>>,
}

#[doc = include_str!("layer_control.doc/ControlState.md")]
#[derive(Default)]
struct ControlState {
    pending: Option<LayerSettings>,
    close_requested: bool,
    size: Size,
    scale: f32,
    state: LayerSurfaceState,
}

impl LayerControl {
    #[doc = include_str!("layer_control.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(ControlState {
                scale: 1.0,
                ..ControlState::default()
            })),
        }
    }

    #[doc = include_str!("layer_control.doc/reconfigure.md")]
    pub fn reconfigure(&self, settings: LayerSettings) {
        self.inner.borrow_mut().pending = Some(settings);
    }

    #[doc = include_str!("layer_control.doc/close.md")]
    pub fn close(&self) {
        self.inner.borrow_mut().close_requested = true;
    }

    #[doc = include_str!("layer_control.doc/size.md")]
    #[must_use]
    pub fn size(&self) -> Size {
        self.inner.borrow().size
    }

    #[doc = include_str!("layer_control.doc/scale.md")]
    #[must_use]
    pub fn scale(&self) -> f32 {
        self.inner.borrow().scale
    }

    #[doc = include_str!("layer_control.doc/state.md")]
    #[must_use]
    pub fn state(&self) -> LayerSurfaceState {
        self.inner.borrow().state
    }

    #[doc = include_str!("layer_control.doc/take_pending.md")]
    pub(super) fn take_pending(&self) -> Option<LayerSettings> {
        self.inner.borrow_mut().pending.take()
    }

    #[doc = include_str!("layer_control.doc/close_requested.md")]
    pub(super) fn close_requested(&self) -> bool {
        self.inner.borrow().close_requested
    }

    #[doc = include_str!("layer_control.doc/publish.md")]
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

    #[test]
    fn a_fresh_control_reports_a_surface_that_has_not_been_configured() {
        let control = LayerControl::new();
        assert_eq!(control.state(), LayerSurfaceState::Configuring);
        assert_eq!(control.scale(), 1.0);
        assert!(!control.close_requested());
    }

    #[test]
    fn the_two_handles_are_one_surface() {
        // Cloned into a handler and kept by the caller: both must see the loop's
        // published geometry, or an application would act on a stale size.
        let control = LayerControl::new();
        let clone = control.clone();
        control.publish(Size::new(1920.0, 36.0), 2.0, LayerSurfaceState::Ready);
        assert_eq!(clone.size(), Size::new(1920.0, 36.0));
        assert_eq!(clone.scale(), 2.0);
        assert!(clone.state().is_drawable());
    }

    #[test]
    fn a_reconfiguration_is_taken_once() {
        let control = LayerControl::new();
        control.reconfigure(LayerSettings::top_bar(48));
        assert_eq!(
            control.take_pending(),
            Some(LayerSettings::top_bar(48)),
            "the loop takes the request"
        );
        assert_eq!(
            control.take_pending(),
            None,
            "and does not apply it a second time on the next turn"
        );
    }

    #[test]
    fn the_last_reconfiguration_before_the_loop_looks_is_the_one_applied() {
        let control = LayerControl::new();
        control.reconfigure(LayerSettings::top_bar(30));
        control.reconfigure(LayerSettings::top_bar(40));
        assert_eq!(control.take_pending(), Some(LayerSettings::top_bar(40)));
    }

    #[test]
    fn asking_to_close_is_sticky() {
        // The loop may not look until the end of its turn, and a request that
        // evaporated in between would leave the surface up.
        let control = LayerControl::new();
        control.close();
        assert!(control.close_requested());
        assert!(control.close_requested());
    }
}
