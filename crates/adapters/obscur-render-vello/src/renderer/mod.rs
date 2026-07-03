//! The Vello/wgpu renderer backend (spec §5.2, §2.7).
//!
//! Implements the `Renderer` port of spec §5.2 on top of Vello and wgpu. Because
//! wgpu device creation is asynchronous and needs a real GPU, the *host* owns the
//! wgpu context and passes the device in (spec §2.7); the adapter never calls
//! `request_adapter`. A [`VelloRenderer`] therefore holds an **optional**
//! [`VelloDevice`] and returns an honest diagnostic when asked to render without
//! one.
//!
//! - [`VelloRenderer`] — the `Renderer` implementation and its optional device.
//! - [`VelloDevice`] — a host-provided wgpu device/queue plus the Vello renderer.
//! - [`VelloSurface`] — an on-screen swapchain surface for windowed present.
//! - [`VelloFrame`] — the per-frame offscreen texture Vello rasterizes into.
mod vello_device;
pub use vello_device::VelloDevice;
mod vello_frame;
pub use vello_frame::VelloFrame;
mod vello_renderer;
pub use vello_renderer::VelloRenderer;
mod vello_surface;
pub use vello_surface::VelloSurface;
