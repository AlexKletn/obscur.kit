Real [`ResourceDecoder`](obscur_resources::port::ResourceDecoder) adapters
(spec §5.15 decode, §5.16 images, §5.17 SVG, §5.18 fonts, §2.7 adapters).

The domain's decode port turns encoded resource bytes into a CPU
representation, but the external crates that actually parse those formats are
forbidden inside the pure domain (spec §5.6): they live here, in an interface
adapter. This crate provides three concrete decoders, one per format family,
each implementing the object-safe
[`ResourceDecoder`](obscur_resources::port::ResourceDecoder) port:

- [`ImageDecoder`](decoder::ImageDecoder) — raster PNG/JPEG via the `image`
  crate, producing decoded RGBA8 pixels and the intrinsic pixel size (spec
  §5.16).
- [`FontDecoder`](decoder::FontDecoder) — TTF/OTF validated with `ttf-parser`,
  producing the face bytes as the shared CPU representation (spec §5.18).
- [`SvgDecoder`](decoder::SvgDecoder) — SVG parsed and validated with `usvg`,
  kept vector as the normalized source bytes plus the intrinsic size taken
  from the resolved `viewBox` (spec §5.17).

A hub registers whichever decoders a build needs; each reports
[`ResourceDecoder::supports`](obscur_resources::port::ResourceDecoder::supports)
for exactly its one [`ResourceKind`](obscur_resources::handle::ResourceKind)
and refuses others honestly.
