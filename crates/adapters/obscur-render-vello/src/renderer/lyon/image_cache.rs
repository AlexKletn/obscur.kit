//! Retain GPU textures for images used by consecutive frames.

use std::borrow::Cow;

use obscur_scene::paint::SceneImage;

#[derive(Default)]
pub(super) struct ImageCache {
    sampler: Option<wgpu::Sampler>,
    bindings: Vec<(SceneImage, wgpu::BindGroup)>,
    previous: Vec<(SceneImage, wgpu::BindGroup)>,
}

impl ImageCache {
    pub(super) fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
        images: &[SceneImage],
    ) -> Result<(), &'static str> {
        let limit = device.limits().max_texture_dimension_2d;
        if images
            .iter()
            .any(|image| image.width().max(image.height()) > limit)
        {
            return Err("image exceeds the device texture limit");
        }
        std::mem::swap(&mut self.bindings, &mut self.previous);
        self.bindings.clear();
        if !images.is_empty() {
            let sampler = self.sampler.get_or_insert_with(|| {
                device.create_sampler(&wgpu::SamplerDescriptor {
                    label: Some("obscur-lyon-image"),
                    mag_filter: wgpu::FilterMode::Linear,
                    min_filter: wgpu::FilterMode::Linear,
                    ..Default::default()
                })
            });
            for image in images {
                let entry =
                    if let Some(index) = self.previous.iter().position(|(old, _)| old == image) {
                        self.previous.swap_remove(index)
                    } else {
                        (image.clone(), upload(device, queue, layout, sampler, image))
                    };
                self.bindings.push(entry);
            }
        }
        self.previous.clear();
        Ok(())
    }

    pub(super) fn binding(&self, index: usize) -> &wgpu::BindGroup {
        &self.bindings[index].1
    }
}

fn upload(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
    image: &SceneImage,
) -> wgpu::BindGroup {
    let size = wgpu::Extent3d {
        width: image.width(),
        height: image.height(),
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("obscur-lyon-image"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let pixels = if image.alpha_mode().is_premultiplied() {
        Cow::Borrowed(image.pixels())
    } else {
        // Premultiply before bilinear filtering so transparent RGB cannot bleed.
        let mut pixels = image.pixels().to_vec();
        for pixel in pixels.chunks_exact_mut(4) {
            let alpha = u16::from(pixel[3]);
            for channel in &mut pixel[..3] {
                *channel = ((u16::from(*channel) * alpha + 127) / 255) as u8;
            }
        }
        Cow::Owned(pixels)
    };
    queue.write_texture(
        texture.as_image_copy(),
        &pixels,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(size.width * 4),
            rows_per_image: Some(size.height),
        },
        size,
    );
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("obscur-lyon-image"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    })
}
