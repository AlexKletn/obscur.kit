Images from a stylesheet url and from an element `src`, and the blocking port
that loads them off the frame (spec §3.3, §3.4, §5.15).

Eight tests whose whole assertion vocabulary is *how many images did this frame
paint* — one when the load resolved, zero when it failed, when no provider was
installed, or when the load has not run yet. Zero is a real answer and most of
these want it: a failed load caches nothing, so an honest runtime paints no image
rather than a placeholder.

The timing half is the §3.3 point. A load is deferred out of the synchronous
frame phase, so the frame that triggers it paints without the image whether or
not a blocking executor is installed; what an executor changes is *where* the
work happens, not whether the first frame waits. The idle-frame test pins the
trap that follows from that: a diagnostics capture recalculates nothing but
still drains the completion channel, so a completion it consumes must also dirty
the frame consuming it — otherwise the result is swallowed and nothing ever asks
for a frame to paint it again.

The seven DPI-variant tests stayed in `ui_runtime.rs`. They read the resident
decoded image out of the private `resources` field, and one of them reads
`ResourceBridge::state_of`, which an earlier pass made `#[cfg(test)]` under this
campaign's own rule against public surface that exists to serve a test.
