A bounded, thread-shared queue of events a host wants pushed to subscribed
DevTools clients (spec §7.6 live inspectable state, §7.8 buffered +
backpressured).

# Why a queue and not a direct write

Subscriptions live inside the [`DevToolsEndpoint`](crate::endpoint::DevToolsEndpoint),
which the serving thread owns — a host on the frame thread cannot ask "who is
subscribed to this?" without sharing the endpoint's whole session state, and
with it the authentication and scope checks. So the host states only *what
happened*; the serving thread decides *who hears it*, still through the
endpoint's own fan-out. Security stays where it was.

# Bounded on purpose

[`MESSAGE_LIMIT`] is the same ceiling §7.7 already imposes elsewhere. A host
that outruns a slow client **drops the oldest** event rather than growing
without bound: an unbounded diagnostics queue turns a stalled inspector into a
memory leak in the app being inspected, which is the opposite of a diagnostic.
[`dropped`](Self::dropped) reports how many were lost, so a client is told
rather than silently shown a gap.
