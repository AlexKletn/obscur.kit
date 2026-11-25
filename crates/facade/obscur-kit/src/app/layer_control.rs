//! A handle to a live desktop surface.

use std::cell::RefCell;
use std::rc::Rc;

use super::layer_control_state::LayerControlState;
use super::popup_content::PopupContent;
use super::popup_request::PopupRequest;
use crate::geometry::Size;
use crate::identity::NodeId;
use crate::runtime::UiRuntime;
use crate::shell_layer::event::PopupActivation;
use crate::view::view::View;
use crate::windowing::layer::{
    LayerEdge, LayerPopupEvent, LayerPopupFrame, LayerSettings, LayerSurfaceState,
};
use crate::windowing::port::LayerError;

#[doc = include_str!("../../docs/api/app/layer_control.doc/LayerControl.md")]
#[derive(Clone, Default)]
pub struct LayerControl {
    inner: Rc<RefCell<LayerControlState>>,
}

impl LayerControl {
    /// A handle to a surface that has not been configured yet.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(LayerControlState {
                scale: 1.0,
                ..LayerControlState::default()
            })),
        }
    }

    #[doc = include_str!("../../docs/api/app/layer_control.doc/reconfigure.md")]
    pub fn reconfigure(&self, settings: LayerSettings) {
        self.inner.borrow_mut().pending = Some(settings);
    }

    /// Queue a popup anchored to a laid-out element, with a logical-pixel RGBA frame.
    pub fn show_popup(&self, anchor: NodeId, side: LayerEdge, frame: LayerPopupFrame) {
        self.inner.borrow_mut().popup = Some(PopupRequest::Show(anchor, side, frame));
    }

    /// Open a popup containing a View. The requested dimensions are in logical pixels.
    pub fn show_popup_view(&self, anchor: NodeId, side: LayerEdge, size: (u32, u32), view: View) {
        self.show_popup_view_with(anchor, side, size, move || view.clone(), |_| {});
    }

    /// Open a popup with a reactive view builder and configure its independent runtime before its first frame.
    pub fn show_popup_view_with(
        &self,
        anchor: NodeId,
        side: LayerEdge,
        size: (u32, u32),
        build: impl FnMut() -> View + 'static,
        configure: impl FnOnce(&mut UiRuntime) + 'static,
    ) {
        self.inner.borrow_mut().popup = Some(PopupRequest::View(
            anchor,
            side,
            size,
            PopupContent {
                build: Box::new(build),
                configure: Box::new(configure),
            },
            None,
        ));
    }

    /// Open a grabbed menu while handling a native panel pointer down, touch down, or key press.
    pub fn show_popup_menu(
        &self,
        anchor: NodeId,
        side: LayerEdge,
        size: (u32, u32),
        view: View,
    ) -> Result<(), LayerError> {
        self.show_popup_menu_with(anchor, side, size, move || view.clone(), |_| {})
    }

    /// Open a grabbed reactive menu and configure its runtime while handling the initiating native panel press.
    pub fn show_popup_menu_with(
        &self,
        anchor: NodeId,
        side: LayerEdge,
        size: (u32, u32),
        build: impl FnMut() -> View + 'static,
        configure: impl FnOnce(&mut UiRuntime) + 'static,
    ) -> Result<(), LayerError> {
        let mut state = self.inner.borrow_mut();
        let activation = state.popup_activation.take().ok_or_else(|| {
            LayerError::InvalidSettings(
                "a grabbed menu must open inside a native panel press handler".into(),
            )
        })?;
        state.popup = Some(PopupRequest::View(
            anchor,
            side,
            size,
            PopupContent {
                build: Box::new(build),
                configure: Box::new(configure),
            },
            Some(activation),
        ));
        Ok(())
    }

    pub(super) fn set_popup_activation(&self, activation: Option<PopupActivation>) {
        self.inner.borrow_mut().popup_activation = activation;
    }

    /// Queue a reactive popup frame after application state changes.
    pub fn request_popup_frame(&self) {
        self.inner.borrow_mut().popup_frame_requested = true;
    }

    pub(super) fn take_popup_frame_request(&self) -> bool {
        core::mem::take(&mut self.inner.borrow_mut().popup_frame_requested)
    }

    /// Queue closure of the native popup.
    pub fn close_popup(&self) {
        self.inner.borrow_mut().popup = Some(PopupRequest::Close);
    }

    /// Register a callback for popup configuration, pointer input, errors and closure.
    pub fn on_popup_event(&self, handler: impl FnMut(&Self, LayerPopupEvent) + 'static) {
        self.inner.borrow_mut().popup_handler = Some(Box::new(handler));
    }

    pub(super) fn has_pending_popup(&self) -> bool {
        let state = self.inner.borrow();
        state.popup.is_some() || state.popup_frame_requested
    }

    pub(super) fn take_popup(&self) -> Option<PopupRequest> {
        self.inner.borrow_mut().popup.take()
    }

    pub(super) fn dispatch_popup_event(&self, event: LayerPopupEvent) {
        let handler = self.inner.borrow_mut().popup_handler.take();
        if let Some(mut handler) = handler {
            handler(self, event);
            let mut state = self.inner.borrow_mut();
            if state.popup_handler.is_none() {
                state.popup_handler = Some(handler);
            }
        }
    }

    #[doc = include_str!("../../docs/api/app/layer_control.doc/close.md")]
    pub fn close(&self) {
        self.inner.borrow_mut().close_requested = true;
    }

    /// The surface's logical size, as the platform last settled it.
    #[must_use]
    pub fn size(&self) -> Size {
        self.inner.borrow().size
    }

    /// The display scale the surface is currently rendering at.
    #[must_use]
    pub fn scale(&self) -> f32 {
        self.inner.borrow().scale
    }

    /// Where the surface is in its lifecycle.
    #[must_use]
    pub fn state(&self) -> LayerSurfaceState {
        self.inner.borrow().state
    }

    /// Take the pending reconfiguration, if a handler left one.
    pub(super) fn take_pending(&self) -> Option<LayerSettings> {
        self.inner.borrow_mut().pending.take()
    }

    #[doc = include_str!("../../docs/api/app/layer_control.doc/has_pending.md")]
    pub(super) fn has_pending(&self) -> bool {
        self.inner.borrow().pending.is_some()
    }

    /// Whether a handler has asked for the surface to be taken down.
    pub(super) fn close_requested(&self) -> bool {
        self.inner.borrow().close_requested
    }

    #[doc = include_str!("../../docs/api/app/layer_control.doc/publish.md")]
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
    #[test]
    fn a_popup_event_handler_can_queue_the_next_popup_operation() {
        let control = LayerControl::new();
        control.on_popup_event(|control, event| {
            if matches!(event, LayerPopupEvent::Closed) {
                control.close_popup();
            }
        });
        control.dispatch_popup_event(LayerPopupEvent::Closed);
        assert!(matches!(control.take_popup(), Some(PopupRequest::Close)));
    }
    #[test]
    fn a_grabbed_menu_cannot_open_without_native_activation() {
        let control = LayerControl::new();
        let result = control.show_popup_menu_with(
            NodeId::from_raw(42),
            LayerEdge::Bottom,
            (100, 100),
            || panic!("must not build without activation"),
            |_| {},
        );
        assert!(matches!(result, Err(LayerError::InvalidSettings(_))));
        assert!(!control.has_pending_popup());
    }
}
