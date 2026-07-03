//! What a renderer backend supports, for effect/fallback selection (spec §5.2).

use obscur_capability::SupportLevel;

use crate::target::PixelFormat;

/// The capability profile a [`Renderer`](crate::renderer::Renderer) backend
/// advertises (spec §5.2).
///
/// Because the renderer abstraction hides backends of very different power
/// (spec §5.2) — from a software/headless rasterizer to a GPU platform backend —
/// callers introspect this profile to pick effects and fallbacks before
/// submitting work. [`Self::custom_shaders`] additionally gates the trusted
/// user-shader extensions of spec §5.3.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RendererCapabilities {
    /// The maximum texture edge length, in physical pixels, the backend accepts.
    pub max_texture_size: u32,
    /// Whether the backend supports multisample anti-aliasing.
    pub msaa: bool,
    /// Whether the backend supports high-dynamic-range output.
    pub hdr: bool,
    /// Whether the backend permits restricted user shaders — gates the trusted
    /// shader extensions of spec §5.3.
    pub custom_shaders: bool,
    /// Whether the backend can render to offscreen targets (spec §5.6).
    pub offscreen: bool,
    /// The pixel format the backend prefers to present in.
    pub preferred_format: PixelFormat,
}

impl RendererCapabilities {
    /// Creates a conservative baseline profile.
    ///
    /// The baseline is what the least-capable backend (a software renderer) can
    /// promise: a `4096` maximum texture size, no MSAA, no HDR, no custom
    /// shaders, offscreen output available, and a [`PixelFormat::Bgra8Unorm`]
    /// preferred format. Backends widen it with the fluent setters.
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

    /// Sets the maximum texture edge length, returning the updated profile.
    #[must_use]
    pub const fn with_max_texture_size(mut self, max_texture_size: u32) -> Self {
        self.max_texture_size = max_texture_size;
        self
    }

    /// Sets whether the backend supports MSAA, returning the updated profile.
    #[must_use]
    pub const fn msaa(mut self, msaa: bool) -> Self {
        self.msaa = msaa;
        self
    }

    /// Sets whether the backend supports HDR output, returning the updated
    /// profile.
    #[must_use]
    pub const fn hdr(mut self, hdr: bool) -> Self {
        self.hdr = hdr;
        self
    }

    /// Sets whether the backend permits custom shaders, returning the updated
    /// profile (spec §5.3).
    #[must_use]
    pub const fn custom_shaders(mut self, custom_shaders: bool) -> Self {
        self.custom_shaders = custom_shaders;
        self
    }

    /// Sets whether the backend can render offscreen, returning the updated
    /// profile (spec §5.6).
    #[must_use]
    pub const fn offscreen(mut self, offscreen: bool) -> Self {
        self.offscreen = offscreen;
        self
    }

    /// Sets the preferred pixel format, returning the updated profile.
    #[must_use]
    pub const fn with_preferred_format(mut self, preferred_format: PixelFormat) -> Self {
        self.preferred_format = preferred_format;
        self
    }

    /// The maximum texture edge length, in physical pixels.
    #[must_use]
    pub const fn max_texture_size(&self) -> u32 {
        self.max_texture_size
    }

    /// Whether the backend supports MSAA.
    #[must_use]
    pub const fn has_msaa(&self) -> bool {
        self.msaa
    }

    /// Whether the backend supports HDR output.
    #[must_use]
    pub const fn has_hdr(&self) -> bool {
        self.hdr
    }

    /// Whether the backend permits custom shaders (spec §5.3).
    #[must_use]
    pub const fn has_custom_shaders(&self) -> bool {
        self.custom_shaders
    }

    /// Whether the backend can render offscreen (spec §5.6).
    #[must_use]
    pub const fn has_offscreen(&self) -> bool {
        self.offscreen
    }

    /// The preferred pixel format.
    #[must_use]
    pub const fn preferred_format(&self) -> PixelFormat {
        self.preferred_format
    }

    /// Whether the backend can allocate a texture of the given edge `size`.
    ///
    /// True when `size` does not exceed [`Self::max_texture_size`].
    #[must_use]
    pub const fn supports_texture_size(&self, size: u32) -> bool {
        size <= self.max_texture_size
    }

    /// The honest [`SupportLevel`] at which the backend offers HDR output.
    ///
    /// [`SupportLevel::Native`] when [`Self::hdr`] is set, else
    /// [`SupportLevel::Unavailable`] — the introspection counterpart of the
    /// explicit diagnostics of spec §5.2.
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
