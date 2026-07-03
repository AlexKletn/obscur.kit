//! The replaceable port for screen / surface / node capture (spec §6.17).
//!
//! [`CaptureProvider`] is the hexagonal port through which the domain asks the
//! host to satisfy a [`CaptureRequest`] and receives a
//! [`CaptureResult`]. Permission and capability are honest: a provider that cannot
//! capture a source answers [`CaptureResult::Unsupported`] (or
//! [`CaptureResult::Denied`]) rather than panicking, and can be pre-checked with
//! [`can_capture`](CaptureProvider::can_capture). The colocated
//! [`NullCaptureProvider`] and [`StubCaptureProvider`] are reference
//! implementations of that contract.

use obscur_geometry::Size;

use crate::capture::{CaptureRequest, CaptureResult, CaptureSource, CapturedImage};

/// The replaceable port for capturing windows, surfaces, nodes, and regions
/// (spec §6.17).
///
/// An adapter implements this trait once per platform and the domain holds it as
/// `Box<dyn CaptureProvider>`. The trait is deliberately **object-safe** — no
/// generic or `Self`-returning methods — so providers can be stored
/// polymorphically.
///
/// Contract: a provider reports honest, typed outcomes and **never panics** on a
/// source it cannot capture — it answers [`CaptureResult::Unsupported`] (or
/// [`CaptureResult::Denied`]) instead (spec §6.17).
pub trait CaptureProvider {
    /// Satisfies `request` and returns the honest outcome (spec §6.17).
    ///
    /// A provider that cannot capture the request's source returns
    /// [`CaptureResult::Unsupported`] rather than panicking.
    fn capture(&mut self, request: CaptureRequest) -> CaptureResult;

    /// Returns whether this provider can capture `source` (spec §6.17).
    #[must_use]
    fn can_capture(&self, source: &CaptureSource) -> bool;
}

/// A reference, no-op [`CaptureProvider`] that captures nothing (spec §6.17).
///
/// `NullCaptureProvider` honestly reports that it can capture no source:
/// [`capture`](Self::capture) returns [`CaptureResult::Unsupported`] and
/// [`can_capture`](Self::can_capture) returns `false`. Its answers are correct
/// behavior — a documented reference for the honest-capability contract — not a
/// stub waiting to be filled in.
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

/// A deterministic, recording [`CaptureProvider`] for tests (spec §6.17).
///
/// `StubCaptureProvider` accepts every source: [`can_capture`](Self::can_capture)
/// is always `true`, and [`capture`](Self::capture) records the request in
/// [`last`](Self::last) and returns a fixed 1x1 `rgba8`
/// [`CaptureResult::Captured`]. Tests inspect the recorded request to assert what
/// the domain attempted, without touching a real capture pipeline.
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
