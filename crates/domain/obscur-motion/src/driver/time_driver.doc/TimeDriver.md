A time-driven [`MotionDriver`]: normalized progress from accumulated logical
time (spec §4.17 "time-driven motion").

The driver holds an `elapsed` count of logical seconds and a fixed
`duration`; its progress is `elapsed / duration`, clamped to `[0, 1]`. It is
**time-agnostic and deterministic**: it never reads a clock. Callers push in
an explicit `delta` via [`advance`](Self::advance), so a **skipped frame** is
simply a bigger delta that still clamps to `1.0` — the same math with no
per-frame assumption baked in.
