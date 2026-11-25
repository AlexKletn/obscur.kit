//! Multisampled composition of isolated Lyon groups, including opacity, filters,
//! and blending with the parent target.

mod crop_target;
mod group_composite;
mod scheduled_crop;

use crop_target::CropTarget;
use group_composite::GroupComposite;
use scheduled_crop::ScheduledCrop;

use super::gpu_stats::GpuStats;
use super::group_blend::GroupBlend;
use super::group_clip_options::GroupClipOptions;
use super::group_content_cache::GroupContentCache;
use super::group_crop::GroupCrop;
use super::group_schedule::GroupSchedule;
use super::layer_blur::LayerBlur;
use super::lyon_renderer::{attachment, projection, raster_pass, transform_binding};
use super::mesh::{Draw, MAX_COLOR_FILTERS, Operation};
use super::pipelines::SAMPLES;

pub(super) struct GroupCompositor {
    pipeline: wgpu::RenderPipeline,
    clipped_pipeline: Option<wgpu::RenderPipeline>,
    clip_scissor: bool,
    layout: wgpu::PipelineLayout,
    source_layout: wgpu::BindGroupLayout,
    blur: Option<LayerBlur>,
    blend: Option<Result<GroupBlend, &'static str>>,
    root_source: Option<wgpu::BindGroup>,
    extent: (u32, u32),
    filter_buffer: wgpu::Buffer,
    filters: Vec<[f32; 24]>,
    cropped: Option<CropTarget>,
    content_cache: GroupContentCache,
    root_prefix: Option<(wgpu::TextureView, wgpu::BindGroup)>,
    targets: Vec<Option<(wgpu::TextureView, wgpu::BindGroup)>>,
    scheduled_crops: Vec<Option<ScheduledCrop>>,
}

impl GroupCompositor {
    pub(super) fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        clip_options: GroupClipOptions,
    ) -> Result<Self, &'static str> {
        let validation = device.push_error_scope(wgpu::ErrorFilter::Validation);
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("obscur-lyon-group"),
            source: wgpu::ShaderSource::Wgsl(
                concat!(include_str!("srgb.wgsl"), include_str!("group.wgsl")).into(),
            ),
        });
        let source_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("obscur-lyon-group"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: true,
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
            label: Some("obscur-lyon-group"),
            bind_group_layouts: &[Some(&source_layout)],
            immediate_size: 0,
        });
        let make_pipeline = |stencil| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("obscur-lyon-group"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vertex"),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fragment"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: Default::default(),
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: super::pipelines::STENCIL,
                    depth_write_enabled: None,
                    depth_compare: None,
                    stencil,
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState {
                    count: SAMPLES,
                    ..Default::default()
                },
                multiview_mask: None,
                cache: None,
            })
        };
        let pipeline = make_pipeline(wgpu::StencilState::default());
        let clipped_pipeline = clip_options.defer_outer_clip.then(|| {
            let face = wgpu::StencilFaceState {
                compare: wgpu::CompareFunction::Equal,
                ..Default::default()
            };
            make_pipeline(wgpu::StencilState {
                front: face,
                back: face,
                read_mask: 0xff,
                write_mask: 0,
            })
        });
        if pollster::block_on(validation.pop()).is_some() {
            return Err("multisampled group composition is unavailable");
        }
        let filter_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("obscur-lyon-color-filters"),
            size: (MAX_COLOR_FILTERS * std::mem::size_of::<[f32; 24]>()) as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Ok(Self {
            filter_buffer,
            filters: Vec::new(),
            pipeline,
            clipped_pipeline,
            clip_scissor: clip_options.scissor,
            layout,
            source_layout,
            blur: None,
            blend: None,
            root_source: None,
            extent: (0, 0),
            cropped: None,
            content_cache: GroupContentCache::default(),
            root_prefix: None,
            targets: Vec::new(),
            scheduled_crops: Vec::new(),
        })
    }

    pub(super) fn prepare(
        &mut self,
        device: &wgpu::Device,
        extent: (u32, u32),
        format: wgpu::TextureFormat,
        depth: usize,
        crop: Option<&GroupCrop>,
        queue: &wgpu::Queue,
        filters: &[[f32; 24]],
        needs_blur: bool,
        root: Option<&wgpu::TextureView>,
        needs_blend: bool,
    ) -> Result<(), &'static str> {
        if self.filters != filters {
            self.filters.clear();
            self.filters.extend_from_slice(filters);
            if !filters.is_empty() {
                queue.write_buffer(&self.filter_buffer, 0, bytemuck::cast_slice(filters));
            }
        }
        if self.extent != extent {
            self.extent = extent;
            self.targets.clear();
            self.scheduled_crops.clear();
            self.root_prefix = None;
            self.content_cache.clear();
            self.cropped = None;
            self.blur = None;
            self.blend = None;
            self.root_source = None;
        }
        if needs_blur && self.blur.is_none() {
            self.blur = Some(LayerBlur::new(device, format, extent, &self.layout));
        }
        if let Some(root) = root {
            self.root_source.get_or_insert_with(|| {
                source_binding(device, &self.source_layout, root, &self.filter_buffer)
            });
        }
        if needs_blend {
            self.blend
                .get_or_insert_with(|| {
                    let (view, binding) = group_target(
                        device,
                        extent,
                        format,
                        &self.source_layout,
                        &self.filter_buffer,
                    );
                    GroupBlend::new(device, format, &self.source_layout, view, binding)
                })
                .as_ref()
                .map_err(|reason| *reason)?;
        }
        if let Some(crop) = crop {
            if self
                .cropped
                .as_ref()
                .is_none_or(|target| target.extent != crop.extent)
            {
                let (color, source) = group_target(
                    device,
                    crop.extent,
                    format,
                    &self.source_layout,
                    &self.filter_buffer,
                );
                self.cropped = Some(CropTarget {
                    extent: crop.extent,
                    color,
                    source,
                    stencil: attachment(device, crop.extent, super::pipelines::STENCIL, SAMPLES),
                });
            }
            return Ok(());
        }
        if depth != 0 {
            self.scheduled_crops.clear();
            if self.targets.len() < depth {
                self.targets.resize_with(depth, || None);
            }
            for slot in &mut self.targets[..depth] {
                slot.get_or_insert_with(|| {
                    group_target(
                        device,
                        extent,
                        format,
                        &self.source_layout,
                        &self.filter_buffer,
                    )
                });
            }
        }
        Ok(())
    }

    pub(super) fn encode(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        root: &wgpu::TextureView,
        stencil: &wgpu::TextureView,
        output: &wgpu::TextureView,
        draws: &[Draw],
        crop: Option<&GroupCrop>,
        stats: Option<(&GpuStats, &wgpu_profiler::GpuProfilerQuery)>,
        draw: impl Fn(&mut wgpu::RenderPass<'_>, &[Draw], bool),
    ) {
        self.content_cache.clear();
        let cropped = crop.map(|_| self.cropped.as_ref().expect("prepared crop target"));
        let target = |depth: usize| {
            if depth == 0 {
                root
            } else if let Some(cropped) = cropped {
                &cropped.color
            } else {
                &self.targets[depth - 1].as_ref().expect("prepared group").0
            }
        };
        let parent_source = |depth: usize| {
            if depth == 0 {
                self.root_source.as_ref().expect("prepared root source")
            } else {
                &self.targets[depth - 1].as_ref().expect("prepared group").1
            }
        };
        let mut depth = 0;
        let mut start = 0;
        let mut clear_color = true;
        let mut clear_stencil = true;
        let mut composite = None;
        // The stencil is shared across groups. Draws obey the same ancestor
        // clips as the parent and the composite reads each MSAA sample separately.
        for (end, marker) in draws.iter().enumerate() {
            if !matches!(
                marker.operation,
                Operation::BeginGroup | Operation::EndGroup { .. } | Operation::Backdrop { .. }
            ) {
                continue;
            }
            if clear_color || start != end || composite.is_some() {
                let label = if depth == 0 { "root" } else { "group" };
                GpuStats::group_pass(stats, encoder, label, |encoder| {
                    let cropped = cropped.filter(|_| depth != 0);
                    let mut pass = raster_pass(
                        encoder,
                        target(depth),
                        cropped.map_or(stencil, |target| &target.stencil),
                        None,
                        clear_color,
                        cropped.is_some() || clear_stencil,
                    );
                    self.draw_segment(
                        &mut pass,
                        composite.take(),
                        &draws[start..end],
                        |pass, draws| draw(pass, draws, cropped.is_some()),
                    );
                });
                clear_stencil = false;
            }
            start = end + 1;
            match marker.operation {
                Operation::BeginGroup => {
                    depth += 1;
                    clear_color = true;
                }
                Operation::EndGroup {
                    encode_srgb,
                    blend,
                    opacity,
                    bounds,
                    color_filter,
                    blur_radius,
                    stencil_depth,
                } => {
                    depth -= 1;
                    clear_color = false;
                    if bounds[2] == 0 || bounds[3] == 0 || (opacity == 0.0 && blend.is_none()) {
                        continue;
                    }
                    let source = cropped.map_or_else(
                        || &self.targets[depth].as_ref().expect("prepared group").1,
                        |target| &target.source,
                    );
                    let mut parameters =
                        color_filter.map_or(opacity.to_bits(), |slot| slot | (1 << 31));
                    if blur_radius > 0.0 || blend.is_some() {
                        GpuStats::group_pass(stats, encoder, "effect", |encoder| {
                            if blur_radius > 0.0 {
                                self.blur.as_ref().expect("prepared blur").encode(
                                    device,
                                    encoder,
                                    source,
                                    if blend.is_some() {
                                        target(depth + 1)
                                    } else {
                                        target(depth)
                                    },
                                    stencil,
                                    stencil_depth,
                                    bounds,
                                    parameters,
                                    blur_radius,
                                    blend.is_some(),
                                );
                                parameters = 1.0_f32.to_bits();
                            }
                            if let Some(mode) = blend {
                                self.blend
                                    .as_ref()
                                    .and_then(|blend| blend.as_ref().ok())
                                    .expect("prepared blend")
                                    .encode(
                                        encoder,
                                        target(depth),
                                        source,
                                        parent_source(depth),
                                        stencil,
                                        stencil_depth,
                                        bounds,
                                        parameters,
                                        mode,
                                    );
                            }
                        });
                    } else {
                        // Apply the composite in the next raster pass on this target.
                        // That pass can also resolve the root when this is the last group.
                        composite = Some(GroupComposite {
                            source,
                            bounds,
                            parameters,
                            vertex_start: crop
                                .map_or(if encode_srgb { 3 } else { 0 }, GroupCrop::vertex_start),
                            stencil_depth: None,
                        });
                    }
                }
                Operation::Backdrop {
                    sample_bounds,
                    radius,
                    stencil_depth,
                } => {
                    clear_color = false;
                    GpuStats::group_pass(stats, encoder, "effect", |encoder| {
                        self.blur.as_ref().expect("prepared blur").encode(
                            device,
                            encoder,
                            parent_source(depth),
                            target(depth),
                            stencil,
                            stencil_depth,
                            sample_bounds,
                            1.0_f32.to_bits(),
                            radius,
                            true,
                        );
                    });
                }
                _ => unreachable!("filtered to group markers"),
            }
        }
        GpuStats::group_pass(stats, encoder, "root", |encoder| {
            let mut pass = raster_pass(
                encoder,
                root,
                stencil,
                Some(output),
                clear_color,
                clear_stencil,
            );
            self.draw_segment(&mut pass, composite, &draws[start..], |pass, draws| {
                draw(pass, draws, false)
            });
        });
    }

    pub(super) fn prepare_scheduled_crops(
        &mut self,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        schedule: &GroupSchedule,
        transforms: (&wgpu::BindGroupLayout, &wgpu::Buffer),
    ) {
        let count = schedule.groups.len();
        self.targets.resize_with(count, || None);
        self.scheduled_crops.resize_with(count, || None);
        for (index, group) in schedule.groups.iter().enumerate() {
            let region = group
                .composite_clip()
                .and_then(|_| GroupCrop::from_bounds(group.bounds, self.extent));
            let crop = &mut self.scheduled_crops[index];
            let mut changed = crop.as_ref().map(|crop| crop.region) != region;
            if let Some(region) = region {
                self.targets[index] = None;
                if crop
                    .as_ref()
                    .is_none_or(|crop| crop.region.extent != region.extent)
                {
                    let (color, source) = group_target(
                        device,
                        region.extent,
                        format,
                        &self.source_layout,
                        &self.filter_buffer,
                    );
                    *crop = Some(ScheduledCrop {
                        region,
                        target: CropTarget {
                            extent: region.extent,
                            color,
                            source,
                            stencil: attachment(
                                device,
                                region.extent,
                                super::pipelines::STENCIL,
                                SAMPLES,
                            ),
                        },
                        transforms: transform_binding(device, transforms.0, transforms.1),
                    });
                } else if let Some(crop) = crop.as_mut() {
                    crop.region = region;
                }
            } else {
                *crop = None;
                if self.targets[index].is_none() {
                    self.targets[index] = Some(group_target(
                        device,
                        self.extent,
                        format,
                        &self.source_layout,
                        &self.filter_buffer,
                    ));
                    changed = true;
                }
            }
            if changed {
                self.content_cache.invalidate(index);
            }
        }
    }

    pub(super) fn encode_scheduled(
        &mut self,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        queue: &wgpu::Queue,
        root_transforms: &wgpu::BindGroup,
        encoder: &mut wgpu::CommandEncoder,
        root: &wgpu::TextureView,
        stencil: &wgpu::TextureView,
        output: &wgpu::TextureView,
        mesh: &super::mesh::Mesh,
        transforms: &[[[f32; 4]; 2]],
        schedule: &GroupSchedule,
        stats: Option<(&GpuStats, &wgpu_profiler::GpuProfilerQuery)>,
        draw: impl Fn(&mut wgpu::RenderPass<'_>, &[Draw], &wgpu::BindGroup),
    ) {
        let draws = &mesh.draws;
        let hits = if GroupContentCache::enabled() {
            let hits = self
                .content_cache
                .update(schedule, mesh, transforms, self.extent);
            if stats.is_some() {
                let reused = hits.iter().filter(|hit| hit.is_some()).count();
                let shifted = hits
                    .iter()
                    .flatten()
                    .filter(|&&shift| shift != [0, 0])
                    .count();
                let deferred = schedule
                    .groups
                    .iter()
                    .filter(|group| group.composite_clip().is_some())
                    .count();
                let clipped_hits = hits
                    .iter()
                    .zip(&schedule.groups)
                    .filter(|(hit, group)| hit.is_some() && group.composite_clip().is_some())
                    .count();
                println!(
                    "[lyon-group-cache] reused={reused} groups={} shifted={shifted} deferred_clip={deferred} clipped_hits={clipped_hits}",
                    hits.len()
                );
            }
            hits
        } else {
            self.content_cache.clear();
            Vec::new()
        };
        let use_clip_scissor = self.clip_scissor;
        let clip_scissors = (use_clip_scissor || stats.is_some())
            .then(|| schedule.clip_scissors(mesh, self.extent));
        if let Some((_, scissors)) = stats.zip(clip_scissors.as_ref()) {
            let area = |[_, _, width, height]: [u32; 4]| u64::from(width) * u64::from(height);
            let target = u64::from(self.extent.0) * u64::from(self.extent.1);
            for (index, group) in schedule.groups.iter().enumerate() {
                if group.composite_clip().is_some() && group.opacity != 0.0 {
                    println!(
                        "[lyon-group-scissor] group={index} target_pixels={target} original_pixels={} clipped_pixels={} enabled={use_clip_scissor}",
                        area(group.bounds),
                        area(scissors[index]),
                    );
                }
            }
        }
        let scissors = clip_scissors.as_deref().filter(|_| use_clip_scissor);
        let paint =
            |pass: &mut wgpu::RenderPass<'_>, draws: &[Draw]| draw(pass, draws, root_transforms);
        for (index, group) in schedule.groups.iter().enumerate().rev() {
            if hits.get(index).is_some_and(Option::is_some) {
                continue;
            }
            let crop = self.scheduled_crops.get(index).and_then(Option::as_ref);
            let (target, stencil, binding) = if let Some(crop) = crop {
                let rows: Vec<_> = mesh
                    .parameters
                    .iter()
                    .map(|parameters| {
                        projection(parameters, crop.region.origin, crop.region.extent)
                    })
                    .collect();
                queue.write_buffer(&crop.transforms.0, 0, bytemuck::cast_slice(&rows));
                if stats.is_some() {
                    let (w, h) = crop.region.extent;
                    let (full_w, full_h) = self.extent;
                    println!(
                        "[lyon-scheduled-crop] group={index} extent={w}x{h} frame={full_w}x{full_h}"
                    );
                }
                (&crop.target.color, &crop.target.stencil, &crop.transforms.1)
            } else {
                (
                    &self.targets[index].as_ref().expect("prepared group").0,
                    stencil,
                    root_transforms,
                )
            };
            GpuStats::group_pass(stats, encoder, "group", |encoder| {
                let mut pass = raster_pass(encoder, target, stencil, None, true, true);
                if let Some(depth) = group.composite_clip() {
                    let local: Vec<_> = draws[group.draws.clone()]
                        .iter()
                        .map(|draw| Draw {
                            operation: draw.operation,
                            depth: draw.depth - depth,
                            indices: draw.indices.clone(),
                        })
                        .collect();
                    draw(&mut pass, &local, binding);
                    return;
                }
                // Replay ancestor clips into the fresh stencil of this pass.
                for &clip in &group.clips {
                    paint(&mut pass, &draws[clip..clip + 1]);
                }
                self.draw_scheduled_segment(
                    &mut pass,
                    draws,
                    group.draws.clone(),
                    &group.children,
                    schedule,
                    scissors,
                    &hits,
                    &paint,
                );
            });
        }
        let prefix = if GroupContentCache::enabled()
            && super::group_schedule::root_cache_enabled(self.targets.len(), self.extent)
        {
            schedule.roots.first().and_then(|&first| {
                self.content_cache.root_prefix(
                    mesh,
                    transforms,
                    schedule.groups[first].draws.start - 1,
                )
            })
        } else {
            None
        };
        if prefix.is_none() {
            self.root_prefix = None;
        }
        if let Some((end, hit)) = prefix {
            let had_target = self.root_prefix.is_some();
            let (target, _) = self.root_prefix.get_or_insert_with(|| {
                group_target(
                    device,
                    self.extent,
                    format,
                    &self.source_layout,
                    &self.filter_buffer,
                )
            });
            if !hit || !had_target {
                GpuStats::group_pass(stats, encoder, "group", |encoder| {
                    let mut pass = raster_pass(encoder, target, stencil, None, true, true);
                    paint(&mut pass, &draws[..end]);
                });
            }
            if stats.is_some() {
                println!("[lyon-root-cache] draws={end} hit={}", hit && had_target);
            }
        }
        GpuStats::group_pass(stats, encoder, "root", |encoder| {
            let mut pass = raster_pass(encoder, root, stencil, Some(output), true, true);
            if let Some((end, _)) = prefix {
                let (_, source) = self.root_prefix.as_ref().expect("prepared root cache");
                // Read each MSAA sample at its original position before blending the suffix.
                pass.set_pipeline(&self.pipeline);
                pass.set_bind_group(0, source, &[]);
                let opacity = 1.0_f32.to_bits();
                pass.draw(0..3, opacity..opacity + 1);
                let first = &schedule.groups[schedule.roots[0]];
                for &clip in first.clips.iter().take_while(|&&clip| clip < end) {
                    paint(&mut pass, &draws[clip..clip + 1]);
                }
            }
            self.draw_scheduled_segment(
                &mut pass,
                draws,
                prefix.map_or(0, |(end, _)| end)..draws.len(),
                &schedule.roots,
                schedule,
                scissors,
                &hits,
                &paint,
            );
        });
    }

    fn draw_scheduled_segment(
        &self,
        pass: &mut wgpu::RenderPass<'_>,
        draws: &[Draw],
        range: std::ops::Range<usize>,
        children: &[usize],
        schedule: &GroupSchedule,
        scissors: Option<&[[u32; 4]]>,
        hits: &[Option<[i32; 2]>],
        paint: &impl Fn(&mut wgpu::RenderPass<'_>, &[Draw]),
    ) {
        let mut start = range.start;
        for &child in children {
            let group = &schedule.groups[child];
            let begin = group.draws.start - 1;
            self.draw_segment(pass, None, &draws[start..begin], paint);
            let bounds = scissors.map_or(group.bounds, |scissors| scissors[child]);
            if bounds[2] != 0 && bounds[3] != 0 && group.opacity != 0.0 {
                let crop = self.scheduled_crops.get(child).and_then(Option::as_ref);
                self.draw_segment(
                    pass,
                    Some(GroupComposite {
                        source: crop.map_or_else(
                            || &self.targets[child].as_ref().expect("prepared group").1,
                            |crop| &crop.target.source,
                        ),
                        bounds,
                        parameters: group.opacity.to_bits(),
                        vertex_start: crop.map_or_else(
                            || {
                                hits.get(child)
                                    .copied()
                                    .flatten()
                                    .map_or(0, translated_vertex)
                            },
                            |crop| crop.region.vertex_start(),
                        ),
                        stencil_depth: group.composite_clip(),
                    }),
                    &[],
                    paint,
                );
                pass.set_scissor_rect(0, 0, self.extent.0, self.extent.1);
            }
            self.draw_segment(pass, None, &group.popped_clips, paint);
            start = group.draws.end + 1;
        }
        self.draw_segment(pass, None, &draws[start..range.end], paint);
    }

    fn draw_segment(
        &self,
        pass: &mut wgpu::RenderPass<'_>,
        composite: Option<GroupComposite<'_>>,
        draws: &[Draw],
        draw: impl FnOnce(&mut wgpu::RenderPass<'_>, &[Draw]),
    ) {
        if let Some(GroupComposite {
            source,
            bounds: [x, y, width, height],
            parameters,
            vertex_start,
            stencil_depth,
        }) = composite
        {
            pass.set_scissor_rect(x, y, width, height);
            if let Some(depth) = stencil_depth {
                pass.set_pipeline(
                    self.clipped_pipeline
                        .as_ref()
                        .expect("prepared clipped pipeline"),
                );
                pass.set_stencil_reference(depth);
            } else {
                pass.set_pipeline(&self.pipeline);
            }
            pass.set_bind_group(0, source, &[]);
            pass.draw(vertex_start..vertex_start + 3, parameters..parameters + 1);
            if !draws.is_empty() {
                pass.set_scissor_rect(0, 0, self.extent.0, self.extent.1);
            }
        }
        if !draws.is_empty() {
            draw(pass, draws);
        }
    }
}

fn translated_vertex([x, y]: [i32; 2]) -> u32 {
    if x == 0 && y == 0 {
        return 0;
    }
    let x = u32::try_from(x + 8192).expect("bounded translation x");
    let y = u32::try_from(y + 8192).expect("bounded translation y");
    // The high bit distinguishes a signed translation from a cropped origin.
    (2 + (1 << 28) + x + (y << 14)) * 3
}

fn source_binding(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    view: &wgpu::TextureView,
    filters: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("obscur-lyon-group-source"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: filters.as_entire_binding(),
            },
        ],
    })
}

fn group_target(
    device: &wgpu::Device,
    extent: (u32, u32),
    format: wgpu::TextureFormat,
    layout: &wgpu::BindGroupLayout,
    filters: &wgpu::Buffer,
) -> (wgpu::TextureView, wgpu::BindGroup) {
    let view = device
        .create_texture(&wgpu::TextureDescriptor {
            label: Some("obscur-lyon-group"),
            size: wgpu::Extent3d {
                width: extent.0,
                height: extent.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: SAMPLES,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        })
        .create_view(&wgpu::TextureViewDescriptor::default());
    let binding = source_binding(device, layout, &view, filters);
    (view, binding)
}
