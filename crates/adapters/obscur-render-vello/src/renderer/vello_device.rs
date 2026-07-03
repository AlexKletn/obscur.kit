//! The host-provided GPU device paired with a live Vello renderer (spec §2.7).

use vello::{AaSupport, Renderer, RendererOptions};

use crate::error::VelloRenderError;

/// A wgpu device/queue plus the [`vello::Renderer`] built on top of them
/// (spec §2.7).
///
/// # Ownership of the GPU context
///
/// wgpu adapter/device creation is asynchronous and requires a real GPU and an
/// event loop, which the adapter deliberately does **not** own. The *host*
/// creates the [`wgpu::Device`] and [`wgpu::Queue`] (it holds the async/windowing
/// context) and hands them to [`VelloDevice::new`], which then builds the Vello
/// renderer synchronously. This keeps the adapter free of any
/// `Instance::request_adapter` call, so the crate compiles and unit-tests run
/// without a GPU (spec §5.2 determinism boundary — only rasterization is
/// device-bound).
///
/// The renderer is initialised with the full anti-aliasing support set
/// ([`AaSupport::all`]) so any [`vello::AaConfig`] chosen at render time is
/// available, `use_cpu` disabled (GPU rasterization), and no pipeline cache.
pub struct VelloDevice {
    device: wgpu::Device,
    queue: wgpu::Queue,
    renderer: Renderer,
}

impl VelloDevice {
    /// Builds a Vello renderer on a host-provided `device`/`queue`.
    ///
    /// # Errors
    ///
    /// Returns [`VelloRenderError::DeviceCreation`] if [`vello::Renderer::new`]
    /// fails (for example, the device lacks a feature Vello requires).
    pub fn new(device: wgpu::Device, queue: wgpu::Queue) -> Result<Self, VelloRenderError> {
        let renderer = Renderer::new(
            &device,
            RendererOptions {
                use_cpu: false,
                antialiasing_support: AaSupport::all(),
                num_init_threads: None,
                pipeline_cache: None,
            },
        )
        .map_err(|error| VelloRenderError::DeviceCreation(error.to_string().into()))?;
        Ok(Self {
            device,
            queue,
            renderer,
        })
    }

    /// The host-provided wgpu device.
    #[must_use]
    pub const fn device(&self) -> &wgpu::Device {
        &self.device
    }

    /// The host-provided wgpu queue.
    #[must_use]
    pub const fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    /// The Vello renderer, mutably (rendering requires `&mut`).
    #[must_use]
    pub const fn renderer_mut(&mut self) -> &mut Renderer {
        &mut self.renderer
    }
}

impl std::fmt::Debug for VelloDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // `vello::Renderer` is not `Debug`; summarise instead of deriving.
        f.debug_struct("VelloDevice")
            .field("device", &"<wgpu::Device>")
            .field("queue", &"<wgpu::Queue>")
            .field("renderer", &"<vello::Renderer>")
            .finish_non_exhaustive()
    }
}
