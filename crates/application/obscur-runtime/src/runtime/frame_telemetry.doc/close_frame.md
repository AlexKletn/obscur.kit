Closes the frame span with the number of phases that actually ran, so an
observer can attribute work to this frame (spec §7.9).

`phases_run` of `0` is the idle frame — a diagnostics capture recalculates
nothing — and is recorded rather than skipped, so a frame that did no work
is still visible as a frame.
