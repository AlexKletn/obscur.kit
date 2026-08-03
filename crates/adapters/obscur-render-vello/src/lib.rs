#![doc = include_str!("lib.doc/_module.md")]
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

// A `cache` module held `GlyphCache`, `PathCache` and `TextureCache`: unbounded
// content-keyed maps that materialised the spec's source tree and nothing else.
// Two of them cannot be filled from here — this adapter rasterizes no glyph and
// creates no `wgpu::Texture`, so `Scene::draw_glyphs` and `Scene::draw_image`
// hand both to Vello, which keeps its own glyph cache (pruned by age) and its
// own image atlas. Filling them meant taking that residency back. The third,
// `PathCache`, had no key at all: `obscur_scene::Path` holds floats and so
// cannot derive `Hash`. Deleted rather than bounded, because a §5.19 budget on
// a map with no producer would bound nothing.

pub mod conversion;
pub mod effects;
pub mod error;
pub mod extension;
pub mod renderer;
