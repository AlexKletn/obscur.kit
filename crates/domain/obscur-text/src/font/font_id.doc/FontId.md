A small `Copy` identifier for a resolved font face held in a registry.

A `FontId` is an opaque handle: it names *which* face a run's glyph indices
belong to without carrying the face data itself. Font fallback produces text
whose glyphs come from several faces; tagging each shaped run with a `FontId`
lets later stages look the concrete face back up in whatever registry issued
the ids. Id `0` is reserved for the primary (first-choice) face; see
[`FontId::PRIMARY`].
