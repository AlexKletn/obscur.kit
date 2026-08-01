Lowers `repeating-linear-gradient( [<angle> | to <side-or-corner>]? ,
<color-stop>{2,} )` into a repeating style-local [`LinearGradient`] (spec §5.2).

A repeating gradient defines **one period** — the extent of the last authored
stop position — and tiles it across the box (the scene coordinator sizes the
gradient axis to one period and the renderer repeats it). The direction is
parsed exactly like [`lower_linear_gradient`]. Stops accept `px` or `%`
positions and, in addition to the ordinary `<color> [<pos>]?` form, the
**two-position** `<color> <pos> <pos>` hard-band form (equivalent to the color
repeated at both positions — the standard way to author a hard stripe edge).

The period is the last stop's position (its px value, or its percentage of the
gradient line). All stop positions are then normalized to `0.0..=1.0` over that
period, so the resulting stops flow through the shared coordinator path
unchanged. Returns [`None`] (degrading to an unsupported-feature diagnostic)
for a malformed direction, an unparseable stop, fewer than two resulting stops,
an unpositioned last stop, mixed px/percent units, or a zero-length period.
