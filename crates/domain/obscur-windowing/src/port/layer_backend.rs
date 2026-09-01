#![doc = include_str!("layer_backend.doc/_module.md")]

use obscur_capability::SupportLevel;
use obscur_geometry::Size;

use crate::layer::{LayerSettings, LayerSurfaceEvent, LayerSurfaceState};
use crate::request::WindowOperationResult;

#[doc = include_str!("layer_backend.doc/LayerFeature.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LayerFeature {
    #[doc = include_str!("layer_backend.doc/Bands.md")]
    Bands,
    #[doc = include_str!("layer_backend.doc/Reservation.md")]
    Reservation,
    #[doc = include_str!("layer_backend.doc/FreeAnchor.md")]
    FreeAnchor,
    #[doc = include_str!("layer_backend.doc/ExclusiveKeyboard.md")]
    ExclusiveKeyboard,
    #[doc = include_str!("layer_backend.doc/DisplaySelection.md")]
    DisplaySelection,
    #[doc = include_str!("layer_backend.doc/NegativeMargin.md")]
    NegativeMargin,
    #[doc = include_str!("layer_backend.doc/FractionalScale.md")]
    FractionalScale,
    #[doc = include_str!("layer_backend.doc/InputTransparency.md")]
    InputTransparency,
}

impl LayerFeature {
    #[doc = include_str!("layer_backend.doc/ALL.md")]
    pub const ALL: [Self; 8] = [
        Self::Bands,
        Self::Reservation,
        Self::FreeAnchor,
        Self::ExclusiveKeyboard,
        Self::DisplaySelection,
        Self::NegativeMargin,
        Self::FractionalScale,
        Self::InputTransparency,
    ];
}

#[doc = include_str!("layer_backend.doc/LayerError.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayerError {
    #[doc = include_str!("layer_backend.doc/Unsupported.md")]
    Unsupported,
    #[doc = include_str!("layer_backend.doc/InvalidSettings.md")]
    InvalidSettings(Box<str>),
    #[doc = include_str!("layer_backend.doc/BackendFailure.md")]
    BackendFailure(Box<str>),
}

impl std::fmt::Display for LayerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unsupported => f.write_str("this platform has no desktop layer surfaces"),
            Self::InvalidSettings(reason) => write!(f, "invalid layer settings: {reason}"),
            Self::BackendFailure(reason) => write!(f, "layer backend failure: {reason}"),
        }
    }
}

impl std::error::Error for LayerError {}

#[doc = include_str!("layer_backend.doc/LayerBackend.md")]
pub trait LayerBackend {
    #[doc = include_str!("layer_backend.doc/open.md")]
    fn open(&mut self, settings: &LayerSettings) -> Result<(), LayerError>;

    #[doc = include_str!("layer_backend.doc/apply.md")]
    fn apply(&mut self, settings: &LayerSettings) -> WindowOperationResult;

    #[doc = include_str!("layer_backend.doc/close.md")]
    fn close(&mut self) -> WindowOperationResult;

    #[doc = include_str!("layer_backend.doc/state.md")]
    fn state(&self) -> LayerSurfaceState;

    #[doc = include_str!("layer_backend.doc/size.md")]
    fn size(&self) -> Size;

    #[doc = include_str!("layer_backend.doc/scale.md")]
    fn scale(&self) -> f32;

    #[doc = include_str!("layer_backend.doc/supports.md")]
    fn supports(&self, feature: LayerFeature) -> SupportLevel;

    #[doc = include_str!("layer_backend.doc/poll_events.md")]
    fn poll_events(&mut self) -> Vec<LayerSurfaceEvent>;

    #[doc = include_str!("layer_backend.doc/physical_size.md")]
    fn physical_size(&self) -> Size {
        let logical = self.size();
        let scale = self.scale();
        Size::new(
            (logical.width * scale).round(),
            (logical.height * scale).round(),
        )
    }

    #[doc = include_str!("layer_backend.doc/is_drawable.md")]
    fn is_drawable(&self) -> bool {
        self.state().is_drawable()
    }
}

#[doc = include_str!("layer_backend.doc/NullLayerBackend.md")]
#[derive(Debug, Default)]
pub struct NullLayerBackend;

impl NullLayerBackend {
    #[doc = include_str!("layer_backend.doc/new.md")]
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
    fn the_port_is_object_safe() {
        fn assert_obj_safe(_: &dyn LayerBackend) {}
        let backend = NullLayerBackend::new();
        assert_obj_safe(&backend);
    }

    #[test]
    fn a_platform_without_desktop_surfaces_says_so_rather_than_failing_oddly() {
        let mut backend = NullLayerBackend::new();
        assert_eq!(
            backend.open(&LayerSettings::top_bar(30)),
            Err(LayerError::Unsupported)
        );
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
    fn the_feature_roster_lists_each_feature_once() {
        let mut seen = LayerFeature::ALL.to_vec();
        seen.sort_by_key(|feature| format!("{feature:?}"));
        seen.dedup();
        assert_eq!(seen.len(), LayerFeature::ALL.len());
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

    #[test]
    fn an_unsupported_operation_is_distinguishable_from_a_failure() {
        let mut backend = NullLayerBackend::new();
        let result = backend.close();
        assert_eq!(result, WindowOperationResult::Unsupported);
        assert_ne!(result, WindowOperationResult::failed("closed too hard"));
    }

    #[test]
    fn the_error_text_names_the_platform_limit_rather_than_a_protocol() {
        let text = LayerError::Unsupported.to_string();
        assert!(text.contains("desktop layer surfaces"), "{text}");
        assert!(!text.to_lowercase().contains("wayland"), "{text}");
    }
}
