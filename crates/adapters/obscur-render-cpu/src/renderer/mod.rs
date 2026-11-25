//! The software/headless renderer backend (spec §5.2, §2.7).

mod brush_fill;
mod clip_mask;
mod cpu_renderer;
mod gaussian_blur;
mod glyph_pen;
mod glyph_raster;
mod group_compositing;
mod image_tiling;
mod layer_composite;
mod render_scope;
mod shadow_approximation;
mod skia_lowering;

pub use cpu_renderer::CpuRenderer;
