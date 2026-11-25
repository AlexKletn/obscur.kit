Set while the acquirer is being dropped.

An acquire may be in flight when the surface is being taken down, and the image
it returns must be dropped on the acquire thread rather than parked in a slot
that nobody will ever empty.
