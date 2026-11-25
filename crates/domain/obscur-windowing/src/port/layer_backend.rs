//! The port a platform implements to put a surface on the desktop.

use obscur_capability::SupportLevel;
use obscur_geometry::{Rect, Size};

use crate::layer::{
    LayerEdge, LayerPopupEvent, LayerPopupFrame, LayerSettings, LayerSurfaceEvent,
    LayerSurfaceState,
};
use crate::request::WindowOperationResult;

use super::{LayerError, LayerFeature};

#[doc = include_str!("../../docs/api/port/layer_backend.doc/LayerBackend.md")]
pub trait LayerBackend {
    #[doc = include_str!("../../docs/api/port/layer_backend.doc/open.md")]
    fn open(&mut self, settings: &LayerSettings) -> Result<(), LayerError>;

    #[doc = include_str!("../../docs/api/port/layer_backend.doc/apply.md")]
    fn apply(&mut self, settings: &LayerSettings) -> WindowOperationResult;

    /// Take the surface down.
    fn close(&mut self) -> WindowOperationResult;

    /// Where the surface is in its lifecycle.
    fn state(&self) -> LayerSurfaceState;

    /// The surface's current size in logical pixels.
    fn size(&self) -> Size;

    /// The display scale the surface is currently rendering at.
    fn scale(&self) -> f32;

    #[doc = include_str!("../../docs/api/port/layer_backend.doc/supports.md")]
    fn supports(&self, feature: LayerFeature) -> SupportLevel;

    #[doc = include_str!("../../docs/api/port/layer_backend.doc/poll_events.md")]
    fn poll_events(&mut self) -> Vec<LayerSurfaceEvent>;

    /// Show or replace the popup anchored to a logical rectangle within the panel.
    fn show_popup(
        &mut self,
        _anchor: Rect,
        _side: LayerEdge,
        _frame: LayerPopupFrame,
    ) -> Result<(), LayerError> {
        Err(LayerError::Unsupported)
    }

    /// Replace the pixels of the configured popup without replacing its surface.
    fn update_popup(&mut self, _frame: LayerPopupFrame) -> Result<(), LayerError> {
        Err(LayerError::Unsupported)
    }

    /// Close the current native popup.
    fn close_popup(&mut self) -> WindowOperationResult {
        WindowOperationResult::Unsupported
    }

    /// Drain native popup events in delivery order.
    fn poll_popup_events(&mut self) -> Vec<LayerPopupEvent> {
        Vec::new()
    }

    #[doc = include_str!("../../docs/api/port/layer_backend.doc/physical_size.md")]
    fn physical_size(&self) -> Size {
        let logical = self.size();
        let scale = self.scale();
        Size::new(
            (logical.width * scale).round(),
            (logical.height * scale).round(),
        )
    }

    /// Whether a frame drawn now would be shown.
    fn is_drawable(&self) -> bool {
        self.state().is_drawable()
    }
}

#[cfg(test)]
mod tests {
    use super::super::NullLayerBackend;
    use super::*;

    #[test]
    fn the_port_is_object_safe() {
        fn assert_obj_safe(_: &dyn LayerBackend) {}
        let backend = NullLayerBackend::new();
        assert_obj_safe(&backend);
    }

    #[test]
    fn physical_size_is_the_logical_size_through_the_scale() {
        struct Scaled(Size, f32);
        impl LayerBackend for Scaled {
            fn open(&mut self, _: &LayerSettings) -> Result<(), LayerError> {
                Ok(())
            }
            fn apply(&mut self, _: &LayerSettings) -> WindowOperationResult {
                WindowOperationResult::native()
            }
            fn close(&mut self) -> WindowOperationResult {
                WindowOperationResult::native()
            }
            fn state(&self) -> LayerSurfaceState {
                LayerSurfaceState::Ready
            }
            fn size(&self) -> Size {
                self.0
            }
            fn scale(&self) -> f32 {
                self.1
            }
            fn supports(&self, _: LayerFeature) -> SupportLevel {
                SupportLevel::Native
            }
            fn poll_events(&mut self) -> Vec<LayerSurfaceEvent> {
                Vec::new()
            }
        }

        let backend = Scaled(Size::new(1000.0, 30.0), 1.5);
        assert_eq!(backend.physical_size(), Size::new(1500.0, 45.0));
        assert!(backend.is_drawable());
    }
}
