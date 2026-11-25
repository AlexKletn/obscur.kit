//! A per-frame offscreen texture Vello rasterizes into (spec §5.2, §5.6).

#[doc = include_str!("../../docs/api/renderer/vello_frame.doc/VelloFrame.md")]
#[derive(Clone)]
pub struct VelloFrame {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    format: wgpu::TextureFormat,
    width: u32,
    height: u32,
}

impl VelloFrame {
    #[doc = include_str!("../../docs/api/renderer/vello_frame.doc/new_offscreen.md")]
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
        Self::from_texture(texture, wgpu::TextureFormat::Rgba8Unorm)
    }

    pub(super) fn from_texture(texture: wgpu::Texture, format: wgpu::TextureFormat) -> Self {
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(format),
            ..Default::default()
        });
        Self {
            width: texture.width(),
            height: texture.height(),
            texture,
            view,
            format,
        }
    }

    #[cfg(feature = "lyon")]
    pub(super) const fn format(&self) -> wgpu::TextureFormat {
        self.format
    }

    /// The backing texture.
    #[must_use]
    pub const fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    /// The texture view Vello renders into.
    #[must_use]
    pub const fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    /// The target width, in physical pixels.
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// The target height, in physical pixels.
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// The target extent as a `(width, height)` pair, in physical pixels.
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
            .field("format", &self.format)
            .finish_non_exhaustive()
    }
}
