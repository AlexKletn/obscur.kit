use super::{CaptureProvider, CaptureRequest, CaptureResult, CaptureSource, CapturedImage};
use obscur_geometry::Size;

#[doc = include_str!("../../docs/api/capture/capture_provider.doc/StubCaptureProvider.md")]
#[derive(Debug, Default, Clone)]
pub struct StubCaptureProvider {
    last: Option<CaptureRequest>,
}

impl StubCaptureProvider {
    /// Creates an empty stub provider that has captured nothing yet (spec §6.17).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the most recent request passed to [`capture`](Self::capture), or
    /// `None` if none has been made (spec §6.17).
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
