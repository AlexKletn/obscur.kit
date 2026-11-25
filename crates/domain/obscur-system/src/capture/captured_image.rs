use obscur_geometry::Size;

#[doc = include_str!("../../docs/api/capture/capture_result.doc/CapturedImage.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct CapturedImage {
    /// The pixel dimensions of the captured image.
    size: Size,
    /// The pixel format token (for example `"rgba8"`).
    format: Box<str>,
    bytes: std::sync::Arc<[u8]>,
}

impl CapturedImage {
    /// Creates a descriptor for pixels of `size` in `format` (spec §6.17).
    #[must_use]
    pub fn new(size: Size, format: impl Into<Box<str>>) -> Self {
        Self {
            size,
            format: format.into(),
            bytes: std::sync::Arc::from([]),
        }
    }

    /// Returns the pixel dimensions of the captured image (spec §6.17).
    #[must_use]
    pub const fn size(&self) -> Size {
        self.size
    }

    /// Returns the pixel format token (spec §6.17).
    #[must_use]
    pub fn format(&self) -> &str {
        &self.format
    }

    /// Attaches the image data in the declared format.
    #[must_use]
    pub fn with_bytes(mut self, bytes: impl Into<std::sync::Arc<[u8]>>) -> Self {
        self.bytes = bytes.into();
        self
    }

    /// Returns the encoded or raw image data.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

#[cfg(test)]
mod tests {
    use super::CapturedImage;
    use obscur_geometry::Size;

    #[test]
    fn captured_image_stores_size_and_format() {
        let image = CapturedImage::new(Size::new(2.0, 3.0), "rgba8");
        assert_eq!(image.size(), Size::new(2.0, 3.0));
        assert_eq!(image.format(), "rgba8");
    }
}
