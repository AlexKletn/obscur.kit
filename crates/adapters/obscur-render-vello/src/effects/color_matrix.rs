//! An owned wgpu 4×5 color-matrix pass layered over Vello (spec §5.2).
//!
//! Vello 0.9 has no color filters, so the CSS color-matrix `filter` functions
//! (`brightness/contrast/saturate/grayscale/sepia/invert/hue-rotate/opacity`) are
//! implemented as our **own** wgpu post-process. Because every one of those
//! functions is an *affine* color transform, the coordinator folds a whole
//! color-op chain into a single 4×5 matrix (a 4×4 linear part + a 4×1 bias); this
//! pass applies it to an element's already-rendered sub-scene in **one** fragment
//! shader (`rgba' = M · rgba + bias`, clamped to `0..1`).
//!
//! It shares the offscreen / composite plumbing with [`GaussianBlur`](super::GaussianBlur):
//! the renderer renders a filter layer's subtree into a margin-padded offscreen,
//! runs this color-matrix pass over it (into a scratch texture), then either blurs
//! and composites (color + blur) or composites directly (color only). The pipeline
//! and shader are built once and cached on the renderer; only the small per-call
//! scratch texture and uniform buffer are per-filter.
//!
//! # Straight alpha
//!
//! Vello's texture output is STRAIGHT (non-premultiplied) alpha, which is exactly
//! the space CSS color filters are defined over, so the matrix is applied directly
//! to the sampled texels (no un-/re-premultiply).

/// The render-target texture format the color-matrix pipeline is built for —
/// matching [`GaussianBlur`](super::GaussianBlur)'s and the Vello offscreen frame's
/// [`wgpu::TextureFormat::Rgba8Unorm`], so the passes chain without a reformat.
const COLOR_MATRIX_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

/// The largest per-axis extent of a color-matrix scratch texture, clamping a
/// pathological size so the pass never allocates an unbounded texture.
const MAX_EXTENT: u32 = 8192;

/// A cached wgpu pipeline applying a folded 4×5 color matrix in one fragment pass.
///
/// Built once via [`ColorMatrix::new`] and reused across frames; each
/// [`ColorMatrix::apply`] call only allocates the small uniform buffer a single
/// filter needs (the destination scratch texture is owned by the caller so it can
/// be fed into a following blur/composite).
pub struct ColorMatrix {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    /// Clamp-to-edge linear sampler (the pass is 1:1, so filtering is a no-op, but
    /// a sampler is still required by the bind group).
    sampler: wgpu::Sampler,
}

impl ColorMatrix {
    /// Builds the color-matrix pipeline on `device` for [`COLOR_MATRIX_FORMAT`].
    #[must_use]
    pub fn new(device: &wgpu::Device) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("obscur-color-matrix-shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("color_matrix.wgsl").into()),
        });

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("obscur-color-matrix-bgl"),
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
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("obscur-color-matrix-pipeline"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("obscur-color-matrix-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("fullscreen_vs"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("color_matrix_fs"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                // The pass writes a fresh texture: replace, no blending.
                targets: &[Some(wgpu::ColorTargetState {
                    format: COLOR_MATRIX_FORMAT,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("obscur-color-matrix-sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        Self {
            pipeline,
            layout,
            sampler,
        }
    }

    /// A scratch color texture (`COLOR_MATRIX_FORMAT`, sampled + render target) the
    /// pass writes its transformed output into, `width`×`height` clamped to
    /// [`MAX_EXTENT`] per axis. The caller owns it so it can feed a following blur /
    /// composite, keeping it alive until the encoder is submitted.
    #[must_use]
    pub fn scratch_texture(&self, device: &wgpu::Device, width: u32, height: u32) -> wgpu::Texture {
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some("obscur-color-matrix-scratch"),
            size: wgpu::Extent3d {
                width: width.clamp(1, MAX_EXTENT),
                height: height.clamp(1, MAX_EXTENT),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: COLOR_MATRIX_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        })
    }

    /// Records a color-matrix pass sampling `source` and writing the transformed
    /// result into `destination` (replace), applying the folded 4×5 `matrix`.
    ///
    /// `matrix` is the flattened 20 entries the shader's uniform expects verbatim:
    /// the four rows of the linear 4×4 part (`matrix[r*4 + c]`) followed by the
    /// 4-entry bias (`matrix[16 + r]`) — exactly five `vec4`s.
    pub fn apply(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        source: &wgpu::TextureView,
        destination: &wgpu::TextureView,
        matrix: &[f32; 20],
    ) {
        let buffer = uniform_buffer(device, matrix, "obscur-color-matrix-uniform");
        let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("obscur-color-matrix-bind"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(source),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: buffer.as_entire_binding(),
                },
            ],
        });
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("obscur-color-matrix-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: destination,
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
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &bind, &[]);
        pass.draw(0..3, 0..1);
    }
}

impl std::fmt::Debug for ColorMatrix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ColorMatrix")
            .field("format", &COLOR_MATRIX_FORMAT)
            .finish_non_exhaustive()
    }
}

/// A transient uniform buffer initialised with the matrix's little-endian bytes
/// (avoiding a `bytemuck` dependency — no new external deps).
fn uniform_buffer(device: &wgpu::Device, matrix: &[f32; 20], label: &'static str) -> wgpu::Buffer {
    use wgpu::util::DeviceExt;
    let mut bytes = Vec::with_capacity(matrix.len() * 4);
    for value in matrix {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: &bytes,
        usage: wgpu::BufferUsages::UNIFORM,
    })
}
