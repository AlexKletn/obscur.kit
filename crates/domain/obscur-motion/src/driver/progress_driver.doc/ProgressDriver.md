A progress-driven [`MotionDriver`]: progress set explicitly from the outside
(spec §4.17 "progress-driven motion").

Unlike a [`TimeDriver`](super::TimeDriver), this driver integrates nothing —
it simply holds a normalized progress that some external control writes
directly, such as a scrubber, slider or timeline-scrubbing gesture. The
stored value is always kept within `[0, 1]`, so [`progress`](MotionDriver::progress)
returns it verbatim.
