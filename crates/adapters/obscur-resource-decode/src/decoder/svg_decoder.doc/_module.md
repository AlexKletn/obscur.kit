An SVG [`ResourceDecoder`] backed by `usvg` (spec §5.17).

[`SvgDecoder`] parses SVG source with `usvg`, which validates the document and
resolves it into a normalized vector tree. Per spec §5.17 an SVG must stay
vector through the scene pipeline, so this decoder does **not** rasterize:
rasterizing here would discard the vector data the scene layer needs. Instead
it keeps the original SVG source bytes as the CPU payload — vector-preserving —
and records the intrinsic pixel size taken from the resolved document (its
`width`/`height`/`viewBox`), which is what the runtime needs for object-fit
layout (spec §5.16 intrinsic size, §5.17 vector).

Text is intentionally left unshaped (`usvg`'s `text` feature is disabled) so
this decoder needs no font database: text elements still parse, but their
glyph outlines are resolved later by the kit's own text stack, keeping the
font provider replaceable (spec §5.18).
