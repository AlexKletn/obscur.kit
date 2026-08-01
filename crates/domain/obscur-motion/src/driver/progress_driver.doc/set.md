Sets the current progress, clamped to `[0, 1]` (spec §4.17).

This is the external control's write path: a scrubber reports its new
position here and the driver reflects it on the next
[`progress`](MotionDriver::progress) read.
