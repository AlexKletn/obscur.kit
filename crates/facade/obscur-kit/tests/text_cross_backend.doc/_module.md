Do the two render backends draw the same text from the same display list?

`clip_cross_backend.rs` exists because the backends' *clip* semantics drifted
apart with nothing watching. Text drifted the same way and had no watcher at
all: `obscur-render-cpu` draws one outline fill per glyph of each `DrawText`
command, while `obscur-render-vello` hands each command to `Scene::draw_glyphs`
— and a Vello glyph run is not a batch. `vello_encoding`'s resolver counts one
path per run, emits every glyph of the run as subpaths, and closes them with a
single `PathTag::PATH`. One run is therefore **one non-zero fill of the union of
its glyphs**, not N composites of them.

That difference is invisible wherever no two glyph outlines reach the same
pixel, which is every line of ordinary text — and it is exactly why a fold of
adjacent commands into one run lived in the Vello converter for four passes
without anyone seeing it. Where outlines *do* meet, the two are not the same
drawing: `1-(1-a)(1-b)` is not `min(a+b, 1)`, and a counter-wound contour that
would be a hole cancels under non-zero instead of being painted over.

**Every number below is measured, on this machine, through the shipping code
paths** — `CpuRenderer::render` into a bitmap target, `VelloRenderer::render_to_image`
onto the adapter. The probe is one 40 px `H` at `ORIGIN`, painted white at
α = ½, and `PROBE` is a pixel deep inside its left stem where both rasterizers
report full coverage.

|                                  | CPU | Vello |
|----------------------------------|-----|-------|
| one command, α = 1                | 255 | 255   |
| **two commands**, α = ½ each      | 192 | 192   |
| **one command, two glyphs**, α = ½| 192 | **128** |

Two things follow, and this file holds both.

**The guard.** Two `DrawText` commands must stay two composites on both
backends. It reads 192 on each, and it is what the merge that was reverted this
round would break: with the fold restored the Vello column reads 128 while the
CPU column still reads 192. The fold is gone, so this test is green today — it
is here for the *next* one, and it was watched failing with the fold temporarily
put back before it was committed to.

**Overlap inside a command.** The CPU renderer combines all glyph outlines
of one command into a winding fill. Vello also emits one glyph run per
command. Duplicate outlines therefore stay within the command's half-alpha
ceiling, including when synthetic bold expands the outlines. Vello can
produce different antialiased edges when the winding magnitude changes.

**What runs where.** The device-independent tests compare CPU pixels and
the Vello encoding. Tests that compare real rasters are ignored by default.
Run them with `cargo test -p obscur-kit --test text_cross_backend -- --ignored`
on a machine with a GPU adapter.
