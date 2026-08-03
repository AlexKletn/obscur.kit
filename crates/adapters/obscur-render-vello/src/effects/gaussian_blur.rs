#![doc = include_str!("gaussian_blur.doc/_module.md")]

use crate::conversion::BlurLayer;
use crate::renderer::VelloFrame;

#[doc = include_str!("gaussian_blur.doc/BLUR_FORMAT.md")]
const BLUR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

#[doc = include_str!("gaussian_blur.doc/MAX_BLUR_EXTENT.md")]
const MAX_BLUR_EXTENT: u32 = 8192;

#[doc = include_str!("gaussian_blur.doc/GaussianBlur.md")]
pub struct GaussianBlur {
    #[doc = include_str!("gaussian_blur.doc/blur_pipeline.md")]
    blur_pipeline: wgpu::RenderPipeline,
    blur_layout: wgpu::BindGroupLayout,
    #[doc = include_str!("gaussian_blur.doc/composite_pipeline.md")]
    composite_pipeline: wgpu::RenderPipeline,
    composite_layout: wgpu::BindGroupLayout,
    #[doc = include_str!("gaussian_blur.doc/sampler.md")]
    sampler: wgpu::Sampler,
}

impl GaussianBlur {
    #[doc = include_str!("gaussian_blur.doc/new.md")]
    #[must_use]
    pub fn new(device: &wgpu::Device) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("obscur-gaussian-blur-shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("gaussian_blur.wgsl").into()),
        });

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

    #[doc = include_str!("gaussian_blur.doc/render.md")]
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
        let src_view = source
            .texture()
            .create_view(&wgpu::TextureViewDescriptor::default());
        self.render_view(
            device,
            encoder,
            &src_view,
            (sw, sh),
            source_origin,
            layer.radius,
            target_view,
            target_size,
        );
    }

    #[doc = include_str!("gaussian_blur.doc/render_view.md")]
    #[allow(clippy::cast_precision_loss, clippy::too_many_arguments)]
    pub fn render_view(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        source: &wgpu::TextureView,
        source_size: (u32, u32),
        source_origin: (f32, f32),
        radius: f32,
        target_view: &wgpu::TextureView,
        target_size: (u32, u32),
    ) {
        let (sw, sh) = source_size;
        let (tw, th) = target_size;
        if sw == 0 || sh == 0 || tw == 0 || th == 0 {
            return;
        }

        let horizontally_blurred = scratch_texture(device, sw, sh, "obscur-gaussian-h");
        let fully_blurred = scratch_texture(device, sw, sh, "obscur-gaussian-v");
        let horizontally_blurred_view =
            horizontally_blurred.create_view(&wgpu::TextureViewDescriptor::default());
        let fully_blurred_view = fully_blurred.create_view(&wgpu::TextureViewDescriptor::default());

        self.blur_pass(
            device,
            encoder,
            source,
            &horizontally_blurred_view,
            [1.0 / sw as f32, 0.0],
            radius,
        );
        self.blur_pass(
            device,
            encoder,
            &horizontally_blurred_view,
            &fully_blurred_view,
            [0.0, 1.0 / sh as f32],
            radius,
        );

        // `filter: blur` composites the whole blurred quad, no rounded mask. The
        // source is a blurred Vello sub-scene (straight alpha); premultiplied `false`
        // keeps the shipped opaque path byte-identical.
        self.composite_texture(
            device,
            encoder,
            &fully_blurred_view,
            (sw, sh),
            source_origin,
            target_view,
            target_size,
            false,
        );
    }

    #[doc = include_str!("gaussian_blur.doc/composite_texture.md")]
    #[allow(clippy::cast_precision_loss, clippy::too_many_arguments)]
    pub fn composite_texture(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        source: &wgpu::TextureView,
        source_size: (u32, u32),
        source_origin: (f32, f32),
        target_view: &wgpu::TextureView,
        target_size: (u32, u32),
        premultiply: bool,
    ) {
        let (sw, sh) = source_size;
        let (tw, th) = target_size;
        if sw == 0 || sh == 0 || tw == 0 || th == 0 {
            return;
        }
        let (ox, oy) = source_origin;
        let x0 = ox / tw as f32 * 2.0 - 1.0;
        let x1 = (ox + sw as f32) / tw as f32 * 2.0 - 1.0;
        let y_top = 1.0 - oy / th as f32 * 2.0;
        let y_bottom = 1.0 - (oy + sh as f32) / th as f32 * 2.0;
        self.composite_pass(
            device,
            encoder,
            source,
            target_view,
            [x0, y_top, x1, y_bottom],
            None,
            premultiply,
        );
    }

    #[doc = include_str!("gaussian_blur.doc/clippy.md")]
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::too_many_arguments
    )]
    pub fn render_backdrop(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        accum: &wgpu::Texture,
        accum_view: &wgpu::TextureView,
        accum_size: (u32, u32),
        bounds: [f32; 4],
        radii: [f32; 4],
        blur_radius: f32,
    ) {
        let (tw, th) = accum_size;
        if tw == 0 || th == 0 || blur_radius <= 0.0 {
            return;
        }
        // Padded region (panel + ~3σ margin), clamped to the frame extent.
        let margin = (blur_radius * 3.0).ceil().max(1.0);
        let rx0 = (bounds[0] - margin).floor().clamp(0.0, tw as f32) as u32;
        let ry0 = (bounds[1] - margin).floor().clamp(0.0, th as f32) as u32;
        let rx1 = (bounds[2] + margin).ceil().clamp(0.0, tw as f32) as u32;
        let ry1 = (bounds[3] + margin).ceil().clamp(0.0, th as f32) as u32;
        if rx1 <= rx0 || ry1 <= ry0 {
            return;
        }
        let sw = rx1 - rx0;
        let sh = ry1 - ry0;

        let copied_region = scratch_texture(device, sw, sh, "obscur-backdrop-src");
        encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: accum,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: rx0,
                    y: ry0,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyTextureInfo {
                texture: &copied_region,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: sw,
                height: sh,
                depth_or_array_layers: 1,
            },
        );
        let copied_region_view = copied_region.create_view(&wgpu::TextureViewDescriptor::default());
        let horizontally_blurred = scratch_texture(device, sw, sh, "obscur-backdrop-h");
        let fully_blurred = scratch_texture(device, sw, sh, "obscur-backdrop-v");
        let horizontally_blurred_view =
            horizontally_blurred.create_view(&wgpu::TextureViewDescriptor::default());
        let fully_blurred_view = fully_blurred.create_view(&wgpu::TextureViewDescriptor::default());
        self.blur_pass(
            device,
            encoder,
            &copied_region_view,
            &horizontally_blurred_view,
            [1.0 / sw as f32, 0.0],
            blur_radius,
        );
        self.blur_pass(
            device,
            encoder,
            &horizontally_blurred_view,
            &fully_blurred_view,
            [0.0, 1.0 / sh as f32],
            blur_radius,
        );

        // Composite the blurred region back over the frame at the clamped region's
        // NDC rect, masked to the rounded panel (in scratch pixel space).
        let x0 = rx0 as f32 / tw as f32 * 2.0 - 1.0;
        let x1 = rx1 as f32 / tw as f32 * 2.0 - 1.0;
        let y_top = 1.0 - ry0 as f32 / th as f32 * 2.0;
        let y_bottom = 1.0 - ry1 as f32 / th as f32 * 2.0;
        let mask = CompositeMask {
            panel: [
                bounds[0] - rx0 as f32,
                bounds[1] - ry0 as f32,
                bounds[2] - rx0 as f32,
                bounds[3] - ry0 as f32,
            ],
            radii,
            quad: [sw as f32, sh as f32],
            aa: 1.0,
        };
        // `fully_blurred` is a blurred copy of the accumulating frame (already
        // premultiplied), so premultiplied `false`; the mask frosts only the panel.
        self.composite_pass(
            device,
            encoder,
            &fully_blurred_view,
            accum_view,
            [x0, y_top, x1, y_bottom],
            Some(mask),
            false,
        );
    }

    #[doc = include_str!("gaussian_blur.doc/composite_over.md")]
    pub fn composite_over(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        source: &wgpu::TextureView,
        target: &wgpu::TextureView,
    ) {
        // Full-target NDC quad: uv (0,0) → (-1, 1) top-left, (1,1) → (1, -1). The
        // source is a Vello segment (STRAIGHT alpha), so premultiply it before the
        // premultiplied-over blend layers it onto the accumulating (premult) frame.
        self.composite_pass(
            device,
            encoder,
            source,
            target,
            [-1.0, 1.0, 1.0, -1.0],
            None,
            true,
        );
    }

    #[doc = include_str!("gaussian_blur.doc/blur_pass.md")]
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

    #[doc = include_str!("gaussian_blur.doc/composite_pass.md")]
    #[allow(clippy::too_many_arguments)]
    fn composite_pass(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        input: &wgpu::TextureView,
        target: &wgpu::TextureView,
        rect: [f32; 4],
        mask: Option<CompositeMask>,
        premultiply: bool,
    ) {
        let mask = mask.unwrap_or_default();
        // CompositeParams: rect(4) + panel(4) + radii(4) + (quad_w, quad_h,
        // mask_enabled, aa) + (premultiply, 0, 0, 0) — 20 f32 / 80 bytes, matching
        // the WGSL struct.
        let uniform = f32s_to_bytes(&[
            rect[0],
            rect[1],
            rect[2],
            rect[3],
            mask.panel[0],
            mask.panel[1],
            mask.panel[2],
            mask.panel[3],
            mask.radii[0],
            mask.radii[1],
            mask.radii[2],
            mask.radii[3],
            mask.quad[0],
            mask.quad[1],
            mask.enabled(),
            mask.aa,
            if premultiply { 1.0 } else { 0.0 },
            0.0,
            0.0,
            0.0,
        ]);
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

    #[doc = include_str!("gaussian_blur.doc/bind_entries.md")]
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

#[doc = include_str!("gaussian_blur.doc/CompositeMask.md")]
#[derive(Clone, Copy, Default)]
struct CompositeMask {
    #[doc = include_str!("gaussian_blur.doc/panel.md")]
    panel: [f32; 4],
    #[doc = include_str!("gaussian_blur.doc/radii.md")]
    radii: [f32; 4],
    #[doc = include_str!("gaussian_blur.doc/quad.md")]
    quad: [f32; 2],
    #[doc = include_str!("gaussian_blur.doc/aa.md")]
    aa: f32,
}

impl CompositeMask {
    #[doc = include_str!("gaussian_blur.doc/enabled.md")]
    fn enabled(&self) -> f32 {
        if self.quad[0] > 0.0 && self.quad[1] > 0.0 {
            1.0
        } else {
            0.0
        }
    }
}

impl std::fmt::Debug for GaussianBlur {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GaussianBlur")
            .field("format", &BLUR_FORMAT)
            .finish_non_exhaustive()
    }
}

#[doc = include_str!("gaussian_blur.doc/build_pipeline.md")]
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

#[doc = include_str!("gaussian_blur.doc/uniform_buffer.md")]
fn uniform_buffer(device: &wgpu::Device, bytes: &[u8], label: &'static str) -> wgpu::Buffer {
    use wgpu::util::DeviceExt;
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: bytes,
        usage: wgpu::BufferUsages::UNIFORM,
    })
}

#[doc = include_str!("gaussian_blur.doc/scratch_texture.md")]
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
        // COPY_DST lets the backdrop-filter pass seed a scratch with a copied
        // region of the accumulating frame before blurring it.
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    })
}

#[doc = include_str!("gaussian_blur.doc/f32s_to_bytes.md")]
fn f32s_to_bytes(values: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(values.len() * 4);
    for value in values {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes
}

#[cfg(test)]
mod tests {
    #![doc = include_str!("gaussian_blur.doc/_module_2.md")]

    /// sRGB electro-optical transfer (decode gamma → linear), matching the shader.
    fn srgb_to_linear(c: f32) -> f32 {
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    }

    /// sRGB opto-electronic transfer (encode linear → gamma), matching the shader.
    fn linear_to_srgb(c: f32) -> f32 {
        if c <= 0.0031308 {
            12.92 * c
        } else {
            1.055 * c.powf(1.0 / 2.4) - 0.055
        }
    }

    /// One separable gaussian tap-sum over a 1-D row of opaque gamma samples at
    /// `center`, done in `linear` light (decode → blur → encode) or naive gamma.
    fn blur_at(row: &[f32], center: usize, sigma: f32, linear: bool) -> f32 {
        let support = (sigma * 3.0).ceil() as i32;
        let mut accum = 0.0_f32;
        let mut wsum = 0.0_f32;
        for i in -support..=support {
            let idx = (center as i32 + i).clamp(0, row.len() as i32 - 1) as usize;
            let fi = i as f32;
            let w = (-(fi * fi) / (2.0 * sigma * sigma)).exp();
            let s = row[idx];
            accum += if linear { srgb_to_linear(s) } else { s } * w;
            wsum += w;
        }
        let norm = accum / wsum.max(0.0001);
        if linear { linear_to_srgb(norm) } else { norm }
    }

    #[test]
    fn transfer_round_trips_near_identity() {
        for step in 0..=255 {
            let c = step as f32 / 255.0;
            let back = linear_to_srgb(srgb_to_linear(c));
            assert!((back - c).abs() < 1e-4, "round-trip drift at {c}: {back}");
        }
    }

    #[test]
    fn linear_light_blur_is_brighter_at_a_bright_dark_boundary() {
        // A hard black→white step (0 on the left, 1 on the right), 41 samples wide.
        let row: Vec<f32> = (0..41).map(|i| if i < 20 { 0.0 } else { 1.0 }).collect();
        let sigma = 4.0;
        // The geometric boundary sample (index 20) is the 50/50 crossover.
        let gamma = blur_at(&row, 20, sigma, false);
        let linear = blur_at(&row, 20, sigma, true);
        // Gamma blur lands the crossover near the arithmetic mean ~0.5; linear light
        // lands it near sRGB-encode(0.5) ≈ 0.735 — visibly brighter. This is the
        // correctness fix, not a regression.
        assert!(
            linear > gamma + 0.15,
            "linear blur ({linear}) should be well above gamma blur ({gamma}) at the boundary"
        );
        assert!(
            linear > 0.70,
            "linear crossover {linear} should be ~sRGB-encode(0.5)"
        );
    }

    #[test]
    fn flat_opaque_region_is_unchanged_by_the_blur() {
        // Blur of a constant is that constant (the transfer round-trips), so a flat
        // region stays byte-identical up to float epsilon in both color spaces.
        let row = vec![0.6_f32; 41];
        let linear = blur_at(&row, 20, 4.0, true);
        assert!((linear - 0.6).abs() < 1e-4, "flat region drifted: {linear}");
    }

    #[test]
    fn degenerate_blur_is_a_pass_through() {
        // A near-zero sigma keeps only the center tap (weight 1), so the sample is
        // returned unchanged — a zero-radius blur is a pass-through.
        let row = vec![0.0, 0.25, 0.5, 0.75, 1.0];
        let linear = blur_at(&row, 2, 0.0001, true);
        assert!(
            (linear - 0.5).abs() < 1e-3,
            "degenerate blur changed the center: {linear}"
        );
    }
}
