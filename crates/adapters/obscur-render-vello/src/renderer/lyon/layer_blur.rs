//! Spatial blur of an isolated, premultiplied Lyon group.

use crate::effects::GaussianBlur;

use super::pipelines::{SAMPLES, STENCIL};

pub(super) struct LayerBlur {
    extent: (u32, u32),
    views: [wgpu::TextureView; 3],
    resolve: wgpu::RenderPipeline,
    composite: wgpu::RenderPipeline,
    replace: wgpu::RenderPipeline,
    composite_group: wgpu::BindGroup,
    gaussian: GaussianBlur,
}

impl LayerBlur {
    pub(super) fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        extent: (u32, u32),
        group_layout: &wgpu::PipelineLayout,
    ) -> Self {
        let views = std::array::from_fn(|_| {
            device
                .create_texture(&wgpu::TextureDescriptor {
                    label: Some("obscur-lyon-blur"),
                    size: wgpu::Extent3d {
                        width: extent.0,
                        height: extent.1,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                        | wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                })
                .create_view(&wgpu::TextureViewDescriptor::default())
        });
        let resolve = pipeline(
            device,
            format,
            Some(group_layout),
            concat!(include_str!("srgb.wgsl"), include_str!("group.wgsl")),
            "resolve_filtered",
            1,
            false,
        );
        let texture_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("obscur-lyon-blur-texture"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            }],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("obscur-lyon-blur-texture"),
            bind_group_layouts: &[Some(&texture_layout)],
            immediate_size: 0,
        });
        let replace = pipeline(
            device,
            format,
            Some(&layout),
            include_str!("layer_blur.wgsl"),
            "fragment",
            SAMPLES,
            false,
        );
        let composite = pipeline(
            device,
            format,
            Some(&layout),
            include_str!("layer_blur.wgsl"),
            "fragment",
            SAMPLES,
            true,
        );
        let composite_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("obscur-lyon-blur-composite"),
            layout: &texture_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&views[2]),
            }],
        });
        Self {
            extent,
            views,
            resolve,
            composite,
            replace,
            composite_group,
            gaussian: GaussianBlur::new(device),
        }
    }

    #[allow(clippy::too_many_arguments, clippy::cast_precision_loss)]
    pub(super) fn encode(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        source: &wgpu::BindGroup,
        target: &wgpu::TextureView,
        stencil: &wgpu::TextureView,
        depth: u32,
        bounds: [u32; 4],
        parameters: u32,
        radius: f32,
        replace: bool,
    ) {
        let [x, y, width, height] = bounds;
        {
            let mut pass = pass(encoder, &self.views[0], None);
            pass.set_scissor_rect(x, y, width, height);
            pass.set_pipeline(&self.resolve);
            pass.set_bind_group(0, source, &[]);
            pass.draw(0..3, parameters..parameters + 1);
        }
        let (w, h) = self.extent;
        self.gaussian.blur_pass(
            device,
            encoder,
            &self.views[0],
            &self.views[1],
            [1.0 / w as f32, 0.0],
            radius,
            Some(bounds),
            false,
        );
        self.gaussian.blur_pass(
            device,
            encoder,
            &self.views[1],
            &self.views[2],
            [0.0, 1.0 / h as f32],
            radius,
            Some(bounds),
            false,
        );
        let mut pass = pass(encoder, target, Some(stencil));
        pass.set_scissor_rect(x, y, width, height);
        pass.set_stencil_reference(depth);
        pass.set_pipeline(if replace {
            &self.replace
        } else {
            &self.composite
        });
        pass.set_bind_group(0, &self.composite_group, &[]);
        pass.draw(0..3, 0..1);
    }
}

fn pass<'a>(
    encoder: &'a mut wgpu::CommandEncoder,
    target: &wgpu::TextureView,
    stencil: Option<&wgpu::TextureView>,
) -> wgpu::RenderPass<'a> {
    let load = if stencil.is_some() {
        wgpu::LoadOp::Load
    } else {
        wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT)
    };
    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("obscur-lyon-blur"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: target,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load,
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: stencil.map(|view| wgpu::RenderPassDepthStencilAttachment {
            view,
            depth_ops: None,
            stencil_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            }),
        }),
        ..Default::default()
    })
}

fn pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    layout: Option<&wgpu::PipelineLayout>,
    source: &str,
    fragment: &str,
    samples: u32,
    blend: bool,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("obscur-lyon-blur"),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    });
    let composite = samples > 1;
    let stencil = wgpu::StencilFaceState {
        compare: wgpu::CompareFunction::Equal,
        ..Default::default()
    };
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("obscur-lyon-blur"),
        layout,
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vertex"),
            compilation_options: Default::default(),
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some(fragment),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: blend.then_some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: Default::default(),
        depth_stencil: composite.then_some(wgpu::DepthStencilState {
            format: STENCIL,
            depth_write_enabled: None,
            depth_compare: None,
            stencil: wgpu::StencilState {
                front: stencil,
                back: stencil,
                read_mask: 0xff,
                write_mask: 0,
            },
            bias: Default::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: samples,
            ..Default::default()
        },
        multiview_mask: None,
        cache: None,
    })
}
