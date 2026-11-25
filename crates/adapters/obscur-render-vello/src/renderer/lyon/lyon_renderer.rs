//! Owns the Lyon backend cache and submits frames to wgpu.

use std::collections::HashSet;
use std::sync::Arc;

use obscur_scene::command::DisplayList;

use crate::conversion::SceneConverter;
use crate::renderer::VelloFrame;

use super::geometry::GeometryCache;
use super::gpu_stats::GpuStats;
use super::gradient::GRADIENT_ROWS;
use super::group_clip_options::GroupClipOptions;
use super::group_compositor::GroupCompositor;
use super::group_crop::GroupCrop;
use super::group_schedule::GroupSchedule;
use super::image_cache::ImageCache;
use super::mesh::{Draw, MAX_DRAW_PARAMETERS, Mesh, Operation, pixel_bounds};
use super::mesh_upload::MeshBuffer;
use super::packed_upload::PackedUpload;
use super::partial_redraw::{self, Mode};
use super::pipelines::{Pipelines, SAMPLES, STENCIL};
use super::resolved_damage_cache::ResolvedDamageCache;
use super::upload_cache::UploadCache;

#[derive(Default)]
pub(crate) struct LyonRenderer {
    geometry: GeometryCache,
    gpu_stats: Option<Option<GpuStats>>,
    images: ImageCache,
    mesh: Mesh,
    prepared: Option<((u32, u32), Arc<DisplayList>)>,
    pipelines: Option<Pipelines>,
    target_config: Option<(wgpu::TextureFormat, u32)>,
    attachments: Option<((u32, u32), Option<wgpu::TextureView>, wgpu::TextureView)>,
    group_clip_options: GroupClipOptions,
    groups: Option<Result<GroupCompositor, &'static str>>,
    reported_fallbacks: HashSet<&'static str>,
    vertices: MeshBuffer,
    indices: MeshBuffer,
    gradient_upload: UploadCache,
    packed_upload: PackedUpload,
    transforms: Option<(wgpu::Buffer, wgpu::Buffer, wgpu::BindGroup)>,
    transform_rows: Vec<[[f32; 4]; 2]>,
    crop_transform: Option<(wgpu::Buffer, wgpu::BindGroup)>,
    crop_rows: Vec<[[f32; 4]; 2]>,
    retained_color: Option<((u32, u32), Arc<DisplayList>)>,
    damage_clear: Option<wgpu::RenderPipeline>,
    resolved_damage: Option<ResolvedDamageCache>,
}

impl LyonRenderer {
    pub(in crate::renderer) fn note_fallback(&mut self, reason: &'static str) -> bool {
        self.reported_fallbacks.insert(reason)
    }

    pub(in crate::renderer) fn text_bounds(&self, extent: (u32, u32)) -> Vec<[u32; 4]> {
        self.mesh
            .text_vertices
            .iter()
            .map(|range| {
                pixel_bounds(
                    &self.mesh.vertices[range.clone()],
                    &self.mesh.parameters,
                    extent,
                )
            })
            .collect()
    }

    pub(in crate::renderer) fn invalidate_fonts(&mut self) {
        self.geometry = GeometryCache::default();
        self.prepared = None;
        self.retained_color = None;
    }

    pub(in crate::renderer) fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        frame: &VelloFrame,
        list: &Arc<DisplayList>,
        fonts: &SceneConverter,
    ) -> Result<(), &'static str> {
        static CROP_GROUPS: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        let crop_groups = *CROP_GROUPS.get_or_init(|| {
            std::env::var_os("OBSCUR_LYON_CROP_GROUPS").is_some_and(|value| value == "1")
        });
        static SCHEDULE_GROUPS: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        let schedule_groups = *SCHEDULE_GROUPS.get_or_init(|| {
            std::env::var_os("OBSCUR_LYON_SCHEDULE_GROUPS").is_some_and(|value| value == "1")
        });
        self.render_with_group_options(
            device,
            queue,
            frame,
            list,
            fonts,
            crop_groups,
            schedule_groups,
        )
    }

    fn render_with_group_options(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        frame: &VelloFrame,
        list: &Arc<DisplayList>,
        fonts: &SceneConverter,
        crop_groups: bool,
        schedule_groups: bool,
    ) -> Result<(), &'static str> {
        let size = frame.size();
        let format = frame.format();
        let retained = self.retained_color.take();
        let mut geometry_changed = false;
        let mut reuse_mesh = false;
        let mut mesh_rebuilt = false;
        if !self.prepared.as_ref().is_some_and(|(extent, previous)| {
            *extent == size && (Arc::ptr_eq(previous, list) || previous.as_ref() == list.as_ref())
        }) {
            let previous = self.prepared.take();
            reuse_mesh = previous.as_ref().is_some_and(|(extent, previous)| {
                *extent == size
                    && self
                        .mesh
                        .update_parameters(previous, list, &mut geometry_changed)
            });
            if !reuse_mesh {
                self.mesh.lower(list, size, &mut self.geometry, fonts)?;
                mesh_rebuilt = true;
                geometry_changed = true;
            }
        }
        let grouped = self.mesh.group_depth != 0 || self.mesh.needs_backdrop;
        let samples = if !grouped && single_sample_requested() {
            1
        } else {
            SAMPLES
        };
        let target_changed = self.target_config != Some((format, samples));
        if target_changed {
            self.target_config = Some((format, samples));
            self.pipelines = None;
            self.damage_clear = None;
            self.resolved_damage = None;
            self.attachments = None;
            self.groups = None;
            self.transforms = None;
            self.crop_transform = None;
            self.crop_rows.clear();
            self.transform_rows.clear();
            self.gradient_upload = UploadCache::default();
            self.images = ImageCache::default();
            self.prepared = None;
            geometry_changed = true;
        }
        let pipelines = self
            .pipelines
            .get_or_insert_with(|| Pipelines::new(device, format, samples));
        let (transform_buffer, gradient_buffer, transform_bind_group) = self
            .transforms
            .get_or_insert_with(|| create_transform_buffer(device, &pipelines.transforms));
        if mesh_rebuilt || target_changed {
            self.images
                .prepare(device, queue, &pipelines.images, &self.mesh.images)?;
        }
        if self.prepared.is_none() {
            let mut changed = self.transform_rows.len() != self.mesh.parameters.len();
            self.transform_rows
                .resize(self.mesh.parameters.len(), [[0.0; 4]; 2]);
            for (rows, parameters) in self.transform_rows.iter_mut().zip(&self.mesh.parameters) {
                let updated = projection(parameters, [0, 0], size);
                changed |= *rows != updated;
                *rows = updated;
            }
            if changed {
                queue.write_buffer(
                    transform_buffer,
                    0,
                    bytemuck::cast_slice(&self.transform_rows),
                );
            }
        }

        if self
            .attachments
            .as_ref()
            .is_some_and(|(extent, ..)| *extent != size)
        {
            self.attachments = None;
        }
        let (_, color, stencil) = self.attachments.get_or_insert_with(|| {
            (
                size,
                (samples > 1).then(|| attachment(device, size, format, samples)),
                attachment(device, size, STENCIL, samples),
            )
        });
        let crop = (grouped && crop_groups)
            .then(|| GroupCrop::for_draws(&self.mesh.draws, size))
            .flatten();
        let schedule = (grouped && schedule_groups && crop.is_none())
            .then(|| GroupSchedule::for_draws(&self.mesh.draws, size, self.group_clip_options))
            .flatten();
        static SCHEDULED_CROP: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        let cropped_schedule = schedule.as_ref().filter(|_| {
            *SCHEDULED_CROP.get_or_init(|| {
                std::env::var_os("OBSCUR_LYON_CROP_SCHEDULED_GROUPS")
                    .is_some_and(|value| value == "1")
            })
        });
        let crop_binding = crop.as_ref().map(|crop| {
            let (buffer, binding) = self.crop_transform.get_or_insert_with(|| {
                transform_binding(device, &pipelines.transforms, gradient_buffer)
            });
            let mut changed = self.crop_rows.len() != self.mesh.parameters.len();
            self.crop_rows
                .resize(self.mesh.parameters.len(), [[0.0; 4]; 2]);
            for (rows, parameters) in self.crop_rows.iter_mut().zip(&self.mesh.parameters) {
                let updated = projection(parameters, crop.origin, crop.extent);
                changed |= *rows != updated;
                *rows = updated;
            }
            if changed {
                queue.write_buffer(buffer, 0, bytemuck::cast_slice(&self.crop_rows));
            }
            &*binding
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("obscur-lyon"),
        });
        // Validate and allocate group resources before opening a profiler query.
        let groups = if grouped {
            let groups = self.groups.get_or_insert_with(|| {
                GroupCompositor::new(device, format, self.group_clip_options)
            });
            let groups = groups.as_mut().map_err(|error| *error)?;
            groups.prepare(
                device,
                size,
                format,
                if cropped_schedule.is_some() {
                    0
                } else {
                    schedule
                        .as_ref()
                        .map_or(self.mesh.group_depth, |plan| plan.groups.len())
                },
                crop.as_ref(),
                queue,
                &self.mesh.color_filters,
                self.mesh.blur_count > 0 || self.mesh.needs_backdrop,
                color
                    .as_ref()
                    .filter(|_| self.mesh.needs_backdrop || self.mesh.needs_blend),
                self.mesh.needs_blend,
            )?;
            if let Some(schedule) = cropped_schedule {
                groups.prepare_scheduled_crops(
                    device,
                    format,
                    schedule,
                    (&pipelines.transforms, gradient_buffer),
                );
            }
            Some(groups)
        } else {
            None
        };
        if geometry_changed {
            let vertices = bytemuck::cast_slice(&self.mesh.vertices);
            let indices = bytemuck::cast_slice(&self.mesh.indices);
            let gradients = bytemuck::cast_slice(&self.mesh.gradients.rows);
            // Check both destination sizes before committing either upload cache.
            let limit = device.limits().max_buffer_size;
            if vertices.len() as u64 > limit || indices.len() as u64 > limit {
                return Err("mesh exceeds the device buffer limit");
            }
            let vertex_bytes = self.vertices.upload(
                device,
                queue,
                &mut self.packed_upload,
                wgpu::BufferUsages::VERTEX,
                vertices,
            );
            let index_bytes = self.indices.upload(
                device,
                queue,
                &mut self.packed_upload,
                wgpu::BufferUsages::INDEX,
                indices,
            );
            let gradient_bytes = self.gradient_upload.write(
                queue,
                &mut self.packed_upload,
                gradient_buffer,
                gradients,
            );
            static TRACE_MESH: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
            if *TRACE_MESH.get_or_init(|| {
                std::env::var_os("OBSCUR_GPU_STATS").is_some_and(|value| value == "1")
            }) {
                if let Some(crop) = &crop {
                    println!(
                        "[lyon-group-crop] origin={},{} extent={}x{} frame={}x{}",
                        crop.origin[0],
                        crop.origin[1],
                        crop.extent.0,
                        crop.extent.1,
                        size.0,
                        size.1,
                    );
                }
                println!(
                    "[lyon-upload] retained={reuse_mesh} samples={samples} vertices={vertex_bytes}/{} indices={index_bytes}/{} gradients={gradient_bytes}/{}",
                    vertices.len(),
                    indices.len(),
                    gradients.len()
                );
            }
            self.packed_upload.encode(device, queue, &mut encoder);
        }
        if self.prepared.is_none() {
            self.prepared = Some((size, Arc::clone(list)));
        }
        let draw_using = |pass: &mut wgpu::RenderPass<'_>,
                          draws: &[Draw],
                          transforms: &wgpu::BindGroup| {
            if let (Some(vertices), Some(indices)) = (self.vertices.buffer(), self.indices.buffer())
            {
                pass.set_bind_group(0, transforms, &[]);
                pass.set_vertex_buffer(0, vertices.slice(..));
                pass.set_index_buffer(indices.slice(..), wgpu::IndexFormat::Uint32);
                let mut operation = None;
                let mut depth = None;
                for draw in draws {
                    if operation != Some(draw.operation) {
                        pass.set_pipeline(pipelines.for_operation(draw.operation));
                        operation = Some(draw.operation);
                        if let Operation::Image(index) = draw.operation {
                            pass.set_bind_group(1, self.images.binding(index), &[]);
                        }
                    }
                    if depth != Some(draw.depth) {
                        pass.set_stencil_reference(draw.depth);
                        depth = Some(draw.depth);
                    }
                    pass.draw_indexed(draw.indices.clone(), 0, 0..1);
                }
            }
        };
        let draw = |pass: &mut wgpu::RenderPass<'_>, draws: &[Draw], cropped: bool| {
            let transforms = if cropped {
                crop_binding.expect("prepared crop transforms")
            } else {
                &*transform_bind_group
            };
            draw_using(pass, draws, transforms);
        };
        let partial_mode = if groups.is_none() && samples == SAMPLES {
            partial_redraw::mode()
        } else {
            Mode::Disabled
        };
        let damage = retained
            .filter(|(extent, _)| {
                partial_mode != Mode::Disabled && !target_changed && *extent == size
            })
            .and_then(|(_, previous)| partial_redraw::regions(&previous, list, size));
        let stats = self.gpu_stats.get_or_insert_with(|| GpuStats::new(device));
        let query = stats.as_mut().map(|stats| stats.begin(&mut encoder));
        let group_stats = stats
            .as_ref()
            .zip(query.as_ref())
            .filter(|(stats, _)| stats.group_timing_enabled());
        if let Some(groups) = groups {
            let root = color.as_ref().expect("groups use MSAA");
            if let Some(schedule) = &schedule {
                groups.encode_scheduled(
                    device,
                    format,
                    queue,
                    transform_bind_group,
                    &mut encoder,
                    root,
                    stencil,
                    frame.view(),
                    &self.mesh,
                    &self.transform_rows,
                    schedule,
                    group_stats,
                    draw_using,
                );
            } else {
                groups.encode(
                    device,
                    &mut encoder,
                    root,
                    stencil,
                    frame.view(),
                    &self.mesh.draws,
                    crop.as_ref(),
                    group_stats,
                    draw,
                );
            }
        } else if partial_mode == Mode::Resolved {
            if self
                .resolved_damage
                .as_ref()
                .is_none_or(|cache| !cache.matches(size))
            {
                self.resolved_damage = Some(ResolvedDamageCache::new(device, size, format));
            }
            let cache = self
                .resolved_damage
                .as_mut()
                .expect("allocated damage cache");
            if let Some(regions) = &damage {
                cache.encode_regions(
                    device,
                    queue,
                    &mut encoder,
                    &pipelines.transforms,
                    gradient_buffer,
                    &self.mesh.parameters,
                    regions,
                    |pass, transforms| draw_using(pass, &self.mesh.draws, transforms),
                );
            } else {
                draw(
                    &mut raster_pass(
                        &mut encoder,
                        color.as_ref().expect("MSAA color"),
                        stencil,
                        Some(cache.view()),
                        true,
                        true,
                    ),
                    &self.mesh.draws,
                    false,
                );
            }
            cache.output(device, &mut encoder, frame);
            self.retained_color = Some((size, Arc::clone(list)));
        } else if partial_mode == Mode::Multisampled {
            let target = color.as_ref().expect("retained MSAA color");
            let mut pass = raster_pass_with_storage(
                &mut encoder,
                target,
                stencil,
                Some(frame.view()),
                damage.is_none(),
                true,
                true,
            );
            if let Some(regions) = &damage {
                if !regions.is_empty() {
                    let clear = self
                        .damage_clear
                        .get_or_insert_with(|| partial_redraw::clear_pipeline(device, format));
                    for [x, y, w, h] in regions.iter().copied() {
                        pass.set_scissor_rect(x, y, w, h);
                        pass.set_pipeline(clear);
                        pass.draw(0..3, 0..1);
                        draw(&mut pass, &self.mesh.draws, false);
                    }
                }
            } else {
                draw(&mut pass, &self.mesh.draws, false);
            }
            self.retained_color = Some((size, Arc::clone(list)));
        } else {
            let (target, resolve) = color
                .as_ref()
                .map_or((frame.view(), None), |msaa| (msaa, Some(frame.view())));
            draw(
                &mut raster_pass(&mut encoder, target, stencil, resolve, true, true),
                &self.mesh.draws,
                false,
            );
        }
        if partial_mode != Mode::Disabled && partial_redraw::trace_enabled() {
            if let Some(regions) = &damage {
                let area: u64 = regions
                    .iter()
                    .map(|r| u64::from(r[2]) * u64::from(r[3]))
                    .sum();
                println!(
                    "[lyon-partial] regions={} pixels={area} extent={}x{}",
                    regions.len(),
                    size.0,
                    size.1
                );
            } else {
                println!("[lyon-partial] full=true extent={}x{}", size.0, size.1);
            }
        }
        if let (Some(stats), Some(query)) = (stats.as_mut(), query) {
            stats.resolve(&mut encoder, query);
        }
        queue.submit([encoder.finish()]);
        if let Some(stats) = stats {
            stats.submitted(device, queue);
        }
        Ok(())
    }
}

fn single_sample_requested() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED
        .get_or_init(|| std::env::var_os("OBSCUR_LYON_SAMPLES").is_some_and(|value| value == "1"))
}

pub(super) fn raster_pass<'a>(
    encoder: &'a mut wgpu::CommandEncoder,
    color: &wgpu::TextureView,
    stencil: &wgpu::TextureView,
    resolve_target: Option<&wgpu::TextureView>,
    clear_color: bool,
    clear_stencil: bool,
) -> wgpu::RenderPass<'a> {
    raster_pass_with_storage(
        encoder,
        color,
        stencil,
        resolve_target,
        clear_color,
        clear_stencil,
        false,
    )
}

fn raster_pass_with_storage<'a>(
    encoder: &'a mut wgpu::CommandEncoder,
    color: &wgpu::TextureView,
    stencil: &wgpu::TextureView,
    resolve_target: Option<&wgpu::TextureView>,
    clear_color: bool,
    clear_stencil: bool,
    retain_color: bool,
) -> wgpu::RenderPass<'a> {
    let store = if resolve_target.is_some() {
        wgpu::StoreOp::Discard
    } else {
        wgpu::StoreOp::Store
    };
    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: color,
            depth_slice: None,
            resolve_target,
            ops: wgpu::Operations {
                load: if clear_color {
                    wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT)
                } else {
                    wgpu::LoadOp::Load
                },
                store: if retain_color {
                    wgpu::StoreOp::Store
                } else {
                    store
                },
            },
        })],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: stencil,
            depth_ops: None,
            stencil_ops: Some(wgpu::Operations {
                load: if clear_stencil {
                    wgpu::LoadOp::Clear(0)
                } else {
                    wgpu::LoadOp::Load
                },
                store,
            }),
        }),
        ..Default::default()
    })
}

pub(super) fn attachment(
    device: &wgpu::Device,
    size: (u32, u32),
    format: wgpu::TextureFormat,
    samples: u32,
) -> wgpu::TextureView {
    device
        .create_texture(&wgpu::TextureDescriptor {
            label: Some("obscur-lyon-msaa"),
            size: wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: samples,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | if format == STENCIL {
                    wgpu::TextureUsages::empty()
                } else {
                    wgpu::TextureUsages::TEXTURE_BINDING
                },
            view_formats: &[],
        })
        .create_view(&wgpu::TextureViewDescriptor::default())
}

#[allow(clippy::cast_precision_loss)]
pub(super) fn projection(
    parameters: &super::mesh::DrawParameters,
    [left, top]: [u32; 2],
    (width, height): (u32, u32),
) -> [[f32; 4]; 2] {
    let (x, y) = (2.0 / width as f32, -2.0 / height as f32);
    let [a, b, c, d, e, f] = parameters.transform.coeffs;
    [
        [
            x * a,
            x * c,
            x * (e - left as f32) - 1.0,
            parameters.opacity,
        ],
        [y * b, y * d, y * (f - top as f32) + 1.0, 0.0],
    ]
}

fn create_transform_buffer(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
) -> (wgpu::Buffer, wgpu::Buffer, wgpu::BindGroup) {
    let gradient_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("obscur-lyon-gradients"),
        size: (GRADIENT_ROWS * std::mem::size_of::<[f32; 4]>()) as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let (buffer, binding) = transform_binding(device, layout, &gradient_buffer);
    (buffer, gradient_buffer, binding)
}

pub(super) fn transform_binding(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    gradient_buffer: &wgpu::Buffer,
) -> (wgpu::Buffer, wgpu::BindGroup) {
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("obscur-lyon-transforms"),
        size: (MAX_DRAW_PARAMETERS * std::mem::size_of::<[[f32; 4]; 2]>()) as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("obscur-lyon-transforms"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: gradient_buffer.as_entire_binding(),
            },
        ],
    });
    (buffer, bind_group)
}

#[cfg(test)]
#[path = "surface_resolve_test.rs"]
mod surface_resolve_test;
