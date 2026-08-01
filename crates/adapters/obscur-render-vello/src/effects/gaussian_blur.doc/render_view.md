Blurs an already-rendered source **texture view** (rather than a
[`VelloFrame`]) and composites it onto `target` — the shared core of
[`Self::render`], reused when a color-matrix pass has already written the
source into a scratch texture (a `filter` that mixes a color op and a blur).

`source_size` is the source's `(width, height)` in pixels; the rest matches
[`Self::render`]. With no color pre-pass this is byte-identical to the shipped
blur (same ping-pong passes, same composite).
