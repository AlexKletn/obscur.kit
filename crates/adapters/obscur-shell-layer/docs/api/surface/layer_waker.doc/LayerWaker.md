A handle that wakes a layer surface's event loop, from any thread.

# Why a loop needs waking at all

The loop sleeps on the compositor's socket. That is the right thing to do —
an idle panel should cost nothing — but it means **nothing outside the
compositor can make the loop turn**. A worker thread that finished, a timer
that fired, a second Wayland connection belonging to some other protocol: all
of them are invisible to a loop parked on one file descriptor.

That last case is not hypothetical. An on-screen keyboard's text-input
activation arrives on its *own* connection, on its own descriptor, and the
surface's loop polls only its own. Without a waker the keyboard would learn
it should appear and have no way to make the frame that shows it.

# Using it

Build one before the surface, hand clones wherever they are needed, and give
one to the surface at open time. [`wake`](Self::wake) is cheap, thread-safe,
and coalesces: several wakes before the loop turns cause one turn, which is
what a caller wants — the loop rebuilds from current state, not once per
notification.
