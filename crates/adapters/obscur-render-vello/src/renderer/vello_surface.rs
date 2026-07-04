//! An on-screen wgpu surface wrapper for windowed presentation (spec §5.2).

use crate::error::VelloRenderError;

/// A [`wgpu::Surface`] with its [`wgpu::SurfaceConfiguration`], for on-screen
/// presentation (spec §5.2).
///
/// # Where the surface comes from
///
/// Building a surface requires a window's raw-window-handle and the wgpu
/// `Instance`, both owned by the *host* (spec §2.7): the adapter never touches a
/// window. The host creates a `Surface<'static>` and its configuration and passes
/// them to [`VelloSurface::new`]; this type then owns the swapchain lifecycle
/// (configure, resize, acquire) that a renderer drives each frame.
///
/// The surface is `'static` because it must outlive individual frames; the host
/// guarantees the window lives at least as long.
pub struct VelloSurface {
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
}

impl VelloSurface {
    /// Wraps a host-created `surface` and its `config`.
    #[must_use]
    pub const fn new(surface: wgpu::Surface<'static>, config: wgpu::SurfaceConfiguration) -> Self {
        Self { surface, config }
    }

    /// The current surface configuration.
    #[must_use]
    pub const fn config(&self) -> &wgpu::SurfaceConfiguration {
        &self.config
    }

    /// Applies the current configuration to the swapchain.
    ///
    /// Must be called once before the first frame and again after any
    /// [`Self::resize`].
    pub fn configure(&self, device: &wgpu::Device) {
        self.surface.configure(device, &self.config);
    }

    /// Resizes the swapchain to `width` × `height` and reconfigures it.
    ///
    /// The extent is clamped to a minimum of `1` in each axis, since wgpu rejects
    /// a zero-sized surface.
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.config.width = width.max(1);
        self.config.height = height.max(1);
        self.configure(device);
    }

    /// Acquires the next swapchain texture to render into.
    ///
    /// A suboptimal texture is still returned (the caller may reconfigure at
    /// leisure); a timed-out, occluded, outdated, lost, or validation-failed
    /// acquisition is surfaced as an error so the caller can reconfigure and
    /// retry.
    ///
    /// # Errors
    ///
    /// Returns [`VelloRenderError::SurfaceLost`] when the swapchain texture cannot
    /// be acquired for presentation.
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
