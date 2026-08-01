The **containing block's content size**, when the caller knows it — the
basis a percentage size resolves against (CSS 2.1 §10.2).

`min`/`max` cannot answer this. They are a *range*, and a range means two
different things at two different moments: "I have decided your size"
while arranging, and "how small could you be?" while measuring — a
`min-content` question arrives as a tight `0`. A box that resolved its own
`width: 100%` against that range therefore read its own border box as its
containing block and came back one padding wider, every level down.

[`None`] means the caller did not say, and a percentage falls back to
resolving against `max` — the behaviour every caller had before this
existed. Servo threads the same distinction as
`SizeConstraint = Definite | MinMax`; this is its first half, carried
alongside the range rather than replacing it.
