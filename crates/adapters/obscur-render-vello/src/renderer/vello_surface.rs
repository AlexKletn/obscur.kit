#![doc = include_str!("vello_surface.doc/_module.md")]

use crate::error::VelloRenderError;

#[doc = include_str!("vello_surface.doc/VelloSurface.md")]
pub struct VelloSurface {
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
}

impl VelloSurface {
    #[doc = include_str!("vello_surface.doc/new.md")]
    #[must_use]
    pub const fn new(surface: wgpu::Surface<'static>, config: wgpu::SurfaceConfiguration) -> Self {
        Self { surface, config }
    }

    #[doc = include_str!("vello_surface.doc/config.md")]
    #[must_use]
    pub const fn config(&self) -> &wgpu::SurfaceConfiguration {
        &self.config
    }

    #[doc = include_str!("vello_surface.doc/configure.md")]
    pub fn configure(&self, device: &wgpu::Device) {
        self.surface.configure(device, &self.config);
    }

    #[doc = include_str!("vello_surface.doc/resize.md")]
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.config.width = width.max(1);
        self.config.height = height.max(1);
        self.configure(device);
    }

    #[doc = include_str!("vello_surface.doc/current_texture.md")]
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
