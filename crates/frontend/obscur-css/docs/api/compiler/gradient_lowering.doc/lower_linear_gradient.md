Lowers `linear-gradient( [<angle> | to <side-or-corner>]? , <color-stop>{2,} )`
into a style-local [`LinearGradient`] (spec §5.2).

The optional first argument is the gradient direction: a `<angle>` (`45deg`,
and the `grad` / `rad` / `turn` units) or a `to <side>` / `to <corner>` keyword
form folded to its CSS-convention angle (`to top` = 0, `to right` = 90, `to
bottom` = 180, `to left` = 270). A `to <corner>` is carried symbolically (as a
[`Corner`]) rather than folded, so the scene coordinator can resolve CSS's
aspect-ratio-aware corner angle against the painted box (spec §3.1). When it is
omitted the
direction defaults to `to bottom` (180°). At least two comma-separated
`<color> [<percentage>]?` stops follow; missing positions are auto-distributed
to explicit `0.0..=1.0` values ([`resolve_stop_positions`]). Returns [`None`]
(so the declaration degrades to an unsupported feature) for a malformed
direction, an unparseable stop, or fewer than two stops.
