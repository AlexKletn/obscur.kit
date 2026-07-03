//! Runtime resource caches for the Vello backend (spec §5.19).
//!
//! Spec §5.19 calls for runtime caches that avoid re-doing per-frame work:
//! re-converting geometry, re-uploading images, and re-rasterizing glyphs. Each
//! cache here is a plain content-hash-keyed map holding no async or GPU-creation
//! logic, so all but the GPU-populated texture cache are fully testable without a
//! device.
//!
//! - [`PathCache`] — converted kurbo `BezPath`s, keyed by geometry hash.
//! - [`TextureCache`] — resident `wgpu::Texture`s, keyed by image content hash.
//! - [`GlyphCache`] — rasterized glyphs ([`GlyphEntry`]), keyed by
//!   font+size+glyph hash.
mod glyph_cache;
pub use glyph_cache::{GlyphCache, GlyphEntry};
mod path_cache;
pub use path_cache::PathCache;
mod texture_cache;
pub use texture_cache::TextureCache;
