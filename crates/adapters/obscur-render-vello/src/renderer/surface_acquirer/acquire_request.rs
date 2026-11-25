/// An instruction to acquire at the latest requested extent or present a rendered image.
pub(super) enum AcquireRequest {
    Image,
    Present(wgpu::SurfaceTexture),
}
