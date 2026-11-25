The extent the swapchain should be configured to, if it is not already.

The authority on resizing, and the message on the channel is only a nudge.
`try_send` may fail — the channel holds one request — and a resize that lived
only in a lost message left the swapchain at the old extent permanently: every
image acquired from it was rejected on arrival by the extent gate in the runner,
the frame that wanted it stayed pending, and nothing in the process said a word.
The thread reads this before every acquire, so the worst a lost message costs is
one turn.
