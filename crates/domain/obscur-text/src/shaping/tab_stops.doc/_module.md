CSS Text 3 `tab-size` (§4.3): a preserved tab (U+0009) advances the pen to
the next tab stop.

Like [`apply_spacing`](crate::shaping::apply_spacing), this is a **pure,
width-independent** rewrite of a shaped paragraph's glyph advances that sits
between the (cached) shaping result and the width-dependent line wrap, so the
runtime's measure pass and its compose pass call the same seam with the same
`tab_width` and can never drift.

A tab stop is a multiple of the `tab_width` measured from the **start of the
line**. Per §4.3 the pen advances to the next tab stop that is *strictly*
greater than the current position, so a tab always advances at least a little:

```text
next = (floor(pen / tab_width) + 1) · tab_width
advance = next − pen
```

(equivalently `ceil((pen + ε) / tab_width) · tab_width`). The pen is measured
from the line start, so it is reset to zero after each preserved segment break
(`\n`) — a documented single-line-per-segment approximation that is exact for
the common case where a run of preserved text has no leading indentation.

Only tabs that are still present in `text` are affected; under a collapsing
`white-space` mode (`normal` / `nowrap` / `pre-line`) a tab has already become
a space at the pre-shape seam, so `text` contains no `\t` and this pass is a
byte-for-byte no-op (it early-returns before touching any run).
