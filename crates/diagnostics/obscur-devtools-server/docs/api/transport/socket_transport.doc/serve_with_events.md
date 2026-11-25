Drives `endpoint` like [`serve`](Self::serve), additionally **pushing**
queued events to subscribers between client messages (spec §7.6).

This is what makes the endpoint live rather than poll-only: a host states
what happened by pushing onto `events` from its own thread, and this loop
fans each out through the endpoint — so subscription state, scopes and
authentication stay exactly where they were, on this thread.

# Why this is what grants a subscription

Entering this loop is what makes `granted: true` true, so it is this call
— not the host's word for it — that marks the endpoint as servicing event
push. An endpoint driven by [`serve`](Self::serve) instead denies every
`Subscribe`, because nothing there would ever drain a queue.

# Why a read timeout

A blocking read parks here until the client speaks, so a queued event could
not be written until the client happened to send something — which is not a
push. `poll` bounds the wait: the loop wakes, drains the queue, and goes
back to reading. It is the *maximum* latency of a push, not a busy-wait —
nothing spins, the thread sleeps in `read` between wakes.

An event for a topic nobody subscribed to fans out to nothing and writes
nothing, so an unwatched host pays only the wake.

# Errors

Returns an [`io::Error`] on any underlying socket read/write failure. A
read that merely timed out is not a failure and does not end the session.
