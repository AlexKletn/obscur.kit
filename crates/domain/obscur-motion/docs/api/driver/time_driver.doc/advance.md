Advances the driver by `delta` logical seconds (spec §4.17).

Only a strictly positive `delta` accumulates; a negative delta (and
`NaN`) is ignored, since time never runs backward here — reversal is a
timeline concern, not a driver one. A single large `delta` from a skipped
frame simply pushes `elapsed` past `duration`, where
[`progress`](MotionDriver::progress) clamps it to `1.0`.
