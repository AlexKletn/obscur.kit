use obscur_capability::SupportLevel;
use obscur_geometry::Size;

use crate::layer::{LayerSettings, LayerSurfaceEvent, LayerSurfaceState};
use crate::request::WindowOperationResult;

use super::{LayerBackend, LayerError, LayerFeature};

#[doc = include_str!("../../docs/api/port/layer_backend.doc/NullLayerBackend.md")]
#[derive(Debug, Default)]
pub struct NullLayerBackend;

impl NullLayerBackend {
    /// A backend that supports nothing.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl LayerBackend for NullLayerBackend {
    fn open(&mut self, _settings: &LayerSettings) -> Result<(), LayerError> {
        Err(LayerError::Unsupported)
    }

    fn apply(&mut self, _settings: &LayerSettings) -> WindowOperationResult {
        WindowOperationResult::Unsupported
    }

    fn close(&mut self) -> WindowOperationResult {
        WindowOperationResult::Unsupported
    }

    fn state(&self) -> LayerSurfaceState {
        LayerSurfaceState::Closed
    }

    fn size(&self) -> Size {
        Size::new(0.0, 0.0)
    }

    fn scale(&self) -> f32 {
        1.0
    }

    fn supports(&self, _feature: LayerFeature) -> SupportLevel {
        SupportLevel::Unavailable
    }

    fn poll_events(&mut self) -> Vec<LayerSurfaceEvent> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_platform_without_desktop_surfaces_says_so_rather_than_failing_oddly() {
        let mut backend = NullLayerBackend::new();
        assert!(matches!(
            backend.open(&LayerSettings::top_bar(30)),
            Err(LayerError::Unsupported)
        ));
        assert_eq!(backend.state(), LayerSurfaceState::Closed);
        assert!(!backend.is_drawable());
    }

    #[test]
    fn every_feature_is_answered_even_when_the_answer_is_no() {
        let backend = NullLayerBackend::new();
        for feature in LayerFeature::ALL {
            assert_eq!(
                backend.supports(feature),
                SupportLevel::Unavailable,
                "{feature:?} went unanswered"
            );
        }
    }

    #[test]
    fn an_unsupported_operation_is_distinguishable_from_a_failure() {
        let mut backend = NullLayerBackend::new();
        let result = backend.close();
        assert_eq!(result, WindowOperationResult::Unsupported);
        assert_ne!(result, WindowOperationResult::failed("closed too hard"));
    }
}
