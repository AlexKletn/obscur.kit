//! An owned wgpu separable-gaussian blur pass layered over Vello (spec §5.2).
//!
//! Vello 0.9 has no content-blur primitive, so `filter: blur()` is implemented as
//! our **own** wgpu post-process: a blur layer's subtree is rendered to an
//! offscreen texture, run through a horizontal then a vertical gaussian pass (a
//! WGSL fragment shader with gaussian weights derived from the radius, ping-ponging
//! between textures), and composited back onto the target with premultiplied-over
//! alpha blending. The pipelines and shader are built once and cached on the
//! renderer; only the small per-blur textures and uniform buffers are per-call.
//!
//! # Scope and limitations (documented, honest)
//!
//! - **Z-order.** The blurred result is composited **on top** of the target at its
//!   bounds. This is correct for the common case — a blurred panel/card with
//!   nothing painted above it — but a later sibling that should paint over the
//!   blurred element would be overdrawn. Full stacking-correct blur needs the
//!   effect folded into the compositor, a follow-up.
//! - **Margin / edges.** The offscreen texture is padded by `ceil(3 * radius)` on
//!   every side so the gaussian tail is not clipped at the element's edge; the
//!   sampler clamps to a transparent border, so nothing bleeds in from outside.
//! - **Coordinate space.** Bounds are the element's *untransformed* border box and
//!   the target is assumed 1:1 logical↔physical pixels; a device scale or an
//!   enclosing transform combined with blur is approximate (documented).
//! - **`backdrop-filter`** (blurring the content BEHIND) is out of scope — it needs
//!   the backdrop, a follow-up that reuses this same offscreen + wgpu-pass path.

use crate::conversion::BlurLayer;
use crate::renderer::VelloFrame;

/// The render-target texture format the blur pipelines are built for. Both the
/// Vello offscreen frame and the per-blur sub-scene textures use
/// [`wgpu::TextureFormat::Rgba8Unorm`], so a single pipeline set suffices.
const BLUR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

/// The largest per-axis extent of a blur offscreen texture, clamping a pathological
/// radius / bounds so the pass never allocates an unbounded texture.
const MAX_BLUR_EXTENT: u32 = 8192;

/// A cached wgpu pipeline set implementing a separable gaussian blur + composite.
///
/// Built once via [`GaussianBlur::new`] and reused across frames; each
/// [`GaussianBlur::render`] call only allocates the small ping-pong textures and
/// uniform buffers a single blur layer needs.
pub struct GaussianBlur {
    /// One separable gaussian pass (horizontal or vertical, per its uniform).
    blur_pipeline: wgpu::RenderPipeline,
    blur_layout: wgpu::BindGroupLayout,
    /// The premultiplied-over composite of a blurred texture onto the target.
    composite_pipeline: wgpu::RenderPipeline,
    composite_layout: wgpu::BindGroupLayout,
    /// Clamp-to-edge linear sampler shared by every pass.
    sampler: wgpu::Sampler,
}

impl GaussianBlur {
    /// Builds the blur + composite pipelines on `device` for [`BLUR_FORMAT`].
    #[must_use]
    pub fn new(device: &wgpu::Device) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("obscur-gaussian-blur-shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("gaussian_blur.wgsl").into()),
        });

        // Both pipelines bind: a sampled texture, a sampler, and a uniform buffer.
        let bind_layout = |label: &'static str| {
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some(label),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            })
        };
        let blur_layout = bind_layout("obscur-gaussian-blur-bgl");
        let composite_layout = bind_layout("obscur-gaussian-composite-bgl");

        let blur_pipeline = build_pipeline(
            device,
            &shader,
            &blur_layout,
            "obscur-gaussian-blur-pipeline",
            "fullscreen_vs",
            "blur_fs",
            // The blur writes a fresh texture: replace, no blending.
            None,
        );
        let composite_pipeline = build_pipeline(
            device,
            &shader,
            &composite_layout,
            "obscur-gaussian-composite-pipeline",
            "composite_vs",
            "composite_fs",
            // Premultiplied-over: src * 1 + dst * (1 - src.a).
            Some(wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add,
                },
            }),
        );

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("obscur-gaussian-sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        Self {
            blur_pipeline,
            blur_layout,
            composite_pipeline,
            composite_layout,
            sampler,
        }
    }

    /// Blurs a single already-rendered [`BlurLayer`] sub-scene texture and
    /// composites it onto `target`.
    ///
    /// `source` is the sub-scene rendered into a margin-padded offscreen texture
    /// whose top-left corresponds to absolute pixel `source_origin`; `layer` gives
    /// the blur radius; `target` is the final frame the blurred result blends onto
    /// (with `target_size` its extent). The work is recorded into `encoder`; the
    /// caller submits it after the Vello render-to-texture passes so ordering holds.
    #[allow(clippy::cast_precision_loss, clippy::too_many_arguments)]
    pub fn render(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        source: &VelloFrame,
        source_origin: (f32, f32),
        layer: &BlurLayer,
        target_view: &wgpu::TextureView,
        target_size: (u32, u32),
    ) {
        let (sw, sh) = source.size();
        let (tw, th) = target_size;
        if sw == 0 || sh == 0 || tw == 0 || th == 0 {
            return;
        }

        // Ping-pong textures: horizontal writes `temp_a`, vertical writes `temp_b`.
        let temp_a = scratch_texture(device, sw, sh, "obscur-gaussian-h");
        let temp_b = scratch_texture(device, sw, sh, "obscur-gaussian-v");
        let temp_a_view = temp_a.create_view(&wgpu::TextureViewDescriptor::default());
        let temp_b_view = temp_b.create_view(&wgpu::TextureViewDescriptor::default());

        // Horizontal pass: sample the source, write `temp_a`.
        self.blur_pass(
            device,
            encoder,
            &source
                .texture()
                .create_view(&wgpu::TextureViewDescriptor::default()),
            &temp_a_view,
            [1.0 / sw as f32, 0.0],
            layer.radius,
        );
        // Vertical pass: sample `temp_a`, write `temp_b`.
        self.blur_pass(
            device,
            encoder,
            &temp_a_view,
            &temp_b_view,
            [0.0, 1.0 / sh as f32],
            layer.radius,
        );

        // Composite `temp_b` back onto the target at its absolute pixel rect.
        let (ox, oy) = source_origin;
        let x0 = ox / tw as f32 * 2.0 - 1.0;
        let x1 = (ox + sw as f32) / tw as f32 * 2.0 - 1.0;
        let y_top = 1.0 - oy / th as f32 * 2.0;
        let y_bottom = 1.0 - (oy + sh as f32) / th as f32 * 2.0;
        self.composite_pass(device, encoder, &temp_b_view, target_view, [
            x0, y_top, x1, y_bottom,
        ]);
    }

    /// Records one separable gaussian pass sampling `input` into `output`.
    fn blur_pass(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        input: &wgpu::TextureView,
        output: &wgpu::TextureView,
        direction: [f32; 2],
        radius: f32,
    ) {
        // BlurParams { direction: vec2, radius: f32, _pad: f32 } — 16 bytes.
        let uniform = f32s_to_bytes(&[direction[0], direction[1], radius, 0.0]);
        let buffer = uniform_buffer(device, &uniform, "obscur-gaussian-blur-uniform");
        let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("obscur-gaussian-blur-bind"),
            layout: &self.blur_layout,
            entries: &Self::bind_entries(input, &self.sampler, &buffer),
        });
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("obscur-gaussian-blur-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: output,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.blur_pipeline);
        pass.set_bind_group(0, &bind, &[]);
        pass.draw(0..3, 0..1);
    }

    /// Records the premultiplied-over composite of `input` onto `target` at the
    /// target-space NDC `rect` (`[x0, y_top, x1, y_bottom]`).
    fn composite_pass(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        input: &wgpu::TextureView,
        target: &wgpu::TextureView,
        rect: [f32; 4],
    ) {
        let uniform = f32s_to_bytes(&rect);
        let buffer = uniform_buffer(device, &uniform, "obscur-gaussian-composite-uniform");
        let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("obscur-gaussian-composite-bind"),
            layout: &self.composite_layout,
            entries: &Self::bind_entries(input, &self.sampler, &buffer),
        });
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("obscur-gaussian-composite-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    // Preserve the already-rendered main scene; blend the blur over.
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.composite_pipeline);
        pass.set_bind_group(0, &bind, &[]);
        pass.draw(0..6, 0..1);
    }

    /// The three bind-group entries (texture, sampler, uniform) both pipelines use.
    fn bind_entries<'a>(
        texture: &'a wgpu::TextureView,
        sampler: &'a wgpu::Sampler,
        uniform: &'a wgpu::Buffer,
    ) -> [wgpu::BindGroupEntry<'a>; 3] {
        [
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(texture),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: uniform.as_entire_binding(),
            },
        ]
    }

}

impl std::fmt::Debug for GaussianBlur {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GaussianBlur")
            .field("format", &BLUR_FORMAT)
            .finish_non_exhaustive()
    }
}

/// Builds one fullscreen render pipeline from `vs`/`fs` entry points, with the
/// given optional color blend (`None` = replace).
fn build_pipeline(
    device: &wgpu::Device,
    shader: &wgpu::ShaderModule,
    bind_layout: &wgpu::BindGroupLayout,
    label: &'static str,
    vs: &'static str,
    fs: &'static str,
    blend: Option<wgpu::BlendState>,
) -> wgpu::RenderPipeline {
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(label),
        bind_group_layouts: &[Some(bind_layout)],
        immediate_size: 0,
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some(vs),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[],
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some(fs),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: BLUR_FORMAT,
                blend,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: None,
    })
}

/// A transient uniform buffer initialised with `bytes`.
fn uniform_buffer(device: &wgpu::Device, bytes: &[u8], label: &'static str) -> wgpu::Buffer {
    use wgpu::util::DeviceExt;
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: bytes,
        usage: wgpu::BufferUsages::UNIFORM,
    })
}

/// A scratch color texture (`BLUR_FORMAT`, sampled + render target) for a blur
/// pass, clamped to [`MAX_BLUR_EXTENT`] per axis.
fn scratch_texture(
    device: &wgpu::Device,
    width: u32,
    height: u32,
    label: &'static str,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: width.clamp(1, MAX_BLUR_EXTENT),
            height: height.clamp(1, MAX_BLUR_EXTENT),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: BLUR_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    })
}

/// Serialises `f32`s to little-endian bytes for a uniform buffer, avoiding a
/// `bytemuck` dependency (no new external deps).
fn f32s_to_bytes(values: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(values.len() * 4);
    for value in values {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes
}
