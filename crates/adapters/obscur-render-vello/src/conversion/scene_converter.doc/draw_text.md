Rasterizes a single **pre-shaped** text run at `origin`.

The run was shaped once in the `obscur-text` domain against this same host
font, so this method neither re-shapes nor remaps: each glyph's
[`glyph_id`](obscur_scene::text::PositionedGlyph::glyph_id) is already a
**font glyph index** and is used **directly**, placed at its domain-supplied
run-relative position offset by `origin`. The assembled glyphs are drawn with
Vello's glyph pipeline, brushed with the command's own [`Brush`].

Degrades gracefully: with no host font set the run is skipped rather than
erroring.
