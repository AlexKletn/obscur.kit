#![doc = include_str!("../../docs/api/renderer/mod.doc/_module.md")]
mod damage_stats;
mod filter_layer_pass;
mod premultiply_blit;
mod segmented_compositor;
mod vello_device;
pub use vello_device::VelloDevice;
mod acquired_image;
pub use acquired_image::AcquiredImage;
mod surface_acquirer;
pub use surface_acquirer::{AcquireWake, SurfaceAcquirer};
mod vello_frame;
pub use vello_frame::VelloFrame;
mod vello_renderer;
pub use vello_renderer::VelloRenderer;
mod vello_surface;
pub use vello_surface::VelloSurface;
#[cfg(feature = "hybrid")]
mod hybrid_renderer;
#[cfg(feature = "lyon")]
mod lyon;

mod readback_layout;
