Sets the current raw signal value (spec §4.17).

This is the interaction subsystem's write path: a drag/cursor/gesture
reports its latest reading here and the driver re-normalizes it on the
next [`progress`](MotionDriver::progress) read.
