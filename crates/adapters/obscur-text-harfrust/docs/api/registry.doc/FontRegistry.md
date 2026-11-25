A real [`FontResolver`] backed by a set of registered fonts and their true
`cmap` coverage (spec §5.10 font fallback).

Fonts are added with [`register`](Self::register), which parses each face
once (via [`read_fonts`]) to read its `units_per_em` and vertical metrics and
hands back a [`FontId`] assigned in registration order (`0, 1, 2, …`). The id
is deterministic and stable, so a later renderer can look the same face back
up by id — and so can a shaper, through the
[`face_metrics`](Self::face_metrics) table, which is how a line box gets sized
from the face's own metrics instead of idealized ratios.

[`resolve`](FontResolver::resolve) walks the request's fallback chain in
order (primary first). For each family it consults every registered font
whose family name matches, and returns the **first** one whose `cmap`
actually covers the run — i.e. every non-whitespace character maps to a
non-zero glyph id. When nothing in the chain covers the run it returns
`None`, leaving the orchestrator to fall back to its default face.

Coverage is checked by re-parsing the font's `cmap` per `resolve` call
(cheap: resolution happens once per itemized run, not per glyph, and an
upstream cache elides repeats), so the registry holds only the raw bytes and
pre-read metrics — no borrowed, self-referential font handle.
