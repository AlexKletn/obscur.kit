#![doc = include_str!("vello_device.doc/_module.md")]

use vello::{AaSupport, Renderer, RendererOptions};

use crate::error::VelloRenderError;

#[doc = include_str!("vello_device.doc/VelloDevice.md")]
pub struct VelloDevice {
    device: wgpu::Device,
    queue: wgpu::Queue,
    renderer: Renderer,
}

impl VelloDevice {
    #[doc = include_str!("vello_device.doc/new.md")]
    pub fn new(device: wgpu::Device, queue: wgpu::Queue) -> Result<Self, VelloRenderError> {
        let renderer = Renderer::new(
            &device,
            RendererOptions {
                use_cpu: false,
                // Area only, because area is all this backend ever asks for:
                // every `render_to_texture` call in the crate passes
                // `AaConfig::Area`. `AaSupport::all()` made `Renderer::new`
                // compile the MSAA 8x and 16x pipeline families too — vello's own
                // doc says it "might increase startup time, as more shader
                // variations must be compiled", and on a mobile driver every
                // extra family is both startup time and one more shader compiler
                // to be unlucky with.
                antialiasing_support: AaSupport::area_only(),
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

    #[doc = include_str!("vello_device.doc/device.md")]
    #[must_use]
    pub const fn device(&self) -> &wgpu::Device {
        &self.device
    }

    #[doc = include_str!("vello_device.doc/queue.md")]
    #[must_use]
    pub const fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    #[doc = include_str!("vello_device.doc/renderer_mut.md")]
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
