One itemized segment of text: a byte range that is uniform in script and
bidirectional direction (spec §5.10).

Itemization walks the source text in *logical* order and cuts it wherever the
script or the resolved bidi direction changes, so every `TextItem` can be
shaped as a single run. Alongside the [`range`](Self::range),
[`script`](Self::script), and [`direction`](Self::direction) it carries the
resolved Unicode Bidirectional Algorithm embedding [`level`](Self::level):
even (`0`, `2`, …) levels are left-to-right, odd levels right-to-left, which
later stages use to reorder runs for display.
