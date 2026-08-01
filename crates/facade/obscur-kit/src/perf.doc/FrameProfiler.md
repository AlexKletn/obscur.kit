A drop-in per-frame profiler a windowed host feeds two measured durations —
the CPU frame and the GPU present — to get a rolling [`FrameStats`] plus a
ready-to-print report line at a fixed cadence.

This is the whole "measure FPS + timings" facility as *kit* code: a host
records each frame's two spans and prints the returned line; it no longer
hand-rolls a ring buffer, an interval timer, and a format string per example.
The real present FPS is `frames this interval / wall-clock interval`, so the
host should request a redraw every frame for a continuous rate.
