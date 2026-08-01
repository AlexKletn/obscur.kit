#![doc = include_str!("foreign_surface_kind.doc/_module.md")]

#[doc = include_str!("foreign_surface_kind.doc/ForeignSurfaceKind.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ForeignSurfaceKind {
    #[doc = include_str!("foreign_surface_kind.doc/Video.md")]
    Video,
    #[doc = include_str!("foreign_surface_kind.doc/WebView.md")]
    WebView,
    #[doc = include_str!("foreign_surface_kind.doc/Camera.md")]
    Camera,
    #[doc = include_str!("foreign_surface_kind.doc/NativeControl.md")]
    NativeControl,
    #[doc = include_str!("foreign_surface_kind.doc/ExternalGpu.md")]
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
