use obscur_geometry::Rect;
use obscur_identity::{NodeId, WindowId};

#[doc = include_str!("../../docs/api/capture/capture_request.doc/CaptureSource.md")]
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
