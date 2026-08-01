Wires the full multi-font fallback stack onto the runtime and the renderer.

Builds a [`FontRegistry`] (the coverage-driven [`FontResolver`]) and registers
the primary under the `sans-serif` generic family — the family the runtime's
default fallback chain (`FontFallback::default_chain`) consults — so it gets
id `0`. If a fallback font is found it is registered under the **same** family
(id `1`), so resolution picks between the two by `cmap` COVERAGE, not by name:
a Latin run resolves to whichever covers it, a CJK run to the CJK face.

The renderer receives the **same** id->bytes assignment via
[`VelloRenderer::register_font`], so the scene's per-run font handles (the
resolved [`FontId`]s) line up with the fonts the renderer rasterizes.

Without a fallback font found, mixed-script glyphs the primary lacks resolve to
no covering font and render as `.notdef`; the fallback path activates only once
a covering font is registered here.
