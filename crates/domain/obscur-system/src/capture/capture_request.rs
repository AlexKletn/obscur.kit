#![doc = include_str!("capture_request.doc/_module.md")]

use obscur_geometry::Rect;
use obscur_identity::{NodeId, WindowId};

#[doc = include_str!("capture_request.doc/CaptureSource.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum CaptureSource {
    #[doc = include_str!("capture_request.doc/Window.md")]
    Window(WindowId),
    #[doc = include_str!("capture_request.doc/Surface.md")]
    Surface,
    #[doc = include_str!("capture_request.doc/Node.md")]
    Node(NodeId),
    #[doc = include_str!("capture_request.doc/Region.md")]
    Region(Rect),
}

#[doc = include_str!("capture_request.doc/CaptureRequest.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct CaptureRequest {
    #[doc = include_str!("capture_request.doc/source.md")]
    source: CaptureSource,
    #[doc = include_str!("capture_request.doc/protected_regions.md")]
    protected_regions: Vec<Rect>,
}

impl CaptureRequest {
    #[doc = include_str!("capture_request.doc/window.md")]
    #[must_use]
    pub fn window(id: WindowId) -> Self {
        Self::with_source(CaptureSource::Window(id))
    }

    #[doc = include_str!("capture_request.doc/surface.md")]
    #[must_use]
    pub fn surface() -> Self {
        Self::with_source(CaptureSource::Surface)
    }

    #[doc = include_str!("capture_request.doc/node.md")]
    #[must_use]
    pub fn node(id: NodeId) -> Self {
        Self::with_source(CaptureSource::Node(id))
    }

    #[doc = include_str!("capture_request.doc/region.md")]
    #[must_use]
    pub fn region(region: Rect) -> Self {
        Self::with_source(CaptureSource::Region(region))
    }

    #[doc = include_str!("capture_request.doc/protecting.md")]
    #[must_use]
    pub fn protecting(mut self, region: Rect) -> Self {
        self.protected_regions.push(region);
        self
    }

    #[doc = include_str!("capture_request.doc/source_2.md")]
    #[must_use]
    pub const fn source(&self) -> &CaptureSource {
        &self.source
    }

    #[doc = include_str!("capture_request.doc/protected_regions_2.md")]
    #[must_use]
    pub fn protected_regions(&self) -> &[Rect] {
        &self.protected_regions
    }

    #[doc = include_str!("capture_request.doc/has_protected_regions.md")]
    #[must_use]
    pub fn has_protected_regions(&self) -> bool {
        !self.protected_regions.is_empty()
    }

    #[doc = include_str!("capture_request.doc/with_source.md")]
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
