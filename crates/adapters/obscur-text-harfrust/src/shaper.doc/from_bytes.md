Creates a shaper wired to a single host font from its raw `bytes`.

Every [`shape`](TextShaper::shape) call uses this font, ignoring the
request face's own source, and its `head`/`hhea` answer
[`line_metrics`](TextShaper::line_metrics).
