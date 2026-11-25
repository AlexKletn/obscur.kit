//! `ForeignSurfaceKind`: the kind of external content a foreign surface hosts (§5.5).

#[doc = include_str!("../../docs/api/surface/foreign_surface_kind.doc/ForeignSurfaceKind.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ForeignSurfaceKind {
    /// A video stream or player surface.
    Video,
    /// An embedded web view.
    WebView,
    /// A live camera feed.
    Camera,
    /// A native platform control (e.g. a native text field or map view).
    NativeControl,
    /// An externally-managed GPU surface.
    ExternalGpu,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variants_are_distinct() {
        assert_ne!(ForeignSurfaceKind::Video, ForeignSurfaceKind::WebView);
        assert_ne!(
            ForeignSurfaceKind::Camera,
            ForeignSurfaceKind::NativeControl
        );
        assert_ne!(ForeignSurfaceKind::ExternalGpu, ForeignSurfaceKind::Video);
    }

    #[test]
    fn is_copy_and_eq() {
        let kind = ForeignSurfaceKind::Camera;
        let copy = kind;
        assert_eq!(kind, copy);
    }
}
