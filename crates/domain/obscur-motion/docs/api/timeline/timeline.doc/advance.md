Advances a running timeline by finite, positive logical seconds. A pending
start delay consumes time first.

When a step crosses iteration boundaries, it counts completed iterations
arithmetically and carries the remaining time into the active iteration.
Alternating playback uses the parity of the crossed boundaries even if the
iteration counter has saturated. A finite repeat settles at the endpoint
of its final iteration.
