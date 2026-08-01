Lowers `conic-gradient( [[from <angle>]? [at <position>]?]? , <color-stop>{2,} )`
into a style-local [`ConicGradient`] (spec §5.2).

The optional leading argument is the gradient prelude: a `from <angle>`
starting angle (CSS convention: `0` = up, increasing clockwise, default `0`)
and an `at <position>` center (default `50% 50%`). When the first comma part is
not a prelude it is a color stop and the prelude defaults apply. At least two
comma-separated `<color> [<percentage>]?` stops follow. Returns [`None`] for a
malformed prelude, an unparseable stop, or fewer than two stops.
