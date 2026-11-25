Closes the frame — the whole of `run_frame`, from its first statement to its
last — and reports when the window is full.

What the four phase marks did not claim is `other`, and it is now measured
rather than derived from a tiling: `start()` is the first thing `run_frame`
does and this is the last, so input dispatch, the resource commit, the plan,
the scene snapshot, the accessibility publish and the scroll refresh are inside
the bracket and outside every phase. The `present` field is the host loop's
number for the previous turn, handed back through `record_present_time`, and it
is **taken**: a present is reported for exactly one frame's line, so a frame
built with nothing waiting behind it prints `present 0.000` instead of
repeating a number that belongs to a different frame. `[phase-stats]` averages
`present` over the frames that carried one and prints how many that was, which
is what makes "built and never shown" a number a reader can see.
