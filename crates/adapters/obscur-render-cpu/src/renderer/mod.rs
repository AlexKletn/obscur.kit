//! The software/headless renderer backend (spec §5.2, §2.7).

mod cpu_renderer;
mod glyph_pen;
pub use cpu_renderer::CpuRenderer;
