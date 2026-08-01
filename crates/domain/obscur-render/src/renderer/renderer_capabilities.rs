#![doc = include_str!("renderer_capabilities.doc/_module.md")]

use obscur_capability::SupportLevel;

use crate::target::PixelFormat;

#[doc = include_str!("renderer_capabilities.doc/RendererCapabilities.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RendererCapabilities {
    #[doc = include_str!("renderer_capabilities.doc/max_texture_size.md")]
    pub max_texture_size: u32,
    #[doc = include_str!("renderer_capabilities.doc/msaa.md")]
    pub msaa: bool,
    #[doc = include_str!("renderer_capabilities.doc/hdr.md")]
    pub hdr: bool,
    #[doc = include_str!("renderer_capabilities.doc/custom_shaders.md")]
    pub custom_shaders: bool,
    #[doc = include_str!("renderer_capabilities.doc/offscreen.md")]
    pub offscreen: bool,
    #[doc = include_str!("renderer_capabilities.doc/preferred_format.md")]
    pub preferred_format: PixelFormat,
}

impl RendererCapabilities {
    #[doc = include_str!("renderer_capabilities.doc/new.md")]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            max_texture_size: 4096,
            msaa: false,
            hdr: false,
            custom_shaders: false,
            offscreen: true,
            preferred_format: PixelFormat::Bgra8Unorm,
        }
    }

    #[doc = include_str!("renderer_capabilities.doc/with_max_texture_size.md")]
    #[must_use]
    pub const fn with_max_texture_size(mut self, max_texture_size: u32) -> Self {
        self.max_texture_size = max_texture_size;
        self
    }

    #[doc = include_str!("renderer_capabilities.doc/msaa_2.md")]
    #[must_use]
    pub const fn msaa(mut self, msaa: bool) -> Self {
        self.msaa = msaa;
        self
    }

    #[doc = include_str!("renderer_capabilities.doc/hdr_2.md")]
    #[must_use]
    pub const fn hdr(mut self, hdr: bool) -> Self {
        self.hdr = hdr;
        self
    }

    #[doc = include_str!("renderer_capabilities.doc/custom_shaders_2.md")]
    #[must_use]
    pub const fn custom_shaders(mut self, custom_shaders: bool) -> Self {
        self.custom_shaders = custom_shaders;
        self
    }

    #[doc = include_str!("renderer_capabilities.doc/offscreen_2.md")]
    #[must_use]
    pub const fn offscreen(mut self, offscreen: bool) -> Self {
        self.offscreen = offscreen;
        self
    }

    #[doc = include_str!("renderer_capabilities.doc/with_preferred_format.md")]
    #[must_use]
    pub const fn with_preferred_format(mut self, preferred_format: PixelFormat) -> Self {
        self.preferred_format = preferred_format;
        self
    }

    #[doc = include_str!("renderer_capabilities.doc/max_texture_size_2.md")]
    #[must_use]
    pub const fn max_texture_size(&self) -> u32 {
        self.max_texture_size
    }

    #[doc = include_str!("renderer_capabilities.doc/has_msaa.md")]
    #[must_use]
    pub const fn has_msaa(&self) -> bool {
        self.msaa
    }

    #[doc = include_str!("renderer_capabilities.doc/has_hdr.md")]
    #[must_use]
    pub const fn has_hdr(&self) -> bool {
        self.hdr
    }

    #[doc = include_str!("renderer_capabilities.doc/has_custom_shaders.md")]
    #[must_use]
    pub const fn has_custom_shaders(&self) -> bool {
        self.custom_shaders
    }

    #[doc = include_str!("renderer_capabilities.doc/has_offscreen.md")]
    #[must_use]
    pub const fn has_offscreen(&self) -> bool {
        self.offscreen
    }

    #[doc = include_str!("renderer_capabilities.doc/preferred_format_2.md")]
    #[must_use]
    pub const fn preferred_format(&self) -> PixelFormat {
        self.preferred_format
    }

    #[doc = include_str!("renderer_capabilities.doc/supports_texture_size.md")]
    #[must_use]
    pub const fn supports_texture_size(&self, size: u32) -> bool {
        size <= self.max_texture_size
    }

    #[doc = include_str!("renderer_capabilities.doc/hdr_support.md")]
    #[must_use]
    pub const fn hdr_support(&self) -> SupportLevel {
        if self.hdr {
            SupportLevel::Native
        } else {
            SupportLevel::Unavailable
        }
    }
}

impl Default for RendererCapabilities {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_conservative_baseline() {
        let caps = RendererCapabilities::default();
        assert_eq!(caps.max_texture_size(), 4096);
        assert!(!caps.has_msaa());
        assert!(!caps.has_hdr());
        assert!(!caps.has_custom_shaders());
        assert!(caps.has_offscreen());
        assert_eq!(caps.preferred_format(), PixelFormat::Bgra8Unorm);
        assert_eq!(caps, RendererCapabilities::new());
    }

    #[test]
    fn builders_override_fields() {
        let caps = RendererCapabilities::new()
            .with_max_texture_size(16384)
            .msaa(true)
            .hdr(true)
            .custom_shaders(true)
            .offscreen(false)
            .with_preferred_format(PixelFormat::Rgba16Float);
        assert_eq!(caps.max_texture_size(), 16384);
        assert!(caps.has_msaa());
        assert!(caps.has_hdr());
        assert!(caps.has_custom_shaders());
        assert!(!caps.has_offscreen());
        assert_eq!(caps.preferred_format(), PixelFormat::Rgba16Float);
    }

    #[test]
    fn supports_texture_size_is_inclusive_bound() {
        let caps = RendererCapabilities::new().with_max_texture_size(2048);
        assert!(caps.supports_texture_size(0));
        assert!(caps.supports_texture_size(2048));
        assert!(!caps.supports_texture_size(2049));
    }

    #[test]
    fn hdr_support_reflects_flag() {
        assert_eq!(
            RendererCapabilities::new().hdr(true).hdr_support(),
            SupportLevel::Native
        );
        assert_eq!(
            RendererCapabilities::new().hdr(false).hdr_support(),
            SupportLevel::Unavailable
        );
    }
}
