Injects the **whole real text stack** into `runtime` over the faces in
`fonts`, and returns the scene font handle assigned to each (spec §5.10,
§5.18; §2.7 adapters keep host reach explicit).

This is the one call that replaces all three of the runtime's honest but
inert text defaults at once, which until now every host re-derived by hand:

* the [`MockTextShaper`](crate::text::shaping::MockTextShaper) → the real
  [`HarfrustShaper`](crate::text_harfrust::HarfrustShaper), so glyphs carry
  font-specific indices with kerning/ligatures/GPOS rather than Unicode
  scalars;
* the empty [`MockFontResolver`](crate::text::font::MockFontResolver) → a
  [`FontRegistry`](crate::text_harfrust::FontRegistry) over `fonts`, so a
  run resolves to a face that actually *covers* it;
* the [`SingleRunItemizer`](crate::text::shaping::SingleRunItemizer) → the
  real [`UnicodeTextItemizer`](crate::text_unicode::UnicodeTextItemizer),
  so text splits into uniform bidi-level + script runs — **the default has
  no bidi at all**, and this is what gives it any.

It also hands the shaper the registry's
[`face_metrics`](crate::text_harfrust::FontRegistry::face_metrics) snapshot,
taken after every face is registered. That is the step easiest to omit and
hardest to notice: a shaper is asked for line metrics by
[`FontId`](crate::text::font::FontId) alone, so without the table it shapes
real glyphs into line boxes sized by idealized 0.8/0.2 ratios — real text in
fake boxes.

The facade constructs the concrete adapters and hands the runtime domain
ports, so the application layer depends only on the ports, never on these
adapter crates.

# The host must still feed the renderer

This installs the resolve/shape half. Rasterization lives in a renderer
this kit does not own, and it must be given the *same* bytes under the
*same* handles or it will paint the wrong outlines — see
[`InstalledFonts`](super::InstalledFonts), which exists to make that
hand-off one loop.

# An empty `fonts` still renders — the bundled last-resort face (§5.18)

The kit ships one vendored face, `assets/fonts/NotoSans-Regular.ttf`,
embedded at compile time so it needs no filesystem — which the isolated /
WASM target does not have. It is appended to the registry **after** the
host's faces, under the generic `sans-serif` family every default fallback
chain terminates in, so:

* a host that supplies its own faces is unaffected — its faces match first
  by family and coverage, and the bundled one only fills a gap nothing else
  covers;
* an **empty** `fonts` (the WASM/isolated case: nothing enumerated) falls
  through to the bundled face and paints, rather than rendering nothing.

This is §5.18's "bundled fallback set for WASM/isolated environments" met,
not the deviation it once was. The bundled face is skipped when the host
already provides those exact bytes, so it never double-registers.

The returned [`InstalledFonts`](super::InstalledFonts) — the renderer
hand-off table — lists the host's faces exactly, and additionally the
bundled face **only when the host supplied none**: that is the one case the
renderer must be handed the bundled bytes to paint from, while a host with
its own faces keeps the handle table it already expects.
