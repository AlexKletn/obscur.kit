Rasterizes a single **pre-shaped** text run at `origin`.

The run was shaped once in the `obscur-text` domain against this same host
font, so this method neither re-shapes nor remaps: each glyph's
[`glyph_id`](obscur_scene::text::PositionedGlyph::glyph_id) is already a
**font glyph index** and is used **directly**, placed at its domain-supplied
run-relative position offset by `origin`. The assembled glyphs are drawn with
Vello's glyph pipeline, brushed with the command's own [`Brush`].

**One command, one glyph run — on purpose.** Gathering adjacent commands that
agree on font, size, synthesis and brush into a single `draw_glyphs` looks free
and is not: `vello_encoding`'s resolver counts one path per glyph run
(`layout.n_paths += 1`) and closes the whole run with a single `PathTag::PATH`,
so every glyph of a run is a subpath of **one** non-zero fill and **one** draw
object. Drawn apart, N commands are N source-over composites; drawn together
they are one non-zero fill of the union. The two agree only where no two glyphs
reach the same pixel — elsewhere `1-(1-a)(1-b)` is not `min(a+b, 1)`, and a
counter wound against a neighbouring outline cancels instead of being painted
over. Measured on this backend, two 15 px legends whose boxes touch differ by
up to 22/255 in alpha and two overlapping 40 px glyphs by up to 58/255; see
`tests/text_run_pixels.rs`, which holds both halves of that line.

Degrades gracefully: with no host font set the run is skipped rather than
erroring, and an empty run never enters `draw_glyphs`.
