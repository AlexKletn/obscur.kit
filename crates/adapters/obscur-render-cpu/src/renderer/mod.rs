#![doc = include_str!("mod.doc/_module.md")]

mod brush_fill;
mod clip_mask;
mod cpu_renderer;
mod glyph_pen;
mod glyph_raster;
mod group_compositing;
mod image_tiling;
mod shadow_approximation;
mod skia_lowering;

pub use cpu_renderer::CpuRenderer;
