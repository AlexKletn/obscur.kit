//! `ForeignSurfaceKind`: the kind of external content a foreign surface hosts (§5.5).

/// The kind of externally-produced content hosted by a foreign surface
/// (Obscur Kit 2 specs §5.5).
///
/// Foreign surfaces integrate content that is produced outside the Paint model,
/// either texture-backed or as a native child surface/window.
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
        assert_ne!(ForeignSurfaceKind::Camera, ForeignSurfaceKind::NativeControl);
        assert_ne!(ForeignSurfaceKind::ExternalGpu, ForeignSurfaceKind::Video);
    }

    #[test]
    fn is_copy_and_eq() {
        let kind = ForeignSurfaceKind::Camera;
        let copy = kind;
        assert_eq!(kind, copy);
    }
}
