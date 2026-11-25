//! A generic typed resource handle (spec §5.15).

use obscur_identity::ResourceId;

use crate::handle::{FontHandle, ImageHandle, ResourceKind, ShaderHandle, StylesheetHandle};

#[doc = include_str!("../../docs/api/handle/resource_handle.doc/ResourceHandle.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceHandle {
    id: ResourceId,
    kind: ResourceKind,
}

impl ResourceHandle {
    /// Creates a handle for the given resource id and kind.
    #[must_use]
    pub const fn new(id: ResourceId, kind: ResourceKind) -> Self {
        Self { id, kind }
    }

    /// Returns the resource identity this handle refers to.
    #[must_use]
    pub const fn id(self) -> ResourceId {
        self.id
    }

    /// Returns the kind of resource this handle refers to.
    #[must_use]
    pub const fn kind(self) -> ResourceKind {
        self.kind
    }

    /// Downcasts to an [`ImageHandle`], or `None` if this is not an image.
    #[must_use]
    pub const fn as_image(self) -> Option<ImageHandle> {
        ImageHandle::from_handle(self)
    }

    /// Downcasts to a [`FontHandle`], or `None` if this is not a font.
    #[must_use]
    pub const fn as_font(self) -> Option<FontHandle> {
        FontHandle::from_handle(self)
    }

    /// Downcasts to a [`StylesheetHandle`], or `None` if this is not a
    /// stylesheet.
    #[must_use]
    pub const fn as_stylesheet(self) -> Option<StylesheetHandle> {
        StylesheetHandle::from_handle(self)
    }

    /// Downcasts to a [`ShaderHandle`], or `None` if this is not a shader.
    #[must_use]
    pub const fn as_shader(self) -> Option<ShaderHandle> {
        ShaderHandle::from_handle(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(raw: u64) -> ResourceId {
        ResourceId::from_raw(raw)
    }

    #[test]
    fn new_exposes_id_and_kind() {
        let h = ResourceHandle::new(id(7), ResourceKind::Image);
        assert_eq!(h.id(), id(7));
        assert_eq!(h.kind(), ResourceKind::Image);
    }

    #[test]
    fn as_image_matches_only_images() {
        let img = ResourceHandle::new(id(1), ResourceKind::Image);
        let font = ResourceHandle::new(id(1), ResourceKind::Font);
        assert_eq!(img.as_image(), Some(ImageHandle::new(id(1))));
        assert_eq!(font.as_image(), None);
    }

    #[test]
    fn as_font_matches_only_fonts() {
        let font = ResourceHandle::new(id(2), ResourceKind::Font);
        let img = ResourceHandle::new(id(2), ResourceKind::Image);
        assert_eq!(font.as_font(), Some(FontHandle::new(id(2))));
        assert_eq!(img.as_font(), None);
    }

    #[test]
    fn as_stylesheet_matches_only_stylesheets() {
        let css = ResourceHandle::new(id(3), ResourceKind::Stylesheet);
        let shader = ResourceHandle::new(id(3), ResourceKind::Shader);
        assert_eq!(css.as_stylesheet(), Some(StylesheetHandle::new(id(3))));
        assert_eq!(shader.as_stylesheet(), None);
    }

    #[test]
    fn as_shader_matches_only_shaders() {
        let shader = ResourceHandle::new(id(4), ResourceKind::Shader);
        let css = ResourceHandle::new(id(4), ResourceKind::Stylesheet);
        assert_eq!(shader.as_shader(), Some(ShaderHandle::new(id(4))));
        assert_eq!(css.as_shader(), None);
    }

    #[test]
    fn kindless_variants_downcast_to_none() {
        let svg = ResourceHandle::new(id(5), ResourceKind::Svg);
        assert_eq!(svg.as_image(), None);
        assert_eq!(svg.as_font(), None);
        assert_eq!(svg.as_stylesheet(), None);
        assert_eq!(svg.as_shader(), None);
    }

    #[test]
    fn equality_and_hash_track_id_and_kind() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let a = ResourceHandle::new(id(9), ResourceKind::Atlas);
        let b = ResourceHandle::new(id(9), ResourceKind::Atlas);
        let c = ResourceHandle::new(id(9), ResourceKind::EmbeddedAsset);
        assert_eq!(a, b);
        assert_ne!(a, c);

        let hash = |h: ResourceHandle| {
            let mut s = DefaultHasher::new();
            h.hash(&mut s);
            s.finish()
        };
        assert_eq!(hash(a), hash(b));
    }
}
