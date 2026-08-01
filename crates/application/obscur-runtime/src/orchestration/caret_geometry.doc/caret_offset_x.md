Returns the caret's pen x (logical pixels) at byte `index` in `content`:
the sum of the shaped glyph advances whose source **cluster** falls before
`index` — i.e. the pen x just left of the glyph the caret sits before (spec
§5.11). A caret at `0` yields `0`; a caret at the text length yields the
whole run advance.

The text is shaped through the **same** paragraph orchestrator (and default
face) the runs are drawn with, unwrapped, so a caret x on the first line is
exact. Multi-line wrapping and RTL reordering are not modeled here — the
caret is placed by the logical (source-order) advance sum, a documented
single-line, left-to-right approximation.
