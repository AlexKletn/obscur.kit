Whether a paragraph alignment offset is **independent of the available (box)
width** — i.e. the line hugs the leading-left edge, so the offset is `0`
regardless of the box width.

This is exactly the case where a same-size text edit can reuse a stale,
content-sized text-node box width without shifting the painted run/caret
(`TextAlignment::x_offset` returns `0` for any available width): `Left` and
`Justify` always, `Start` under left-to-right, and `End` under right-to-left.
Every other combination distributes the remaining space and therefore depends
on the box width.
