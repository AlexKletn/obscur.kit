Shapes `0` in the face `font` selects, at the size `font` resolved.

Reads exactly the four properties the port's contract promises are present
— `font-family` (via [`families_for`]), `font-weight` / `font-style` (via
[`attributes_for`]), and `font-size` (via [`font_size_px`]) — and nothing
else, because nothing else is resolved yet when the resolver calls this.

These are the same three helpers the layout measure pass and the scene
compose pass use to build *their* shaping requests, which is what makes the
`ch` answer consistent with the text: they are not re-derived here.

# Standing limitation

`ch` **does** follow `font-family`: the node's families are passed below and
the resolver's fallback chain picks the face, so a monospace node measures
its own `0` (Noto Sans Mono's 600/1000 vs Noto Sans's 572/1000 — measurably
different at every size). Only the `FontFace` *carrier* is ignored.

What is still open is narrower and worth stating exactly, because it is the
seam this port exists to close: a family that resolves to **no registered
face** — `font-family: Helvetica` with neither Helvetica nor a fallback
registered — measures nothing. `zero_advance` answers `None`, layout then
takes `ch_base`'s 0.5em substitute, and paint leaves `Ch` symbolic and reads
0. Layout and paint disagree again, exactly as they did for `em` before 4-A.
This coincides with a general fallback gap (such a node renders no glyphs at
all), so `ch` inherits it rather than causing it — but the seam is live for
an unresolvable family even though every production driver supplies metrics.
