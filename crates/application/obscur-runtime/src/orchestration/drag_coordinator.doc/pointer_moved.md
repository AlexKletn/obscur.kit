Tracks the pointer to `position`, with `under` the node it is currently
over (as hit-tested by the caller).

Drives the session's `enter` / `leave` and restages the provisional work
for the new hover. Returns whether anything observable changed, so a caller
can request a frame only when the drag actually moved between targets.

A no-op when no drag is in flight.
