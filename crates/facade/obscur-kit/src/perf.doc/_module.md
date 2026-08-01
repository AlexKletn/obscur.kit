Frame performance measurement for windowed hosts (spec §7.9 telemetry, host
side).

The runtime never reads a clock (it records structured phase *counts*, not
wall time — the host owns timing). [`FrameStats`] is the small reusable
aggregator a windowed host feeds two per-frame durations — the **CPU** phase
(build → reconcile → style → layout → compose) and the **present** phase
(GPU submit + blit) — to get a rolling FPS and timing breakdown it can print
or draw, instead of every example hand-rolling the same ring buffer.
