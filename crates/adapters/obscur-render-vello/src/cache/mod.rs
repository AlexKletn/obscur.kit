//! Unused content-hash-keyed maps, kept only pending a decision to delete them.
//!
//! Each was *intended* to hold, and none ever has (see below — nothing constructs
//! them):
//!
//! - [`PathCache`] — converted kurbo `BezPath`s, keyed by geometry hash.
//! - [`TextureCache`] — resident `wgpu::Texture`s, keyed by image content hash.
//! - [`GlyphCache`] — rasterized glyphs ([`GlyphEntry`]), keyed by
//!   font+size+glyph hash.
//!
//! # These are not §5.19 caches, nothing calls them, and two of them cannot be filled
//!
//! Each is an unbounded `HashMap` newtype. None of §5.19's requirements — memory
//! budgets, eviction, visibility priority, `DevTools` metrics, a disable switch —
//! is implemented here, and none of the three has a caller. That absence is not a
//! §5.19 shortfall: §5.19's list is permissive ("runtime caches **may** include …
//! paths; glyphs; … textures"), and its requirements bind the caches that *run*.
//! None of these three is ever constructed on a live path, so none holds a byte,
//! and putting a budget on them would bound nothing. The §5.19 policy tier
//! (`obscur_resources::cache`) is real and is wired to the runtime's decoded-image
//! cache; it is not wired here, because there is no cache here to bound.
//!
//! The blocker is **not** that the renderer is stateless — it is not.
//! `VelloRenderer` keeps its blitter, blur and color-matrix pipelines across
//! frames, and `SceneConverter` keeps the host's registered fonts. Those are
//! bounded by construction (at most one pipeline each; the font map is a host
//! registry, not a content-keyed cache). The blocker is that for two of the three
//! **the cached value has no producer in this crate**, and would only get one by
//! taking work back from Vello:
//!
//! - [`TextureCache`] holds a `wgpu::Texture` per image, and this adapter never
//!   creates one. `draw_image` wraps the decoded pixels in a `peniko::ImageData`
//!   and calls `Scene::draw_image`, which takes image *data*, not a texture; Vello
//!   uploads it into **its own** image atlas, keyed by the blob id. Filling this
//!   map means taking image residency back via `Renderer::override_image`.
//! - [`GlyphCache`] holds rasterized coverage per glyph, and this adapter never
//!   rasterizes one. Runs arrive pre-shaped from `obscur-text` and go to
//!   `Scene::draw_glyphs`; `vello_encoding` runs its own glyph cache and already
//!   prunes it by age. Filling this map would duplicate a cache the dependency
//!   already keeps *and bounds*.
//! - [`PathCache`] is the only one whose value this adapter does produce
//!   (`PathConverter` makes the `BezPath`s). It still has no key: nothing hashes a
//!   path's geometry, and `obscur_scene::Path` derives `PartialEq` but not `Hash`
//!   — holding floats, it cannot derive one — so the `u64` would need a bespoke
//!   bit-level hash. No measurement says conversion is worth that.
//!
//! So the options are not "make the renderer use these or delete them": they are to
//! re-implement what Vello already does for glyphs and images, or delete those two;
//! and for [`PathCache`], to measure before building. Deletion is the recommendation
//! (audit finding 5-C); the decision is the owner's and is unmade.
mod glyph_cache;
pub use glyph_cache::{GlyphCache, GlyphEntry};
mod path_cache;
pub use path_cache::PathCache;
mod texture_cache;
pub use texture_cache::TextureCache;
