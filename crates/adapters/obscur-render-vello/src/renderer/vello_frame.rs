//! A per-frame offscreen texture Vello rasterizes into (spec §5.2, §5.6).

/// An offscreen [`wgpu::Texture`] and its view, sized for one frame
/// (spec §5.6 offscreen output).
///
/// Vello's [`render_to_texture`](vello::Renderer::render_to_texture) draws into a
/// texture that must have the [`wgpu::TextureFormat::Rgba8Unorm`] format and the
/// [`wgpu::TextureUsages::STORAGE_BINDING`] flag (Vello's fine-rasterization
/// stage writes it with a compute pipeline). This type allocates exactly such a
/// texture plus the matching [`wgpu::TextureView`] Vello binds. Additional
/// usages — render-attachment, texture-binding, and copy-source — let a host
/// blit the result to a surface or read it back for an offscreen/bitmap target.
pub struct VelloFrame {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    width: u32,
    height: u32,
}

impl VelloFrame {
    /// Allocates an offscreen render target of `width` × `height` on `device`.
    ///
    /// The extent is clamped to a minimum of `1` per axis (wgpu rejects a
    /// zero-sized texture). The texture is created with the format and usages
    /// Vello requires plus the copy/binding usages a host needs to consume the
    /// result.
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
            .field("format", &"Rgba8Unorm")
            .finish_non_exhaustive()
    }
}
