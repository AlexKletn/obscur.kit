//! Store resolved pixels and use transient multisampling to refresh dirty regions.

use super::target_extent::rounded_target_extent;

mod damage_scratch;

use damage_scratch::DamageScratch;

use crate::renderer::VelloFrame;
use crate::renderer::premultiply_blit::PremultiplyBlitter;

use super::lyon_renderer::{projection, raster_pass, transform_binding};
use super::mesh::DrawParameters;
use super::pipelines::{SAMPLES, STENCIL};

pub(super) struct ResolvedDamageCache {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    blitter: PremultiplyBlitter,
    scratch: Option<DamageScratch>,
    transforms: Vec<(wgpu::Buffer, wgpu::BindGroup)>,
    rows: Vec<[[f32; 4]; 2]>,
}

impl ResolvedDamageCache {
    pub(super) fn new(
        device: &wgpu::Device,
        extent: (u32, u32),
        format: wgpu::TextureFormat,
    ) -> Self {
        let texture = texture(
            device,
            extent,
            format,
            1,
            wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
        );
        let view = texture.create_view(&Default::default());
        Self {
            texture,
            view,
            blitter: PremultiplyBlitter::with_alpha_conversion(device, format, false),
            scratch: None,
            transforms: Vec::new(),
            rows: Vec::new(),
        }
    }

    pub(super) fn matches(&self, extent: (u32, u32)) -> bool {
        (self.texture.width(), self.texture.height()) == extent
    }

    pub(super) fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    pub(super) fn encode_regions(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        layout: &wgpu::BindGroupLayout,
        gradients: &wgpu::Buffer,
        parameters: &[DrawParameters],
        regions: &[[u32; 4]],
        draw: impl Fn(&mut wgpu::RenderPass<'_>, &wgpu::BindGroup),
    ) {
        let Some((width, height)) = regions
            .iter()
            .map(|r| (r[2], r[3]))
            .reduce(|(w, h), (next_w, next_h)| (w.max(next_w), h.max(next_h)))
        else {
            return;
        };
        let needed = (
            rounded_target_extent(width, self.texture.width()),
            rounded_target_extent(height, self.texture.height()),
        );
        if self
            .scratch
            .as_ref()
            .is_none_or(|slot| slot.extent.0 < needed.0 || slot.extent.1 < needed.1)
        {
            self.scratch = Some(DamageScratch::new(device, needed, self.texture.format()));
        }
        let scratch = self.scratch.as_ref().expect("allocated region scratch");
        if self.transforms.len() < regions.len() {
            self.transforms.resize_with(regions.len(), || {
                transform_binding(device, layout, gradients)
            });
        }
        for (&[x, y, w, h], (buffer, binding)) in regions.iter().zip(&self.transforms) {
            self.rows.clear();
            self.rows.extend(
                parameters
                    .iter()
                    .map(|parameter| projection(parameter, [x, y], (w, h))),
            );
            queue.write_buffer(buffer, 0, bytemuck::cast_slice(&self.rows));
            {
                let mut pass = raster_pass(
                    encoder,
                    &scratch.color,
                    &scratch.stencil,
                    Some(&scratch.resolve_view),
                    true,
                    true,
                );
                #[allow(clippy::cast_precision_loss)]
                pass.set_viewport(0.0, 0.0, w as f32, h as f32, 0.0, 1.0);
                pass.set_scissor_rect(0, 0, w, h);
                draw(&mut pass, binding);
            }
            encoder.copy_texture_to_texture(
                scratch.resolved.as_image_copy(),
                wgpu::TexelCopyTextureInfo {
                    origin: wgpu::Origin3d { x, y, z: 0 },
                    ..self.texture.as_image_copy()
                },
                wgpu::Extent3d {
                    width: w,
                    height: h,
                    depth_or_array_layers: 1,
                },
            );
        }
    }

    pub(super) fn output(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        frame: &VelloFrame,
    ) {
        if frame.texture().format() == self.texture.format()
            && frame
                .texture()
                .usage()
                .contains(wgpu::TextureUsages::COPY_DST)
        {
            encoder.copy_texture_to_texture(
                self.texture.as_image_copy(),
                frame.texture().as_image_copy(),
                self.texture.size(),
            );
        } else {
            self.blitter.copy(device, encoder, &self.view, frame.view());
        }
    }
}

fn texture(
    device: &wgpu::Device,
    (width, height): (u32, u32),
    format: wgpu::TextureFormat,
    sample_count: u32,
    usage: wgpu::TextureUsages,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("obscur-lyon-damage-cache"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage,
        view_formats: &[],
    })
}
