Shapes `text` end to end, returning one [`ShapedRun`] per itemized run in
logical (reading) order (spec §5.10).

For each itemized run the resolver is asked for a covering font from
`families` matching `attributes`; when it returns `None` the run falls
back to [`FontId::PRIMARY`] paired with `default_face`. Every run is
shaped at `font_size` with the item's own direction (itself seeded by
`base_direction`), and its glyph clusters and source range are normalized
to full-text byte coordinates. Empty `text` yields a single empty
`ShapedRun` (the itemizer emits one empty item), never a panic.

The result is memoized: the first call for a given set of inputs runs the
full itemize → resolve → shape pipeline and stores the runs; every later
identical call (a second phase this frame, or any future frame) returns a
clone of the cached runs without re-itemizing or re-resolving. Any change
to `text`, `families`, `attributes`, `default_face`, `font_size`, or
`base_direction` is a distinct key and re-shapes.
