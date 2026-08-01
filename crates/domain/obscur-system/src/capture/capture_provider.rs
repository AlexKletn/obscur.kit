#![doc = include_str!("capture_provider.doc/_module.md")]

use obscur_geometry::Size;

use crate::capture::{CaptureRequest, CaptureResult, CaptureSource, CapturedImage};

#[doc = include_str!("capture_provider.doc/CaptureProvider.md")]
pub trait CaptureProvider {
    #[doc = include_str!("capture_provider.doc/capture.md")]
    fn capture(&mut self, request: CaptureRequest) -> CaptureResult;

    #[doc = include_str!("capture_provider.doc/can_capture.md")]
    #[must_use]
    fn can_capture(&self, source: &CaptureSource) -> bool;
}

#[doc = include_str!("capture_provider.doc/NullCaptureProvider.md")]
#[derive(Debug, Default, Clone)]
pub struct NullCaptureProvider;

impl CaptureProvider for NullCaptureProvider {
    fn capture(&mut self, _request: CaptureRequest) -> CaptureResult {
        // Honest: a null provider can capture no source.
        CaptureResult::Unsupported
    }

    fn can_capture(&self, _source: &CaptureSource) -> bool {
        false
    }
}

#[doc = include_str!("capture_provider.doc/StubCaptureProvider.md")]
#[derive(Debug, Default, Clone)]
pub struct StubCaptureProvider {
    last: Option<CaptureRequest>,
}

impl StubCaptureProvider {
    #[doc = include_str!("capture_provider.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[doc = include_str!("capture_provider.doc/last.md")]
    #[must_use]
    pub const fn last(&self) -> Option<&CaptureRequest> {
        self.last.as_ref()
    }
}

impl CaptureProvider for StubCaptureProvider {
    fn capture(&mut self, request: CaptureRequest) -> CaptureResult {
        self.last = Some(request);
        CaptureResult::Captured(CapturedImage::new(Size::new(1.0, 1.0), "rgba8"))
    }

    fn can_capture(&self, _source: &CaptureSource) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_identity::WindowId;

    #[test]
    fn capture_provider_is_object_safe() {
        let _: Box<dyn CaptureProvider> = Box::new(NullCaptureProvider);
    }

    #[test]
    fn null_provider_captures_nothing() {
        let mut provider = NullCaptureProvider;
        let source = CaptureSource::Surface;
        assert!(!provider.can_capture(&source));
        assert_eq!(
            provider.capture(CaptureRequest::surface()),
            CaptureResult::Unsupported
        );
    }

    #[test]
    fn stub_provider_captures_and_records_last() {
        let mut provider = StubCaptureProvider::new();
        assert!(provider.last().is_none());
        assert!(provider.can_capture(&CaptureSource::Surface));

        let request = CaptureRequest::window(WindowId::from_raw(1));
        let result = provider.capture(request.clone());
        assert_eq!(
            result,
            CaptureResult::Captured(CapturedImage::new(Size::new(1.0, 1.0), "rgba8"))
        );
        assert_eq!(provider.last(), Some(&request));

        // The latest request replaces the previous one.
        let second = CaptureRequest::surface();
        provider.capture(second.clone());
        assert_eq!(provider.last(), Some(&second));
    }
}
