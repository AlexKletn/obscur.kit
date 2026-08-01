The §7.9 performance channel: the telemetry sink each frame's phases
record into and the per-phase cost budget their counts are checked
against.

One field rather than the two it replaces, because a phase's cost count is
both of them at once — see [`FrameTelemetry`] for why a sink and a budget
held apart let a violation contradict the counter beside it.
