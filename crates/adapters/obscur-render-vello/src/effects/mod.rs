//! Owned wgpu post-process effect passes layered over Vello (spec §5.2).
//!
//! Vello 0.9 rasterizes vector content but lacks some GPU-class effects (content
//! blur, later color-matrix / `backdrop-filter`). Rather than fake them, this
//! module adds our **own** wgpu render passes that run over Vello's output: each
//! effect renders an offscreen sub-scene and post-processes it with a small WGSL
//! pipeline set cached on the renderer.
//!
//! - [`GaussianBlur`] — a separable gaussian blur + premultiplied composite,
//!   implementing CSS `filter: blur()`.
//! - [`ColorMatrix`] — a folded 4×5 affine color-matrix pass, implementing the CSS
//!   color-matrix `filter` functions (grayscale/brightness/contrast/saturate/
//!   sepia/invert/hue-rotate/opacity).
mod color_matrix;
mod gaussian_blur;
pub use color_matrix::ColorMatrix;
pub use gaussian_blur::GaussianBlur;
