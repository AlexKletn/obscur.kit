Sets the current scroll offset (spec §4.17).

The scroll subsystem reports its latest scroll position here and the
driver re-normalizes it on the next [`progress`](MotionDriver::progress)
read.
