//! Descriptive metadata about a loaded resource (spec §5.16).

use obscur_geometry::Size;

/// Descriptive metadata about a loaded resource (spec §5.16).
///
/// Captures the intrinsic, backend-agnostic properties a ready resource exposes:
/// its byte size, MIME type, intrinsic pixel size and DPI-variant scale. Per spec
/// §5.16, images carry an intrinsic size and thus an aspect ratio; non-visual
/// resources such as CSS leave [`ResourceMetadata::intrinsic_size`] as `None`.
///
/// Because it holds `f32`/[`Size`] fields it derives only `Debug`, `Clone`, and
/// `PartialEq` (floats preclude `Eq`).
#[derive(Debug, Clone, PartialEq)]
pub struct ResourceMetadata {
    /// The encoded or CPU byte size of the resource, or `None` if unknown.
    pub byte_size: Option<u64>,
    /// The MIME type of the resource (for example `image/png`), or `None` if
    /// unknown.
    pub mime_type: Option<Box<str>>,
    /// The intrinsic size in pixels (spec §5.16), or `None` for non-visual
    /// resources such as CSS.
    pub intrinsic_size: Option<Size>,
    /// The DPI-variant scale of this representation (spec §5.16); `1.0` is the
    /// base scale.
    pub scale: f32,
}

impl ResourceMetadata {
    /// Creates empty metadata: all fields unknown and `scale` at the base `1.0`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            byte_size: None,
            mime_type: None,
            intrinsic_size: None,
            scale: 1.0,
        }
    }

    /// Returns this metadata with its byte size set (spec §5.16).
    #[must_use]
    pub fn with_byte_size(mut self, byte_size: u64) -> Self {
        self.byte_size = Some(byte_size);
        self
    }

    /// Returns this metadata with its MIME type set (spec §5.16).
    #[must_use]
    pub fn with_mime_type(mut self, mime_type: impl Into<Box<str>>) -> Self {
        self.mime_type = Some(mime_type.into());
        self
    }

    /// Returns this metadata with its intrinsic pixel size set (spec §5.16).
    #[must_use]
    pub fn with_intrinsic_size(mut self, intrinsic_size: Size) -> Self {
        self.intrinsic_size = Some(intrinsic_size);
        self
    }

    /// Returns this metadata with its DPI-variant scale set (spec §5.16).
    #[must_use]
    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    /// Returns the byte size, if known.
    #[must_use]
    pub const fn byte_size(&self) -> Option<u64> {
        self.byte_size
    }

    /// Returns the MIME type, if known.
    #[must_use]
    pub fn mime_type(&self) -> Option<&str> {
        self.mime_type.as_deref()
    }

    /// Returns the intrinsic pixel size, if any (spec §5.16).
    #[must_use]
    pub const fn intrinsic_size(&self) -> Option<Size> {
        self.intrinsic_size
    }

    /// Returns the DPI-variant scale (spec §5.16).
    #[must_use]
    pub const fn scale(&self) -> f32 {
        self.scale
    }

    /// Returns the intrinsic aspect ratio (width / height) when an intrinsic
    /// size is present with both dimensions strictly positive, else `None`
    /// (spec §5.16).
    #[must_use]
    pub fn aspect_ratio(&self) -> Option<f32> {
        match self.intrinsic_size {
            Some(size) if size.width > 0.0 && size.height > 0.0 => Some(size.width / size.height),
            _ => None,
        }
    }
}

impl Default for ResourceMetadata {
    /// The default metadata is [`ResourceMetadata::new`].
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
