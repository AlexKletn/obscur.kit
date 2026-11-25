Blurs an already-rendered source **texture view** (rather than a
[`VelloFrame`]) and composites it onto `target` — the shared core of
[`Self::render`], reused when a color-matrix pass has already written the
source into a scratch texture (a `filter` that mixes a color op and a blur).

The source contains straight RGBA and `source_size` gives its pixel extent.
The first pass converts to premultiplied RGBA, which the second pass and
compositor preserve.
