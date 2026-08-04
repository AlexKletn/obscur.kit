A slider's value model: its bounds, its step, and the two conversions between a value and the fraction a thumb sits at.

It lives in the domain and not in the widget because BOTH ends need it. The
widget converts a value into a position to draw; the runtime converts a drag's
position back into a value — and an application crate may not depend on a
components one. Two copies of this arithmetic would drift, and the drift would
look like a thumb that creeps.

Everything here fails to the minimum rather than to a NaN. `min == max` is a
real state an application renders while its data loads, and a fraction computed
by dividing by nothing would paint nowhere at all.
