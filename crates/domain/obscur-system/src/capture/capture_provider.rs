#![doc = include_str!("../../docs/api/capture/capture_provider.doc/_module.md")]

use super::{CaptureRequest, CaptureResult, CaptureSource};

#[doc = include_str!("../../docs/api/capture/capture_provider.doc/CaptureProvider.md")]
pub trait CaptureProvider {
    #[doc = include_str!("../../docs/api/capture/capture_provider.doc/capture.md")]
    fn capture(&mut self, request: CaptureRequest) -> CaptureResult;

    /// Returns whether this provider can capture `source` (spec §6.17).
    #[must_use]
    fn can_capture(&self, source: &CaptureSource) -> bool;

    /// Whether this provider can prevent protected regions from entering the result.
    #[must_use]
    fn protects_regions(&self, _source: &CaptureSource) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::super::NullCaptureProvider;
    use super::*;

    #[test]
    fn capture_provider_is_object_safe() {
        let _: Box<dyn CaptureProvider> = Box::new(NullCaptureProvider);
    }
}
