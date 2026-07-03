//! Target module.
mod bitmap_target;
pub use bitmap_target::BitmapTarget;
mod render_target;
pub use render_target::{PixelFormat, RenderTarget};
mod texture_target;
pub use texture_target::TextureTarget;
mod window_target;
pub use window_target::WindowTarget;
