The root render transform `scale(s) · translate(−ox, −oy)`, or [`None`]
when it is the identity.

[`None`] is not an optimisation detail the caller may ignore: at scale
exactly `1.0` and offset `(0, 0)` the composed display list must be emitted
**unwrapped**, byte-identical to the pre-scroll path.

Scroll first (translate the content up/left by the offset), then scale to
device resolution. When the offset is `(0, 0)` the matrix reduces exactly to
`scale(s, s)`.
