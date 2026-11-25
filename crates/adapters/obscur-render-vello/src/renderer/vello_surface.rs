//! An on-screen wgpu surface wrapper for windowed presentation (spec §5.2).

use crate::error::VelloRenderError;

#[doc = include_str!("../../docs/api/renderer/vello_surface.doc/VelloSurface.md")]
pub struct VelloSurface {
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
}

impl VelloSurface {
    /// Wraps a host-created `surface` and its `config`.
    #[must_use]
    pub fn new(surface: wgpu::Surface<'static>, mut config: wgpu::SurfaceConfiguration) -> Self {
        // A surface's default format is usually an `*UnormSrgb` one, and writing
        // to a view in that format makes the hardware apply the sRGB transfer
        // function. Vello's output is ALREADY sRGB-encoded, so that write encodes
        // it a second time and every colour reaches the screen too light — an
        // 8%-lightness panel painted at 32%, which is a dark theme that is not
        // dark.
        //
        // The fix is to blit through a view that does not encode. That view is
        // only legal if the surface was configured to allow it, which is what
        // this does — here rather than at each call site, because a host that
        // forgot would get a defect with no error attached to it.
        let plain = config.format.remove_srgb_suffix();
        if plain != config.format && !config.view_formats.contains(&plain) {
            config.view_formats.push(plain);
        }
        Self { surface, config }
    }

    #[doc = include_str!("../../docs/api/renderer/vello_surface.doc/blit_format.md")]
    #[must_use]
    pub fn blit_format(&self) -> wgpu::TextureFormat {
        self.config.format.remove_srgb_suffix()
    }

    /// The current surface configuration.
    #[must_use]
    pub const fn config(&self) -> &wgpu::SurfaceConfiguration {
        &self.config
    }

    #[doc = include_str!("../../docs/api/renderer/vello_surface.doc/configure.md")]
    pub fn configure(&self, device: &wgpu::Device) {
        self.surface.configure(device, &self.config);
    }

    #[doc = include_str!("../../docs/api/renderer/vello_surface.doc/resize.md")]
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.config.width = width.max(1);
        self.config.height = height.max(1);
        self.configure(device);
    }

    #[doc = include_str!("../../docs/api/renderer/vello_surface.doc/current_texture.md")]
    pub fn current_texture(&self) -> Result<wgpu::SurfaceTexture, VelloRenderError> {
        match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture)
            | wgpu::CurrentSurfaceTexture::Suboptimal(texture) => Ok(texture),
            wgpu::CurrentSurfaceTexture::Timeout => Err(VelloRenderError::SurfaceLost(
                "surface acquire timed out".into(),
            )),
            wgpu::CurrentSurfaceTexture::Occluded => {
                Err(VelloRenderError::SurfaceLost("surface occluded".into()))
            }
            wgpu::CurrentSurfaceTexture::Outdated => Err(VelloRenderError::SurfaceLost(
                "surface configuration outdated".into(),
            )),
            wgpu::CurrentSurfaceTexture::Lost => {
                Err(VelloRenderError::SurfaceLost("surface lost".into()))
            }
            wgpu::CurrentSurfaceTexture::Validation => Err(VelloRenderError::SurfaceLost(
                "surface validation error".into(),
            )),
        }
    }
}

impl std::fmt::Debug for VelloSurface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // `wgpu::Surface` is not `Debug`; summarise the configured extent/format.
        f.debug_struct("VelloSurface")
            .field("surface", &"<wgpu::Surface>")
            .field("width", &self.config.width)
            .field("height", &self.config.height)
            .field("format", &self.config.format)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    fn config(format: wgpu::TextureFormat) -> wgpu::SurfaceConfiguration {
        wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: 8,
            height: 8,
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: Vec::new(),
        }
    }

    /// `VelloSurface::new` needs a real `wgpu::Surface`, which needs a window.
    /// The part that can fail silently is the config arithmetic, so it is
    /// factored out here and checked directly — a GPU is not required to know
    /// whether the right format was asked for.
    fn normalized(format: wgpu::TextureFormat) -> wgpu::SurfaceConfiguration {
        let mut config = config(format);
        let plain = config.format.remove_srgb_suffix();
        if plain != config.format && !config.view_formats.contains(&plain) {
            config.view_formats.push(plain);
        }
        config
    }

    #[test]
    fn an_srgb_surface_gains_a_view_format_that_does_not_encode() {
        // Without this the blit target re-applies the transfer function and
        // every authored colour reaches the screen too light.
        let config = normalized(wgpu::TextureFormat::Bgra8UnormSrgb);
        assert!(
            config
                .view_formats
                .contains(&wgpu::TextureFormat::Bgra8Unorm),
            "{:?}",
            config.view_formats
        );
    }

    #[test]
    fn a_surface_that_never_encoded_is_left_alone() {
        let config = normalized(wgpu::TextureFormat::Bgra8Unorm);
        assert!(
            config.view_formats.is_empty(),
            "nothing to add: {:?}",
            config.view_formats
        );
    }

    #[test]
    fn the_view_format_is_not_listed_twice() {
        let mut config = config(wgpu::TextureFormat::Rgba8UnormSrgb);
        config.view_formats.push(wgpu::TextureFormat::Rgba8Unorm);
        let plain = config.format.remove_srgb_suffix();
        if plain != config.format && !config.view_formats.contains(&plain) {
            config.view_formats.push(plain);
        }
        assert_eq!(config.view_formats.len(), 1);
    }

    #[test]
    fn stripping_the_suffix_keeps_the_channel_order() {
        // `Bgra` must not become `Rgba` on the way through: a swapped view
        // format is a red interface, which is at least loud. Kept as an
        // assertion anyway, because the blit is what would silently do it.
        assert_eq!(
            wgpu::TextureFormat::Bgra8UnormSrgb.remove_srgb_suffix(),
            wgpu::TextureFormat::Bgra8Unorm
        );
        assert_eq!(
            wgpu::TextureFormat::Rgba8UnormSrgb.remove_srgb_suffix(),
            wgpu::TextureFormat::Rgba8Unorm
        );
    }
}
