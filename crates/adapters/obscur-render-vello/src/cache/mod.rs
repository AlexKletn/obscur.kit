//! Content-hash-keyed maps for the Vello backend's per-frame work.
//!
//! - [`PathCache`] — converted kurbo `BezPath`s, keyed by geometry hash.
//! - [`TextureCache`] — resident `wgpu::Texture`s, keyed by image content hash.
//! - [`GlyphCache`] — rasterized glyphs ([`GlyphEntry`]), keyed by
//!   font+size+glyph hash.
//!
//! # These are not §5.19 caches, and nothing calls them
//!
//! Each is an unbounded `HashMap` newtype. None of §5.19's requirements —
//! memory budgets, eviction, visibility priority, `DevTools` metrics, a disable
//! switch — is implemented here, and none of the three has a caller: the Vello
//! renderer builds a fresh `Scene` every frame and converts every path, uploads
//! every image, and shapes every run again. The §5.19 policy tier
//! (`obscur_resources::cache`) is real and is wired to the runtime's decoded-image
//! cache; it is not wired here, because there is no cache here to bound.
//!
//! Adopting it would need two things these types do not have: a key the
//! converter actually computes (nothing hashes a `Path`'s geometry today — the
//! `u64` is entirely caller-supplied and no caller exists), and a decision about
//! whether the renderer should hold state across frames at all. Until one of
//! those happens these are scaffolding, and the honest options are to make the
//! renderer use them or to delete them (audit finding 5-C).
mod glyph_cache;
pub use glyph_cache::{GlyphCache, GlyphEntry};
mod path_cache;
pub use path_cache::PathCache;
mod texture_cache;
pub use texture_cache::TextureCache;
