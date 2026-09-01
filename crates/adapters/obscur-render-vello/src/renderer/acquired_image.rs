#![doc = include_str!("acquired_image.doc/_module.md")]

#[doc = include_str!("acquired_image.doc/AcquiredImage.md")]
pub struct AcquiredImage {
    #[doc = include_str!("acquired_image.doc/texture.md")]
    pub texture: wgpu::SurfaceTexture,
    #[doc = include_str!("acquired_image.doc/width.md")]
    pub width: u32,
    #[doc = include_str!("acquired_image.doc/height.md")]
    pub height: u32,
    #[doc = include_str!("acquired_image.doc/format.md")]
    pub format: wgpu::TextureFormat,
}

impl AcquiredImage {
    #[doc = include_str!("acquired_image.doc/matches.md")]
    #[must_use]
    pub const fn matches(&self, width: u32, height: u32) -> bool {
        self.width == width && self.height == height
    }
}

impl std::fmt::Debug for AcquiredImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AcquiredImage")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("format", &self.format)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn an_image_knows_the_extent_it_was_acquired_at() {
        // Not a convenience: the surface lives on the acquire thread, so the
        // caller cannot ask it, and an image blitted against the wrong extent is
        // a stretched frame with nothing to catch it.
        fn extent_matches(width: u32, height: u32, against: (u32, u32)) -> bool {
            width == against.0 && height == against.1
        }
        assert!(extent_matches(800, 600, (800, 600)));
        assert!(!extent_matches(800, 600, (1024, 600)));
    }
}
