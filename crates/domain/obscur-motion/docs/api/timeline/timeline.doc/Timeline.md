A deterministic, time-agnostic playable timeline (spec §4.17 "motion
engine").

A `Timeline` integrates an explicit `delta` of elapsed *logical seconds*
(`f32`) supplied by a driver; it never reads a clock. This keeps it
deterministic — feeding two identical timelines the same sequence of deltas
yields identical positions — and robust under event-driven scheduling and
**skipped frames**: a skipped frame is simply a larger `delta` and is handled
by the same integration math, with no per-frame assumption baked in.

The timeline owns its lifecycle ([`TimelineState`]), its raw linear
[`TimelinePosition`], a [`PlaybackDirection`], a [`RepeatCount`], and whether
it `alternates` direction each iteration. It exposes the primitives the spec
requires — start, pause/resume, [`advance`](Self::advance),
[`reverse`](Self::reverse) and [`seek`](Self::seek) — from which
interruption, reversal, chaining and synchronization are built downstream.
