#![doc = include_str!("vello_frame.doc/_module.md")]

#[doc = include_str!("vello_frame.doc/VelloFrame.md")]
#[derive(Clone)]
pub struct VelloFrame {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    width: u32,
    height: u32,
}

impl VelloFrame {
    #[doc = include_str!("vello_frame.doc/new_offscreen.md")]
    #[must_use]
    pub fn new_offscreen(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let width = width.max(1);
        let height = height.max(1);
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("obscur-vello-frame"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Self {
            texture,
            view,
            width,
            height,
        }
    }

    #[doc = include_str!("vello_frame.doc/texture.md")]
    #[must_use]
    pub const fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    #[doc = include_str!("vello_frame.doc/view.md")]
    #[must_use]
    pub const fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    #[doc = include_str!("vello_frame.doc/width.md")]
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    #[doc = include_str!("vello_frame.doc/height.md")]
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }

    #[doc = include_str!("vello_frame.doc/size.md")]
    #[must_use]
    pub const fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

impl std::fmt::Debug for VelloFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // `wgpu::TextureView` is not `Debug`; summarise the extent instead.
        f.debug_struct("VelloFrame")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("format", &"Rgba8Unorm")
            .finish_non_exhaustive()
    }
}
