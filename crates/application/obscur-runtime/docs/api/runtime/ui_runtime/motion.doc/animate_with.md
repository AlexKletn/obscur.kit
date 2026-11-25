Starts an animation of `property` on `node` over `track`, with full control
of its playback (delay, repeat, direction, fill) — the general form
[`animate`](Self::animate) is the common case of (spec §4.17).

Supersedes any animation already running for the same `(node, property)`
target, resolving the incoming track against the in-flight one per
`interruption`. Requests an [`FrameReason::Animation`] frame, so the
animation reaches paint without the host arranging it.

Returns the [`TimelineId`] the animation runs under.

Eight parameters because this is the *general* form: every one of them is
an independent axis of css-animations-1 playback, and bundling them into a
config struct would only move the same eight to a different line.
[`animate`](Self::animate) is the short form for the common case.
