//! Pipelines for vector fills and nested stencil clips.

use super::mesh::{Operation, Vertex};

pub(super) const SAMPLES: u32 = 4;
pub(super) const STENCIL: wgpu::TextureFormat = wgpu::TextureFormat::Stencil8;

pub(super) struct Pipelines {
    pub(super) transforms: wgpu::BindGroupLayout,
    pub(super) images: wgpu::BindGroupLayout,
    image: wgpu::RenderPipeline,
    paint: wgpu::RenderPipeline,
    push: wgpu::RenderPipeline,
    pop: wgpu::RenderPipeline,
}

impl Pipelines {
    pub(super) fn new(device: &wgpu::Device, format: wgpu::TextureFormat, samples: u32) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("obscur-lyon"),
            source: wgpu::ShaderSource::Wgsl(
                concat!(include_str!("srgb.wgsl"), include_str!("vector.wgsl")).into(),
            ),
        });
        let transforms = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("obscur-lyon-transforms"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
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
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("obscur-lyon"),
            bind_group_layouts: &[Some(&transforms)],
            immediate_size: 0,
        });
        let images = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("obscur-lyon-image"),
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
            ],
        });
        let image_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("obscur-lyon-image"),
            bind_group_layouts: &[Some(&transforms), Some(&images)],
            immediate_size: 0,
        });
        Self {
            transforms,
            images,
            image: pipeline(
                device,
                &shader,
                &image_layout,
                format,
                samples,
                Operation::Image(0),
            ),
            paint: pipeline(device, &shader, &layout, format, samples, Operation::Paint),
            push: pipeline(device, &shader, &layout, format, samples, Operation::Push),
            pop: pipeline(device, &shader, &layout, format, samples, Operation::Pop),
        }
    }

    pub(super) fn for_operation(&self, operation: Operation) -> &wgpu::RenderPipeline {
        match operation {
            Operation::Paint => &self.paint,
            Operation::Image(_) => &self.image,
            Operation::Push => &self.push,
            Operation::Pop => &self.pop,
            Operation::BeginGroup | Operation::EndGroup { .. } | Operation::Backdrop { .. } => {
                unreachable!("group marker")
            }
        }
    }
}

fn pipeline(
    device: &wgpu::Device,
    shader: &wgpu::ShaderModule,
    layout: &wgpu::PipelineLayout,
    format: wgpu::TextureFormat,
    samples: u32,
    operation: Operation,
) -> wgpu::RenderPipeline {
    const ATTRIBUTES: [wgpu::VertexAttribute; 6] = wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x4, 2 => Float32x4, 3 => Float32x2, 4 => Uint32, 5 => Uint32];
    let paints = matches!(operation, Operation::Paint | Operation::Image(_));
    let pass_op = match operation {
        Operation::Paint | Operation::Image(_) => wgpu::StencilOperation::Keep,
        Operation::Push => wgpu::StencilOperation::IncrementClamp,
        Operation::Pop => wgpu::StencilOperation::DecrementClamp,
        Operation::BeginGroup | Operation::EndGroup { .. } | Operation::Backdrop { .. } => {
            unreachable!("group marker")
        }
    };
    let face = wgpu::StencilFaceState {
        compare: wgpu::CompareFunction::Equal,
        fail_op: wgpu::StencilOperation::Keep,
        depth_fail_op: wgpu::StencilOperation::Keep,
        pass_op,
    };
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("obscur-lyon"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vertex"),
            compilation_options: Default::default(),
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<Vertex>() as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &ATTRIBUTES,
            }],
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some(if matches!(operation, Operation::Image(_)) {
                "fragment_image"
            } else {
                "fragment"
            }),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                write_mask: if paints {
                    wgpu::ColorWrites::ALL
                } else {
                    wgpu::ColorWrites::empty()
                },
            })],
        }),
        primitive: Default::default(),
        depth_stencil: Some(wgpu::DepthStencilState {
            format: STENCIL,
            depth_write_enabled: None,
            depth_compare: None,
            stencil: wgpu::StencilState {
                front: face,
                back: face,
                read_mask: 0xff,
                write_mask: if paints { 0 } else { 0xff },
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
