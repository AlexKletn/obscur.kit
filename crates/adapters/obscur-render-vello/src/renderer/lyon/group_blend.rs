//! Per-sample blending of an isolated layer with the paint below it.

use super::pipelines::{SAMPLES, STENCIL};

pub(super) struct GroupBlend {
    scratch: wgpu::TextureView,
    scratch_binding: wgpu::BindGroup,
    blend: wgpu::RenderPipeline,
    replace: wgpu::RenderPipeline,
}

impl GroupBlend {
    pub(super) fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        source_layout: &wgpu::BindGroupLayout,
        scratch: wgpu::TextureView,
        scratch_binding: wgpu::BindGroup,
    ) -> Result<Self, &'static str> {
        let validation = device.push_error_scope(wgpu::ErrorFilter::Validation);
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("obscur-lyon-group-blend"),
            source: wgpu::ShaderSource::Wgsl(
                concat!(
                    include_str!("srgb.wgsl"),
                    include_str!("group.wgsl"),
                    include_str!("group_blend.wgsl")
                )
                .into(),
            ),
        });
        let pipeline = |entry_point, layouts: &[Option<&wgpu::BindGroupLayout>], depth_stencil| {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("obscur-lyon-group-blend"),
                bind_group_layouts: layouts,
                immediate_size: 0,
            });
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("obscur-lyon-group-blend"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vertex"),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some(entry_point),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: Default::default(),
                depth_stencil,
                multisample: wgpu::MultisampleState {
                    count: SAMPLES,
                    ..Default::default()
                },
                multiview_mask: None,
                cache: None,
            })
        };
        let blend = pipeline(
            "blend_fragment",
            &[Some(source_layout), Some(source_layout)],
            None,
        );
        let face = wgpu::StencilFaceState {
            compare: wgpu::CompareFunction::Equal,
            ..Default::default()
        };
        let replace = pipeline(
            "fragment",
            &[Some(source_layout)],
            Some(wgpu::DepthStencilState {
                format: STENCIL,
                depth_write_enabled: None,
                depth_compare: None,
                stencil: wgpu::StencilState {
                    front: face,
                    back: face,
                    read_mask: 0xff,
                    write_mask: 0,
                },
                bias: Default::default(),
            }),
        );
        if pollster::block_on(validation.pop()).is_some() {
            return Err("multisampled layer blending is unavailable");
        }
        Ok(Self {
            scratch,
            scratch_binding,
            blend,
            replace,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn encode(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        source: &wgpu::BindGroup,
        backdrop: &wgpu::BindGroup,
        stencil: &wgpu::TextureView,
        stencil_depth: u32,
        [x, y, width, height]: [u32; 4],
        parameters: u32,
        mode: u32,
    ) {
        // WebGPU cannot sample an active color attachment. Stage the result,
        // then replace the parent samples under the active clip.
        {
            let mut pass = pass(encoder, &self.scratch, None);
            pass.set_scissor_rect(x, y, width, height);
            pass.set_pipeline(&self.blend);
            pass.set_bind_group(0, source, &[]);
            pass.set_bind_group(1, backdrop, &[]);
            pass.draw(mode * 3..mode * 3 + 3, parameters..parameters + 1);
        }
        let mut pass = pass(encoder, target, Some(stencil));
        pass.set_scissor_rect(x, y, width, height);
        pass.set_stencil_reference(stencil_depth);
        pass.set_pipeline(&self.replace);
        pass.set_bind_group(0, &self.scratch_binding, &[]);
        let opacity = 1.0_f32.to_bits();
        pass.draw(0..3, opacity..opacity + 1);
    }
}

fn pass<'a>(
    encoder: &'a mut wgpu::CommandEncoder,
    view: &'a wgpu::TextureView,
    stencil: Option<&'a wgpu::TextureView>,
) -> wgpu::RenderPass<'a> {
    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("obscur-lyon-group-blend"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load: if stencil.is_some() {
                    wgpu::LoadOp::Load
                } else {
                    wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT)
                },
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
