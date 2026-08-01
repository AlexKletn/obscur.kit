§4.16: "`::before` and `::after` are supported for decoration only." and
"Meaningful structure or behavior must use real nodes."

`.badge::before { content: "!" }` on a childless `.badge` holding "New".
The "!" is painted before "New" in inline order, yet `.badge` reports **0**
children, the accessibility tree gains no node for it, and hit-testing the
glyph returns `.badge` itself — a materialized real child would leak into
all three.
