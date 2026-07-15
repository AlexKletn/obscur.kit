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
//! - **`backdrop-filter`** (blurring the content BEHIND the element) reuses this
//!   same pipeline set via [`GaussianBlur::render_backdrop`]: the renderer's
//!   segmented compositing path copies a region of the accumulated frame, runs the
//!   same separable gaussian over it, and composites it back masked to the panel's
//!   rounded shape (the frosted-glass / acrylic effect, spec §5.2).

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

    /// Blurs an already-rendered source **texture view** (rather than a
    /// [`VelloFrame`]) and composites it onto `target` — the shared core of
    /// [`Self::render`], reused when a color-matrix pass has already written the
    /// source into a scratch texture (a `filter` that mixes a color op and a blur).
    ///
    /// `source_size` is the source's `(width, height)` in pixels; the rest matches
    /// [`Self::render`]. With no color pre-pass this is byte-identical to the shipped
    /// blur (same ping-pong passes, same composite).
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

        // Ping-pong textures: horizontal writes `temp_a`, vertical writes `temp_b`.
        let temp_a = scratch_texture(device, sw, sh, "obscur-gaussian-h");
        let temp_b = scratch_texture(device, sw, sh, "obscur-gaussian-v");
        let temp_a_view = temp_a.create_view(&wgpu::TextureViewDescriptor::default());
        let temp_b_view = temp_b.create_view(&wgpu::TextureViewDescriptor::default());

        // Horizontal pass: sample the source, write `temp_a`.
        self.blur_pass(
            device,
            encoder,
            source,
            &temp_a_view,
            [1.0 / sw as f32, 0.0],
            radius,
        );
        // Vertical pass: sample `temp_a`, write `temp_b`.
        self.blur_pass(
            device,
            encoder,
            &temp_a_view,
            &temp_b_view,
            [0.0, 1.0 / sh as f32],
            radius,
        );

        // Composite `temp_b` back onto the target at its absolute pixel rect.
        // `filter: blur` composites the whole blurred quad, no rounded mask. The
        // source is a blurred Vello sub-scene (straight alpha); premultiplied `false`
        // keeps the shipped opaque path byte-identical.
        self.composite_texture(
            device,
            encoder,
            &temp_b_view,
            (sw, sh),
            source_origin,
            target_view,
            target_size,
            false,
        );
    }

    /// Composites an already-post-processed source **texture** onto `target` at its
    /// absolute pixel rect (top-left `source_origin`, extent `source_size`), with no
    /// rounded mask — the composite step shared by the blur path and the color-only
    /// `filter` path (a color matrix with no blur).
    ///
    /// `premultiply` selects whether the (straight-alpha Vello) source is
    /// premultiplied before the premultiplied-over blend: the blur path passes
    /// `false` (byte-identical to the shipped opaque composite), the color-only path
    /// passes `true` so a translucent color result (e.g. `opacity()`) composites
    /// correctly.
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

    /// Frosts the **backdrop** in place: blurs the region of the accumulating frame
    /// `accum` under `bounds` (the panel's border box, physical pixels
    /// `(x0, y0, x1, y1)`), clipped to the rounded `radii`, and writes the blurred
    /// result back over `accum` (spec §5.2, CSS `backdrop-filter`).
    ///
    /// Unlike [`Self::render`] (which blurs a *sub-scene* and composites it), this
    /// samples the already-composited frame itself: a padded region of `accum` is
    /// copied into a scratch texture (the `~3σ` margin keeps the gaussian tail from
    /// clipping at the panel edge, clamped to the frame), run through the same
    /// separable gaussian, and composited back over `accum` masked to the rounded
    /// panel shape — so only the panel area is frosted and its corners stay sharp
    /// outside the radius. Recorded into `encoder`; the caller submits after the
    /// prior segment composites so the backdrop is fully painted first.
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

        // Copy the padded region of the frame into a scratch, then H+V blur it.
        let src = scratch_texture(device, sw, sh, "obscur-backdrop-src");
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
                texture: &src,
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
        let src_view = src.create_view(&wgpu::TextureViewDescriptor::default());
        let temp_a = scratch_texture(device, sw, sh, "obscur-backdrop-h");
        let temp_b = scratch_texture(device, sw, sh, "obscur-backdrop-v");
        let temp_a_view = temp_a.create_view(&wgpu::TextureViewDescriptor::default());
        let temp_b_view = temp_b.create_view(&wgpu::TextureViewDescriptor::default());
        self.blur_pass(
            device,
            encoder,
            &src_view,
            &temp_a_view,
            [1.0 / sw as f32, 0.0],
            blur_radius,
        );
        self.blur_pass(
            device,
            encoder,
            &temp_a_view,
            &temp_b_view,
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
        // `temp_b` is a blurred copy of the accumulating frame (already
        // premultiplied), so premultiplied `false`; the mask frosts only the panel.
        self.composite_pass(
            device,
            encoder,
            &temp_b_view,
            accum_view,
            [x0, y_top, x1, y_bottom],
            Some(mask),
            false,
        );
    }

    /// Composites the whole of `source` premultiplied-over `target` (both the same
    /// extent) — the per-segment layering of the backdrop-filter path, which stacks
    /// each rendered segment onto the accumulating frame in paint order.
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
    ///
    /// `mask` optionally clips the composite to a rounded-rect panel shape (the
    /// backdrop-filter frost); `None` composites the whole quad (the `filter: blur`
    /// path), byte-identical to the pre-mask uniform (coverage is a constant `1`).
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

/// The optional rounded-rect mask a [`composite_pass`](GaussianBlur::composite_pass)
/// clips its blurred quad to — the backdrop-filter panel shape.
///
/// [`Default`] is the **disabled** mask (`quad == [0, 0]` → coverage constant `1`),
/// so the `filter: blur` composite stays byte-identical. A constructed mask carries
/// the panel border box + per-corner radii in the composited quad's own pixel
/// space, plus the antialiased edge half-width `aa`.
#[derive(Clone, Copy, Default)]
struct CompositeMask {
    /// Panel border box `[x0, y0, x1, y1]` in the quad's pixel space.
    panel: [f32; 4],
    /// Per-corner radii `(tl, tr, br, bl)` in quad pixels.
    radii: [f32; 4],
    /// The composited quad's `(width, height)` in pixels; `[0, 0]` disables the mask.
    quad: [f32; 2],
    /// Antialiased edge half-width, in pixels.
    aa: f32,
}

impl CompositeMask {
    /// The shader's `mask_enabled` flag: `1` when a non-degenerate quad is set,
    /// else `0` (the disabled default → whole-quad composite).
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
        // COPY_DST lets the backdrop-filter pass seed a scratch with a copied
        // region of the accumulating frame before blurring it.
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST,
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

#[cfg(test)]
mod tests {
    //! Deterministic CPU port of the `blur_fs` decode → blur → encode pipeline,
    //! validating (without a GPU) that convolving in **linear light** — per the
    //! Filter Effects Module default `color-interpolation-filters: linearRGB` — keeps
    //! a bright/dark boundary brighter than a naive gamma-sRGB gaussian, and that a
    //! flat opaque region round-trips the transfer unchanged.
    //!
    //! These mirror the exact WGSL helpers/formulas in `gaussian_blur.wgsl`; the GPU
    //! render `examples/blur_linear_light_proof.rs` proves the shader itself.

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
