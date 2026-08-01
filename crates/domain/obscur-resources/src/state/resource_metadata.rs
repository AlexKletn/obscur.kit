#![doc = include_str!("resource_metadata.doc/_module.md")]

use obscur_geometry::Size;

#[doc = include_str!("resource_metadata.doc/ResourceMetadata.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct ResourceMetadata {
    #[doc = include_str!("resource_metadata.doc/byte_size.md")]
    pub byte_size: Option<u64>,
    #[doc = include_str!("resource_metadata.doc/mime_type.md")]
    pub mime_type: Option<Box<str>>,
    #[doc = include_str!("resource_metadata.doc/intrinsic_size.md")]
    pub intrinsic_size: Option<Size>,
    #[doc = include_str!("resource_metadata.doc/scale.md")]
    pub scale: f32,
}

impl ResourceMetadata {
    #[doc = include_str!("resource_metadata.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self {
            byte_size: None,
            mime_type: None,
            intrinsic_size: None,
            scale: 1.0,
        }
    }

    #[doc = include_str!("resource_metadata.doc/with_byte_size.md")]
    #[must_use]
    pub fn with_byte_size(mut self, byte_size: u64) -> Self {
        self.byte_size = Some(byte_size);
        self
    }

    #[doc = include_str!("resource_metadata.doc/with_mime_type.md")]
    #[must_use]
    pub fn with_mime_type(mut self, mime_type: impl Into<Box<str>>) -> Self {
        self.mime_type = Some(mime_type.into());
        self
    }

    #[doc = include_str!("resource_metadata.doc/with_intrinsic_size.md")]
    #[must_use]
    pub fn with_intrinsic_size(mut self, intrinsic_size: Size) -> Self {
        self.intrinsic_size = Some(intrinsic_size);
        self
    }

    #[doc = include_str!("resource_metadata.doc/with_scale.md")]
    #[must_use]
    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    #[doc = include_str!("resource_metadata.doc/byte_size_2.md")]
    #[must_use]
    pub const fn byte_size(&self) -> Option<u64> {
        self.byte_size
    }

    #[doc = include_str!("resource_metadata.doc/mime_type_2.md")]
    #[must_use]
    pub fn mime_type(&self) -> Option<&str> {
        self.mime_type.as_deref()
    }

    #[doc = include_str!("resource_metadata.doc/intrinsic_size_2.md")]
    #[must_use]
    pub const fn intrinsic_size(&self) -> Option<Size> {
        self.intrinsic_size
    }

    #[doc = include_str!("resource_metadata.doc/scale_2.md")]
    #[must_use]
    pub const fn scale(&self) -> f32 {
        self.scale
    }

    #[doc = include_str!("resource_metadata.doc/aspect_ratio.md")]
    #[must_use]
    pub fn aspect_ratio(&self) -> Option<f32> {
        match self.intrinsic_size {
            Some(size) if size.width > 0.0 && size.height > 0.0 => Some(size.width / size.height),
            _ => None,
        }
    }
}

impl Default for ResourceMetadata {
    #[doc = include_str!("resource_metadata.doc/default.md")]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_all_none_scale_one() {
        let meta = ResourceMetadata::default();
        assert_eq!(meta.byte_size(), None);
        assert_eq!(meta.mime_type(), None);
        assert_eq!(meta.intrinsic_size(), None);
        assert_eq!(meta.scale(), 1.0);
        assert_eq!(meta, ResourceMetadata::new());
    }

    #[test]
    fn builder_sets_fields() {
        let meta = ResourceMetadata::new()
            .with_byte_size(2048)
            .with_mime_type("image/png")
            .with_intrinsic_size(Size::new(200.0, 100.0))
            .with_scale(2.0);
        assert_eq!(meta.byte_size(), Some(2048));
        assert_eq!(meta.mime_type(), Some("image/png"));
        assert_eq!(meta.intrinsic_size(), Some(Size::new(200.0, 100.0)));
        assert_eq!(meta.scale(), 2.0);
    }

    #[test]
    fn aspect_ratio_from_intrinsic_size() {
        let meta = ResourceMetadata::new().with_intrinsic_size(Size::new(200.0, 100.0));
        assert_eq!(meta.aspect_ratio(), Some(2.0));
    }

    #[test]
    fn aspect_ratio_none_without_size_or_zero_dim() {
        assert_eq!(ResourceMetadata::new().aspect_ratio(), None);
        let zero = ResourceMetadata::new().with_intrinsic_size(Size::new(0.0, 100.0));
        assert_eq!(zero.aspect_ratio(), None);
        let zero_h = ResourceMetadata::new().with_intrinsic_size(Size::new(100.0, 0.0));
        assert_eq!(zero_h.aspect_ratio(), None);
    }
}
