Lowers `radial-gradient( [<shape>? <size>? [at <position>]?]? , <color-stop>{2,} )`
into a style-local [`RadialGradient`] (spec §5.2).

The optional leading argument is the gradient prelude: an ending shape
(`circle` / `ellipse`, default `ellipse`), an ending size (one of the four
extent keywords `closest-side` / `closest-corner` / `farthest-side` /
`farthest-corner`, default `farthest-corner`, or an explicit `<length>` /
`<percentage>` radius), and an `at <position>` center (default `50% 50%`). When
the first comma part is not a prelude it is a color stop and the prelude
defaults apply. At least two comma-separated `<color> [<percentage>]?` stops
follow. Returns [`None`] (so the declaration degrades to an unsupported
feature) for a malformed prelude, an unparseable stop, or fewer than two stops.

For repeating gradients, stop positions retain their length or percentage
units until the ending ellipse is known. This also preserves the first stop’s
distance from the center and accepts two-position color stops.
