//! What the app asks the host to capture (spec §6.17 — capture and privacy).

use obscur_geometry::Rect;
use obscur_identity::{NodeId, WindowId};

/// What a [`CaptureRequest`] targets (spec §6.17).
///
/// The variants mirror the spec's capture surfaces: a whole window, the app's
/// drawing surface, a single scene node, or an arbitrary image region.
#[derive(Debug, Clone, PartialEq)]
pub enum CaptureSource {
    /// Capture an entire window by identity (spec §6.17 — capture window).
    Window(WindowId),
    /// Capture the app's drawing surface (spec §6.17 — capture surface).
    Surface,
    /// Capture a single scene node by identity (spec §6.17 — capture node).
    Node(NodeId),
    /// Capture an arbitrary rectangular image region (spec §6.17 — capture
    /// image).
    Region(Rect),
}

/// A request describing what to capture and which regions to protect
/// (spec §6.17).
///
/// A request is a pure value: a [`source`](Self::source) naming the target and a
/// set of [`protected_regions`](Self::protected_regions) the host must
/// exclude/redact because they are privacy-sensitive or capture-protected
/// (spec §6.17). Whether the capture succeeds is the host's decision, reported via
/// [`CaptureResult`](crate::capture::CaptureResult).
#[derive(Debug, Clone, PartialEq)]
pub struct CaptureRequest {
    /// What to capture.
    source: CaptureSource,
    /// Regions to exclude or redact as privacy-sensitive / capture-protected.
    protected_regions: Vec<Rect>,
}

impl CaptureRequest {
    /// Creates a request capturing the window `id` with no protected regions
    /// (spec §6.17).
    #[must_use]
    pub fn window(id: WindowId) -> Self {
        Self::with_source(CaptureSource::Window(id))
    }

    /// Creates a request capturing the app's drawing surface (spec §6.17).
    #[must_use]
    pub fn surface() -> Self {
        Self::with_source(CaptureSource::Surface)
    }

    /// Creates a request capturing the scene node `id` (spec §6.17).
    #[must_use]
    pub fn node(id: NodeId) -> Self {
        Self::with_source(CaptureSource::Node(id))
    }

    /// Creates a request capturing the image `region` (spec §6.17).
    #[must_use]
    pub fn region(region: Rect) -> Self {
        Self::with_source(CaptureSource::Region(region))
    }

    /// Returns this request with `region` added to its protected regions
    /// (spec §6.17).
    #[must_use]
    pub fn protecting(mut self, region: Rect) -> Self {
        self.protected_regions.push(region);
        self
    }

    /// Returns what this request captures (spec §6.17).
    #[must_use]
    pub const fn source(&self) -> &CaptureSource {
        &self.source
    }

    /// Returns the privacy-sensitive / capture-protected regions (spec §6.17).
    #[must_use]
    pub fn protected_regions(&self) -> &[Rect] {
        &self.protected_regions
    }

    /// Returns whether this request carries any protected regions (spec §6.17).
    #[must_use]
    pub fn has_protected_regions(&self) -> bool {
        !self.protected_regions.is_empty()
    }

    /// Builds a request from `source` with no protected regions.
    #[must_use]
    fn with_source(source: CaptureSource) -> Self {
        Self {
            source,
            protected_regions: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_geometry::{Point, Size};

    fn rect(x: f32, y: f32) -> Rect {
        Rect::new(Point::new(x, y), Size::new(10.0, 10.0))
    }

    #[test]
    fn constructors_set_source() {
        assert_eq!(
            CaptureRequest::window(WindowId::from_raw(1)).source(),
            &CaptureSource::Window(WindowId::from_raw(1))
        );
        assert_eq!(CaptureRequest::surface().source(), &CaptureSource::Surface);
        assert_eq!(
            CaptureRequest::node(NodeId::from_raw(2)).source(),
            &CaptureSource::Node(NodeId::from_raw(2))
        );
        let r = rect(0.0, 0.0);
        assert_eq!(
            CaptureRequest::region(r).source(),
            &CaptureSource::Region(r)
        );
    }

    #[test]
    fn new_request_has_no_protected_regions() {
        let req = CaptureRequest::surface();
        assert!(!req.has_protected_regions());
        assert!(req.protected_regions().is_empty());
    }

    #[test]
    fn protecting_appends_regions_in_order() {
        let req = CaptureRequest::surface()
            .protecting(rect(0.0, 0.0))
            .protecting(rect(5.0, 5.0));
        assert!(req.has_protected_regions());
        assert_eq!(req.protected_regions().len(), 2);
        assert_eq!(req.protected_regions()[0], rect(0.0, 0.0));
        assert_eq!(req.protected_regions()[1], rect(5.0, 5.0));
    }
}
