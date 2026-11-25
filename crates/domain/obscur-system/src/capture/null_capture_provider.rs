use super::{CaptureProvider, CaptureRequest, CaptureResult, CaptureSource};

#[doc = include_str!("../../docs/api/capture/capture_provider.doc/NullCaptureProvider.md")]
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
