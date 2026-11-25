//! The host-provided GPU device paired with a live Vello renderer (spec §2.7).

use vello::{AaSupport, Renderer, RendererOptions};

use crate::error::VelloRenderError;

#[doc = include_str!("../../docs/api/renderer/vello_device.doc/VelloDevice.md")]
pub struct VelloDevice {
    device: wgpu::Device,
    queue: wgpu::Queue,
    renderer: Option<Renderer>,
    #[cfg(feature = "lyon")]
    pub(super) lyon: Option<Box<super::lyon::LyonRenderer>>,
    #[cfg(feature = "lyon")]
    pub(super) lyon_fallback: bool,
    #[cfg(feature = "hybrid")]
    pub(super) hybrid: Option<Box<super::hybrid_renderer::HybridRenderer>>,
}

impl VelloDevice {
    /// Returns the adapter's supported timestamp features for opt-in Lyon GPU
    /// profiling when `OBSCUR_GPU_STATS=1`.
    #[must_use]
    pub fn diagnostic_features(adapter: &wgpu::Adapter) -> wgpu::Features {
        #[cfg(feature = "lyon")]
        if std::env::var_os("OBSCUR_GPU_STATS").is_some_and(|value| value == "1") {
            return adapter.features()
                & (wgpu::Features::TIMESTAMP_QUERY
                    | wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS);
        }
        let _ = adapter;
        wgpu::Features::empty()
    }

    #[doc = include_str!("../../docs/api/renderer/vello_device.doc/new.md")]
    pub fn new(device: wgpu::Device, queue: wgpu::Queue) -> Result<Self, VelloRenderError> {
        let renderer = Self::classic_renderer(&device)?;
        Ok(Self {
            device,
            queue,
            renderer: Some(renderer),
            #[cfg(feature = "lyon")]
            lyon: None,
            #[cfg(feature = "lyon")]
            lyon_fallback: true,
            #[cfg(feature = "hybrid")]
            hybrid: None,
        })
    }

    #[cfg(feature = "hybrid")]
    /// Create a hybrid backend on an existing GPU device and queue.
    pub fn hybrid(device: wgpu::Device, queue: wgpu::Queue) -> Self {
        Self {
            device,
            queue,
            renderer: None,
            hybrid: Some(Box::default()),
            #[cfg(feature = "lyon")]
            lyon: None,
            #[cfg(feature = "lyon")]
            lyon_fallback: true,
        }
    }

    fn classic_renderer(device: &wgpu::Device) -> Result<Renderer, VelloRenderError> {
        Renderer::new(
            device,
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
        .map_err(VelloRenderError::RendererInitialization)
    }

    #[cfg(feature = "lyon")]
    /// Create a lyon backend on an existing GPU device and queue.
    pub fn lyon(device: wgpu::Device, queue: wgpu::Queue) -> Self {
        Self {
            device,
            queue,
            renderer: None,
            lyon: Some(Box::default()),
            lyon_fallback: true,
            #[cfg(feature = "hybrid")]
            hybrid: None,
        }
    }

    #[cfg(feature = "lyon")]
    /// Configures a Lyon device to report unsupported frames as render errors.
    /// Classic Vello is never initialized as a fallback on this device.
    #[must_use]
    pub fn without_lyon_fallback(mut self) -> Self {
        self.lyon_fallback = false;
        self
    }

    #[cfg(any(feature = "lyon", feature = "hybrid"))]
    pub(super) fn ensure_classic(&mut self) -> Result<(), VelloRenderError> {
        if self.renderer.is_none() {
            self.renderer = Some(Self::classic_renderer(&self.device)?);
        }
        Ok(())
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
        self.renderer.as_mut().expect("classic Vello device")
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
